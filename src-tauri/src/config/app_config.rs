use crate::utils::AppError;
use chrono::Utc;
use dirs::config_local_dir;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::any::Any;
use std::collections::HashMap;
use std::path::PathBuf;
use thiserror::Error;
use tokio::fs;

const APP_CONFIG_FILE_NAME: &str = "app_config.json";
pub const APP_CONFIG_DIRECTORY_NAME: &str = "formation-docs";

/// 「最新版に変換できる」トレイト
pub trait ConfigMigratable: Any {
    /// 次のバージョンの型（最新版なら Self を返す）
    fn migrate_boxed(self: Box<Self>) -> Box<dyn ConfigMigratable>;
    // where
    //     Self: Sized;

    /// 今が最新版なら true
    fn is_latest(&self) -> bool;

    /// Any型としてダウンキャストできるようにする
    fn as_any(self: Box<Self>) -> Box<dyn Any>;

    /// 最新版になるまでマイグレーション
    fn migrate_until_latest(self: Box<Self>) -> AppConfig
    where
        Self: Sized,
    {
        let mut current: Box<dyn ConfigMigratable> = self;
        while !current.is_latest() {
            current = current.migrate_boxed();
        }
        // 最後は AppConfig に downcast
        *current
            .as_any()
            .downcast::<AppConfig>()
            .expect("must be Conf at latest")
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfigV1 {
    pub version: u32,
    pub workspaces: HashMap<String, String>,
}
impl AppConfigV1 {
    pub fn new() -> Self {
        AppConfigV1 {
            version: 1,
            workspaces: HashMap::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfigV2 {
    pub version: u32,
    pub workspaces: HashMap<String, String>,
    pub initialized: bool,
    pub initialized_at: String,
}
impl AppConfigV2 {
    pub fn new() -> Self {
        AppConfigV2 {
            version: 2,
            workspaces: HashMap::new(),
            initialized: false,
            initialized_at: Utc::now().to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub version: u32,
    pub workspaces: HashMap<String, String>,
    pub initialized: bool,
    pub initialized_at: String,
}
impl AppConfig {
    pub fn new() -> Self {
        let config = AppConfigV2::new();
        AppConfig {
            version: config.version,
            workspaces: config.workspaces,
            initialized: config.initialized,
            initialized_at: config.initialized_at,
        }
    }
}

impl ConfigMigratable for AppConfigV1 {
    fn migrate_boxed(self: Box<Self>) -> Box<dyn ConfigMigratable> {
        Box::new(AppConfigV2 {
            version: 2,
            workspaces: self.workspaces,
            initialized: false,
            initialized_at: Utc::now().to_string(),
        })
    }
    fn is_latest(&self) -> bool {
        false
    }
    fn as_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }
}

impl ConfigMigratable for AppConfigV2 {
    fn migrate_boxed(self: Box<Self>) -> Box<dyn ConfigMigratable> {
        Box::new(AppConfig {
            version: 2,
            workspaces: self.workspaces,
            initialized: self.initialized,
            initialized_at: self.initialized_at,
        })
    }
    fn is_latest(&self) -> bool {
        false
    }
    fn as_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }
}

impl ConfigMigratable for AppConfig {
    fn migrate_boxed(self: Box<Self>) -> Box<dyn ConfigMigratable> {
        self
    }
    fn is_latest(&self) -> bool {
        true
    }
    fn as_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }
}

#[derive(Debug, Error)]
pub enum AppConfigError {
    #[error("app error: {0}")]
    App(#[from] AppError),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}

fn app_config_path() -> Result<PathBuf, AppConfigError> {
    let dir = config_local_dir().ok_or(AppConfigError::App(AppError::new(
        "Failed to get local config directory",
    )))?;
    return Ok(dir
        .join(APP_CONFIG_DIRECTORY_NAME)
        .join(APP_CONFIG_FILE_NAME));
}

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

async fn write_app_config(new_app_config: &AppConfig) -> Result<(), AppConfigError> {
    let app_config_json = serde_json::to_string::<AppConfig>(new_app_config)?;
    let app_config_path = app_config_path()?;
    fs::write(app_config_path, app_config_json).await?;
    return Ok(());
}

pub async fn read_app_config() -> Result<AppConfig, AppConfigError> {
    let version = read_config_version().await?;
    let config_path = app_config_path()?;
    let config_json = fs::read_to_string(&config_path).await?;

    match version {
        1 => {
            let config_json = serde_json::from_str::<AppConfigV1>(&config_json)?;
            let boxed = Box::new(config_json);
            let config: AppConfig = boxed.migrate_until_latest();
            write_app_config(&config).await?;
            Ok(config)
        }
        2 => {
            let config_json = serde_json::from_str::<AppConfigV2>(&config_json)?;
            let boxed = Box::new(config_json);
            Ok(boxed.migrate_until_latest())
        }
        _ => {
            let config_json = AppConfig::new();
            write_app_config(&config_json).await?;
            Ok(config_json)
        }
    }
}
