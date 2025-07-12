use std::sync::LazyLock;

pub static DB: LazyLock<sled::Db> =
    LazyLock::new(|| sled::open("data/db").expect("Failed to open database"));
