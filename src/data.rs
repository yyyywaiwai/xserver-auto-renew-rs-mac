use std::{
    io::BufReader,
    sync::{LazyLock, Mutex},
};

use bincode::{
    Decode, Encode,
    config::{Configuration, standard},
};
use ua_generator::ua::spoof_ua;

use crate::account::Account;

#[derive(Clone, Debug, Encode, Decode)]
pub struct Data {
    account: Account,
    ua: String,
    cookie: Option<String>,
}

const CONF: Configuration = standard();
const SAVE_PATH: &str = "data/data.bin";

pub struct OptionData(Option<Data>);

pub static DATA: LazyLock<Mutex<OptionData>> = LazyLock::new(|| {
    Mutex::new(OptionData(if std::fs::metadata(SAVE_PATH).is_ok() {
        let reader =
            BufReader::new(std::fs::File::open(SAVE_PATH).expect("Failed to open data file"));
        let data: Data =
            bincode::decode_from_reader(reader, CONF).expect("Failed to decode data file");
        Some(data)
    } else {
        None
    }))
});

impl OptionData {
    fn save(&self) {
        if let Some(ref data) = self.0 {
            let mut writer = Vec::new();
            bincode::encode_into_std_write(data, &mut writer, CONF).expect("Failed to encode data");
            std::fs::write(SAVE_PATH, writer).expect("Failed to write data file");
        } else {
            std::fs::remove_file(SAVE_PATH).ok(); // Remove file if no data
        }
    }

    pub fn save_account(&mut self, account: Account) {
        self.0 = Some(Data {
            account,
            ua: spoof_ua().to_string(),
            cookie: None,
        });
        self.save();
    }

    pub fn save_cookie(&mut self, cookie: String) {
        if let Some(ref mut data) = self.0 {
            data.cookie = Some(cookie);
        } else {
            panic!("No data to save cookie to");
        }
        self.save();
    }

    pub fn unwrap(&self) -> &Data {
        self.0.as_ref().expect("Data is not initialized")
    }

    pub fn is_some(&self) -> bool {
        self.0.is_some()
    }
}

impl Data {
    pub fn get_account(&self) -> &Account {
        &self.account
    }

    pub fn get_ua(&self) -> String {
        self.ua.clone()
    }

    pub fn get_cookie(&self) -> Option<String> {
        self.cookie.clone()
    }
}
