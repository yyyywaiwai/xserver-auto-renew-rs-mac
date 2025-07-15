use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::client::Captcha;

const API: &str = "https://xrenew.hiro.red";

#[derive(Debug, Serialize)]
pub struct Request {
    #[serde(rename = "mimeType")]
    pub mime_type: String,
    pub data: String,
}

#[derive(Debug, Deserialize)]
pub struct Response {
    pub code: i32,
}

#[derive(Debug, thiserror::Error)]
pub enum CaptchaError {
    #[error("invalid src format")]
    InvalidSrcFormat,
    #[error("Failed to send captcha request: {0}")]
    RequestError(#[from] reqwest::Error),
    #[error("Failed to parse captcha response: {code} - {message}")]
    ServerError { code: StatusCode, message: String },
}

pub async fn solve_captcha(captcha: &Captcha) -> Result<i32, CaptchaError> {
    let client = reqwest::Client::new();
    let request = Request {
        mime_type: captcha
            .mime_type()
            .unwrap_or_else(|| "image/png".to_string()),
        data: captcha
            .base64_image()
            .ok_or(CaptchaError::InvalidSrcFormat)?,
    };
    let res = client
        .post(format!("{}/solve", API))
        .json(&request)
        .send()
        .await?;

    let code = res.status();

    if !code.is_success() {
        let error_text = res
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(CaptchaError::ServerError {
            code: code,
            message: error_text,
        });
    }

    let result = res.json::<Response>().await?;
    Ok(result.code)
}
