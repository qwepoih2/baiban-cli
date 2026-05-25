# OXC CLI - Rust 环境安装与项目初始化指南

## 1. 安装 Rust 工具链

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --profile minimal
```

安装提示选择 **1) Proceed with standard installation**（直接回车）。

## 2. 加载环境变量

```bash
source ~/.zshenv
```

## 3. 验证安装

```bash
rustc --version
cargo --version
```

看到类似输出表示安装成功：

```
rustc 1.xx.x (xxxxxxxxx 2026-xx-xx)
cargo 1.xx.x (xxxxxxxxx 2026-xx-xx)
```

## 4. 初始化项目

```bash
cd /Users/rr/Documents/demo/cli/oxc-cli
cargo init --name oxc-cli
```

验证项目结构：

```bash
ls -la
```

应看到：

```
Cargo.toml
src/
  main.rs
```

## 5. 验证项目可编译

```bash
cargo run
```

应看到输出：

```
Hello, world!
```

## 6. （可选）补装常用工具

```bash
rustup component add rustfmt clippy
```

- `rustfmt` — 代码格式化
- `clippy` — 代码检查

## 7. （可选）查看/管理安装

```bash
# 查看已安装的工具链
rustup show

# 查看磁盘占用
du -sh ~/.cargo/

# 更新 Rust
rustup update

# 卸载
rustup self uninstall
```
