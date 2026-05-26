# Baiban CLI

🚀 基于 Rust 构建的高性能项目脚手架工具，一行命令创建标准化项目模板。

## 特性

- ⚡ **极速** - Rust 编译为独立二进制，启动 < 10ms，体积仅 1.7 MB
- 🚫 **零依赖** - 无需 Node.js/Python 运行时环境
- 🎨 **交互式** - 彩色终端 + 箭头选择器，开发体验友好
- 🪞 **内置镜像** - 默认使用 GitHub 镜像加速，国内开箱即用
- 🔧 **可扩展** - TOML 配置模板列表，支持内置 + 用户自定义
- 🌍 **跨平台** - macOS / Linux / Windows，x64 / ARM64

## 快速开始

```bash
# 安装
pnpm add -g baiban-cli

# 创建项目（交互式）
baiban create

# 创建项目（直接指定模板）
baiban create my-app -t react-admin
```

## 安装

### 通过 npm（推荐）

```bash
# pnpm
pnpm add -g baiban-cli

# npm
npm install -g baiban-cli
```

> **提示**：安装后若提示 `command not found`，需将 pnpm 全局目录加入 PATH：
> ```bash
> echo 'export PATH="$HOME/Library/pnpm:$PATH"' >> ~/.zshrc
> source ~/.zshrc
> ```

### 直接下载二进制

