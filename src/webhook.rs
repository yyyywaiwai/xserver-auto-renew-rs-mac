use crate::data::DATA;
use serde_json::json;

pub async fn send(content: &str) {
    let url = {
        let data = DATA.lock().unwrap();
        if data.is_some() {
            data.unwrap().get_webhook()
        } else {
            None
        }
    };
    if let Some(url) = url {
        let client = reqwest::Client::new();
        let _ = client
            .post(&url)
            .json(&json!({"content": content}))
            .send()
            .await;
    }
}
