# 用 Rust 从零打造一个 CLI 脚手架工具

> 从想法到发布 npm，记录用 Rust 构建跨平台 CLI 脚手架工具 Baiban 的完整过程，以及我们踩过的每一个坑。

## 为什么选 Rust

团队内部有多个项目模板（React Admin、Vue3 Admin 等），每次新建项目都要手动 clone、删 .git、改 package.json，重复且容易出错。

市面上的脚手架工具（Yeoman、create-react-app）要么太重，要么不够灵活。我们想要一个：

- **快**：二进制文件 < 2MB，启动即用
- **轻**：零运行时依赖，不需要 Node.js/Python 环境
- **灵活**：TOML 配置模板列表，随时添加

Rust 编译出的独立二进制完美满足这些需求。对比一下运行时内存：

| 方案 | 产物体积 | 运行时依赖 |
|------|---------|-----------|
| **Rust** | ~1.7 MB | 无 |
| Node.js CLI | ~50 MB | Node.js 运行时 |
| Go CLI | ~8 MB | 无 |
| Python CLI | ~30 MB | Python 运行时 |

Rust 的编译产物最小，且不需要任何运行时——一个二进制文件扔到服务器上就能跑。

## 技术选型

| 功能 | 选择 | 理由 |
|------|------|------|
| CLI 框架 | `clap` (derive) | Rust CLI 事实标准，derive 宏零样板 |
| 交互 prompt | `dialoguer` | 终端交互组件丰富（Input/Select/Confirm） |
| 终端样式 | `console` | dialoguer 的姊妹库，彩色输出 + spinner |
| 配置解析 | `serde` + `toml` | Rust 生态标配，类型安全 |
| 错误处理 | `anyhow` | 简洁的 `?` 传播，开发阶段足够用 |
| Git 操作 | `std::process::Command` | 直接调用系统 git，避免 git2 的编译开销 |

最终 `Cargo.toml` 非常干净，只有 7 个依赖：

```toml
[dependencies]
clap = { version = "4", features = ["derive"] }
dialoguer = "0.11"
console = "0.15"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"
anyhow = "1"
```

## 架构设计

项目分为 5 个模块，职责清晰：

```
src/
├── main.rs        # 入口：命令分发 + 错误处理
├── cli.rs         # clap derive 定义命令树
├── config.rs      # 模板和设置的读写、合并、镜像处理
├── template.rs    # git clone + 后处理
└── ui.rs          # 交互 prompt + 格式化输出
```

### 命令树

```
baiban
├── create [name]          创建项目（交互式 or 参数式）
│   ├── --template, -t     指定模板
│   ├── --mirror, -m       指定 GitHub 镜像
│   ├── --dir, -d          指定输出目录
│   └── --yes, -y          跳过确认
├── template
│   ├── list               列出可用模板
│   ├── add                交互式添加模板
│   └── remove [name]      移除模板
└── config
    ├── set-mirror <url>   设置镜像
    ├── clear-mirror       清除镜像
    └── show               查看配置
```

用 `clap` 的 derive 宏定义这个命令树，只需要一个 enum：

```rust
#[derive(Subcommand)]
pub enum Commands {
    Create {
        name: Option<String>,
        #[arg(short, long)] template: Option<String>,
        #[arg(short, long)] dir: Option<String>,
        #[arg(short, long)] mirror: Option<String>,
        #[arg(short, long)] yes: bool,
    },
    #[command(subcommand)] Template(TemplateCommands),
    #[command(subcommand)] Config(ConfigCommands),
}
```

注释直接变成 `--help` 的输出文案，非常丝滑。

### 配置三层覆盖

这是我们在实践中设计出的核心机制——**三层配置覆盖**：

```
优先级从高到低：

1. CLI 参数     baiban create --mirror https://xxx/
                    ↓
2. 用户设置     ~/.baiban/settings.toml
                    ↓
3. 项目内置     templates.toml（编译时嵌入二进制）
```

为什么这样设计？

- **项目内置**：镜像地址写在 `templates.toml` 里，编译时通过 `include_str!` 嵌入二进制。用户拿到 CLI 直接能用，国内开箱即用。
- **用户设置**：有些人想用其他镜像，`baiban config set-mirror` 写到 `~/.baiban/settings.toml`，覆盖内置。
- **CLI 参数**：临时换一个镜像，最高优先级。

```rust
pub fn get_mirror(cli_mirror: &Option<String>) -> Option<String> {
    if let Some(m) = cli_mirror { return Some(m.clone()); }      // CLI
    if let Ok(user) = load_user_settings() {
        if user.mirror.is_some() { return user.mirror; }         // 用户
    }
    load_builtin_settings().ok().and_then(|s| s.mirror)          // 内置
}
```

### 模板下载与后处理

`template.rs` 做两件事：clone + post_process。

```rust
// clone：先尝试指定分支，失败则用默认分支
git clone --depth 1 --branch <branch> <mirror_url> <dir>
// 失败回退
git clone --depth 1 <mirror_url> <dir>

// post_process：清理 + 初始化
rm -rf .git                      // 删掉模板的 git 历史
sed package.json name=项目名     // 替换项目名
git init                         // 新仓库
git add . && git commit          // 初始 commit
```

镜像转换只对 `https://github.com/` 开头的 URL 生效：

```rust
pub fn apply_mirror(url: &str, mirror: &Option<String>) -> String {
    match mirror {
        Some(prefix) if url.starts_with("https://github.com/") => {
            format!("{}/{}", prefix.trim_end_matches('/'), url)
        }
        _ => url.to_string(),
    }
}
```

