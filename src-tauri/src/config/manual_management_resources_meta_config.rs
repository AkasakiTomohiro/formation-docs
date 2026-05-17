use crate::utils::context::file::FileSystem;
use crate::utils::{AppError, ConfigMigratable};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::any::Any;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use thiserror::Error;

const STACK_META_CONFIG_LATEST_VERSION: u32 = 1;

/// 手動管理リソースのmetaファイル名
const MANUAL_MANAGEMENT_RESOURCES_META_FILE: &str = "manual_management_resources.meta.json";

///
/// 手動管理リソースメタコンフィグ v1
///

#[derive(Debug, Serialize, Deserialize)]
pub struct ManualManagementResourcesMetaConfigResource {
    pub description: String,
    pub reasons: HashMap<String, String>,
}
#[derive(Debug, Serialize, Deserialize)]
struct ManualManagementResourcesMetaConfigV1 {
    pub version: u32,
    pub description: String,
    pub resources: HashMap<String, ManualManagementResourcesMetaConfigResource>,
}

impl ManualManagementResourcesMetaConfigV1 {
    pub fn new() -> Self {
        ManualManagementResourcesMetaConfigV1 {
            version: 1,
            description: "".to_string(),
            resources: HashMap::new(),
        }
    }
}

impl ConfigMigratable for ManualManagementResourcesMetaConfigV1 {
    type Latest = ManualManagementResourcesMetaConfig;

    fn migrate_boxed(self: Box<Self>) -> Box<dyn ConfigMigratable<Latest = Self::Latest>> {
        Box::new(ManualManagementResourcesMetaConfig {
            version: STACK_META_CONFIG_LATEST_VERSION,
            description: self.description,
            resources: self.resources,
        })
    }
    fn as_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }
}

///
/// 手動管理リソースメタコンフィグ 最新バージョン
///

#[derive(Debug, Serialize, Deserialize)]
pub struct ManualManagementResourcesMetaConfig {
    pub version: u32,
    pub description: String,
    pub resources: HashMap<String, ManualManagementResourcesMetaConfigResource>,
}

impl ManualManagementResourcesMetaConfig {
    pub fn new() -> Self {
        let config: Box<dyn ConfigMigratable<Latest = ManualManagementResourcesMetaConfig>> =
            Box::new(ManualManagementResourcesMetaConfigV1::new());
        *config
            .migrate_boxed()
            .as_any()
            .downcast::<ManualManagementResourcesMetaConfig>()
            .expect("must be ManualManagementResourcesMetaConfig at latest")
    }

    /// ${スタック名}.meta.jsonを読み込む
    ///
    /// versionが最新でない場合はマイグレーションして最新版に変換し、保存する
    /// - 戻り値: 読み込んだManualManagementResourcesMetaConfig
    ///
    /// - `workspace_directory`: ワークスペースディレクトリ
    /// - `stack_name`: スタック名
    pub async fn read(
        file_system: Arc<dyn FileSystem>,
        workspace_directory: &str,
    ) -> Result<ManualManagementResourcesMetaConfig, ManualManagementResourcesMetaConfigError> {
        let version = read_config_version(file_system.clone(), workspace_directory).await?;
        let config_path = manual_management_resources_meta_config_path(workspace_directory);

        match version {
            1 => {
                let config_json = file_system.read_file(&config_path).await?;
                let config_json =
                    serde_json::from_str::<ManualManagementResourcesMetaConfigV1>(&config_json)?;
                let boxed = Box::new(config_json);
                Ok(boxed.migrate_until_latest())
            }
            _ => {
                let config_json = ManualManagementResourcesMetaConfig::new();
                Ok(ManualManagementResourcesMetaConfig::write(
                    config_json,
                    file_system.clone(),
                    workspace_directory,
                )
                .await?)
            }
        }
    }

    /// ${スタック名}.meta.jsonへ書き込む
    ///
    /// - `workspace_directory`: ワークスペースディレクトリ
    /// - `stack_name`: スタック名
    pub async fn write(
        config: ManualManagementResourcesMetaConfig,
        file_system: Arc<dyn FileSystem>,
        workspace_directory: &str,
    ) -> Result<ManualManagementResourcesMetaConfig, ManualManagementResourcesMetaConfigError> {
        let config_path = manual_management_resources_meta_config_path(workspace_directory);
        let config_json = serde_json::to_string(&config)?;
        file_system
            .write_file(&config_path, config_json.as_bytes())
            .await?;
        return Ok(config);
    }
}

