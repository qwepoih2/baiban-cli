use anyhow::{bail, Context, Result};
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::config::{apply_mirror, Template};

/// 下载模板到目标目录
pub fn clone_template(template: &Template, target_dir: &Path, mirror: &Option<String>) -> Result<()> {
    // 检查目标目录是否已存在且非空
    if target_dir.exists() {
        let entries: Vec<_> = fs::read_dir(target_dir)
            .context("读取目标目录失败")?
            .collect();
        if !entries.is_empty() {
            bail!(
                "目录 {} 已存在且不为空，请选择其他名称",
                target_dir.display()
            );
        }
    }

    // 检查 git 是否已安装
    let git_check = Command::new("git").arg("--version").output();
    if git_check.is_err() {
        bail!("未找到 git，请先安装 git: https://git-scm.com/downloads");
    }

    // 应用镜像地址
    let repo_url = apply_mirror(&template.repository, mirror);

    if mirror.is_some() {
        println!("  使用镜像: {}", repo_url);
    }

    // 尝试 1: 指定分支 clone
    let mut cmd = Command::new("git");
    cmd.arg("clone")
        .arg("--depth")
        .arg("1")
        .arg("--branch")
        .arg(&template.branch)
        .arg(&repo_url)
        .arg(target_dir);

    let status = cmd.status().context("执行 git clone 失败")?;

    if !status.success() {
        // 清理可能残留的目录
        if target_dir.exists() {
            fs::remove_dir_all(target_dir).ok();
        }

        // 尝试 2: 不指定分支（使用默认分支）
        let mut cmd2 = Command::new("git");
        cmd2.arg("clone")
            .arg("--depth")
            .arg("1")
            .arg(&repo_url)
            .arg(target_dir);

        let status2 = cmd2.status().context("执行 git clone 失败")?;
        if !status2.success() {
            bail!(
                "git clone 失败，请检查仓库地址和网络连接。\n\
                 提示: 如果无法访问 GitHub，可使用镜像：\n\
                 baiban create --mirror https://ghfast.top/\n\
                 或永久设置: baiban config set-mirror https://ghfast.top/"
            );
        }
    }

    Ok(())
}

/// 后处理：清理 .git 并重新初始化
pub fn post_process(target_dir: &Path, project_name: &str) -> Result<()> {
    // 1. 删除 .git 目录
    let git_dir = target_dir.join(".git");
    if git_dir.exists() {
        fs::remove_dir_all(&git_dir).context("删除 .git 目录失败")?;
    }

    // 2. 替换 package.json 中的 name 字段
    let package_json_path = target_dir.join("package.json");
    if package_json_path.exists() {
        let content = fs::read_to_string(&package_json_path).context("读取 package.json 失败")?;
        let mut pkg: serde_json::Value =
            serde_json::from_str(&content).context("解析 package.json 失败")?;

        if let Some(obj) = pkg.as_object_mut() {
            obj.insert(
                "name".to_string(),
                serde_json::Value::String(project_name.to_string()),
            );
        }

        let updated =
            serde_json::to_string_pretty(&pkg).context("序列化 package.json 失败")?;
        fs::write(&package_json_path, updated).context("写入 package.json 失败")?;
    }

    // 3. 初始化新的 git 仓库
    let init_status = Command::new("git")
        .arg("init")
        .current_dir(target_dir)
        .status()
        .context("执行 git init 失败")?;

    if !init_status.success() {
        eprintln!("警告: git init 失败，跳过初始化");
    }

    // 4. 创建初始 commit
    let add_status = Command::new("git")
        .args(["add", "."])
        .current_dir(target_dir)
        .status();

    if let Ok(status) = add_status {
        if status.success() {
            let _ = Command::new("git")
                .args(["commit", "-m", "init from baiban template"])
                .current_dir(target_dir)
                .status();
        }
    }

    Ok(())
}
