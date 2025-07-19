use scraper::{ElementRef, Html, Selector};
use url::Url;

use super::Client;
use super::form::{Form, extract_forms};

pub fn get_server_id(html: &str) -> Option<String> {
    let doc = Html::parse_document(html);

    let h3_sel = Selector::parse("h3").unwrap();
    let link_sel = Selector::parse("a[href]").unwrap();

    for h3 in doc.select(&h3_sel) {
        if !h3.text().any(|t| t.contains("無料")) {
            continue;
        }
        let mut next = h3.next_sibling();
        while let Some(node) = next {
            if let Some(el) = node.value().as_element() {
                if el.name() == "table" {
                    let table = ElementRef::wrap(node).unwrap();

                    for a in table.select(&link_sel) {
                        if let Some(href) = a.value().attr("href") {
                            let full = Url::parse(&format!("http://dummy.local{}", href))
                                .or_else(|_| Url::parse(href));

                            if let Ok(url) = full {
                                if let Some(id_val) = url
                                    .query_pairs()
                                    .find(|(k, _)| k == "id")
                                    .map(|(_, v)| v.into_owned())
                                {
                                    return Some(id_val);
                                }
                            }
                        }
                    }
                    break;
                } else {
                    break;
                }
            }
            next = node.next_sibling();
        }
    }
    None
}

pub fn get_message(html: &str) -> Option<String> {
    let doc = Html::parse_document(html);
    let message_sel = Selector::parse("section").unwrap();

    if let Some(section) = doc.select(&message_sel).next() {
        let msg = section.text().collect::<Vec<_>>().join(" ");
        let lines = msg
            .lines()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>();
        if !lines.is_empty() {
            return Some(lines.join(" "));
        }
    }
    None
}

pub fn get_captcha_image(doc: &Html) -> Option<String> {
    let img_sel = Selector::parse("img").unwrap();

    for img in doc.select(&img_sel) {
        if let Some(src) = img.value().attr("src") {
            if src.contains("base64") {
                return Some(src.to_string());
            }
        }
    }
    None
}

pub fn get_cloudflare_challenge(doc: &Html) -> Option<String> {
    let sel = Selector::parse("div.cf-turnstile").unwrap();
    let elm = doc.select(&sel).next()?;
    let data = elm.value().attr("data-sitekey")?;
    Some(data.to_string())
}

