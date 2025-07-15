use std::{
    path::{Path, PathBuf},
    sync::LazyLock,
};

use directories::ProjectDirs;

pub static SAVE_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    if cfg!(target_os = "windows") && Path::new("data").is_dir() {
        PathBuf::from("data")
    } else {
        let d = ProjectDirs::from("", "", "xrenew").map(|p| p.data_dir().to_owned());
        if let Some(d) = d {
            std::fs::create_dir_all(&d).ok();
            d
        } else {
            panic!("Failed to determine save directory");
        }
    }
});