## 发布到 npm

Rust 项目发布到 npm，核心思路是：**npm 包只做薄壳，安装时下载对应平台的二进制**。

### 方案：单包 + install.js

```
npm install -g baiban-cli
        ↓
postinstall → node install.js
        ↓
检测系统平台（os.platform() + os.arch()）
        ↓
从 GitHub Releases 下载对应二进制
        ↓
保存到 bin/baiban-bin
```

`install.js` 核心逻辑：

```javascript
const platformMap = {
  "darwin arm64":  "baiban-darwin-arm64",
  "darwin x64":    "baiban-darwin-x64",
  "linux x64":     "baiban-linux-x64",
  "linux arm64":   "baiban-linux-arm64",
  "win32 x64":     "baiban-win32-x64.exe",
  "win32 arm64":   "baiban-win32-arm64.exe",
};

const key = `${os.platform()} ${os.arch()}`;
const url = `https://github.com/${REPO}/releases/download/v${VERSION}/${platformMap[key]}`;
```

key 用 `${platform} ${arch}` 空格拼接，这是参考了 ogito 等项目的做法。

### Wrapper 脚本的符号链接坑

`bin/baiban` 是个 shell wrapper，转发给实际二进制。这里我们踩了一个大坑：

npm/pnpm 全局安装时会创建符号链接：

```
/usr/local/bin/baiban → /usr/local/lib/.../node_modules/baiban-cli/bin/baiban
```

如果 wrapper 脚本用 `$0` 直接取路径，拿到的是链接位置而不是实际包目录，`baiban-bin` 自然找不到。

修复方法——解析符号链接：

```bash
#!/bin/sh
SOURCE="$0"
while [ -L "$SOURCE" ]; do
  DIR="$(cd "$(dirname "$SOURCE")" && pwd)"
  SOURCE="$(readlink "$SOURCE")"
  case "$SOURCE" in
    /*) ;;
    *) SOURCE="$DIR/$SOURCE" ;;
  esac
done
DIR="$(cd "$(dirname "$SOURCE")" && pwd)"

exec "$DIR/baiban-bin" "$@"
```

### macOS Gatekeeper 拦截

下载的未签名二进制会被 macOS 的 Gatekeeper 拦截。用户首次运行需要：

```bash
xattr -d com.apple.quarantine $(which baiban-bin)
```

这是一个已知问题，目前还没有完美解法。正式方案是用 Apple Developer 证书做签名（需要 $99/年）。

## GitHub Actions 跨平台编译

5 个平台 × 2 种编译方式，一个 workflow 搞定：

```yaml
strategy:
  fail-fast: false    # 一个平台失败不影响其他
  matrix:
    include:
      # macOS: 原生编译
      - target: aarch64-apple-darwin   # Apple Silicon
      - target: x86_64-apple-darwin    # Intel
      # Linux: x64 原生，arm64 用 cross
      - target: x86_64-unknown-linux-musl
      - target: aarch64-unknown-linux-musl   # cross: true
      # Windows: MSVC
      - target: x86_64-pc-windows-msvc
```

`fail-fast: false` 很关键——我们第一次发布时，Linux ARM64 编译失败导致所有平台都被取消了。

ARM64 Linux 需要交叉编译工具链，用 `houseabsolute/actions-rust-cross` action 处理：

```yaml
- name: Build (cross)
  if: matrix.cross
  uses: houseabsolute/actions-rust-cross@v1
  with:
    target: ${{ matrix.target }}
    args: "--release"
```

另一个坑是 **GitHub Actions 默认没有创建 Release 的权限**，需要在 workflow 顶部加：

```yaml
permissions:
  contents: write
```

## 踩坑总结

| 坑 | 原因 | 解法 |
|----|------|------|
| git clone 超时 | 国内访问 GitHub 不稳定 | 内置 ghfast.top 镜像 |
| dialoguer `Input` move 报错 | Rust 所有权，`with_prompt` 消费了 self | 用 `input = input.with_prompt(...)` 重新赋值 |
| CI 权限不足 | Actions 默认只读 | 加 `permissions: contents: write` |
| CI 一个失败全部取消 | `fail-fast` 默认 true | 设为 `false` |
| Linux ARM64 编译失败 | 缺少交叉编译工具链 | 用 `actions-rust-cross` action |
| npm 安装后找不到命令 | pnpm 全局 bin 不在 PATH | `export PATH="$HOME/Library/pnpm:$PATH"` |
| 二进制找不到 | npm 符号链接未解析 | wrapper 脚本循环解析 readlink |
| macOS 打不开二进制 | Gatekeeper 安全限制 | `xattr -d com.apple.quarantine` |
| git push 卡住 | SSH 没配 key 或网络问题 | 用 HTTPS + 镜像代理 |

## 最终效果

```
$ baiban create

🚀 Baiban 项目脚手架工具

? 请输入项目名称: › my-admin
? 请选择模板: ›
  ❯ react-admin - React Admin 后台管理模板 [react, admin, typescript]
? 确认创建项目 my-admin？(Y/n) › Y

✓ 模板克隆完成！
✓ 项目初始化完成！

✓ 项目已创建: /Users/rr/my-admin

后续步骤:
  cd my-admin
  npm install
  npm run dev
```

整个项目从想法到发布，不到一天。Rust 的学习曲线确实陡，但一旦编译通过，代码质量就很有保障——编译器替你挡掉了大部分 bug。

如果你们团队也需要一个轻量、快速的脚手架工具，不妨试试这个思路：Rust CLI + GitHub Actions + npm 薄壳。
