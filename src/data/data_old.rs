use std::{
    io::BufReader,
    path::{Path, PathBuf},
};

use bincode::{
    Decode, Encode,
    config::{Configuration, standard},
};

use crate::{
    client::Account,
    data::value::{set_account, set_cookie, set_ua, set_webhook},
};

use super::SAVE_DIR;

#[derive(Clone, Debug, Encode, Decode)]
pub struct OldDataV1 {
    account: Account,
    ua: String,
    cookie: Option<String>,
}

#[derive(Clone, Debug, Encode, Decode)]
pub struct OldDataV2 {
    account: Account,
    ua: String,
    cookie: Option<String>,
    webhook: Option<String>,
}

const CONF: Configuration = standard();

pub fn old_save_path() -> PathBuf {
    SAVE_DIR.join("data.bin")
}

pub enum OldData {
    V1(OldDataV1),
    V2(OldDataV2),
    None,
}

pub fn load_old_data(path: &Path) -> OldData {
    if std::fs::metadata(path).is_ok() {
        let mut reader =
            BufReader::new(std::fs::File::open(path).expect("Failed to open old data file"));
        if let Ok(data) = bincode::decode_from_std_read::<OldDataV1, _, _>(&mut reader, CONF) {
            return OldData::V1(data);
        } else if let Ok(data) = bincode::decode_from_std_read::<OldDataV2, _, _>(&mut reader, CONF)
        {
            return OldData::V2(data);
        } else {
            eprintln!("Failed to decode old data");
            return OldData::None;
        }
    } else {
        return OldData::None;
    }
}

pub fn transfer_old_data() {
    let path = old_save_path();
    let old_data = load_old_data(&path);
    match old_data {
        OldData::V1(data) => {
            set_account(&data.account);
            if let Some(cookie) = data.cookie {
                set_cookie(&cookie);
            }
            set_ua(&data.ua);
        }
        OldData::V2(data) => {
            set_account(&data.account);
            if let Some(cookie) = data.cookie {
                set_cookie(&cookie);
            }
            set_ua(&data.ua);
            if let Some(webhook) = data.webhook {
                set_webhook(&webhook);
            }
        }
        OldData::None => return,
    }
    println!("Old data transferred successfully");
    std::fs::remove_file(&path).ok();
}
