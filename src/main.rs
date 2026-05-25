mod cli;
mod config;
mod template;
mod ui;

use anyhow::{bail, Result};
use clap::Parser;
use console::style;
use std::path::PathBuf;

use cli::{Cli, Commands, ConfigCommands, TemplateCommands};
use config::{
    add_user_template, find_template, get_mirror, load_all_templates, load_builtin_settings,
    load_user_settings, load_user_templates, remove_user_template, save_user_settings,
    Template,
};

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Create {
            name,
            template,
            dir,
            mirror,
            yes,
        } => cmd_create(name, template, dir, mirror, yes),
        Commands::Template(sub) => match sub {
            TemplateCommands::List => cmd_template_list(),
            TemplateCommands::Add => cmd_template_add(),
            TemplateCommands::Remove { name } => cmd_template_remove(name),
        },
        Commands::Config(sub) => match sub {
            ConfigCommands::SetMirror { url } => cmd_config_set_mirror(url),
            ConfigCommands::ClearMirror => cmd_config_clear_mirror(),
            ConfigCommands::Show => cmd_config_show(),
        },
    };

    if let Err(err) = result {
        eprintln!("{} {}", style("错误:").red().bold(), err);
        std::process::exit(1);
    }
}

/// 创建新项目
fn cmd_create(
    name: Option<String>,
    template_name: Option<String>,
    dir: Option<String>,
    mirror: Option<String>,
    skip_confirm: bool,
) -> Result<()> {
    println!();
    println!(
        "{} {}",
        style("🚀 Baiban").bold(),
        style("项目脚手架工具").dim()
    );
    println!();

    // 1. 获取项目名称
    let project_name = match name {
        Some(n) => n,
        None => ui::prompt_project_name(None)?,
    };

    // 2. 选择模板
    let templates = load_all_templates()?;
    let selected_template: Template = match template_name {
        Some(tn) => match find_template(&tn)? {
            Some(t) => t,
            None => bail!(
                "未找到模板 '{}'，使用 `baiban template list` 查看可用模板",
                tn
            ),
        },
        None => ui::prompt_select_template(&templates)?.clone(),
    };

    // 3. 确认
    if !skip_confirm {
        let confirmed = ui::prompt_confirm(&project_name)?;
        if !confirmed {
            println!("{}", style("已取消").dim());
            return Ok(());
        }
    }

    // 4. 确定目标目录
    let target_dir = match dir {
        Some(d) => PathBuf::from(d).join(&project_name),
        None => PathBuf::from(&project_name),
    };

    // 5. 获取镜像配置
    let mirror_opt = get_mirror(&mirror);

    // 6. 克隆模板
    println!();
    println!(
        "{} 正在克隆模板 {} ...",
        style("⠋").cyan(),
        style(&selected_template.name).green().bold()
    );

    template::clone_template(&selected_template, &target_dir, &mirror_opt)?;

    println!("{} 模板克隆完成！", style("✓").green().bold());

    // 7. 后处理
    println!("{} 正在初始化项目 ...", style("⠋").cyan());

    template::post_process(&target_dir, &project_name)?;

    println!("{} 项目初始化完成！", style("✓").green().bold());

    // 8. 打印成功信息
    let abs_path = std::fs::canonicalize(&target_dir)
        .unwrap_or_else(|_| target_dir.clone())
        .to_string_lossy()
        .to_string();

    ui::print_success(&project_name, &abs_path);

    Ok(())
}

/// 列出所有模板
fn cmd_template_list() -> Result<()> {
    let templates = load_all_templates()?;
    ui::print_templates(&templates);
    Ok(())
}

/// 添加自定义模板
fn cmd_template_add() -> Result<()> {
    println!();
    println!("{}", style("添加自定义模板").bold());
    println!();

    let name = ui::prompt_template_name()?;
    let description = ui::prompt_template_description()?;
    let repository = ui::prompt_template_repository()?;
    let branch = ui::prompt_template_branch()?;

    let template = Template {
        name: name.clone(),
        description,
        repository,
        branch,
        tags: Vec::new(),
    };

    add_user_template(template)?;

    println!();
    println!(
        "{} 模板 {} 已添加！",
        style("✓").green().bold(),
        style(&name).green()
    );

    Ok(())
}

/// 移除自定义模板
fn cmd_template_remove(name: Option<String>) -> Result<()> {
    let template_name = match name {
        Some(n) => n,
        None => {
            let user_templates = load_user_templates()?;
            if user_templates.is_empty() {
                bail!("没有可移除的用户自定义模板");
            }
            let idx = ui::prompt_select_remove(&user_templates)?;
            user_templates[idx].name.clone()
        }
    };

    let removed = remove_user_template(&template_name)?;
    if removed {
        println!(
            "{} 模板 {} 已移除",
            style("✓").green().bold(),
            style(&template_name).red()
        );
    } else {
        bail!(
            "模板 '{}' 不在用户自定义模板中，内置模板无法移除",
            template_name
        );
    }

    Ok(())
}

/// 设置 GitHub 镜像（用户级覆盖）
fn cmd_config_set_mirror(url: String) -> Result<()> {
    let mut settings = load_user_settings()?;
    settings.mirror = Some(url.clone());
    save_user_settings(&settings)?;

    println!(
        "{} GitHub 镜像已设置（用户级覆盖）: {}",
        style("✓").green().bold(),
        style(&url).cyan()
    );
    Ok(())
}

/// 清除 GitHub 镜像（用户级覆盖）
fn cmd_config_clear_mirror() -> Result<()> {
    let mut settings = load_user_settings()?;
    settings.mirror = None;
    save_user_settings(&settings)?;

    println!(
        "{} 用户镜像覆盖已清除（将使用项目内置镜像）",
        style("✓").green().bold()
    );
    Ok(())
}

/// 显示当前配置
fn cmd_config_show() -> Result<()> {
    let builtin = load_builtin_settings()?;
    let user = load_user_settings()?;

    println!();
    println!("{}", style("当前配置:").bold());
    println!("{}", style("─".repeat(40)).dim());

    println!("  {}", style("项目内置:").bold());
    println!(
        "    GitHub 镜像: {}",
        match &builtin.mirror {
            Some(m) => style(m).cyan().to_string(),
            None => style("未设置").dim().to_string(),
        }
    );

    println!();
    println!("  {}", style("用户覆盖:").bold());
    println!(
        "    GitHub 镜像: {}",
        match &user.mirror {
            Some(m) => style(m).yellow().to_string(),
            None => style("未设置（使用内置）").dim().to_string(),
        }
    );

    println!();
    println!(
        "  用户配置文件: {}",
        style(config::user_config_dir().display()).dim()
    );
    println!();
    Ok(())
}
