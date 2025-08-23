use crate::data::value::get_account;
use rand::Rng;
use std::path::Path;

pub fn enable_auto() {
    {
        if get_account().is_none() {
            println!("No account configured. Run 'xrenew login' first.");
            return;
        }
    }
    _enable_auto();
}

fn _enable_auto() {
    let exe = std::env::current_exe().expect("get exe path");
    
    if cfg!(target_os = "macos") {
        _enable_auto_macos(&exe);
    } else if cfg!(target_os = "linux") {
        _enable_auto_linux(&exe);
    } else {
        println!("Unsupported operating system");
    }
}

fn _enable_auto_linux(exe: &Path) {
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

fn _enable_auto_macos(exe: &Path) {
    let home_dir = std::env::var("HOME").expect("HOME environment variable not set");
    let log_dir = format!("{}/.local/share/xrenew", home_dir);
    let user_bin_path = format!("{}/.local/bin", home_dir);
    
    std::fs::create_dir_all(&log_dir).expect("create log dir");
    
    let plist_content = include_str!("../launchd/com.xrenew.timer.plist")
        .replace("{{EXEC_PATH}}", exe.to_str().expect("exe path to str"))
        .replace("{{LOG_PATH}}", &log_dir)
        .replace("{{USER_BIN_PATH}}", &user_bin_path);
    
    let launch_agents_dir = format!("{}/Library/LaunchAgents", home_dir);
    std::fs::create_dir_all(&launch_agents_dir).expect("create LaunchAgents dir");
    
    let plist_path = format!("{}/com.xrenew.timer.plist", launch_agents_dir);
    std::fs::write(&plist_path, plist_content).expect("write plist file");
    
    let _ = std::process::Command::new("launchctl")
        .args(["bootstrap", "gui/501", &plist_path])
        .status();
    
    let _ = std::process::Command::new("launchctl")
        .args(["enable", "gui/501/com.xrenew.timer"])
        .status();
    
    println!("Automatic extension enabled (macOS)");
}

pub fn disable_auto() {
    if cfg!(target_os = "macos") {
        _disable_auto_macos();
    } else if cfg!(target_os = "linux") {
        _disable_auto_linux();
    } else {
        println!("Unsupported operating system");
    }
}

fn _disable_auto_linux() {
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

fn _disable_auto_macos() {
    let home_dir = std::env::var("HOME").expect("HOME environment variable not set");
    let plist_path = format!("{}/Library/LaunchAgents/com.xrenew.timer.plist", home_dir);
    
    let _ = std::process::Command::new("launchctl")
        .args(["disable", "gui/501/com.xrenew.timer"])
        .status();
    
    let _ = std::process::Command::new("launchctl")
        .args(["bootout", "gui/501", &plist_path])
        .status();
    
    std::fs::remove_file(&plist_path).ok();
    println!("Automatic extension disabled (macOS)");
}

pub fn refresh_auto() {
    let enabled = is_auto_enabled();
    if enabled {
        disable_auto();
        _enable_auto();
        println!("Automatic extension refreshed");
    } else {
        println!("Automatic extension not configured");
    }
}

pub fn is_auto_enabled() -> bool {
    if cfg!(target_os = "macos") {
        _is_auto_enabled_macos()
    } else if cfg!(target_os = "linux") {
        _is_auto_enabled_linux()
    } else {
        false
    }
}

fn _is_auto_enabled_linux() -> bool {
    std::process::Command::new("systemctl")
        .args(["--user", "is-enabled", "xrenew.timer"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn _is_auto_enabled_macos() -> bool {
    let home_dir = std::env::var("HOME").unwrap_or_default();
    let plist_path = format!("{}/Library/LaunchAgents/com.xrenew.timer.plist", home_dir);
    Path::new(&plist_path).exists()
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
