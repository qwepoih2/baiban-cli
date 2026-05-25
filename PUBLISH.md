# Baiban CLI - 发布指南

## 发布方案概述

Baiban 是 Rust 项目（编译为原生二进制），发布到 npm 的思路是：

```
GitHub Releases（存放各平台二进制文件）
        ↓
npm 包（安装脚本检测系统，下载对应平台的二进制）
        ↓
用户执行 npx baiban create
```

**发布分两阶段：**
1. 编译 + 上传二进制到 GitHub Releases
2. 发布 npm wrapper 包

---

## 阶段一：编译跨平台二进制

### 本地编译（当前机器）

```bash
# macOS ARM (Apple Silicon)
cargo build --release --target aarch64-apple-darwin

# macOS Intel
cargo build --release --target x86_64-apple-darwin
```

### 交叉编译其它平台（需要额外工具链）

```bash
# 安装交叉编译目标
rustup target add x86_64-unknown-linux-gnu
rustup target add x86_64-pc-windows-gnu

# Linux x64（需要安装 musl 或 gnu 交叉编译器）
cargo build --release --target x86_64-unknown-linux-musl

# Windows x64
cargo build --release --target x86_64-pc-windows-gnu
```

### 推荐：用 GitHub Actions 自动编译

在项目根目录创建 `.github/workflows/release.yml`，推送 tag 时自动编译所有平台并上传到 GitHub Releases（见下方 CI 配置）。

### 编译产物位置

```
target/release/baiban                    # macOS/Linux
target/release/baiban.exe                # Windows
```

---

## 阶段二：发布 npm 包

### npm 包结构

在项目中创建 `npm/` 目录作为 npm 包的根：

```
npm/
├── package.json
├── install.js      # 安装脚本：检测平台，下载二进制
├── bin/
│   └── baiban      # shell 入口（转发给下载的二进制）
└── README.md
```

### package.json

```json
{
  "name": "baiban",
  "version": "0.1.0",
  "description": "Baiban CLI - 项目脚手架工具",
  "bin": {
    "baiban": "bin/baiban"
  },
  "scripts": {
    "postinstall": "node install.js"
  },
  "engines": {
    "node": ">=14"
  },
  "files": [
    "bin/",
    "install.js",
    "README.md"
  ],
  "repository": {
    "type": "git",
    "url": "https://github.com/your-org/baiban"
  },
  "keywords": ["cli", "scaffold", "template", "react"],
  "license": "MIT"
}
```

### install.js（核心安装脚本）

```js
#!/usr/bin/env node
const { execSync } = require("child_process");
const fs = require("fs");
const path = require("path");
const https = require("https");
const os = require("os");

const VERSION = require("./package.json").version;
const REPO = "your-org/baiban"; // 改为你的 GitHub 仓库

// 检测平台
function getPlatformInfo() {
  const platform = os.platform();
  const arch = os.arch();

  const platformMap = {
    "darwin-arm64": "baiban-darwin-arm64",
    "darwin-x64": "baiban-darwin-x64",
    "linux-x64": "baiban-linux-x64",
    "win32-x64": "baiban-win32-x64.exe",
  };

  const key = `${platform}-${arch}`;
  const binaryName = platformMap[key];

  if (!binaryName) {
    throw new Error(`不支持的平台: ${platform}-${arch}`);
  }

  return { binaryName, isWindows: platform === "win32" };
}

// 下载文件
function download(url, dest) {
  return new Promise((resolve, reject) => {
    const file = fs.createWriteStream(dest);
    https.get(url, (res) => {
      if (res.statusCode === 302 || res.statusCode === 301) {
        download(res.headers.location, dest).then(resolve).catch(reject);
        return;
      }
      if (res.statusCode !== 200) {
        reject(new Error(`下载失败: HTTP ${res.statusCode}`));
        return;
      }
      res.pipe(file);
      file.on("finish", () => {
        file.close(resolve);
      });
    }).on("error", reject);
  });
}

async function install() {
  const { binaryName, isWindows } = getPlatformInfo();
  const binDir = path.join(__dirname, "bin");
  const binaryPath = path.join(binDir, isWindows ? "baiban.exe" : "baiban-bin");

  // GitHub Release 下载地址
  const url = `https://github.com/${REPO}/releases/download/v${VERSION}/${binaryName}`;

  console.log(`正在下载 baiban v${VERSION} (${binaryName})...`);

  try {
    fs.mkdirSync(binDir, { recursive: true });
    await download(url, binaryPath);

    // 设置可执行权限（非 Windows）
    if (!isWindows) {
      fs.chmodSync(binaryPath, 0o755);
    }

    console.log("baiban 安装成功！");
  } catch (err) {
    console.error("安装失败:", err.message);
    process.exit(1);
  }
}

install();
```

### bin/baiban（Shell 入口）

```bash
#!/bin/sh
# 转发给下载的二进制文件
DIR="$(cd "$(dirname "$0")" && pwd)"