前往 [GitHub Releases](https://github.com/qwepoih2/baiban-cli/releases) 下载对应平台文件：

| 平台 | 文件 |
|------|------|
| macOS Apple Silicon | `baiban-darwin-arm64` |
| macOS Intel | `baiban-darwin-x64` |
| Linux x64 | `baiban-linux-x64` |
| Linux ARM64 | `baiban-linux-arm64` |
| Windows x64 | `baiban-win32-x64.exe` |

```bash
# macOS / Linux 示例
curl -L -o /usr/local/bin/baiban \
  https://github.com/qwepoih2/baiban-cli/releases/download/v0.1.0/baiban-darwin-arm64
chmod +x /usr/local/bin/baiban
```

> **macOS 用户**：如果提示"无法验证开发者"，执行：
> ```bash
> xattr -d com.apple.quarantine /usr/local/bin/baiban
> ```

## 命令

### `baiban create [name]` - 创建项目

```bash
# 交互式
baiban create

# 直接指定项目名和模板
baiban create my-app -t react-admin

# 指定输出目录
baiban create my-app -t react-admin -d ~/projects

# 指定 GitHub 镜像
baiban create my-app --mirror https://ghfast.top/

# 跳过确认提示
baiban create my-app -t react-admin --yes
```

**选项：**

| 参数 | 说明 |
|------|------|
| `name` | 项目名称（可选，未指定则交互输入） |
| `-t, --template` | 指定模板名（跳过选择） |
| `-d, --dir` | 指定输出目录 |
| `-m, --mirror` | 指定 GitHub 镜像 |
| `-y, --yes` | 跳过确认提示 |

### `baiban template` - 管理模板

```bash
# 列出所有模板
baiban template list

# 交互式添加模板
baiban template add

# 移除模板
baiban template remove [name]
```

### `baiban config` - 管理配置

```bash
# 查看配置
baiban config show

# 设置 GitHub 镜像
baiban config set-mirror https://ghfast.top/

# 清除镜像设置
baiban config clear-mirror
```

## 内置模板

| 名称 | 说明 | 标签 |
|------|------|------|
| `react-admin` | React Admin 后台管理模板 | react, admin, typescript |

> 持续扩充中，欢迎提交 PR 添加更多模板。

## 添加自定义模板

### 交互式

```bash
baiban template add

? 模板名称 › vue3-admin
? 模板描述 › Vue3 后台管理模板
? Git 仓库地址 › https://github.com/your-org/vue3-template
? 分支名称 › main
```

### 编辑配置文件

用户模板保存在 `~/.baiban/templates.toml`：

```toml
[[templates]]
name = "vue3-admin"
description = "Vue3 后台管理模板"
repository = "https://github.com/your-org/vue3-template"
branch = "main"
```

## 镜像配置

内置 GitHub 镜像（`https://ghfast.top/`），开箱即用。

**三级覆盖（优先级从高到低）：**

1. **CLI 参数** - 临时使用
   ```bash
   baiban create my-app --mirror https://xxx/
   ```

2. **用户设置** - 永久覆盖
   ```bash
   baiban config set-mirror https://xxx/
   ```

3. **项目内置** - 编译时嵌入二进制

## 工作原理

```
baiban create my-app -t react-admin
│
├─ 1. 解析参数 / 交互式输入
├─ 2. 加载模板列表（内置 + 用户）
├─ 3. 应用镜像地址
├─ 4. git clone --depth 1 <mirror_url> my-app
├─ 5. 后处理：
│   ├── 删除 .git（清除模板历史）
│   ├── 替换 package.json name
│   ├── git init（新仓库）
│   └── git commit -m "init from baiban template"
└─ 6. 打印成功信息
```

## 本地开发

```bash
# 安装 Rust（推荐最小安装）
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --profile minimal

# 开发运行
cargo run -- create
cargo run -- template list

# 编译 release（产物约 1.7MB）
cargo build --release
ls -lh target/release/baiban

# 代码检查
cargo clippy
cargo fmt --check
```

## 发布新版本

```bash
# 1. 更新版本号（Cargo.toml 和 npm/package.json 保持一致）

# 2. 提交代码
git commit -am "chore: bump version to 0.2.0"
git push origin main

# 3. 打 tag 触发 GitHub Actions（自动编译 5 平台 + 创建 Release）
git tag v0.2.0
git push origin v0.2.0

# 4. 发布 npm
cd npm
npm version patch
npm publish
```

详细的发布指南见 [PUBLISH.md](./PUBLISH.md)。

## 技术栈

| 组件 | 说明 |
|------|------|
| [clap](https://github.com/clap-rs/clap) | CLI 框架 |
| [dialoguer](https://github.com/console-rs/dialoguer) | 交互 prompt |
| [console](https://github.com/console-rs/console) | 终端样式 |
| [serde](https://serde.rs/) | 序列化 |
| [anyhow](https://github.com/dtolnay/anyhow) | 错误处理 |

## 项目结构

```
baiban-cli/
├── Cargo.toml              # Rust 依赖配置
├── templates.toml          # 内置模板（编译时嵌入）
├── src/
│   ├── main.rs             # 入口 + 命令分发
│   ├── cli.rs              # 命令定义
│   ├── config.rs           # 配置管理
│   ├── template.rs         # 模板下载与后处理
│   └── ui.rs               # 终端交互
├── npm/
│   ├── package.json        # npm 包配置
│   ├── install.js          # 平台检测 + 二进制下载
│   └── bin/baiban          # Shell wrapper
└── .github/workflows/
    └── release.yml         # 跨平台编译 CI
```

## 常见问题

<details>
<summary><b>安装后找不到 baiban 命令</b></summary>

将 pnpm/npm 全局目录加入 PATH：
```bash
# pnpm
echo 'export PATH="$HOME/Library/pnpm:$PATH"' >> ~/.zshrc

# npm（先查看路径）
echo "export PATH=\"$(npm config get prefix)/bin:\$PATH\"" >> ~/.zshrc

source ~/.zshrc
```
</details>

<details>
<summary><b>macOS 提示"无法验证开发者"</b></summary>

```bash
BIN=$(find $(pnpm root -g) -name "baiban-bin" 2>/dev/null | head -1)
xattr -d com.apple.quarantine "$BIN"
```
</details>

<details>
<summary><b>clone 模板超时</b></summary>

```bash
# 检查镜像配置
baiban config show

# 临时指定镜像
baiban create my-app --mirror https://ghfast.top/
```
</details>

<details>
<summary><b>支持非 GitHub 仓库吗？</b></summary>

支持。添加模板时直接输入 GitLab/Gitee 等地址即可，非 `github.com` 的 URL 不会走镜像。
</details>

## 许可证

[MIT](./LICENSE)

## 相关链接

- [GitHub 仓库](https://github.com/qwepoih2/baiban-cli)
- [GitHub Releases](https://github.com/qwepoih2/baiban-cli/releases)
- [模板示例 - React Admin](https://github.com/qwepoih2/react-template-admin)
- [开发文档](./DEVELOPMENT.md)
- [发布指南](./PUBLISH.md)
- [技术博客](./BLOG.md)
- [团队分享](./TEAM-SHARING.md)
