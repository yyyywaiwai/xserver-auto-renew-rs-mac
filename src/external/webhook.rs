use serde_json::json;

use crate::data::value::get_webhook;

pub async fn send(content: &str) {
    let url = get_webhook();
    if let Some(url) = url {
        let client = reqwest::Client::new();
        let _ = client
            .post(&url)
            .json(&json!({"content": content}))
            .send()
            .await;
    }
}
