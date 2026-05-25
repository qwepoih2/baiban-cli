# Baiban CLI - 开发文档

## 项目简介

Baiban 是一个基于 Rust 构建的 CLI 脚手架工具，用于快速下载和管理项目模板。

- 通过 `git clone --depth 1` 浅克隆 GitHub 模板仓库
- 内置 GitHub 镜像加速（国内用户开箱即用）
- 支持内置模板 + 用户自定义模板，双层配置覆盖
- 交互式终端界面，彩色输出

## 技术栈

| 组件 | 技术 | 版本 |
|------|------|------|
| 语言 | Rust | 2024 Edition |
| CLI 解析 | clap (derive) | 4.x |
| 交互式 prompt | dialoguer | 0.11 |
| 终端样式 | console | 0.15 |
| 序列化 | serde + serde_json | 1.x |
| TOML 解析 | toml | 0.8 |
| 错误处理 | anyhow | 1.x |

## 项目结构

```
baiban/
├── Cargo.toml              # Rust 项目配置 + 依赖
├── templates.toml           # 内置模板和全局设置（编译时嵌入二进制）
└── src/
    ├── main.rs              # 入口：命令分发 + 业务逻辑
    ├── cli.rs               # CLI 参数定义（clap derive 宏）
    ├── config.rs            # 配置管理：模板读写、设置管理、镜像处理
    ├── template.rs          # 模板下载（git clone）+ 后处理
    └── ui.rs                # 终端交互：prompt、彩色输出
```

## 核心模块说明

### cli.rs — 命令定义

使用 `clap` derive 宏定义命令结构，零样板代码：

```rust
#[derive(Parser)]
#[command(name = "baiban", version, about = "Baiban CLI - 项目脚手架工具")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}
```

**命令树：**

```
baiban
├── create [name]
│   ├── --template, -t     指定模板
│   ├── --dir, -d          指定输出目录
│   ├── --mirror, -m       指定 GitHub 镜像
│   └── --yes, -y          跳过确认
├── template
│   ├── list               列出所有模板
│   ├── add                添加自定义模板
│   └── remove [name]      移除自定义模板
└── config
    ├── set-mirror <url>   设置 GitHub 镜像
    ├── clear-mirror       清除镜像设置
    └── show               查看当前配置
```

### config.rs — 配置管理

**配置层级（优先级从高到低）：**

```
CLI --mirror 参数
    ↓
~/.baiban/settings.toml（用户级覆盖）
    ↓
templates.toml（项目内置，编译进二进制）
```

**关键数据结构：**

```rust
// 模板定义
pub struct Template {
    pub name: String,           // 模板标识名
    pub description: String,    // 描述（交互列表中显示）
    pub repository: String,     // Git 仓库地址
    pub branch: String,         // 分支名（默认 main）
    pub tags: Vec<String>,      // 标签（交互列表中显示）
}

// 全局设置
pub struct Settings {
    pub mirror: Option<String>, // GitHub 镜像前缀
}
```

**配置合并逻辑：**

1. `load_builtin_templates()` — 通过 `include_str!` 宏在编译时嵌入 `templates.toml`
2. `load_user_templates()` — 运行时读取 `~/.baiban/templates.toml`
3. `load_all_templates()` — 合并去重，用户模板覆盖同名内置模板，按名称排序

**镜像转换逻辑：**

```rust
// 将 https://github.com/user/repo 转为 https://ghfast.top/https://github.com/user/repo
pub fn apply_mirror(url: &str, mirror: &Option<String>) -> String { ... }
```

仅对 `https://github.com/` 开头的 URL 生效，其它地址原样返回。

### template.rs — 模板下载与后处理

**`clone_template()` 流程：**

```
1. 检查目标目录（不存在或非空则报错）
2. 检查 git 是否安装
3. 应用镜像地址
4. git clone --depth 1 --branch <branch> <url> <dir>
5. 失败则回退：git clone --depth 1 <url> <dir>（不指定分支，用默认分支）
6. 仍然失败则输出错误 + 镜像使用提示
```

