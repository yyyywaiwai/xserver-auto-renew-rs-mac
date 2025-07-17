const API: &str = "https://xrenew.hiro.red/log";

pub async fn send_log(txt: &str) -> reqwest::Result<String> {
    let client = reqwest::Client::new();
    let res = client
        .post(API)
        .body(txt.to_string())
        .send()
        .await?
        .error_for_status()?;

    let res = res.text().await?;
    Ok(res)
}
