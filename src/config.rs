use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// 单个模板定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    pub name: String,
    pub description: String,
    pub repository: String,
    #[serde(default = "default_branch")]
    pub branch: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

fn default_branch() -> String {
    "main".to_string()
}

/// 项目级设置（嵌入 templates.toml，随项目分发）
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Settings {
    /// GitHub 镜像前缀，如 "https://ghfast.top/"
    #[serde(default)]
    pub mirror: Option<String>,
}

/// TOML 配置文件结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateConfig {
    #[serde(default)]
    pub settings: Settings,
    #[serde(default)]
    pub templates: Vec<Template>,
}

/// 内置默认模板（编译时嵌入）
const BUILTIN_TEMPLATES: &str = include_str!("../templates.toml");

/// 获取用户配置目录路径
pub fn user_config_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".baiban")
}

/// 获取用户配置文件路径
pub fn user_config_path() -> PathBuf {
    user_config_dir().join("templates.toml")
}

/// 加载内置模板
fn load_builtin_templates() -> Result<Vec<Template>> {
    let config: TemplateConfig =
        toml::from_str(BUILTIN_TEMPLATES).context("解析内置模板配置失败")?;
    Ok(config.templates)
}

/// 加载用户自定义模板
pub fn load_user_templates() -> Result<Vec<Template>> {
    let path = user_config_path();
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(&path).context("读取用户配置文件失败")?;
    let config: TemplateConfig = toml::from_str(&content).context("解析用户模板配置失败")?;
    Ok(config.templates)
}

/// 获取所有模板（用户模板覆盖同名内置模板）
pub fn load_all_templates() -> Result<Vec<Template>> {
    let mut map: HashMap<String, Template> = HashMap::new();

    // 先加载内置模板
    for t in load_builtin_templates()? {
        map.insert(t.name.clone(), t);
    }

    // 用户模板覆盖同名内置模板
    for t in load_user_templates()? {
        map.insert(t.name.clone(), t);
    }

    // 按名称排序
    let mut templates: Vec<Template> = map.into_values().collect();
    templates.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(templates)
}

/// 根据名称查找模板
pub fn find_template(name: &str) -> Result<Option<Template>> {
    let templates = load_all_templates()?;
    Ok(templates.into_iter().find(|t| t.name == name))
}

/// 保存用户模板配置
pub fn save_user_templates(templates: &[Template]) -> Result<()> {
    let dir = user_config_dir();
    fs::create_dir_all(&dir).context("创建配置目录失败")?;

    let config = TemplateConfig {
        settings: Settings::default(),
        templates: templates.to_vec(),
    };
    let content = toml::to_string_pretty(&config).context("序列化模板配置失败")?;
    fs::write(user_config_path(), content).context("写入用户配置文件失败")?;
    Ok(())
}

/// 添加用户模板
pub fn add_user_template(template: Template) -> Result<()> {
    let mut templates = load_user_templates()?;

    // 检查是否已存在
    if let Some(existing) = templates.iter_mut().find(|t| t.name == template.name) {
        *existing = template;
    } else {
        templates.push(template);
    }

    save_user_templates(&templates)
}

/// 移除用户模板
pub fn remove_user_template(name: &str) -> Result<bool> {
    let mut templates = load_user_templates()?;
    let len_before = templates.len();
    templates.retain(|t| t.name != name);

    if templates.len() < len_before {
        save_user_templates(&templates)?;
        Ok(true)
    } else {
        Ok(false)
    }
}

// ─── 配置管理（镜像等） ───

/// 获取用户设置文件路径（用于覆盖项目内置配置）
fn user_settings_path() -> PathBuf {
    user_config_dir().join("settings.toml")
}

/// 加载项目内置设置（编译时嵌入）
pub fn load_builtin_settings() -> Result<Settings> {
    let config: TemplateConfig =
        toml::from_str(BUILTIN_TEMPLATES).context("解析内置配置失败")?;
    Ok(config.settings)
}

/// 加载用户设置（覆盖内置配置）
pub fn load_user_settings() -> Result<Settings> {
    let path = user_settings_path();
    if !path.exists() {
        return Ok(Settings::default());
    }
    let content = fs::read_to_string(&path).context("读取用户设置失败")?;
    let settings: Settings = toml::from_str(&content).context("解析用户设置失败")?;
    Ok(settings)
}

/// 保存用户设置
pub fn save_user_settings(settings: &Settings) -> Result<()> {
    let dir = user_config_dir();
    fs::create_dir_all(&dir).context("创建配置目录失败")?;

    let content = toml::to_string_pretty(settings).context("序列化设置失败")?;
    fs::write(user_settings_path(), content).context("写入用户设置失败")?;
    Ok(())
}

/// 获取镜像地址（优先级：CLI 参数 > 用户设置 > 项目内置）
pub fn get_mirror(cli_mirror: &Option<String>) -> Option<String> {
    // 1. CLI 参数优先
    if let Some(m) = cli_mirror {
        return Some(m.clone());
    }

    // 2. 用户设置覆盖
    if let Ok(user) = load_user_settings() {
        if user.mirror.is_some() {
            return user.mirror;
        }
    }

    // 3. 项目内置设置
    load_builtin_settings().ok().and_then(|s| s.mirror)
}

/// 将 GitHub URL 转换为镜像 URL
pub fn apply_mirror(url: &str, mirror: &Option<String>) -> String {
    match mirror {
        Some(prefix) if url.starts_with("https://github.com/") => {
            let prefix = prefix.trim_end_matches('/');
            format!("{}/{}", prefix, url)
        }
        _ => url.to_string(),
    }
}
