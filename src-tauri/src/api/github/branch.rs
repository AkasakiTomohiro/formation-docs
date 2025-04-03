use serde_json::Value;

pub async fn get_branch(owner: &str, repo: &str, branch: &str) -> Result<Value, ()> {
    let url = format!(
        "https://api.github.com/repos/{}/{}/branches/{}",
        owner, repo, branch
    );
    let client = reqwest::Client::new();
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("User-Agent", "formation-docs".parse().unwrap());
    headers.insert("X-GitHub-Api-Version", "2022-11-28".parse().unwrap());

    let response = match client.get(url).headers(headers).send().await {
        Ok(res) => res,
        Err(_) => return Err(()),
    };
    if response.status() != reqwest::StatusCode::OK {
        return Err(());
    }
    let response = match response.text().await {
        Ok(res) => res,
        Err(_) => return Err(()),
    };

    return match serde_json::from_str(&response) {
        Ok(json) => Ok(json),
        Err(_) => Err(()),
    };
}
