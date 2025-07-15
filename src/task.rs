use crate::data::value::get_account;
use rand::Rng;

pub fn enable_auto() {
    {
        if get_account().is_none() {
            println!("No account configured. Run 'xrenew login' first.");
            return;
        }
    }

    let exe = std::env::current_exe().expect("get exe path");
    let service = include_str!("../systemd/xrenew.service")
        .replace("{{EXEC_PATH}}", exe.to_str().expect("exe path to str"));

    let mut rng = rand::rng();
    let hour: u8 = rng.random_range(0..12);
    let minute: u8 = rng.random_range(0..60);
    let timer = include_str!("../systemd/xrenew.timer")
        .replace("{{HOUR}}", &format!("{:02}", hour))
        .replace("{{MINUTE}}", &format!("{:02}", minute));
    let dir = directories::BaseDirs::new()
        .expect("get base dirs")
        .config_dir()
        .join("systemd/user");
    std::fs::create_dir_all(&dir).expect("create systemd dir");
    std::fs::write(dir.join("xrenew.service"), service).expect("write service");
    std::fs::write(dir.join("xrenew.timer"), timer).expect("write timer");
    let _ = std::process::Command::new("systemctl")
        .args(["--user", "daemon-reload"])
        .status();
    let _ = std::process::Command::new("systemctl")
        .args(["--user", "enable", "--now", "xrenew.timer"])
        .status();
    println!("Automatic extension enabled");
}

pub fn disable_auto() {
    let dir = directories::BaseDirs::new()
        .expect("get base dirs")
        .config_dir()
        .join("systemd/user");
    let _ = std::process::Command::new("systemctl")
        .args(["--user", "disable", "--now", "xrenew.timer"])
        .status();
    let _ = std::process::Command::new("systemctl")
        .args(["--user", "stop", "xrenew.timer"])
        .status();
    std::fs::remove_file(dir.join("xrenew.service")).ok();
    std::fs::remove_file(dir.join("xrenew.timer")).ok();
    println!("Automatic extension disabled");
}

pub fn refresh_auto() {
    let enabled = std::process::Command::new("systemctl")
        .args(["--user", "is-enabled", "xrenew.timer"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    if enabled {
        disable_auto();
        enable_auto();
        println!("Automatic extension refreshed");
    } else {
        println!("Automatic extension not configured");
    }
}

pub fn should_run() -> bool {
    if let Some((ts, _)) = crate::logger::read_logs()
        .iter()
        .rev()
        .find(|(_, m)| m.starts_with("SUCCESS"))
    {
        let diff = chrono::Local::now() - *ts;
        diff.num_hours() >= 23
    } else {
        true
    }
}
