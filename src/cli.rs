use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "baiban", version, about = "Baiban CLI - 项目脚手架工具")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// 创建新项目
    Create {
        /// 项目名称
        name: Option<String>,

        /// 指定模板名称（跳过交互选择）
        #[arg(short, long)]
        template: Option<String>,

        /// 指定输出目录（默认当前目录）
        #[arg(short, long)]
        dir: Option<String>,

        /// GitHub 镜像前缀（如 https://ghfast.top/），加速下载
        #[arg(short, long)]
        mirror: Option<String>,

        /// 跳过确认提示，直接创建
        #[arg(short, long)]
        yes: bool,
    },

    /// 管理模板
    #[command(subcommand)]
    Template(TemplateCommands),

    /// 管理配置
    #[command(subcommand)]
    Config(ConfigCommands),
}

#[derive(Subcommand)]
pub enum TemplateCommands {
    /// 列出所有可用模板
    List,

    /// 添加自定义模板
    Add,

    /// 移除自定义模板
    Remove {
        /// 模板名称
        name: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum ConfigCommands {
    /// 设置 GitHub 镜像地址
    SetMirror {
        /// 镜像 URL（如 https://ghfast.top/）
        url: String,
    },

    /// 清除 GitHub 镜像设置
    ClearMirror,

    /// 查看当前配置
    Show,
}
