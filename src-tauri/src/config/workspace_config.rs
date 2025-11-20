use crate::utils::context::file::FileSystem;
use crate::utils::{AppError, ConfigMigratable};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::any::Any;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use thiserror::Error;

const WORKSPACE_FILE_NAME: &str = "workspace.json";
const WORKSPACE_CONFIG_LATEST_VERSION: u32 = 1;

///
/// ワークスペースコンフィグ v1
///

#[derive(Debug, Serialize, Deserialize, Clone)]
struct WorkspaceConfigV1 {
    pub version: u32,
    // stack_id: stack_file_nameのマッピング
    pub stacks: HashMap<String, String>,
    pub name: String,
    pub description: String,
}

impl WorkspaceConfigV1 {
    pub fn new(name: &str) -> Self {
        WorkspaceConfigV1 {
            version: 1,
            stacks: HashMap::new(),
            name: name.to_string().chars().take(256).collect(),
            description: "".to_string().chars().take(256).collect(),
        }
    }
}

impl ConfigMigratable for WorkspaceConfigV1 {
    type Latest = WorkspaceConfig;

    fn migrate_boxed(self: Box<Self>) -> Box<dyn ConfigMigratable<Latest = Self::Latest>> {
        Box::new(WorkspaceConfig {
            version: WORKSPACE_CONFIG_LATEST_VERSION,
            stacks: self.stacks,
            name: self.name,
            description: self.description,
        })
    }
    fn as_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }
}

///
/// ワークスペースコンフィグ 最新バージョン
///

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkspaceConfig {
    pub version: u32,
    pub stacks: HashMap<String, String>,
    pub name: String,
    pub description: String,
}
impl WorkspaceConfig {
    pub fn new(name: &str) -> Self {
        let config: Box<dyn ConfigMigratable<Latest = WorkspaceConfig>> =
            Box::new(WorkspaceConfigV1::new(name));
        *config
            .migrate_boxed()
            .as_any()
            .downcast::<WorkspaceConfig>()
            .unwrap()
    }

    /// workspace.jsonを読み込む
    ///
    /// versionが最新でない場合はマイグレーションして最新版に変換し、保存する
    /// - 戻り値: 読み込んだWorkspaceConfig
    pub async fn read(
        file_system: Arc<dyn FileSystem>,
        workspace_directory: &str,
    ) -> Result<WorkspaceConfig, WorkspaceConfigError> {
        let version = read_config_version(file_system.clone(), workspace_directory).await?;
        let config_path = workspace_config_path(workspace_directory);

        match version {
            1 => {
                let config_json = file_system.read_file(&config_path).await?;
                let config_json = serde_json::from_str::<WorkspaceConfigV1>(&config_json)?;
                let boxed = Box::new(config_json);
                Ok(boxed.migrate_until_latest())
            }
            _ => {
                let name = Path::new(workspace_directory)
                    .file_name()
                    .and_then(|f| f.to_str());
                if name.is_none() {
                    return Err(WorkspaceConfigError::App(AppError::new(
                        "Invalid directory",
                    )));
                }
                let name = name.unwrap();
                let config_json = WorkspaceConfig::new(name);
                config_json
                    .write(file_system.clone(), workspace_directory)
                    .await?;
                Ok(config_json)
            }
        }
    }

    /// workspace.jsonへ書き込む
    ///
    /// - `workspace_directory`: ワークスペースディレクトリ
    pub async fn write(
        &self,
        file_system: Arc<dyn FileSystem>,
        workspace_directory: &str,
    ) -> Result<(), WorkspaceConfigError> {
        let workspace_config_json = serde_json::to_string::<WorkspaceConfig>(self)?;
        let workspace_config_path = workspace_config_path(workspace_directory);
        file_system
            .write_file(&workspace_config_path, workspace_config_json.as_bytes())
            .await?;
        return Ok(());
    }
}

impl ConfigMigratable for WorkspaceConfig {
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
pub enum WorkspaceConfigError {
    #[error("app error: {0}")]
    App(#[from] AppError),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}

/// workspace.jsonのパスを取得
fn workspace_config_path(workspace_directory: &str) -> PathBuf {
    return PathBuf::from(workspace_directory).join(WORKSPACE_FILE_NAME);
}

/// workspace.jsonのバージョンを取得
async fn read_config_version(
    file_system: Arc<dyn FileSystem>,
    workspace_directory: &str,
) -> Result<u32, WorkspaceConfigError> {
    let config_path = workspace_config_path(workspace_directory);
    match file_system.path_exists(&config_path) {
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
#[coverage(off)]
mod workspace_config_v1 {
    use super::*;

    /// WorkspaceConfigV1::new で構造体が正しく作成されること
    #[test]
    fn workspace_config_v1_new() {
        // ######### 準備 #########

        // ######### 実行 #########
        let result = WorkspaceConfigV1::new("test");

        // ######### 検証 #########
        assert_eq!(result.version, 1);
        assert_eq!(result.stacks, HashMap::new());
        assert_eq!(result.name, "test");
        assert_eq!(result.description, "");
    }

    /// WorkspaceConfigV1::migrate_boxed で最新バージョンにマイグレーションされること
    #[test]
    fn workspace_config_v1_migrate_boxed() {
        // ######### 準備 #########
        let mut config = WorkspaceConfigV1::new("test");
        config
            .stacks
            .insert("stack1".to_string(), "stack1.json".to_string());
        config.description = "This is a test workspace".to_string();

        // ######### 実行 #########
        let migrated = Box::new(config.clone()).migrate_boxed();

        // ######### 検証 #########
        assert!(migrated.is_latest());

        let downcast = migrated.as_any().downcast::<WorkspaceConfig>().unwrap();
        assert_eq!(downcast.version, WORKSPACE_CONFIG_LATEST_VERSION);
        assert_eq!(downcast.stacks, config.stacks);
        assert_eq!(downcast.name, config.name);
        assert_eq!(downcast.description, config.description);
    }

    /// WorkspaceConfigV1::as_any を使ってダウンキャストできること
    #[test]
    fn workspace_config_v1_as_any() {
        // ######### 準備 #########
        let config = WorkspaceConfigV1::new("test");

        // ######### 実行 #########
        let any_box = Box::new(config).as_any();
        let downcast = any_box.downcast::<WorkspaceConfigV1>();

        // ######### 検証 #########
        assert!(downcast.is_ok());
    }
}
