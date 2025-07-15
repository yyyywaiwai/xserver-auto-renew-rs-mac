use std::sync::{Arc, LazyLock};

use cookie_store::CookieStore;
use reqwest_cookie_store::CookieStoreMutex;
use ua_generator::ua::spoof_ua;

use crate::data::value::{get_cookie, get_ua, set_cookie, set_ua};

pub struct Client {
    cookie_store: Arc<CookieStoreMutex>,
    pub client: reqwest::Client,
}

pub fn create_client(ua: String, cookie: Option<String>) -> Client {
    let cookie_store = match cookie {
        Some(cookie) => {
            let store = CookieStore::default();
            let mut reader = std::io::Cursor::new(cookie);
            cookie_store::serde::json::load(&mut reader).expect("Failed to load cookie store");
            Arc::new(CookieStoreMutex::new(store))
        }
        None => Arc::new(CookieStoreMutex::new(CookieStore::default())),
    };
    let ua = ua.clone();
    let client = reqwest::Client::builder()
        .cookie_provider(cookie_store.clone())
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
        .expect("Failed to build HTTP client");

    Client {
        cookie_store,
        client,
    }
}

impl Client {
    pub fn get_cookie(&self) -> String {
        let store = self
            .cookie_store
            .lock()
            .expect("Failed to lock cookie store");
        let mut writer = Vec::new();
        cookie_store::serde::json::save(&*store, &mut writer)
            .expect("Failed to save cookie store to JSON");
        String::from_utf8(writer).expect("Failed to convert cookie store to string")
    }
}

pub static DEFAULT_CLIENT: LazyLock<Client> = LazyLock::new(|| {
    let ua = match get_ua() {
        Some(ua) => ua,
        None => {
            let ua = spoof_ua().to_string();
            set_ua(&ua);
            ua
        }
    };
    let cookie = get_cookie();
    create_client(ua, cookie)
});

pub fn save_default_client() {
    let cookie = DEFAULT_CLIENT.get_cookie();
    set_cookie(&cookie);
}