impl ConfigMigratable for ManualManagementResourcesMetaConfig {
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

#[derive(Debug, Error)]
pub enum ManualManagementResourcesMetaConfigError {
    #[error("app error: {0}")]
    App(#[from] AppError),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}

/// 手動管理リソースメタコンフィグのパスを取得
fn manual_management_resources_meta_config_path(workspace_directory: &str) -> PathBuf {
    let meta_path = PathBuf::from(workspace_directory).join(MANUAL_MANAGEMENT_RESOURCES_META_FILE);
    return meta_path;
}

/// 手動管理リソースメタコンフィグのバージョンを取得
async fn read_config_version(
    file_system: Arc<dyn FileSystem>,
    workspace_directory: &str,
) -> Result<u32, ManualManagementResourcesMetaConfigError> {
    let meta_path = manual_management_resources_meta_config_path(workspace_directory);
    match file_system.path_exists(&meta_path) {
        true => {
            let config_json = file_system.read_file(&meta_path).await?;
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
#[coverage(off)]
/// ManualManagementResourcesMetaConfigV1構造体用のテスト
mod manual_management_resources_meta_config_v1_tests {
    use super::*;

    /// ManualManagementResourcesMetaConfigV1の初期生成データが正しいことを確認
    #[test]
    fn stack_meta_config_v1_new() {
        // ######### 実行 #########
        let config_v1 = ManualManagementResourcesMetaConfigV1::new();

        // ######### 検証 #########
        assert_eq!(config_v1.version, 1);
        assert_eq!(config_v1.description, "".to_string());
        assert_eq!(config_v1.resources.is_empty(), true);
    }

    /// ManualManagementResourcesMetaConfigV1をマイグレーションしたときに、ManualManagementResourcesMetaConfigに変換できることを確認
    #[test]
    fn stack_meta_config_v1_migrate() {
        // ######### 準備 #########
        let config_v1 = ManualManagementResourcesMetaConfigV1::new();

        // ######### 実行 #########
        let boxed: Box<dyn ConfigMigratable<Latest = ManualManagementResourcesMetaConfig>> =
            Box::new(config_v1);
        let migrated = boxed.migrate_boxed();

        // V1 -> ManualManagementResourcesMetaConfigへマイグレーション
        let config_latest = migrated
            .as_any()
            .downcast::<ManualManagementResourcesMetaConfig>()
            .expect("must be ManualManagementResourcesMetaConfig at latest");

        // ######### 検証 #########
        assert_eq!(config_latest.version, STACK_META_CONFIG_LATEST_VERSION);
        assert_eq!(config_latest.description, "".to_string());
        assert_eq!(config_latest.resources.is_empty(), true);
    }

    #[test]
    fn stack_meta_config_v1_as_any() {
        // ######### 準備 #########
        let config_v1 = ManualManagementResourcesMetaConfigV1::new();

        // ######### 実行 #########
        let boxed: Box<dyn ConfigMigratable<Latest = ManualManagementResourcesMetaConfig>> =
            Box::new(config_v1);
        let any_boxed = boxed.as_any();
        let downcasted = any_boxed
            .downcast::<ManualManagementResourcesMetaConfigV1>()
            .expect("must be ManualManagementResourcesMetaConfigV1");

        // ######### 検証 #########
        assert_eq!(downcasted.version, 1);
        assert_eq!(downcasted.description, "".to_string());
        assert_eq!(downcasted.resources.is_empty(), true);
    }
}

#[cfg(test)]
#[coverage(off)]
/// ManualManagementResourcesMetaConfig構造体用のテスト
mod manual_management_resources_meta_config_tests {
    use super::*;

    #[test]
    fn manual_management_resources_meta_config_new() {
        // ######### 準備 #########

        // ######### 実行 #########
        let config_latest = ManualManagementResourcesMetaConfig::new();

        // ######### 検証 #########
        assert_eq!(config_latest.version, STACK_META_CONFIG_LATEST_VERSION);
        assert_eq!(config_latest.description, "".to_string());
        assert_eq!(config_latest.resources.is_empty(), true);
    }
}
