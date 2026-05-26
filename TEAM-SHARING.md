# Baiban CLI - 团队分享

> 团队内部项目脚手架工具，一行命令创建标准化项目模板。

## 一句话介绍

```bash
baiban create my-project
# 交互式选择模板 → 从 GitHub 克隆 → 自动初始化 → 开箱即用
```

## 它能做什么

| 功能 | 命令 | 说明 |
|------|------|------|
| 创建项目 | `baiban create` | 交互式选择模板，一键创建 |
| 直接创建 | `baiban create my-app -t react-admin` | 跳过交互，直接指定模板 |
| 查看模板 | `baiban template list` | 列出所有可用模板 |
| 添加模板 | `baiban template add` | 交互式添加自定义模板 |
| 移除模板 | `baiban template remove` | 移除自定义模板 |
| 查看配置 | `baiban config show` | 查看当前镜像等配置 |
| 设置镜像 | `baiban config set-mirror <url>` | 设置 GitHub 镜像加速 |

## 安装方式

### 方式一：npm（推荐）

```bash
# 公司内网私有源
pnpm add -g baiban-cli

# 验证
baiban --version
```

> pnpm 全局安装后如果找不到 `baiban` 命令，需要加 PATH：
> ```bash
> echo 'export PATH="$HOME/Library/pnpm:$PATH"' >> ~/.zshrc
> source ~/.zshrc
> ```

### 方式二：直接下载二进制

