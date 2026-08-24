use crate::model::StoredConfig;
use std::{fs, path::PathBuf};
use tauri::{AppHandle, Manager};

const KEYRING_SERVICE: &str = "io.github.lingyenbird.sub2poolcard";
const KEYRING_ACCOUNT: &str = "sub2pool-admin-api-token";
const SETTINGS_FILE: &str = "settings.json";

fn settings_path(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_config_dir()
        .map(|path| path.join(SETTINGS_FILE))
        .map_err(|error| format!("无法定位配置目录：{error}"))
}

pub fn load_config(app: &AppHandle) -> Result<StoredConfig, String> {
    let path = settings_path(app)?;
    if !path.exists() {
        return Ok(StoredConfig::default());
    }
    let content =
        fs::read_to_string(&path).map_err(|error| format!("读取本地配置失败：{error}"))?;
    serde_json::from_str(&content).map_err(|error| format!("本地配置格式无效：{error}"))
}

pub fn save_config(app: &AppHandle, config: &StoredConfig) -> Result<(), String> {
    let path = settings_path(app)?;
    let parent = path
        .parent()
        .ok_or_else(|| "无法定位配置目录".to_string())?;
    fs::create_dir_all(parent).map_err(|error| format!("创建配置目录失败：{error}"))?;
    let content = serde_json::to_vec_pretty(config)
        .map_err(|error| format!("序列化本地配置失败：{error}"))?;
    fs::write(path, content).map_err(|error| format!("保存本地配置失败：{error}"))
}

fn token_entry() -> Result<keyring::Entry, String> {
    keyring::Entry::new(KEYRING_SERVICE, KEYRING_ACCOUNT)
        .map_err(|error| format!("无法访问系统凭据库：{error}"))
}

pub fn read_token() -> Result<Option<String>, String> {
    match token_entry()?.get_password() {
        Ok(token) if token.trim().is_empty() => Ok(None),
        Ok(token) => Ok(Some(token)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(error) => Err(format!("读取 Sub2Pool API Key 失败：{error}")),
    }
}

pub fn save_token(token: &str) -> Result<(), String> {
    token_entry()?
        .set_password(token)
        .map_err(|error| format!("保存 Sub2Pool API Key 失败：{error}"))
}
