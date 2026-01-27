use super::super::super::utils::AppError;
use crate::utils::context::app_context::AppContext;
use crate::utils::context::http_client;
use crate::{config::app_config::APP_CONFIG_DIRECTORY_NAME, utils::context::file::FileSystem};
use regex::Regex;
use serde_json::Value;
use std::sync::Arc;
use std::{
    collections::{HashMap, HashSet},
    io::Cursor,
    path::PathBuf,
};
use thiserror::Error;
use zip::ZipArchive;

#[derive(Debug, Error)]
pub enum DlSchemaError {
    #[error("app error: {0}")]
    App(#[from] AppError),
    #[error("http client error: {0}")]
    Reqwest(#[from] http_client::HttpClientError),
    #[error("io error: {0}")]
    Io(#[from] tokio::io::Error),
    #[error("zip error: {0:?}")]
    Zip(#[from] zip::result::ZipError),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("task join error: {0}")]
    Join(#[from] tokio::task::JoinError),
}

/// CloudFormationのサービスとリソース種別のサマリー結果を保存するファイル名
pub const SUMMARY_SERVICE_LIST_FILE: &str = "summary_service_list.json";

fn get_resource_provider_dl_path(region: &str) -> String {
    return format!(
        "https://schema.cloudformation.{}.amazonaws.com/CloudformationSchema.zip",
        region
    );
}

fn get_resource_provider_save_path(
    file_system: Arc<dyn FileSystem>,
    region: &str,
) -> Result<PathBuf, DlSchemaError> {
    return match file_system.config_local_dir() {
        Some(dir) => Ok(dir
            .join(APP_CONFIG_DIRECTORY_NAME)
            .join(format!("CloudformationSchema-{}", region))),
        None => Err(DlSchemaError::App(AppError::new(
            "Failed to get config local dir",
        ))),
    };
}

pub async fn generate_summary_service_list(
    cxt: &AppContext,
    output_dir: PathBuf,
) -> Result<(), DlSchemaError> {
    let mut result_map: HashMap<String, HashSet<String>> = HashMap::new();
    let re = Regex::new(r"aws-([a-z\d]+)-([a-z\d]+)\.json").unwrap();
    let entries = cxt.file_system.read_dir(&output_dir).await?;
    for path in entries {
        if let Some(entry) = path.file_name().and_then(|f| f.to_str()) {
            if re.is_match(entry) {
                let file_path = output_dir.join(entry);
                let schema_json = cxt.file_system.read_file(&file_path).await?;
                let schema_json: Value = serde_json::from_str(&schema_json)?;
                if schema_json.is_object() == true {
                    let type_name = schema_json["typeName"]
                        .as_str()
                        .unwrap_or_default()
                        .to_string();
                    let parts: Vec<&str> = type_name.split("::").collect();
                    let (_, service_name, resource_type): (&str, &str, &str) = match parts[..] {
                        [a, b, c] => (a, b, c),
                        _ => continue,
                    };
                    if let Some(original) = result_map.get_mut(service_name) {
                        original.insert(resource_type.to_string());
                    } else {
                        let mut resource_types = HashSet::new();
                        resource_types.insert(resource_type.to_string());
                        result_map.insert(service_name.to_string(), resource_types);
                    }
                }
            }
        }
    }
    let summary_file_path = output_dir.join(SUMMARY_SERVICE_LIST_FILE);
    let summary_json = serde_json::to_string(&result_map)?;
    cxt.file_system
        .write_file(&summary_file_path, summary_json.as_bytes())
        .await?;
    return Ok(());
}

pub fn get_resource_provider_save_dir(
    file_system: Arc<dyn FileSystem>,
    region: &str,
) -> Result<PathBuf, DlSchemaError> {
    return match file_system.config_local_dir() {
        Some(dir) => Ok(dir
            .join(APP_CONFIG_DIRECTORY_NAME)
            .join("CloudformationSchema")
            .join(region)),
        None => Err(DlSchemaError::App(AppError::new(
            "Failed to get config local dir",
        ))),
    };
}

pub async fn dl_resource_provider(ctx: &AppContext, region: &str) -> Result<(), DlSchemaError> {
    let url = get_resource_provider_dl_path(region);
    let save_path = get_resource_provider_save_path(ctx.file_system.clone(), region)?;
    if ctx.file_system.path_exists(&save_path) {
        ctx.file_system.remove_file(&save_path).await?
    }

    // Zipファイルをダウンロード
    let response = ctx.http_client.get(url).await?;

    let output_dir = get_resource_provider_save_dir(ctx.file_system.clone(), region)?;
    let output_dir_tmp = output_dir.clone();
    let file_system = ctx.file_system.clone();

    // ZIPファイルを解凍
    // 非同期処理内で同期処理を行うため（ZipArchiveが同期処理）、spawn_blockingで別スレッドに処理を移す
    // spawn_blocking内では非同期関数は使えないため、tokioではなくstdクレートを使用
    tokio::task::spawn_blocking(move || {
        let content = Cursor::new(response);
        let mut archive = ZipArchive::new(content)?;
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let out_path = output_dir_tmp.join(file.name());

            if file.is_dir() {
                file_system.create_dir_all_sync(&out_path)?;
            } else {
                if let Some(parent) = out_path.parent() {
                    file_system.create_dir_all_sync(parent)?;
                }

                let mut outfile = file_system.touch_and_open_file(&out_path)?;
                file_system.copy_file_stream(&mut file, &mut outfile)?;
            }
        }
        Ok::<(), DlSchemaError>(())
    })
    .await??;

    generate_summary_service_list(&ctx, output_dir).await?;
    return Ok(());
}

#[cfg(test)]
#[coverage(off)]
mod get_resource_provider_save_path_tests {
    use std::sync::Arc;

    use crate::utils::context::{file::MockFileSystem, http_client::RealHttpClient};

    use super::*;

    /// resource provider を保存するファイルパスの作成に成功すること
    #[test]
    fn success() {
        // ######### 準備 #########
        let mut mock_file_system = MockFileSystem::new();

        let config_local_dir = PathBuf::from("config_local_dir");
        mock_file_system
            .expect_config_local_dir()
            .return_const(Some(config_local_dir.clone()));

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(RealHttpClient {}),
        };

        // ######### 実行 #########
        let result = get_resource_provider_save_path(app_context.file_system, "us-west-1");

        // ######### 検証 #########
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            config_local_dir
                .join(APP_CONFIG_DIRECTORY_NAME)
                .join("CloudformationSchema-us-west-1")
        );
    }

    /// config_local_dir に失敗したとき、エラーを返すこと
    #[test]
    fn error() {
        // ######### 準備 #########
        let mut mock_file_system = MockFileSystem::new();

        mock_file_system
            .expect_config_local_dir()
            .return_const(None);

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(RealHttpClient {}),
        };

        // ######### 実行 #########
        let result = get_resource_provider_save_path(app_context.file_system, "us-west-1");

        // ######### 検証 #########
        assert!(result.is_err());
    }
}

#[cfg(test)]
#[coverage(off)]
mod generate_summary_service_list_tests {
    use std::sync::Arc;

