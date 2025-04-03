use super::super::api::cloudformation;
use super::super::api::github;

async fn get_service_data_dl() -> Result<(), ()> {
    let branch = github::branch::get_branch("aws", "aws-cli", "v2")
        .await
        .unwrap();

    let commit_sha = match branch["commit"]["sha"].as_str() {
        Some(sha) => sha,
        None => {
            println!("Failed to get commit SHA from branch data");
            return Err(());
        }
    };
    println!("Branch commit SHA: {:?}", commit_sha);

    return Ok(());
}

#[tauri::command]
pub async fn setup_app() -> Result<(), ()> {
    if let Err(_) = cloudformation::schema::dl_resource_provider("us-east-1").await {
        return Err(());
    }
    if let Err(_) = get_service_data_dl().await {
        return Err(());
    }
    if let Err(_) = super::app_config::initialized_app_config().await {
        return Err(());
    }
    return Ok(());
}
