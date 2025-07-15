use std::sync::LazyLock;

use crate::data::{SAVE_DIR, data_old::transfer_old_data};

use super::BIN_CONF;

pub static DB: LazyLock<sled::Db> = LazyLock::new(|| {
    let db_path = SAVE_DIR.join("xrenew.db");
    sled::open(db_path).expect("Failed to open database")
});

pub fn put<K, V>(key: K, value: &V)
where
    K: AsRef<[u8]>,
    V: bincode::Encode,
{
    let serialized_value =
        bincode::encode_to_vec(value, BIN_CONF).expect("Failed to serialize value");
    DB.insert(key.as_ref(), serialized_value)
        .expect("Failed to insert into database");
}

pub fn get<K, V>(key: K) -> Option<V>
where
    K: AsRef<[u8]>,
    V: bincode::Decode<()>,
{
    let val = DB.get(key.as_ref()).expect("Failed to get from database")?;
    let (data, _): (V, usize) =
        bincode::decode_from_slice(&val, BIN_CONF).expect("Failed to deserialize value");
    Some(data)
}

pub fn remove<K>(key: K) -> bool
where
    K: AsRef<[u8]>,
{
    DB.remove(key.as_ref())
        .expect("Failed to remove from database")
        .is_some()
}

pub fn initialize_db() {
    transfer_old_data();
}

pub fn remove_all() {
    DB.clear().expect("Failed to clear database");
}