#[derive(Debug, thiserror::Error)]
pub enum ExtendError {
    #[error("Failed to send extend request: {0}")]
    RequestError(#[from] reqwest::Error),
    #[error("Failed to parse extend response: {0}")]
    ParseError(&'static str),
}

#[derive(Debug)]
pub struct Captcha {
    pub form: Form,
    pub image: Option<String>,
    pub cloudflare: Option<String>,
    pub url: Url,
}

impl Captcha {
    pub fn has_image(&self) -> bool {
        self.image.is_some()
    }

    pub fn base64_image(&self) -> Option<String> {
        let image = self.image.as_ref()?;
        let split = ";base64,";
        if let Some(pos) = image.find(split) {
            Some(image[pos + split.len()..].to_string())
        } else {
            None
        }
    }

    pub fn mime_type(&self) -> Option<String> {
        let image = self.image.as_ref()?;
        if let Some(pos) = image.find(';') {
            Some(image[..pos].replace("data:", ""))
        } else {
            None
        }
    }

    pub fn cloudflare_challenge(&self) -> Option<&str> {
        self.cloudflare.as_deref()
    }
}

#[derive(Debug)]
pub enum ExtendResponse {
    Success(String),
    Failure(String),
    CaptchaRequired(Captcha),
}

#[derive(Debug)]
pub enum CaptchaResponse {
    Success(String),
    Failure(String),
}

const EXTEND_URL: &str = "https://secure.xserver.ne.jp/xapanel/xvps/server/freevps/extend/index";

pub type ExtendResult<T> = Result<T, ExtendError>;

impl Client {
    pub async fn extend_vps(&self, id: &str) -> ExtendResult<Form> {
        let url = format!("{}?id_vps={}", EXTEND_URL, id);
        let res = self.client.get(&url).send().await?.error_for_status()?;
        let url = res.url().clone();
        let text = res.text().await?;
        let forms = extract_forms(&text, Some(&url));

        for form in forms {
            if form
                .action
                .as_ref()
                .map_or(false, |a| a.contains("extend") && !a.contains("change"))
            {
                return Ok(form);
            }
        }
        Err(ExtendError::ParseError("No valid extend form found"))
    }

    pub async fn submit_extend_form(&self, form: &Form) -> ExtendResult<ExtendResponse> {
        let mut params = std::collections::HashMap::new();
        for field in &form.fields {
            params.insert(field.name.clone(), field.value.clone().unwrap_or_default());
        }
        let res = self
            .client
            .post(form.action.as_ref().unwrap())
            .form(&params)
            .send()
            .await?
            .error_for_status()?;

        let url = res.url().clone();
        let text = res.text().await?;

        let extend_unavailable = ["以降にお試し", "継続される場合は", "利用期限の1日前"];
        if extend_unavailable.iter().any(|s| text.contains(s)) {
            return Ok(ExtendResponse::Failure(
                get_message(&text).unwrap_or_else(|| "Extend unavailable".to_string()),
            ));
        }

        let success_message = ["完了しました", "成功しました"];
        if success_message.iter().any(|s| text.contains(s)) {
            return Ok(ExtendResponse::Success(
                get_message(&text).unwrap_or_else(|| "Extend successful".to_string()),
            ));
        }

        if text.contains("画像認証") {
            let forms = extract_forms(&text, Some(&url));
            for form in forms {
                let html = Html::parse_document(&text);
                let image = get_captcha_image(&html);
                let cloudflare = get_cloudflare_challenge(&html);
                if form.action.as_ref().map_or(false, |a| a.contains("/do")) {
                    return Ok(ExtendResponse::CaptchaRequired(Captcha {
                        form,
                        image,
                        cloudflare,
                        url,
                    }));
                }
            }
            return Err(ExtendError::ParseError(
                "Captcha required but no image found",
            ));
        }

        return Err(ExtendError::ParseError("Extend failed"));
    }

    pub async fn submit_captcha(
        &self,
        captcha: &Captcha,
        code: Option<i32>,
        turnstile_response: Option<String>,
    ) -> ExtendResult<CaptchaResponse> {
        let form = &captcha.form;
        let mut params = std::collections::HashMap::new();
        for field in &form.fields {
            params.insert(field.name.clone(), field.value.clone().unwrap_or_default());
        }
        if let Some(code) = code {
            if let Some(field) = params.get_mut("auth_code") {
                *field = code.to_string();
            } else {
                'f: {
                    for (name, value) in &mut params {
                        if name.contains("code") || name.contains("auth") {
                            *value = code.to_string();
                            break 'f;
                        }
                        return Err(ExtendError::ParseError(
                            "Captcha code field not found in form",
                        ));
                    }
                }
            }
        }
        if let Some(turnstile) = turnstile_response {
            params.insert("cf-turnstile-response".to_string(), turnstile);
        }

        let res = self
            .client
            .post(form.action.as_ref().unwrap())
            .form(&params)
            .send()
            .await?
            .error_for_status()?;

        let text = res.text().await?;

        if text.contains("入力された認証コードが正しくありません") {
            return Err(ExtendError::ParseError("Invalid captcha code"));
        }

        let extend_unavailable = ["以降にお試し", "継続される場合は", "利用期限の1日前"];
        if extend_unavailable.iter().any(|s| text.contains(s)) {
            return Ok(CaptchaResponse::Failure(
                get_message(&text).unwrap_or_else(|| "Extend unavailable".to_string()),
            ));
        }

        if text.contains("利用期限の更新手続きが完了しました") {
            return Ok(CaptchaResponse::Success(
                "利用期限の更新手続きが完了しました".to_string(),
            ));
        }

        let success_message = ["完了しました", "成功しました"];
        if success_message.iter().any(|s| text.contains(s)) {
            return Ok(CaptchaResponse::Success(
                get_message(&text).unwrap_or_else(|| "Extend successful".to_string()),
            ));
        }

        return Err(ExtendError::ParseError("Captcha submission failed"));
    }
}