去 [GitHub Releases](https://github.com/qwepoih2/baiban-cli/releases) 下载对应平台的文件：

| 平台 | 文件名 |
|------|--------|
| macOS Apple Silicon | `baiban-darwin-arm64` |
| macOS Intel | `baiban-darwin-x64` |
| Linux x64 | `baiban-linux-x64` |
| Linux ARM64 | `baiban-linux-arm64` |
| Windows x64 | `baiban-win32-x64.exe` |

```bash
# macOS 示例
curl -L -o /usr/local/bin/baiban https://github.com/qwepoih2/baiban-cli/releases/download/v0.1.0/baiban-darwin-arm64
chmod +x /usr/local/bin/baiban

# 如果 macOS 提示"无法验证开发者"
xattr -d com.apple.quarantine /usr/local/bin/baiban
```

## 内置模板

| 名称 | 说明 | 仓库 |
|------|------|------|
| `react-admin` | React Admin 后台管理模板 | [react-template-admin](https://github.com/qwepoih2/react-template-admin) |

> 后续会持续添加更多模板（Vue3、Next.js、小程序等），在 `templates.toml` 里配置即可。

## 添加自定义模板

### 交互式添加

```bash
baiban template add

? 模板名称 › vue3-admin
? 模板描述 › Vue3 后台管理模板
? Git 仓库地址 › https://github.com/your-team/vue3-template
? 分支名称 › main
```

模板会保存到 `~/.baiban/templates.toml`，仅本机生效。

### 直接编辑配置文件

```toml
# ~/.baiban/templates.toml
[[templates]]
name = "vue3-admin"
description = "Vue3 后台管理模板"
repository = "https://github.com/your-team/vue3-template"
branch = "main"
```

### 添加内置模板（需要重新发版）

编辑项目中的 `templates.toml`，重新编译发布：

```toml
[[templates]]
name = "nextjs-starter"
description = "Next.js 项目模板"
repository = "https://github.com/your-team/nextjs-starter"
branch = "main"
tags = ["nextjs", "react", "ssr"]
```

## GitHub 镜像配置

内置了 `https://ghfast.top/` 镜像，国内开箱即用。

```bash
# 查看当前配置
baiban config show

# 修改镜像
baiban config set-mirror https://mirror.ghproxy.com/

# 清除用户设置（回退到内置镜像）
baiban config clear-mirror

# 临时使用（不修改配置）
baiban create --mirror https://other-mirror.com/
```

**优先级：CLI 参数 > 用户设置 > 项目内置**

## 架构概览

```
┌─────────────────────────────────────────────┐
│                用户执行 baiban               │
├─────────────────────────────────────────────┤
│  npm 包 (baiban-cli)                    │
│  ├── bin/baiban (shell wrapper)             │
│  ├── install.js (下载对应平台二进制)          │
│  └── package.json                           │
├─────────────────────────────────────────────┤
│  GitHub Releases (二进制文件存储)             │
│  ├── baiban-darwin-arm64                    │
│  ├── baiban-darwin-x64                      │
│  ├── baiban-linux-x64                       │
│  ├── baiban-linux-arm64                     │
│  └── baiban-win32-x64.exe                  │
├─────────────────────────────────────────────┤
│  GitHub Actions (CI 自动编译)                │
│  └── push tag v* → 编译 5 平台 → Release    │
└─────────────────────────────────────────────┘
```

### 配置覆盖层级

```
baiban create --mirror xxx    ← 最高优先级（临时）
        ↓ 未指定
~/.baiban/settings.toml       ← 用户级覆盖
        ↓ 未指定
templates.toml（编译进二进制） ← 项目内置（开箱即用）
```

### 创建项目流程

```
baiban create my-app -t react-admin
        │
        ├── 1. 解析参数 / 交互式获取项目名
        ├── 2. 加载模板列表（内置 + 用户）
        ├── 3. 选择模板（参数指定或交互选择）
        ├── 4. 确认创建（可 --yes 跳过）
        ├── 5. 获取镜像配置（三级覆盖）
        ├── 6. git clone --depth 1 <mirror_url> <dir>
        ├── 7. 后处理：
        │   ├── 删除 .git
        │   ├── 替换 package.json name
        │   ├── git init
        │   └── git add . && git commit
        └── 8. 打印成功信息 + 后续步骤
```

## 如何贡献

### 添加新的内置模板

1. 编辑 `templates.toml`，添加 `[[templates]]`
2. 本地测试：`cargo run -- template list`
3. 提交 PR

### 发布新版本

```bash
# 1. 更新版本号（Cargo.toml + npm/package.json 保持一致）
# 2. 提交代码
git add .
git commit -m "feat: xxx"
git push origin main

# 3. 打 tag 触发 CI
git tag v0.2.0
git push origin v0.2.0

# 4. 去 GitHub Actions 确认编译成功
# 5. 发布 npm
cd npm
npm version patch  # 或 minor/major
npm publish
```

### 本地开发

```bash
# 安装 Rust（如果没装）
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --profile minimal

# 编译运行
cargo run -- create
cargo run -- template list

# 编译 release
cargo build --release
# 产物在 target/release/baiban（约 1.7MB）
```

## 常见问题

### Q: 安装后输入 `baiban` 提示 command not found？

**pnpm 用户：**
```bash
echo 'export PATH="$HOME/Library/pnpm:$PATH"' >> ~/.zshrc
source ~/.zshrc
```

**npm 用户：**
```bash
# 查看 npm 全局 bin 路径
npm config get prefix
# 把这个路径加到 PATH
```

### Q: macOS 提示"无法打开，因为无法验证开发者"？

```bash
# 找到二进制位置
BIN=$(find $(pnpm root -g) -name "baiban-bin" 2>/dev/null | head -1)

# 移除隔离标记
xattr -d com.apple.quarantine "$BIN"
```

### Q: clone 模板失败或超时？

```bash
# 检查镜像是否生效
baiban config show

# 手动指定镜像
baiban create my-app --mirror https://ghfast.top/

# 检查网络
curl -I https://ghfast.top/
```

### Q: 想加一个非 GitHub 的模板（GitLab、Gitee）？

```bash
baiban template add

# 直接输入对应的仓库地址即可，非 github.com 的地址不会走镜像
? Git 仓库地址 › https://gitlab.com/your-team/template
```

### Q: 如何删除用户添加的模板？

```bash
baiban template remove

# 交互式选择要移除的模板
# 注意：内置模板无法通过此命令移除
```

## 对比其他方案

| 对比项 | Baiban (Rust) | Node.js CLI | Shell 脚本 |
|--------|--------------|-------------|-----------|
| 安装方式 | npm / 直接下载 | npm | 手动下载 |
| 运行时依赖 | 无 | Node.js | git + bash |
| 产物体积 | ~1.7 MB | ~50 MB | < 10 KB |
| 启动速度 | < 10ms | ~200ms | < 10ms |
| 交互体验 | 丰富（彩色 + 选择器） | 丰富 | 简陋 |
| 跨平台 | macOS/Linux/Windows | 全平台 | 仅 Unix-like |
| 维护成本 | 低（改 TOML） | 低 | 中 |
| 可扩展性 | 强（模板 + 配置） | 强 | 弱 |

## 仓库地址

- GitHub: https://github.com/qwepoih2/baiban-cli
- npm: baiban-cli
- 模板仓库: https://github.com/qwepoih2/react-template-admin
