use anyhow::{bail, Result};
use console::style;
use dialoguer::{Confirm, Input, Select};

use crate::config::Template;

/// 交互式获取项目名称
pub fn prompt_project_name(default: Option<&str>) -> Result<String> {
    let mut input = Input::<String>::new();
    input = input.with_prompt(format!("{} 请输入项目名称", style("?").cyan().bold()));

    if let Some(d) = default {
        input = input.default(d.to_string());
    }

    let name = input.interact_text()?;

    if name.trim().is_empty() {
        bail!("项目名称不能为空");
    }

    Ok(name.trim().to_string())
}

/// 交互式选择模板
pub fn prompt_select_template(templates: &[Template]) -> Result<&Template> {
    if templates.is_empty() {
        bail!("没有可用的模板，请先通过 `baiban template add` 添加模板");
    }

    let items: Vec<String> = templates
        .iter()
        .map(|t| {
            if t.tags.is_empty() {
                format!("{} - {}", style(&t.name).green(), t.description)
            } else {
                format!(
                    "{} - {} [{}]",
                    style(&t.name).green(),
                    t.description,
                    t.tags.join(", ")
                )
            }
        })
        .collect();

    let selection = Select::new()
        .with_prompt(format!("{} 请选择模板", style("?").cyan().bold()))
        .items(&items)
        .default(0)
        .interact()?;

    Ok(&templates[selection])
}

/// 确认创建
pub fn prompt_confirm(project_name: &str) -> Result<bool> {
    let confirmed = Confirm::new()
        .with_prompt(format!(
            "{} 确认创建项目 {} ？",
            style("?").cyan().bold(),
            style(project_name).yellow().bold()
        ))
        .default(true)
        .interact()?;

    Ok(confirmed)
}

/// 交互式添加模板 - 获取模板名称
pub fn prompt_template_name() -> Result<String> {
    let name = Input::<String>::new()
        .with_prompt(format!("{} 模板名称", style("?").cyan().bold()))
        .interact_text()?;

    if name.trim().is_empty() {
        bail!("模板名称不能为空");
    }

    Ok(name.trim().to_string())
}

/// 交互式添加模板 - 获取描述
pub fn prompt_template_description() -> Result<String> {
    let desc = Input::<String>::new()
        .with_prompt(format!("{} 模板描述", style("?").cyan().bold()))
        .interact_text()?;

    Ok(desc.trim().to_string())
}

/// 交互式添加模板 - 获取仓库地址
pub fn prompt_template_repository() -> Result<String> {
    let repo = Input::<String>::new()
        .with_prompt(format!("{} Git 仓库地址", style("?").cyan().bold()))
        .interact_text()?;

    if repo.trim().is_empty() {
        bail!("仓库地址不能为空");
    }

    Ok(repo.trim().to_string())
}

/// 交互式添加模板 - 获取分支
pub fn prompt_template_branch() -> Result<String> {
    let branch: String = Input::new()
        .with_prompt(format!("{} 分支名称", style("?").cyan().bold()))
        .default("main".to_string())
        .interact_text()?;

    Ok(branch.trim().to_string())
}

/// 交互式选择要移除的模板
pub fn prompt_select_remove(templates: &[Template]) -> Result<usize> {
    if templates.is_empty() {
        bail!("没有可移除的用户模板");
    }

    let items: Vec<String> = templates
        .iter()
        .map(|t| format!("{} - {}", style(&t.name).red(), t.description))
        .collect();

    let selection = Select::new()
        .with_prompt(format!("{} 请选择要移除的模板", style("?").cyan().bold()))
        .items(&items)
        .default(0)
        .interact()?;

    Ok(selection)
}

/// 打印成功信息
pub fn print_success(project_name: &str, target_path: &str) {
    println!();
    println!(
        "{} 项目已创建: {}",
        style("✓").green().bold(),
        style(target_path).cyan()
    );
    println!();
    println!("后续步骤:");
    println!("  {}", style(format!("cd {}", project_name)).cyan());
    println!("  {}", style("npm install").cyan());
    println!("  {}", style("npm run dev").cyan());
    println!();
}

/// 打印模板列表
pub fn print_templates(templates: &[Template]) {
    if templates.is_empty() {
        println!("没有可用的模板");
        return;
    }

    println!();
    println!("{} 可用模板:", style("📋").bold());
    println!("{}", style("─".repeat(50)).dim());

    for t in templates {
        println!(
            "  {} {}",
            style(&t.name).green().bold(),
            style(&t.description).dim()
        );
        println!(
            "    {} {}",
            style("仓库:").dim(),
            style(&t.repository).cyan()
        );
        if !t.tags.is_empty() {
            println!(
                "    {} {}",
                style("标签:").dim(),
                t.tags.join(", ")
            );
        }
        println!();
    }
}
