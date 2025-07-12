use std::sync::{Arc, LazyLock};

use cookie_store::CookieStore;
use reqwest_cookie_store::CookieStoreMutex;
use ua_generator::ua::spoof_ua;

use crate::db::DB;

pub static COOKIE: LazyLock<Arc<CookieStoreMutex>> = LazyLock::new(|| {
    let text = DB.get("cookie").unwrap();
    Arc::new(CookieStoreMutex::new(if let Some(text) = text {
        let reader = std::io::Cursor::new(text);
        cookie_store::serde::json::load(reader).expect("Failed to load cookie store from database")
    } else {
        CookieStore::default()
    }))
});

pub fn save_cookie() -> sled::Result<()> {
    let mut writer = Vec::new();
    let cookie_store = COOKIE.lock().expect("Failed to lock cookie store");
    cookie_store::serde::json::save(&*cookie_store, &mut writer)
        .expect("Failed to save cookie store to JSON");
    DB.insert("cookie", writer)?;
    DB.flush()?;
    Ok(())
}

pub static UA: LazyLock<String> = LazyLock::new(|| match DB.get("user_agent").unwrap() {
    Some(text) => {
        String::from_utf8(text.to_vec()).expect("Failed to parse user agent from database")
    }
    None => spoof_ua().to_string(),
});

pub static CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
    let cookie_store = Arc::clone(&COOKIE);
    let ua = UA.clone();
    reqwest::Client::builder()
        .cookie_provider(cookie_store)
        .default_headers({
            let mut headers = reqwest::header::HeaderMap::new();
            headers.insert(reqwest::header::USER_AGENT, ua.parse().unwrap());
            headers.insert(
                reqwest::header::ACCEPT_LANGUAGE,
                "ja,en-US;q=0.9,en;q=0.8".parse().unwrap(),
            );
            headers
        })
        .build()
        .expect("Failed to build HTTP client")
});
