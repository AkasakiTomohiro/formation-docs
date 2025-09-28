use crate::utils::{AppError, ConfigMigratable};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::any::Any;
use std::collections::HashMap;
use std::path::PathBuf;
use thiserror::Error;
use tokio::fs;

///
/// スタックメタコンフィグ v1
///

#[derive(Debug, Serialize, Deserialize)]
struct StackMetaConfigV1 {
    pub version: u32,
    pub name: String,
    pub description: String,
}

impl ConfigMigratable for StackMetaConfigV1 {
    type Latest = StackMetaConfig;

    fn migrate_boxed(self: Box<Self>) -> Box<dyn ConfigMigratable<Latest = Self::Latest>> {
        Box::new(StackMetaConfigV2 {
            version: 2,
            name: self.name,
            description: self.description,
            reasons: HashMap::new(),
        })
    }
    fn is_latest(&self) -> bool {
        false
    }
    fn as_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }

    fn migrate_until_latest(self: Box<Self>) -> Self::Latest
    where
        Self: Sized,
    {
        let mut current: Box<dyn ConfigMigratable<Latest = Self::Latest>> = self;
        while !current.is_latest() {
            current = current.migrate_boxed();
        }
        // 最後にdowncast
        *current.as_any().downcast().expect("must be Conf at latest")
    }
}

///
/// スタックメタコンフィグ v2
///

#[derive(Debug, Serialize, Deserialize)]
struct StackMetaConfigV2 {
    pub version: u32,
    pub name: String,
    pub description: String,
    pub reasons: HashMap<String, HashMap<String, String>>,
}

impl StackMetaConfigV2 {
    pub fn new(name: &str) -> Self {
        StackMetaConfigV2 {
            version: 2,
            name: name.to_string(),
            description: "".to_string(),
            reasons: HashMap::new(),
        }
    }
}

impl ConfigMigratable for StackMetaConfigV2 {
    type Latest = StackMetaConfig;

    fn migrate_boxed(self: Box<Self>) -> Box<dyn ConfigMigratable<Latest = Self::Latest>> {
        Box::new(StackMetaConfig {
            version: 2,
            name: self.name,
            description: self.description,
            reasons: self.reasons,
        })
    }
    fn is_latest(&self) -> bool {
        false
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
            Box::new(StackMetaConfigV2::new(name));
        *config
            .migrate_boxed()
            .as_any()
            .downcast::<StackMetaConfig>()
            .expect("must be StackMeta at latest")
    }
    pub fn new_stack_meta(
        name: &str,
        description: &str,
        reasons: HashMap<String, HashMap<String, String>>,
    ) -> Self {
        let mut config = StackMetaConfig::new(name);
        config.description = description.to_string();
        config.reasons = reasons;
        config
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
pub fn stack_meta_config_path(
    workspace_directory: &str,
    stack_name: &str,
) -> Result<PathBuf, StackMetaConfigError> {
    let meta_path = format!("{}/{}.meta.json", workspace_directory, stack_name);
    let meta_path = PathBuf::from(&meta_path);
    return Ok(meta_path);
}

/// ${スタック名}.meta.jsonのバージョンを取得
async fn read_config_version(
    workspace_directory: &str,
    stack_name: &str,
) -> Result<u32, StackMetaConfigError> {
    let meta_path = stack_meta_config_path(workspace_directory, stack_name)?;
    match meta_path.exists() {
        true => {
            let config_json = fs::read_to_string(&meta_path).await?;
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

/// ${スタック名}.meta.jsonへ書き込む
///
/// - `stack_meta_path`: 書き込むパス
/// - `new_config`: 書き込む内容
pub async fn write_stack_meta_config(
    stack_meta_path: &PathBuf,
    new_config: &StackMetaConfig,
) -> Result<(), StackMetaConfigError> {
    if !stack_meta_path.exists() {
        fs::File::create(stack_meta_path).await?;
    }
    let config_json = serde_json::to_string(new_config)?;
    fs::write(stack_meta_path, config_json).await?;
    return Ok(());
}

/// ${スタック名}.meta.jsonを読み込む
///
/// versionが最新でない場合はマイグレーションして最新版に変換し、保存する
/// - 戻り値: 読み込んだStackMetaConfig
pub async fn read_stack_meta_config(
    workspace_directory: &str,
    stack_name: &str,
) -> Result<StackMetaConfig, StackMetaConfigError> {
    let version = read_config_version(workspace_directory, stack_name).await?;
    let config_path = stack_meta_config_path(workspace_directory, stack_name)?;

    match version {
        1 => {
            let config_json = fs::read_to_string(&config_path).await?;
            let config_json = serde_json::from_str::<StackMetaConfigV1>(&config_json)?;
            let boxed = Box::new(config_json);
            let config: StackMetaConfig = boxed.migrate_until_latest();
            write_stack_meta_config(&config_path, &config).await?;
            Ok(config)
        }
        2 => {
            let config_json = fs::read_to_string(&config_path).await?;
            let config_json = serde_json::from_str::<StackMetaConfigV2>(&config_json)?;
            let boxed = Box::new(config_json);
            Ok(boxed.migrate_until_latest())
        }
        _ => {
            let config_json = StackMetaConfig::new(stack_name);
            write_stack_meta_config(&config_path, &config_json).await?;
            Ok(config_json)
        }
    }
}