    use crate::utils::context::{file::MockFileSystem, http_client::RealHttpClient};

    use super::*;

    /// summary_service_list.json の生成に成功すること
    #[tokio::test]
    async fn success() {
        // ######### 準備 #########
        let mut mock_file_system = MockFileSystem::new();
        let output_dir = PathBuf::from("output_dir");

        mock_file_system.expect_read_dir().returning(|_| {
            Ok(vec![
                PathBuf::from("aws-s3-bucket.json"),
                PathBuf::from("aws-lambda-function.json"),
                PathBuf::from("aws-lambda-version.json"),
            ])
        });

        mock_file_system
            .expect_read_file()
            .times(3)
            .returning(|path| {
                let file_name = path.file_name().unwrap().to_str().unwrap();
                match file_name {
                    "aws-s3-bucket.json" => Ok(r#"{ "typeName": "AWS::S3::Bucket" }"#.to_string()),
                    "aws-lambda-function.json" => {
                        Ok(r#"{ "typeName": "AWS::Lambda::Function" }"#.to_string())
                    }
                    "aws-lambda-version.json" => {
                        Ok(r#"{ "typeName": "AWS::Lambda::Version" }"#.to_string())
                    }
                    _ => panic!("Unexpected file: {}", file_name),
                }
            });

        mock_file_system
            .expect_write_file()
            // 書き込み内容が期待通りであること
            .withf(|_, contents| {
                let contents_str = std::str::from_utf8(contents).unwrap();
                let summary: HashMap<String, HashSet<String>> =
                    serde_json::from_str(&contents_str).unwrap();
                return summary.len() == 2
                // S3サービスが["Bucket"]であるか
                && summary.get("S3").map_or(false, |s| s.contains("Bucket") && s.len() == 1)
                // Lambdaサービスが["Function", "Version"]であるか
                && summary.get("Lambda").map_or(false, |s| {
                    s.contains("Function") && s.contains("Version") && s.len() == 2
                });
            })
            .returning(|_, _| Ok(()));

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(RealHttpClient {}),
        };

        // ######### 実行 #########
        let result = generate_summary_service_list(&app_context, output_dir).await;

        // ######### 検証 #########
        assert!(result.is_ok());
    }

    /// read_dir に失敗したとき、エラーを返すこと
    #[tokio::test]
    async fn read_dir_err() {
        // ######### 準備 #########
        let mut mock_file_system = MockFileSystem::new();
        let output_dir = PathBuf::from("output_dir");

        mock_file_system.expect_read_dir().returning(|_| {
            Err(tokio::io::Error::new(
                tokio::io::ErrorKind::Other,
                "read_dir error",
            ))
        });

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(RealHttpClient {}),
        };

        // ######### 実行 #########
        let result = generate_summary_service_list(&app_context, output_dir).await;

        // ######### 検証 #########
        assert!(result.is_err());
    }

    /// file_name で None が返されたとき、無視して処理を継続すること
    #[tokio::test]
    async fn file_name_none() {
        // ######### 準備 #########
        let mut mock_file_system = MockFileSystem::new();
        let output_dir = PathBuf::from("output_dir");

        mock_file_system.expect_read_dir().returning(|_| {
            Ok(vec![
                PathBuf::from("aws-s3-bucket.json"),
                PathBuf::from("aws-lambda-function.json"), // TODO: file_nameがNoneになるケースを模擬する方法を検討
                PathBuf::from("aws-lambda-version.json"),
            ])
        });

        mock_file_system
            .expect_read_file()
            .times(3)
            .returning(|path| {
                let file_name = path.file_name().unwrap().to_str().unwrap();
                match file_name {
                    "aws-s3-bucket.json" => Ok(r#"{ "typeName": "AWS::S3::Bucket" }"#.to_string()),
                    "aws-lambda-function.json" => {
                        Ok(r#"{ "typeName": "AWS::Lambda::Function" }"#.to_string())
                    }
                    "aws-lambda-version.json" => {
                        Ok(r#"{ "typeName": "AWS::Lambda::Version" }"#.to_string())
                    }
                    _ => panic!("Unexpected file: {}", file_name),
                }
            });

        mock_file_system
            .expect_write_file()
            // 書き込み内容が期待通りであること
            .withf(|_, contents| {
                let contents_str = std::str::from_utf8(contents).unwrap();
                let summary: HashMap<String, HashSet<String>> =
                    serde_json::from_str(&contents_str).unwrap();
                return summary.len() == 2
                // S3サービスが["Bucket"]であるか
                && summary.get("S3").map_or(false, |s| s.contains("Bucket") && s.len() == 1)
                // Lambdaサービスが["Function", "Version"]であるか
                && summary.get("Lambda").map_or(false, |s| {
                    s.contains("Function") && s.contains("Version") && s.len() == 2
                });
            })
            .returning(|_, _| Ok(()));

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(RealHttpClient {}),
        };

        // ######### 実行 #########
        let result = generate_summary_service_list(&app_context, output_dir).await;

        // ######### 検証 #########
        assert!(result.is_ok());
    }

    /// file_name().and_then(|f| f.to_str()) で None が返されたとき、無視して処理を継続すること
    ///
    /// to_strは有効なUTF-8文字列でない場合にNoneを返すが、PathBufから無効なUTF-8ファイル名を作成するのは困難なため、このテストは実装しない
    /// 実際は無効なUTF-8ファイル名が存在する可能性があるが、その場合は正規表現にマッチしないため影響はない
    #[tokio::test]
    #[ignore]
    async fn file_name_to_str_none() {
        // 実装不可能
    }

    /// ファイル名が正規表現にマッチしないとき、無視して処理を継続すること
    #[tokio::test]
    async fn file_name_not_matching_regex() {
        // ######### 準備 #########
        let mut mock_file_system = MockFileSystem::new();
        let output_dir = PathBuf::from("output_dir");

        mock_file_system.expect_read_dir().returning(|_| {
            Ok(vec![
                PathBuf::from("aws-s3-bucket.json"),
                PathBuf::from("aws-lambda-function.yaml"), // 正規表現にマッチしないファイル名
                PathBuf::from("aws-lambda-version.json"),
            ])
        });

        mock_file_system
            .expect_read_file()
            .times(2)
            .returning(|path| {
                let file_name = path.file_name().unwrap().to_str().unwrap();
                match file_name {
                    "aws-s3-bucket.json" => Ok(r#"{ "typeName": "AWS::S3::Bucket" }"#.to_string()),
                    "aws-lambda-version.json" => {
                        Ok(r#"{ "typeName": "AWS::Lambda::Version" }"#.to_string())
                    }
                    _ => panic!("Unexpected file: {}", file_name),
                }
            });

        mock_file_system
            .expect_write_file()
            // 書き込み内容が期待通りであること
            .withf(|_, contents| {
                let contents_str = std::str::from_utf8(contents).unwrap();
                let summary: HashMap<String, HashSet<String>> =
                    serde_json::from_str(&contents_str).unwrap();
                return summary.len() == 2
                // S3サービスが["Bucket"]であるか
                && summary.get("S3").map_or(false, |s| s.contains("Bucket") && s.len() == 1)
                // Lambdaサービスが["Version"]であるか
                && summary.get("Lambda").map_or(false, |s| {
                    s.contains("Version") && s.len() == 1
                });
            })
            .returning(|_, _| Ok(()));

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(RealHttpClient {}),
        };

        // ######### 実行 #########
        let result = generate_summary_service_list(&app_context, output_dir).await;

        // ######### 検証 #########
        assert!(result.is_ok());
    }

    /// read_file に失敗したとき、エラーを返すこと
    #[tokio::test]
    async fn read_file_err() {
        // ######### 準備 #########
        let mut mock_file_system = MockFileSystem::new();
        let output_dir = PathBuf::from("output_dir");

        mock_file_system.expect_read_dir().returning(|_| {
            Ok(vec![
                PathBuf::from("aws-s3-bucket.json"),
                PathBuf::from("aws-lambda-function.json"),
                PathBuf::from("aws-lambda-version.json"),
            ])
        });

        mock_file_system.expect_read_file().times(1).returning(|_| {
            Err(tokio::io::Error::new(
                tokio::io::ErrorKind::Other,
                "read_file error",
            ))
        });

        mock_file_system.expect_write_file().times(0);

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(RealHttpClient {}),
        };

        // ######### 実行 #########
        let result = generate_summary_service_list(&app_context, output_dir).await;

        // ######### 検証 #########
        assert!(result.is_err());
    }

    /// serde_json::from_str に失敗したとき、エラーを返すこと
    #[tokio::test]
    async fn from_str_err() {
        // ######### 準備 #########
        let mut mock_file_system = MockFileSystem::new();
        let output_dir = PathBuf::from("output_dir");

        mock_file_system.expect_read_dir().returning(|_| {
            Ok(vec![
                PathBuf::from("aws-s3-bucket.json"),
                PathBuf::from("aws-lambda-function.json"),
                PathBuf::from("aws-lambda-version.json"),
            ])
        });

        mock_file_system
            .expect_read_file()
            .times(3)
            .returning(|path| {
                let file_name = path.file_name().unwrap().to_str().unwrap();
                match file_name {
                    "aws-s3-bucket.json" => Ok(r#"{ "typeName": "AWS::S3::Bucket" }"#.to_string()),
                    "aws-lambda-function.json" => {
                        Ok(r#"{ "typeName": "AWS::Lambda::Function" }"#.to_string())
                    }
                    "aws-lambda-version.json" => {
                        // 不正なJSON
                        Ok(r#"{ "typeName": "AWS::Lambda::Version", }"#.to_string())
                    }
                    _ => panic!("Unexpected file: {}", file_name),
                }
            });

        mock_file_system.expect_write_file().times(0);

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(RealHttpClient {}),
        };

        // ######### 実行 #########
        let result = generate_summary_service_list(&app_context, output_dir).await;

        // ######### 検証 #########
        assert!(result.is_err());
    }

    /// 読み込んだスキーマファイルが Object でないとき、無視して処理を継続すること
    #[tokio::test]
    async fn non_object_schema_file() {
        // ######### 準備 #########
        let mut mock_file_system = MockFileSystem::new();
        let output_dir = PathBuf::from("output_dir");

        mock_file_system.expect_read_dir().returning(|_| {
            Ok(vec![
                PathBuf::from("aws-s3-bucket.json"),
                PathBuf::from("aws-lambda-function.json"),
                PathBuf::from("aws-lambda-version.json"),
            ])
        });

        mock_file_system
            .expect_read_file()
            .times(3)
            .returning(|path| {
                let file_name = path.file_name().unwrap().to_str().unwrap();
                match file_name {
                    "aws-s3-bucket.json" => Ok(r#"{ "typeName": "AWS::S3::Bucket" }"#.to_string()),
                    "aws-lambda-function.json" => {
                        Ok(r#"{ "typeName": "AWS::Lambda::Function" }"#.to_string())
                    }
                    "aws-lambda-version.json" => {
                        // Objectでない
                        Ok(r#"["AWS::Lambda::Version"]"#.to_string())
                    }
                    _ => panic!("Unexpected file: {}", file_name),
                }
            });

        mock_file_system
            .expect_write_file()
            // 書き込み内容が期待通りであること
            .withf(|_, contents| {
                let contents_str = std::str::from_utf8(contents).unwrap();
                let summary: HashMap<String, HashSet<String>> =
                    serde_json::from_str(&contents_str).unwrap();
                return summary.len() == 2
                // S3サービスが["Bucket"]であるか
                && summary.get("S3").map_or(false, |s| s.contains("Bucket") && s.len() == 1)
                // Lambdaサービスが["Function"]であるか
                && summary.get("Lambda").map_or(false, |s| {
                    s.contains("Function") && s.len() == 1
                });
            })
            .returning(|_, _| Ok(()));

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(RealHttpClient {}),
        };

        // ######### 実行 #########
        let result = generate_summary_service_list(&app_context, output_dir).await;

        // ######### 検証 #########
        assert!(result.is_ok());
    }

    /// スキーマファイルの "typeName" が文字列でないとき、無視して処理を継続すること
    #[tokio::test]
    async fn non_string_type_name() {
        // ######### 準備 #########
        let mut mock_file_system = MockFileSystem::new();
        let output_dir = PathBuf::from("output_dir");

        mock_file_system.expect_read_dir().returning(|_| {
            Ok(vec![
                PathBuf::from("aws-s3-bucket.json"),
                PathBuf::from("aws-lambda-function.json"),
                PathBuf::from("aws-lambda-version.json"),
            ])
        });

        mock_file_system
            .expect_read_file()
            .times(3)
            .returning(|path| {
                let file_name = path.file_name().unwrap().to_str().unwrap();
                match file_name {
                    "aws-s3-bucket.json" => Ok(r#"{ "typeName": "AWS::S3::Bucket" }"#.to_string()),
                    "aws-lambda-function.json" => {
                        Ok(r#"{ "typeName": "AWS::Lambda::Function" }"#.to_string())
                    }
                    "aws-lambda-version.json" => {
                        // "typeName" が文字列でない
                        Ok(r#"{ "typeName": 1 }"#.to_string())
                    }
                    _ => panic!("Unexpected file: {}", file_name),
                }
            });

        mock_file_system
            .expect_write_file()
            // 書き込み内容が期待通りであること
            .withf(|_, contents| {
                let contents_str = std::str::from_utf8(contents).unwrap();
                let summary: HashMap<String, HashSet<String>> =
                    serde_json::from_str(&contents_str).unwrap();
                return summary.len() == 2
                // S3サービスが["Bucket"]であるか
                && summary.get("S3").map_or(false, |s| s.contains("Bucket") && s.len() == 1)
                // Lambdaサービスが["Function"]であるか
                && summary.get("Lambda").map_or(false, |s| {
                    s.contains("Function") && s.len() == 1
                });
            })
            .returning(|_, _| Ok(()));

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(RealHttpClient {}),
        };

        // ######### 実行 #########
        let result = generate_summary_service_list(&app_context, output_dir).await;

        // ######### 検証 #########
        assert!(result.is_ok());
    }

    /// スキーマファイルの "typeName" の形式が A::B::C でないとき、無視して処理を継続すること
    #[tokio::test]
    async fn non_a_b_c_type_name() {
        // ######### 準備 #########
        let mut mock_file_system = MockFileSystem::new();
        let output_dir = PathBuf::from("output_dir");

        mock_file_system.expect_read_dir().returning(|_| {
            Ok(vec![
                PathBuf::from("aws-s3-bucket.json"),
                PathBuf::from("aws-lambda-function.json"),
                PathBuf::from("aws-lambda-version.json"),
            ])
        });

        mock_file_system
            .expect_read_file()
            .times(3)
            .returning(|path| {
                let file_name = path.file_name().unwrap().to_str().unwrap();
                match file_name {
                    "aws-s3-bucket.json" => Ok(r#"{ "typeName": "AWS::S3::Bucket" }"#.to_string()),
                    "aws-lambda-function.json" => {
                        Ok(r#"{ "typeName": "AWS::Lambda::Function" }"#.to_string())
                    }
                    "aws-lambda-version.json" => {
                        // "typeName" の形式が A::B::C でない
                        Ok(r#"{ "typeName": "AWS::Lambda" }"#.to_string())
                    }
                    _ => panic!("Unexpected file: {}", file_name),
                }
            });

        mock_file_system
            .expect_write_file()
            // 書き込み内容が期待通りであること
            .withf(|_, contents| {
                let contents_str = std::str::from_utf8(contents).unwrap();
                let summary: HashMap<String, HashSet<String>> =
                    serde_json::from_str(&contents_str).unwrap();
                return summary.len() == 2
                // S3サービスが["Bucket"]であるか
                && summary.get("S3").map_or(false, |s| s.contains("Bucket") && s.len() == 1)
                // Lambdaサービスが["Function"]であるか
                && summary.get("Lambda").map_or(false, |s| {
                    s.contains("Function") && s.len() == 1
                });
            })
            .returning(|_, _| Ok(()));

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(RealHttpClient {}),
        };

        // ######### 実行 #########
        let result = generate_summary_service_list(&app_context, output_dir).await;

        // ######### 検証 #########
        assert!(result.is_ok());
    }

    /// serde_json::to_string に失敗したとき、エラーを返すこと
    ///
    /// 実際は失敗するパターンは起こりえないため、このテストは実装しない
    #[tokio::test]
    #[ignore]
    async fn to_string_err() {
        // このテストケースは実装不可
    }

    /// write_file に失敗したとき、エラーを返すこと
    #[tokio::test]
    async fn write_file_err() {
        // ######### 準備 #########
        let mut mock_file_system = MockFileSystem::new();
        let output_dir = PathBuf::from("output_dir");

        mock_file_system.expect_read_dir().returning(|_| {
            Ok(vec![
                PathBuf::from("aws-s3-bucket.json"),
                PathBuf::from("aws-lambda-function.json"),
                PathBuf::from("aws-lambda-version.json"),
            ])
        });

        mock_file_system
            .expect_read_file()
            .times(3)
            .returning(|path| {
                let file_name = path.file_name().unwrap().to_str().unwrap();
                match file_name {
                    "aws-s3-bucket.json" => Ok(r#"{ "typeName": "AWS::S3::Bucket" }"#.to_string()),
                    "aws-lambda-function.json" => {
                        Ok(r#"{ "typeName": "AWS::Lambda::Function" }"#.to_string())
                    }
                    "aws-lambda-version.json" => {
                        Ok(r#"{ "typeName": "AWS::Lambda::Version" }"#.to_string())
                    }
                    _ => panic!("Unexpected file: {}", file_name),
                }
            });

        mock_file_system.expect_write_file().returning(|_, _| {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "write_file error",
            ))
        });

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(RealHttpClient {}),
        };

        // ######### 実行 #########
        let result = generate_summary_service_list(&app_context, output_dir).await;

        // ######### 検証 #########
        assert!(result.is_err());
    }
}

#[cfg(test)]
#[coverage(off)]
mod get_resource_provider_save_dir_tests {
    use std::sync::Arc;

    use crate::utils::context::{file::MockFileSystem, http_client::RealHttpClient};

    use super::*;

    /// resource provider を保存するディレクトリ名の作成に成功すること
    #[test]
    fn success() {
        // ######### 準備 #########
        let mut mock_file_system = MockFileSystem::new();

        let config_local_dir = PathBuf::from("config_local_dir");
        mock_file_system
            .expect_config_local_dir()
            .return_const(Some(config_local_dir.clone()));

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(RealHttpClient {}),
        };

        // ######### 実行 #########
        let result = get_resource_provider_save_dir(app_context.file_system, "us-west-1");

        // ######### 検証 #########
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            config_local_dir
                .join(APP_CONFIG_DIRECTORY_NAME)
                .join("CloudformationSchema")
                .join("us-west-1")
        );
    }

    /// config_local_dir に失敗したとき、エラーを返すこと
    #[test]
    fn error() {
        // ######### 準備 #########
        let mut mock_file_system = MockFileSystem::new();

        mock_file_system
            .expect_config_local_dir()
            .return_const(None);

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(RealHttpClient {}),
        };

        // ######### 実行 #########
        let result = get_resource_provider_save_dir(app_context.file_system, "us-west-1");

        // ######### 検証 #########
        assert!(result.is_err());
    }
}

#[cfg(test)]
#[coverage(off)]
mod dl_resource_provider_tests {
    use crate::utils::context::{
        file::MockFileSystem,
        http_client::{HttpClientError, MockHttpClient},
    };
    use bytes::Bytes;
    use std::{fs::File, io::Write};
    use zip::write::SimpleFileOptions;

    use super::*;

    /// テスト用のZIPバイトデータを作成
    fn create_test_zip_bytes() -> Vec<u8> {
        let mut buffer = Vec::new();
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(&mut buffer));

        let options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);

        // テスト用のスキーマファイルを追加
        zip.start_file("aws-s3-bucket.json", options).unwrap();
        zip.write_all(br#"{"typeName":"AWS::S3::Bucket"}"#).unwrap();

        zip.start_file("aws-lambda-function.json", options).unwrap();
        zip.write_all(br#"{"typeName":"AWS::Lambda::Function"}"#)
            .unwrap();

        zip.start_file("aws-ec2-instance.json", options).unwrap();
        zip.write_all(br#"{"typeName":"AWS::EC2::Instance"}"#)
            .unwrap();

        zip.finish().unwrap();
        return buffer;
    }

    /// テスト用のZIPバイトデータを作成
    fn create_test_zip_bytes_with_dir() -> Vec<u8> {
        let mut buffer = Vec::new();
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(&mut buffer));

        let options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);

        // ディレクトリエントリを追加
        zip.add_directory("schemas/", options).unwrap();

        // テスト用のスキーマファイルを追加
        zip.start_file("schemas/aws-s3-bucket.json", options)
            .unwrap();
        zip.write_all(br#"{"typeName":"AWS::S3::Bucket"}"#).unwrap();

        zip.start_file("schemas/aws-lambda-function.json", options)
            .unwrap();
        zip.write_all(br#"{"typeName":"AWS::Lambda::Function"}"#)
            .unwrap();

        zip.start_file("schemas/aws-ec2-instance.json", options)
            .unwrap();
        zip.write_all(br#"{"typeName":"AWS::EC2::Instance"}"#)
            .unwrap();

        zip.finish().unwrap();
        return buffer;
    }

    /// resource provider の1回目のダウンロードと展開に成功すること
    #[tokio::test]
    async fn success_first() {
        // ######### 準備 #########
        let mut mock_file_system = MockFileSystem::new();
        let mut mock_http_client = MockHttpClient::new();

        mock_file_system
            .expect_config_local_dir()
            .return_const(Some(PathBuf::from("config_local_dir")));

        mock_file_system.expect_path_exists().return_const(false);

        mock_file_system.expect_remove_file().times(0); // 呼び出しなし

        let zip_bytes = create_test_zip_bytes();
        mock_http_client
            .expect_get()
            .returning(move |_| Ok(Bytes::from(zip_bytes.clone())));

        mock_file_system
            .expect_create_dir_all_sync()
            .returning(|_| Ok(()));

        mock_file_system
            .expect_touch_and_open_file()
            .returning(|_| Ok(File::create(std::env::temp_dir().join("temp_file")).unwrap()));

        mock_file_system
            .expect_copy_file_stream()
            .returning(|_, _| Ok(100));

        // generate_summary_service_list内で呼び出される関数のモック

        mock_file_system.expect_read_dir().returning(|_| {
            Ok(vec![
                PathBuf::from("aws-s3-bucket.json"),
                PathBuf::from("aws-lambda-function.json"),
            ])
        });

        mock_file_system.expect_read_file().returning(|path| {
            let file_name = path.file_name().unwrap().to_str().unwrap();
            match file_name {
                "aws-s3-bucket.json" => Ok(r#"{ "typeName": "AWS::S3::Bucket" }"#.to_string()),
                "aws-lambda-function.json" => {
                    Ok(r#"{ "typeName": "AWS::Lambda::Function" }"#.to_string())
                }
                _ => panic!("Unexpected file: {}", file_name),
            }
        });

        mock_file_system
            .expect_write_file()
            .returning(|_, _| Ok(()));

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
        };

        // ######### 実行 #########
        let result = dl_resource_provider(&app_context, "us-west-1").await;

        // ######### 検証 #########
        assert!(result.is_ok());
    }

    /// resource provider の2回目以降のダウンロードと展開に成功すること
    #[tokio::test]
    async fn success_second_and_later() {
        // ######### 準備 #########
        let mut mock_file_system = MockFileSystem::new();
        let mut mock_http_client = MockHttpClient::new();

        mock_file_system
            .expect_config_local_dir()
            .return_const(Some(PathBuf::from("config_local_dir")));

        mock_file_system.expect_path_exists().return_const(true);

        mock_file_system
            .expect_remove_file()
            .times(1)
            .returning(|_| Ok(()));

        let zip_bytes = create_test_zip_bytes();
        mock_http_client
            .expect_get()
            .returning(move |_| Ok(Bytes::from(zip_bytes.clone())));

        mock_file_system
            .expect_create_dir_all_sync()
            .returning(|_| Ok(()));

        mock_file_system
            .expect_touch_and_open_file()
            .returning(|_| Ok(File::create(std::env::temp_dir().join("temp_file")).unwrap()));

        mock_file_system
            .expect_copy_file_stream()
            .returning(|_, _| Ok(100));

        // generate_summary_service_list内で呼び出される関数のモック

        mock_file_system.expect_read_dir().returning(|_| {
            Ok(vec![
                PathBuf::from("aws-s3-bucket.json"),
                PathBuf::from("aws-lambda-function.json"),
            ])
        });

        mock_file_system.expect_read_file().returning(|path| {
            let file_name = path.file_name().unwrap().to_str().unwrap();
            match file_name {
                "aws-s3-bucket.json" => Ok(r#"{ "typeName": "AWS::S3::Bucket" }"#.to_string()),
                "aws-lambda-function.json" => {
                    Ok(r#"{ "typeName": "AWS::Lambda::Function" }"#.to_string())
                }
                _ => panic!("Unexpected file: {}", file_name),
            }
        });

        mock_file_system
            .expect_write_file()
            .returning(|_, _| Ok(()));

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
        };

        // ######### 実行 #########
        let result = dl_resource_provider(&app_context, "us-west-1").await;

        // ######### 検証 #########
        assert!(result.is_ok());
    }

    /// Zipファイルにディレクトリが含まれる場合、resource provider のダウンロードと展開に成功すること
    #[tokio::test]
    async fn success_with_dir() {
        // ######### 準備 #########
        let mut mock_file_system = MockFileSystem::new();
        let mut mock_http_client = MockHttpClient::new();

        mock_file_system
            .expect_config_local_dir()
            .return_const(Some(PathBuf::from("config_local_dir")));

        mock_file_system.expect_path_exists().return_const(false);

        mock_file_system.expect_remove_file().times(0); // 呼び出しなし

        let zip_bytes = create_test_zip_bytes_with_dir();
        mock_http_client
            .expect_get()
            .returning(move |_| Ok(Bytes::from(zip_bytes.clone())));

        mock_file_system
            .expect_create_dir_all_sync()
            .returning(|_| Ok(()));

        mock_file_system
            .expect_touch_and_open_file()
            .returning(|_| Ok(File::create(std::env::temp_dir().join("temp_file")).unwrap()));

        mock_file_system
            .expect_copy_file_stream()
            .returning(|_, _| Ok(100));

        // generate_summary_service_list内で呼び出される関数のモック

        mock_file_system.expect_read_dir().returning(|_| {
            Ok(vec![
                PathBuf::from("aws-s3-bucket.json"),
                PathBuf::from("aws-lambda-function.json"),
            ])
        });

        mock_file_system.expect_read_file().returning(|path| {
            let file_name = path.file_name().unwrap().to_str().unwrap();
            match file_name {
                "aws-s3-bucket.json" => Ok(r#"{ "typeName": "AWS::S3::Bucket" }"#.to_string()),
                "aws-lambda-function.json" => {
                    Ok(r#"{ "typeName": "AWS::Lambda::Function" }"#.to_string())
                }
                _ => panic!("Unexpected file: {}", file_name),
            }
        });

        mock_file_system
            .expect_write_file()
            .returning(|_, _| Ok(()));

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
        };

        // ######### 実行 #########
        let result = dl_resource_provider(&app_context, "us-west-1").await;

        // ######### 検証 #########
        assert!(result.is_ok());
    }

    /// get_resource_provider_save_path に失敗したとき、エラーを返すこと
    #[tokio::test]
    async fn get_resource_provider_save_path_err() {
        // ######### 準備 #########
        let mut mock_file_system = MockFileSystem::new();

        mock_file_system
            .expect_config_local_dir()
            .return_const(None);

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(MockHttpClient::new()),
        };

        // ######### 実行 #########
        let result = dl_resource_provider(&app_context, "us-west-1").await;

        // ######### 検証 #########
        assert!(result.is_err());
    }

    /// remove_file に失敗したとき、エラーを返すこと
    #[tokio::test]
    async fn remove_file_err() {
        // ######### 準備 #########
        let mut mock_file_system = MockFileSystem::new();

        mock_file_system
            .expect_config_local_dir()
            .return_const(Some(PathBuf::from("config_local_dir")));

        mock_file_system.expect_path_exists().return_const(true);

        mock_file_system
            .expect_remove_file()
            .times(1)
            .returning(|_| {
                Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "remove_file error",
                ))
            });

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(MockHttpClient::new()),
        };

        // ######### 実行 #########
        let result = dl_resource_provider(&app_context, "us-west-1").await;

        // ######### 検証 #########
        assert!(result.is_err());
    }

    /// resource provider のZipファイルダウンロードに失敗したとき、エラーを返すこと
    #[tokio::test]
    async fn http_client_get_err() {
        // ######### 準備 #########
        let mut mock_file_system = MockFileSystem::new();
        let mut mock_http_client = MockHttpClient::new();

        mock_file_system
            .expect_config_local_dir()
            .return_const(Some(PathBuf::from("config_local_dir")));

        mock_file_system.expect_path_exists().return_const(false);

        mock_http_client.expect_get().returning(|_| {
            Err(HttpClientError::Reqwest(
                "http client get error".to_string(),
            ))
        });

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
        };

        // ######### 実行 #########
        let result = dl_resource_provider(&app_context, "us-west-1").await;

        // ######### 検証 #########
        assert!(result.is_err());
    }

    /// get_resource_provider_save_dir に失敗したとき、エラーを返すこと
    #[tokio::test]
    async fn get_resource_provider_save_dir_err() {
        // ######### 準備 #########
        let mut mock_file_system = MockFileSystem::new();
        let mut mock_http_client = MockHttpClient::new();

        let mut count = 0;
        mock_file_system
            .expect_config_local_dir()
            .returning(move || {
                count += 1;
                if count < 2 {
                    Some(PathBuf::from("config_local_dir"))
                } else {
                    None
                }
            });

        mock_file_system.expect_path_exists().return_const(false);

        mock_file_system.expect_remove_file().times(0);

        let zip_bytes = create_test_zip_bytes();
        mock_http_client
            .expect_get()
            .times(1)
            .returning(move |_| Ok(Bytes::from(zip_bytes.clone())));
        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
        };

        // ######### 実行 #########
        let result = dl_resource_provider(&app_context, "us-west-1").await;

        // ######### 検証 #########
        assert!(result.is_err());
    }

    /// ZipArchive::new に失敗したとき、エラーを返すこと
    #[tokio::test]
    async fn zip_archive_new_err() {
        // ######### 準備 #########
        let mut mock_file_system = MockFileSystem::new();
        let mut mock_http_client = MockHttpClient::new();

        mock_file_system
            .expect_config_local_dir()
            .return_const(Some(PathBuf::from("config_local_dir")));

        mock_file_system.expect_path_exists().return_const(false);

        mock_file_system.expect_remove_file().times(0);

        // 不正なZIPデータ（ZIPマジックナンバー 0x50 0x4B で始まっていない）
        let invalid_zip_bytes = vec![0u8, 1u8, 2u8, 3u8, 4u8];
        mock_http_client
            .expect_get()
            .returning(move |_| Ok(Bytes::from(invalid_zip_bytes.clone())));

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
        };

        // ######### 実行 #########
        let result = dl_resource_provider(&app_context, "us-west-1").await;

        // ######### 検証 #########
        assert!(result.is_err());
    }

    /// ZipArchive.by_index に失敗したとき、エラーを返すこと
    ///
    /// by_index がエラーを返すのは以下の場合
    /// 1. インデックスが範囲外（本コードでは起こりえない）
    /// 2. ZIPファイルが破損（通常は ZipArchive::new で検出される）
    /// 3. 暗号化されたエントリ（テスト用ZIPの作成が困難）
    /// 4. 未サポートの圧縮方式（テスト用ZIPの作成が困難）
    ///
    /// これらのケースをテストすることは困難なため、このテストは実装しない
    #[tokio::test]
    #[ignore]
    async fn zip_archive_by_index_err() {
        // 実装不可能
    }

    /// Zipファイルにディレクトリが含まれ create_dir_all_sync に失敗したとき、エラーを返すこと
    #[tokio::test]
    async fn create_dir_all_sync_err_with_dir() {
        // ######### 準備 #########
        let mut mock_file_system = MockFileSystem::new();
        let mut mock_http_client = MockHttpClient::new();

        mock_file_system
            .expect_config_local_dir()
            .return_const(Some(PathBuf::from("config_local_dir")));

        mock_file_system.expect_path_exists().return_const(false);

        mock_file_system.expect_remove_file().times(0);

        let zip_bytes = create_test_zip_bytes_with_dir();
        mock_http_client
            .expect_get()
            .returning(move |_| Ok(Bytes::from(zip_bytes.clone())));

        mock_file_system
            .expect_create_dir_all_sync()
            .returning(|_| {
                Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "create_dir_all_sync error",
                ))
            });

        mock_file_system
            .expect_touch_and_open_file()
            .returning(|_| Ok(File::create(std::env::temp_dir().join("temp_file")).unwrap()));

        mock_file_system
            .expect_copy_file_stream()
            .returning(|_, _| Ok(100));

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
        };

        // ######### 実行 #########
        let result = dl_resource_provider(&app_context, "us-west-1").await;

        // ######### 検証 #########
        assert!(result.is_err());
    }

    /// Zipファイルにディレクトリが含まれず create_dir_all_sync に失敗したとき、エラーを返すこと
    #[tokio::test]
    async fn create_dir_all_sync_err_without_dir() {
        // ######### 準備 #########
        let mut mock_file_system = MockFileSystem::new();
        let mut mock_http_client = MockHttpClient::new();

        mock_file_system
            .expect_config_local_dir()
            .return_const(Some(PathBuf::from("config_local_dir")));

        mock_file_system.expect_path_exists().return_const(false);

        mock_file_system.expect_remove_file().times(0); // 呼び出しなし

        let zip_bytes = create_test_zip_bytes();
        mock_http_client
            .expect_get()
            .returning(move |_| Ok(Bytes::from(zip_bytes.clone())));

        mock_file_system
            .expect_create_dir_all_sync()
            .returning(|_| {
                Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "create_dir_all_sync error",
                ))
            });

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
        };

        // ######### 実行 #########
        let result = dl_resource_provider(&app_context, "us-west-1").await;

        // ######### 検証 #########
        assert!(result.is_err());
    }

