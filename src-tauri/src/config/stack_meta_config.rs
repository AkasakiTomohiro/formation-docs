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

///
/// スタックメタコンフィグ v1
///

#[derive(Debug, Serialize, Deserialize)]
struct StackMetaConfigV1 {
    pub version: u32,
    pub name: String,
    pub description: String,
    pub reasons: HashMap<String, HashMap<String, String>>,
}

impl StackMetaConfigV1 {
    pub fn new(name: &str) -> Self {
        StackMetaConfigV1 {
            version: 1,
            name: name.to_string(),
            description: "".to_string(),
            reasons: HashMap::new(),
        }
    }
}

impl ConfigMigratable for StackMetaConfigV1 {
    type Latest = StackMetaConfig;

    fn migrate_boxed(self: Box<Self>) -> Box<dyn ConfigMigratable<Latest = Self::Latest>> {
        Box::new(StackMetaConfig {
            version: STACK_META_CONFIG_LATEST_VERSION,
            name: self.name,
            description: self.description,
            reasons: self.reasons,
        })
    }
    fn as_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }
}

///
/// スタックメタコンフィグ 最新バージョン
///

#[derive(Debug, Serialize, Deserialize)]
pub struct StackMetaConfig {
    pub version: u32,
    pub name: String,
    pub description: String,
    pub reasons: HashMap<String, HashMap<String, String>>,
}

impl StackMetaConfig {
    pub fn new(name: &str) -> Self {
        let config: Box<dyn ConfigMigratable<Latest = StackMetaConfig>> =
            Box::new(StackMetaConfigV1::new(name));
        *config
            .migrate_boxed()
            .as_any()
            .downcast::<StackMetaConfig>()
            .expect("must be StackMeta at latest")
    }

    /// ${スタック名}.meta.jsonを読み込む
    ///
    /// versionが最新でない場合はマイグレーションして最新版に変換し、保存する
    /// - 戻り値: 読み込んだStackMetaConfig
    ///
    /// - `workspace_directory`: ワークスペースディレクトリ
    /// - `stack_name`: スタック名
    pub async fn read(
        file_system: Arc<dyn FileSystem>,
        workspace_directory: &str,
        stack_name: &str,
    ) -> Result<StackMetaConfig, StackMetaConfigError> {
        let version =
            read_config_version(file_system.clone(), workspace_directory, stack_name).await?;
        let config_path = stack_meta_config_path(workspace_directory, &stack_name)?;

        match version {
            1 => {
                let config_json = file_system.read_file(&config_path).await?;
                let config_json = serde_json::from_str::<StackMetaConfigV1>(&config_json)?;
                let boxed = Box::new(config_json);
                Ok(boxed.migrate_until_latest())
            }
            _ => {
                let config_json = StackMetaConfig::new(stack_name);
                config_json
                    .write(file_system.clone(), workspace_directory, stack_name)
                    .await?;
                Ok(config_json)
            }
        }
    }

    /// ${スタック名}.meta.jsonへ書き込む
    ///
    /// - `workspace_directory`: ワークスペースディレクトリ
    /// - `stack_name`: スタック名
    pub async fn write(
        &self,
        file_system: Arc<dyn FileSystem>,
        workspace_directory: &str,
        stack_name: &str,
    ) -> Result<(), StackMetaConfigError> {
        let config_path = stack_meta_config_path(workspace_directory, stack_name)?;
        let config_json = serde_json::to_string(self)?;
        file_system
            .write_file(&config_path, config_json.as_bytes())
            .await?;
        return Ok(());
    }
}

impl ConfigMigratable for StackMetaConfig {
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
pub enum StackMetaConfigError {
    #[error("app error: {0}")]
    App(#[from] AppError),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}

/// ${スタック名}.meta.jsonのパスを取得
fn stack_meta_config_path(
    workspace_directory: &str,
    stack_name: &str,
) -> Result<PathBuf, StackMetaConfigError> {
    let meta_path = PathBuf::from(workspace_directory)
        .join(stack_name)
        .with_extension("meta.json");
    let meta_path = PathBuf::from(&meta_path);
    return Ok(meta_path);
}

/// ${スタック名}.meta.jsonのバージョンを取得
async fn read_config_version(
    file_system: Arc<dyn FileSystem>,
    workspace_directory: &str,
    stack_name: &str,
) -> Result<u32, StackMetaConfigError> {
    let meta_path = stack_meta_config_path(workspace_directory, stack_name)?;
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