# 优先使用下载的二进制，否则尝试本地编译版
if [ -f "$DIR/baiban-bin" ]; then
  exec "$DIR/baiban-bin" "$@"
elif [ -f "$DIR/baiban.exe" ]; then
  exec "$DIR/baiban.exe" "$@"
else
  echo "错误: baiban 二进制未找到，请尝试重新安装: npm install -g baiban"
  exit 1
fi
```

---

## GitHub Actions 自动发布（推荐）

在仓库根目录创建 `.github/workflows/release.yml`：

```yaml
name: Release

on:
  push:
    tags:
      - "v*"

jobs:
  build:
    strategy:
      matrix:
        include:
          - target: x86_64-unknown-linux-musl
            os: ubuntu-latest
            name: baiban-linux-x64
          - target: aarch64-apple-darwin
            os: macos-latest
            name: baiban-darwin-arm64
          - target: x86_64-apple-darwin
            os: macos-latest
            name: baiban-darwin-x64
          - target: x86_64-pc-windows-gnu
            os: windows-latest
            name: baiban-win32-x64.exe

    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4

      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}

      - run: cargo build --release --target ${{ matrix.target }}

      - name: Rename binary
        shell: bash
        run: |
          mkdir -p dist
          if [ "${{ matrix.os }}" = "windows-latest" ]; then
            cp target/${{ matrix.target }}/release/baiban.exe dist/${{ matrix.name }}
          else
            cp target/${{ matrix.target }}/release/baiban dist/${{ matrix.name }}
          fi

      - uses: softprops/action-gh-release@v1
        with:
          files: dist/${{ matrix.name }}
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}

  publish-npm:
    needs: build
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - uses: actions/setup-node@v4
        with:
          node-version: "20"
          registry-url: "https://registry.npmjs.org"

      - name: Publish to npm
        run: |
          cd npm
          npm publish
        env:
          NODE_AUTH_TOKEN: ${{ secrets.NPM_TOKEN }}
```

---

## 完整发布流程（手动）

### 1. 更新版本号

```bash
# 更新 Cargo.toml
# version = "0.1.0" → "0.2.0"

# 更新 npm/package.json
# "version": "0.1.0" → "0.2.0"
```

### 2. 编译并上传二进制

```bash
# 编译当前平台
cargo build --release

# 上传到 GitHub Releases（需要 gh CLI）
gh release create v0.2.0 \
  target/release/baiban#baiban-darwin-arm64 \
  --title "v0.2.0" \
  --notes "Release v0.2.0"

# 其它平台的二进制需要在对应系统上编译后手动上传，或用 GitHub Actions
```

### 3. 发布 npm 包

```bash
cd npm

# 首次发布需要登录
npm login

# 发布
npm publish

# 验证
npx baiban --version
```

### 4. 创建 git tag

```bash
git tag v0.2.0
git push origin v0.2.0
# 触发 GitHub Actions 自动编译 + 发布（如果配置了 CI）
```

---

## 用户使用方式

### npm 安装

```bash
# 全局安装
npm install -g baiban

# 使用
baiban create
baiban template list
```

### npx 免安装使用

```bash
npx baiban create my-project
```

### Rust 原生安装（不依赖 Node.js）

```bash
# 从 GitHub Releases 下载
curl -L https://github.com/your-org/baiban/releases/download/v0.1.0/baiban-darwin-arm64 -o /usr/local/bin/baiban
chmod +x /usr/local/bin/baiban

# 验证
baiban --version
```

### Homebrew（可选，需要创建 tap）

```bash
brew tap your-org/baiban
brew install baiban
```

---

## 版本号规范

遵循 [SemVer](https://semver.org/) 语义化版本：

| 变更 | 示例 |
|------|------|
| 修复 bug | 0.1.0 → 0.1.1 |
| 新增功能（向后兼容） | 0.1.0 → 0.2.0 |
| 破坏性变更 | 0.1.0 → 1.0.0 |

**Cargo.toml 和 npm/package.json 的版本号必须保持一致。**

---

## 注意事项

1. **npm 包名冲突**：发布前先检查包名是否可用：
   ```bash
   npm view baiban
   # 如果返回 404 说明可用
   ```

2. **GitHub Token**：CI 发布 npm 需要配置 `NPM_TOKEN`：
   ```bash
   # 在 GitHub 仓库 Settings → Secrets 中添加
   NPM_TOKEN = npm_xxxxx
   ```

3. **Windows 支持**：Windows 交叉编译需要安装 `mingw-w64`：
   ```bash
   # macOS
   brew install mingw-w64

   # Ubuntu
   apt install gcc-mingw-w64
   ```

4. **Linux 静态链接**：推荐使用 `musl` 目标，生成静态二进制，兼容性更好：
   ```bash
   # macOS 上安装 musl 交叉编译器
   brew install filosottile/musl-cross/musl-cross
   ```
