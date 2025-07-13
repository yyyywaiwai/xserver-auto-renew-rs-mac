use scraper::{ElementRef, Html, Selector};
use url::Url;

use crate::{
    client::Client,
    form::{Form, extract_forms},
};

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
        if !msg.is_empty() {
            return Some(msg);
        }
    }
    None
}

#[derive(Debug, thiserror::Error)]
pub enum ExtendError {
    #[error("Failed to send extend request: {0}")]
    RequestError(#[from] reqwest::Error),
    #[error("Failed to parse extend response: {0}")]
    ParseError(&'static str),
}

#[derive(Debug)]
pub enum ExtendResponse {
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
        std::fs::write("data/extend.html", &text).expect("Failed to write extend.html");
        for form in forms {
            let action_url = form.action.as_ref();
            println!("Form action: {:?}", action_url);
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

        return Err(ExtendError::ParseError("Extend failed"));
    }
}
