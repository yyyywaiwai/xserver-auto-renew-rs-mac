use std::collections::HashMap;

use crate::{
    account::Account,
    client::Client,
    form::{FieldType, Form, classify_field, extract_forms, get_mailaddress},
};

const LOGIN_URL: &str = "https://secure.xserver.ne.jp/xapanel/login/xvps/";
const TOP_PAGE: &str = "https://secure.xserver.ne.jp/xapanel/xvps/index";
const AUTH_URL: &str = "https://secure.xserver.ne.jp/xapanel/myaccount/loginauth/index";

#[derive(Debug, thiserror::Error)]
pub enum LoginError {
    #[error("Failed to send login request: {0}")]
    RequestError(#[from] reqwest::Error),
    #[error("Failed to parse login response: {0}")]
    ParseError(&'static str),
}

#[derive(Debug)]
pub enum LoginStatus {
    Success(String),
    Failure(String),
    TowWayAuthRequired(Form, Option<String>),
}

pub type LoginResult<T> = Result<T, LoginError>;

impl Client {
    pub async fn login_page(&self) -> LoginResult<Form> {
        let res = self.client.get(LOGIN_URL).send().await?;
        let res = res.error_for_status()?;
        let text = res.text().await?;
        let url = LOGIN_URL.parse::<url::Url>().expect("Invalid login URL");
        for form in extract_forms(&text, Some(&url)).into_iter() {
            if form.action.as_ref().map_or(false, |a| a.contains("login")) {
                return Ok(form);
            }
        }
        Err(LoginError::ParseError("No login form found"))
    }

    pub async fn try_login(&self, form: &Form, account: &Account) -> LoginResult<LoginStatus> {
        let mut params = std::collections::HashMap::new();
        for field in &form.fields {
            match classify_field(field) {
                FieldType::Id => {
                    params.insert(field.name.clone(), account.email.clone());
                }
                FieldType::Password => {
                    params.insert(field.name.clone(), account.password.clone());
                }
                _ => {
                    params.insert(field.name.clone(), field.value.clone().unwrap_or_default());
                }
            }
        }
        let res = self
            .client
            .post(form.action.as_ref().unwrap())
            .form(&params)
            .send()
            .await?;

        let res = res.error_for_status()?;

        let url = res.url().clone();

        if url.as_str().starts_with(LOGIN_URL) {
            return Ok(LoginStatus::Failure(
                "Login failed(アカウントが間違っている可能性があります)".to_string(),
            ));
        }

        if url.as_str().starts_with(AUTH_URL) {
            let text = res.text().await?;
            let forms = extract_forms(&text, Some(&url));
            if forms.is_empty() {
                return Err(LoginError::ParseError("No two-way auth form found"));
            }
            return Ok(LoginStatus::TowWayAuthRequired(
                forms.into_iter().next().unwrap(),
                get_mailaddress(&text),
            ));
        }

        if url.as_str().starts_with(TOP_PAGE) {
            let text = res.text().await?;
            return Ok(LoginStatus::Success(text));
        }

        return Err(LoginError::ParseError("Unknown login status"));
    }

    pub async fn two_way_select_email(&self, form: &Form) -> LoginResult<Form> {
        let mut params: HashMap<String, String> = HashMap::from_iter(
            form.fields
                .iter()
                .filter_map(|f| f.value.as_ref().map(|v| (f.name.clone(), v.clone()))),
        );
        if let Some(auth_type) = params.get_mut("auth_type") {
            *auth_type = "auth_mail".into();
        } else {
            return Err(LoginError::ParseError("auth_type not found in form"));
        }

        let res = self
            .client
            .post(form.action.as_ref().unwrap())
            .form(&params)
            .send()
            .await?;

        let res = res.error_for_status()?;
        let url = res.url().clone();
        let text = res.text().await?;

        let forms = extract_forms(&text, Some(&url));
        if forms.is_empty() {
            return Err(LoginError::ParseError(
                "No forms found in two-way auth response",
            ));
        }

        for form in forms {
            if form.action.as_ref().map_or(false, |a| a.contains("/do")) {
                return Ok(form);
            }
        }

        Err(LoginError::ParseError(
            "No valid form found in two-way auth response",
        ))
    }

    pub async fn two_way_auth(&self, form: &Form, code: &str) -> LoginResult<LoginStatus> {
        let mut params: HashMap<String, String> = HashMap::from_iter(
            form.fields
                .iter()
                .filter_map(|f| f.value.as_ref().map(|v| (f.name.clone(), v.clone()))),
        );
        if let Some(auth_code) = params.get_mut("auth_code") {
            *auth_code = code.to_string();
        } else {
            return Err(LoginError::ParseError("auth_code not found in form"));
        }

        let res = self
            .client
            .post(form.action.as_ref().unwrap())
            .form(&params)
            .send()
            .await?;

        let res = res.error_for_status()?;
        let url = res.url().clone();

        if url.as_str().starts_with(TOP_PAGE) {
            let text = res.text().await?;
            return Ok(LoginStatus::Success(text));
        }

        Err(LoginError::ParseError("Two-way auth failed"))
    }
}