**`post_process()` 流程：**

```
1. 删除 .git 目录（清除模板仓库的 git 历史）
2. 替换 package.json 中的 name 字段为用户输入的项目名
3. git init（初始化新的 git 仓库）
4. git add . && git commit -m "init from baiban template"
```

### ui.rs — 终端交互

基于 `dialoguer` 封装的交互组件：

| 函数 | 组件 | 用途 |
|------|------|------|
| `prompt_project_name()` | `Input` | 获取项目名称 |
| `prompt_select_template()` | `Select` | 模板选择列表 |
| `prompt_confirm()` | `Confirm` | 创建确认 |
| `prompt_template_name()` | `Input` | 添加模板 - 名称 |
| `prompt_template_description()` | `Input` | 添加模板 - 描述 |
| `prompt_template_repository()` | `Input` | 添加模板 - 仓库地址 |
| `prompt_template_branch()` | `Input` | 添加模板 - 分支 |
| `prompt_select_remove()` | `Select` | 选择要移除的模板 |
| `print_success()` | - | 创建成功信息 + 后续步骤 |
| `print_templates()` | - | 格式化模板列表 |

### main.rs — 入口与命令分发

`main()` 函数职责：

1. `Cli::parse()` 解析命令行参数
2. `match` 分发到对应处理函数（`cmd_create`、`cmd_template_*`、`cmd_config_*`）
3. 统一错误处理：`Err` 时红色输出 + `exit(1)`

## 开发指南

### 环境要求

- Rust 1.85+ (2024 Edition)
- git

### 安装 Rust

```bash
# 最小安装
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --profile minimal

# 加载环境变量
source ~/.zshenv

# 验证
rustc --version
cargo --version
```

### 常用命令

```bash
# 开发编译（debug 模式，编译快）
cargo build

# 开发运行
cargo run -- create
cargo run -- template list
cargo run -- create my-app -t react-admin --yes

# 发布编译（release 模式，优化后体积小）
cargo build --release
# 产物位于 target/release/baiban

# 代码检查
cargo clippy          # lint 检查
cargo fmt             # 格式化

# 清理编译产物（释放磁盘空间）
cargo clean
```

### 添加内置模板

编辑 `templates.toml`，添加新的 `[[templates]]` 段：

```toml
[[templates]]
name = "vue3-admin"
description = "Vue3 后台管理模板"
repository = "https://github.com/your-org/vue3-admin"
branch = "main"
tags = ["vue", "admin", "typescript"]
```

重新编译后生效。

### 修改镜像设置

编辑 `templates.toml` 中的 `[settings]` 段：

```toml
[settings]
mirror = "https://ghfast.top/"   # 修改为其它镜像或删除此行直连 GitHub
```

## 配置文件说明

### templates.toml（项目内置）

编译时通过 `include_str!` 嵌入二进制，用户无需配置即可使用。

```toml
# 全局设置
[settings]
mirror = "https://ghfast.top/"

# 模板列表
[[templates]]
name = "react-admin"
description = "React Admin 后台管理模板"
repository = "https://github.com/qwepoih2/react-template-admin"
branch = "main"
tags = ["react", "admin", "typescript"]
```

### ~/.baiban/templates.toml（用户模板）

运行时通过 `baiban template add` 创建，存储用户自定义模板。与内置模板同名时覆盖内置。

### ~/.baiban/settings.toml（用户设置）

运行时通过 `baiban config set-mirror` 创建，覆盖内置设置。

```toml
mirror = "https://other-mirror.com/"
```

## 错误处理策略

项目统一使用 `anyhow::Result<T>` 进行错误传播：

- `bail!()` — 业务逻辑错误（目录已存在、模板未找到等）
- `.context("说明")` — 附加上下文（IO 失败、解析失败等）
- `main()` 统一捕获，红色输出后 `exit(1)`
