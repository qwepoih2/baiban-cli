# Baiban CLI

🚀 基于 Rust 构建的高性能项目脚手架工具，一行命令创建标准化项目模板。

## 特性

- ⚡ **极速** - 独立二进制，启动 < 10ms，体积仅 1.7 MB
- 🚫 **零依赖** - 无需 Node.js/Python 运行时
- 🎨 **交互式** - 彩色终端 + 箭头选择器
- 🪞 **内置镜像** - 默认使用 GitHub 镜像，国内开箱即用
- 🌍 **跨平台** - macOS / Linux / Windows，x64 / ARM64

## 安装

```bash
# pnpm
pnpm add -g baiban-cli

# npm
npm install -g baiban-cli
```

> **提示**：安装后若提示 `command not found`，需将全局 bin 目录加入 PATH：
> ```bash
> # pnpm 用户
> echo 'export PATH="$HOME/Library/pnpm:$PATH"' >> ~/.zshrc
> source ~/.zshrc
>
> # npm 用户（先查看路径）
> echo "export PATH=\"$(npm config get prefix)/bin:\$PATH\"" >> ~/.zshrc
> source ~/.zshrc
> ```

## 快速开始

```bash
# 创建项目（交互式）
baiban create

# 创建项目（直接指定模板）
baiban create my-app -t react-admin
```

## 命令

### `baiban create [name]`

创建新项目。

```bash
baiban create                                  # 交互式
baiban create my-app -t react-admin            # 指定模板
baiban create my-app -t react-admin -d ~/dir   # 指定目录
baiban create my-app --mirror https://xxx/     # 指定镜像
baiban create my-app -t react-admin --yes      # 跳过确认
```

**选项：**

| 参数 | 说明 |
|------|------|
| `-t, --template` | 指定模板名（跳过选择） |
| `-d, --dir` | 指定输出目录 |
| `-m, --mirror` | 指定 GitHub 镜像 |
| `-y, --yes` | 跳过确认提示 |

### `baiban template`

管理模板。

```bash
baiban template list               # 列出所有模板
baiban template add                # 交互式添加模板
baiban template remove [name]      # 移除模板
```

### `baiban config`

管理配置。

```bash
baiban config show                              # 查看配置
baiban config set-mirror https://ghfast.top/    # 设置镜像
baiban config clear-mirror                      # 清除镜像
```

## 内置模板

| 名称 | 说明 | 标签 |
|------|------|------|
| `react-admin` | React Admin 后台管理模板 | react, admin, typescript |

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

## 常见问题

### 安装后找不到 `baiban` 命令

将全局 bin 目录加入 PATH（见上文安装说明）。

### macOS 提示"无法验证开发者"

```bash
# 找到二进制位置
BIN=$(find $(npm root -g)/baiban-cli -name "baiban-bin" 2>/dev/null | head -1)

# 移除隔离标记
xattr -d com.apple.quarantine "$BIN"
```

### clone 模板超时

```bash
# 检查镜像配置
baiban config show

# 临时指定镜像
baiban create my-app --mirror https://ghfast.top/
```

### 支持非 GitHub 仓库吗？

支持。添加模板时直接输入 GitLab/Gitee 等地址即可，非 `github.com` 的 URL 不会走镜像。

## 相关链接

- [GitHub 仓库](https://github.com/qwepoih2/baiban-cli)
- [GitHub Releases](https://github.com/qwepoih2/baiban-cli/releases)
- [模板示例 - React Admin](https://github.com/qwepoih2/react-template-admin)

## 许可证

[MIT](./LICENSE)
