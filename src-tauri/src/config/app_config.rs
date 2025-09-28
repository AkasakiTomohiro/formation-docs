use crate::utils::{AppError, ConfigMigratable};
use chrono::Utc;
use dirs::config_local_dir;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::any::Any;
use std::collections::HashMap;
use std::path::PathBuf;
use thiserror::Error;
use tokio::fs;

const APP_CONFIG_LATEST_VERSION: u32 = 1;
const APP_CONFIG_FILE_NAME: &str = "app_config.json";
pub const APP_CONFIG_DIRECTORY_NAME: &str = "formation-docs";

///
/// アプリコンフィグ v1
///

#[derive(Debug, Serialize, Deserialize)]
struct AppConfigV1 {
    pub version: u32,
    pub workspaces: HashMap<String, String>,
    pub initialized: bool,
    pub initialized_at: String,
}

impl AppConfigV1 {
    pub fn new() -> Self {
        AppConfigV1 {
            version: 1,
            workspaces: HashMap::new(),
            initialized: false,
            initialized_at: Utc::now().to_string(),
        }
    }
}

impl ConfigMigratable for AppConfigV1 {
    type Latest = AppConfig;

    fn migrate_boxed(self: Box<Self>) -> Box<dyn ConfigMigratable<Latest = Self::Latest>> {
        Box::new(AppConfig {
            version: APP_CONFIG_LATEST_VERSION,
            workspaces: self.workspaces,
            initialized: self.initialized,
            initialized_at: self.initialized_at,
        })
    }
    fn as_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }
}

///
/// アプリコンフィグ 最新バージョン
///

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub version: u32,
    pub workspaces: HashMap<String, String>,
    pub initialized: bool,
    pub initialized_at: String,
}
impl AppConfig {
    pub fn new() -> Self {
        let config: Box<dyn ConfigMigratable<Latest = AppConfig>> = Box::new(AppConfigV1::new());
        *config
            .migrate_boxed()
            .as_any()
            .downcast::<AppConfig>()
            .unwrap()
    }

    /// app_config.jsonを読み込む
    ///
    /// versionが最新でない場合はマイグレーションして最新版に変換し、保存する
    /// - 戻り値: 読み込んだAppConfig
    pub async fn read() -> Result<AppConfig, AppConfigError> {
        let version = read_config_version().await?;
        let config_path = app_config_path()?;
        match version {
            1 => {
                let config_json = fs::read_to_string(&config_path).await?;
                let config_json = serde_json::from_str::<AppConfigV1>(&config_json)?;
                let boxed = Box::new(config_json);
                Ok(boxed.migrate_until_latest())
            }
            _ => {
                let config_json = AppConfig::new();
                config_json.write().await?;
                Ok(config_json)
            }
        }
    }

    /// app_config.jsonへ書き込む
    pub async fn write(&self) -> Result<(), AppConfigError> {
        let app_config_json = serde_json::to_string::<AppConfig>(self)?;
        let app_config_path = app_config_path()?;
        fs::write(app_config_path, app_config_json).await?;
        return Ok(());
    }
}

impl ConfigMigratable for AppConfig {
    type Latest = Self;

    fn migrate_boxed(self: Box<Self>) -> Box<dyn ConfigMigratable<Latest = Self::Latest>> {
        self
    }
    fn is_latest(&self) -> bool {
        true
    }
    fn as_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }
}

///
/// アプリコンフィグ エラー
///

#[derive(Debug, Error)]
pub enum AppConfigError {
    #[error("app error: {0}")]
    App(#[from] AppError),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}

/// app_config.jsonのパスを取得
fn app_config_path() -> Result<PathBuf, AppConfigError> {
    let dir = config_local_dir().ok_or(AppConfigError::App(AppError::new(
        "Failed to get local config directory",
    )))?;
    return Ok(dir
        .join(APP_CONFIG_DIRECTORY_NAME)
        .join(APP_CONFIG_FILE_NAME));
}

/// app_config.jsonのバージョンを取得
async fn read_config_version() -> Result<u32, AppConfigError> {
    let config_path = app_config_path()?;
    match config_path.exists() {
        true => {
            let config_json = fs::read_to_string(&config_path).await?;
            let config_json: Value = serde_json::from_str(&config_json)?;
            let version = config_json
                .get("version")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as u32;
            Ok(version)
        }
        false => Ok(0),
    }
}
