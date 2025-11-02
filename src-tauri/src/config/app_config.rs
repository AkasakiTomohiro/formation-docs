use crate::utils::context::file::FileSystem;
use crate::utils::{AppError, ConfigMigratable};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::any::Any;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use thiserror::Error;

const APP_CONFIG_LATEST_VERSION: u32 = 1;
const APP_CONFIG_FILE_NAME: &str = "app_config.json";
pub const APP_CONFIG_DIRECTORY_NAME: &str = "formation-docs";

///
/// アプリコンフィグ v1
///

#[derive(Debug, Serialize, Deserialize, Clone)]
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
            initialized_at: "".to_string(),
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

#[derive(Debug, Serialize, Deserialize, Clone)]
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
    pub async fn read(file_system: Arc<dyn FileSystem>) -> Result<AppConfig, AppConfigError> {
        let version = read_config_version(file_system.clone()).await?;
        let config_path = app_config_path(file_system.clone())?;
        match version {
            1 => {
                let config_json = file_system.read_file(&config_path).await?;
                let config_json = serde_json::from_str::<AppConfigV1>(&config_json)?;
                let boxed = Box::new(config_json);
                Ok(boxed.migrate_until_latest())
            }
            _ => {
                let config_json = AppConfig::new();
                config_json.write(file_system).await?;
                Ok(config_json)
            }
        }
    }

    /// app_config.jsonへ書き込む
    pub async fn write(&self, file_system: Arc<dyn FileSystem>) -> Result<(), AppConfigError> {
        let app_config_json = serde_json::to_string::<AppConfig>(self)?;
        let app_config_path = app_config_path(file_system.clone())?;

        // ディレクトリが存在しない場合は作成する
        if let Some(parent_dir) = app_config_path.parent() {
            file_system.create_dir_all(parent_dir).await?;
        }

        file_system
            .write_file(&app_config_path, app_config_json.as_bytes())
            .await?;
        Ok(())
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
fn app_config_path(file_system: Arc<dyn FileSystem>) -> Result<PathBuf, AppConfigError> {
    let dir = file_system
        .config_local_dir()
        .ok_or(AppConfigError::App(AppError::new(
            "Failed to get local config directory",
        )))?;

    return Ok(dir
        .join(APP_CONFIG_DIRECTORY_NAME)
        .join(APP_CONFIG_FILE_NAME));
}

/// app_config.jsonのバージョンを取得
async fn read_config_version(file_system: Arc<dyn FileSystem>) -> Result<u32, AppConfigError> {
    let config_path = app_config_path(file_system.clone())?;
    match config_path.exists() {
        true => {
            let config_json = file_system.read_file(&config_path).await?;
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

#[cfg(test)]
mod tests {

    use crate::utils::ConfigMigratable;

    /// AppConfigV1の初期生成データが正しいことを確認
    #[test]
    fn app_config_v1_new() {
        // ######### 準備 #########

        // ######### 実行 #########
        let config_v1 = super::AppConfigV1::new();

        // ######### 検証 #########
        assert_eq!(config_v1.version, 1);
        assert!(config_v1.workspaces.is_empty());
        assert!(config_v1.initialized == false);
        assert!(config_v1.initialized_at.is_empty());
        assert!(config_v1.is_latest() == false);
    }

    /// AppConfigV1をマイグレーションしたときに、AppConfigに変換できることを確認
    #[test]
    fn app_config_v1_migrate_boxed() {
        // ######### 準備 #########
        let mut config_v1 = super::AppConfigV1::new();
        config_v1
            .workspaces
            .insert("default".to_string(), "Default Workspace".to_string());
        config_v1.initialized_at = "2024-01-01T00:00:00Z".to_string();

        // ######### 実行 #########
        let boxed = Box::new(config_v1.clone()).migrate_boxed();

        // ######### 検証 #########
        let downcasted = boxed.as_any().downcast::<super::AppConfig>();

        // downcastに成功していること
        assert!(downcasted.is_ok());

        // AppConfigV1がAppConfigに変換されていること
        let migrate_config = downcasted.unwrap();
        assert!(migrate_config.is_latest());
        assert_eq!(migrate_config.version, super::APP_CONFIG_LATEST_VERSION);
        assert_eq!(migrate_config.workspaces, config_v1.workspaces);
        assert_eq!(migrate_config.initialized, config_v1.initialized);
        assert_eq!(migrate_config.initialized_at, config_v1.initialized_at);
    }

    // AppConfigの初期生成データが正しいことを確認
    #[test]
    fn app_config_new() {
        // ######### 準備 #########

        // ######### 実行 #########
        let config = super::AppConfig::new();

        // ######### 検証 #########
        assert_eq!(config.version, super::APP_CONFIG_LATEST_VERSION);
        assert!(config.workspaces.is_empty());
        assert!(config.initialized == false);
        assert!(config.initialized_at.is_empty());
        assert!(config.is_latest());
    }

    /// AppConfigをマイグレーションしたときに、同じデータが返ってくることを確認
    #[test]
    fn app_config_migrate_boxed() {
        // ######### 準備 #########
        let mut config = super::AppConfig::new();
        config
            .workspaces
            .insert("default".to_string(), "Default Workspace".to_string());
        config.initialized_at = "2024-01-01T00:00:00Z".to_string();

        // ######### 実行 #########
        let boxed = Box::new(config.clone()).migrate_boxed();

        // ######### 検証 #########
        let downcasted = boxed.as_any().downcast::<super::AppConfig>();

        // downcastに成功していること
        assert!(downcasted.is_ok());

        // AppConfigがAppConfigに変換されていること
        let migrate_config = downcasted.unwrap();
        assert!(migrate_config.is_latest());
        assert_eq!(migrate_config.version, config.version);
        assert_eq!(migrate_config.workspaces, config.workspaces);
        assert_eq!(migrate_config.initialized, config.initialized);
        assert_eq!(migrate_config.initialized_at, config.initialized_at);
    }
}