    /// touch_and_open_file に失敗したとき、エラーを返すこと
    #[tokio::test]
    async fn touch_and_open_file_err() {
        // ######### 準備 #########
        let mut mock_file_system = MockFileSystem::new();
        let mut mock_http_client = MockHttpClient::new();

        mock_file_system
            .expect_config_local_dir()
            .return_const(Some(PathBuf::from("config_local_dir")));

        mock_file_system.expect_path_exists().return_const(false);

        mock_file_system.expect_remove_file().times(0); // 呼び出しなし

        let zip_bytes = create_test_zip_bytes();
        mock_http_client
            .expect_get()
            .returning(move |_| Ok(Bytes::from(zip_bytes.clone())));

        mock_file_system
            .expect_create_dir_all_sync()
            .returning(|_| Ok(()));

        mock_file_system
            .expect_touch_and_open_file()
            .returning(|_| {
                Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "remove_file error",
                ))
            });

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
        };

        // ######### 実行 #########
        let result = dl_resource_provider(&app_context, "us-west-1").await;

        // ######### 検証 #########
        assert!(result.is_err());
    }

    /// copy_file_stream に失敗したとき、エラーを返すこと
    #[tokio::test]
    async fn copy_file_stream_err() {
        // ######### 準備 #########
        let mut mock_file_system = MockFileSystem::new();
        let mut mock_http_client = MockHttpClient::new();

        mock_file_system
            .expect_config_local_dir()
            .return_const(Some(PathBuf::from("config_local_dir")));

        mock_file_system.expect_path_exists().return_const(false);

        mock_file_system.expect_remove_file().times(0); // 呼び出しなし

        let zip_bytes = create_test_zip_bytes();
        mock_http_client
            .expect_get()
            .returning(move |_| Ok(Bytes::from(zip_bytes.clone())));

        mock_file_system
            .expect_create_dir_all_sync()
            .returning(|_| Ok(()));

        mock_file_system
            .expect_touch_and_open_file()
            .returning(|_| Ok(File::create(std::env::temp_dir().join("temp_file")).unwrap()));

        mock_file_system
            .expect_copy_file_stream()
            .returning(|_, _| {
                Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "remove_file error",
                ))
            });

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
        };

        // ######### 実行 #########
        let result = dl_resource_provider(&app_context, "us-west-1").await;

        // ######### 検証 #########
        assert!(result.is_err());
    }

    /// out_path.parent() が None を返したとき、親ディレクトリの作成をスキップすること
    ///
    /// 本コードでは常に親ディレクトリが存在し None になることはないため、テストは実装しない
    #[tokio::test]
    #[ignore]
    async fn out_path_parent_none() {
        // 実装不要（起こりえないケース）
    }

    /// generate_summary_service_list に失敗したとき、エラーを返すこと
    #[tokio::test]
    async fn generate_summary_service_list_err() {
        // ######### 準備 #########
        let mut mock_file_system = MockFileSystem::new();
        let mut mock_http_client = MockHttpClient::new();

        mock_file_system
            .expect_config_local_dir()
            .return_const(Some(PathBuf::from("config_local_dir")));

        mock_file_system.expect_path_exists().return_const(false);

        mock_file_system.expect_remove_file().times(0); // 呼び出しなし

        let zip_bytes = create_test_zip_bytes();
        mock_http_client
            .expect_get()
            .returning(move |_| Ok(Bytes::from(zip_bytes.clone())));

        mock_file_system
            .expect_create_dir_all_sync()
            .returning(|_| Ok(()));

        mock_file_system
            .expect_touch_and_open_file()
            .returning(|_| Ok(File::create(std::env::temp_dir().join("temp_file")).unwrap()));

        mock_file_system
            .expect_copy_file_stream()
            .returning(|_, _| Ok(100));

        // generate_summary_service_list内で呼び出される関数のモック
        mock_file_system.expect_read_dir().returning(|_| {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "read_dir error",
            ))
        });

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
        };

        // ######### 実行 #########
        let result = dl_resource_provider(&app_context, "us-west-1").await;

        // ######### 検証 #########
        assert!(result.is_err());
    }

    /// spawn_blocking に失敗したとき、エラーを返すこと
    ///
    /// spawn_blocking はクロージャ内でパニックが発生した場合にエラーを返すが、
    /// 本コードではパニックが発生するケースはないため、このテストは実装しない
    #[tokio::test]
    #[ignore]
    async fn spawn_blocking_err() {
        // 実装不可能
    }
}
