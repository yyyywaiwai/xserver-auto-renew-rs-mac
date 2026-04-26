use crate::data::value::get_account;
use rand::Rng;
use std::path::{Path, PathBuf};

struct MacosLaunchdUser {
    username: String,
    uid: String,
    home_dir: PathBuf,
}

fn command_stdout(command: &mut std::process::Command) -> Option<String> {
    let output = command.output().ok()?;
    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8(output.stdout).ok()?;
    let stdout = stdout.trim();
    if stdout.is_empty() {
        None
    } else {
        Some(stdout.to_owned())
    }
}

fn macos_launchd_user() -> Option<MacosLaunchdUser> {
    let username = std::env::var("SUDO_USER")
        .ok()
        .filter(|user| user != "root")
        .or_else(|| command_stdout(std::process::Command::new("id").arg("-un")))?;

    let uid = command_stdout(std::process::Command::new("id").args(["-u", &username]))?;
    let home_dir = command_stdout(std::process::Command::new("dscl").args([
        ".",
        "-read",
        &format!("/Users/{}", username),
        "NFSHomeDirectory",
    ]))
    .and_then(|line| {
        line.split_once(": ")
            .map(|(_, home)| PathBuf::from(home.trim()))
    })
    .or_else(|| {
        std::env::var("HOME")
            .ok()
            .filter(|home| {
                !home.is_empty() && username == std::env::var("USER").unwrap_or_default()
            })
            .map(PathBuf::from)
    })?;

    Some(MacosLaunchdUser {
        username,
        uid,
        home_dir,
    })
}

fn launchctl_domain_target(user: &MacosLaunchdUser) -> String {
    format!("gui/{}", user.uid)
}

fn run_launchctl(args: &[&str]) -> bool {
    std::process::Command::new("launchctl")
        .args(args)
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

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
    let Some(user) = macos_launchd_user() else {
        println!("Could not determine the macOS user for launchd");
        return;
    };

    let log_dir = user.home_dir.join(".local/share/xrenew");
    let user_bin_path = user.home_dir.join(".local/bin");

    std::fs::create_dir_all(&log_dir).expect("create log dir");

    let plist_content = include_str!("../launchd/com.xrenew.timer.plist")
        .replace("{{EXEC_PATH}}", exe.to_str().expect("exe path to str"))
        .replace("{{LOG_PATH}}", log_dir.to_str().expect("log path to str"))
        .replace(
            "{{USER_BIN_PATH}}",
            user_bin_path.to_str().expect("user bin path to str"),
        );

    let launch_agents_dir = user.home_dir.join("Library/LaunchAgents");
    std::fs::create_dir_all(&launch_agents_dir).expect("create LaunchAgents dir");

    let plist_path = launch_agents_dir.join("com.xrenew.timer.plist");
    std::fs::write(&plist_path, plist_content).expect("write plist file");

    if std::env::var("SUDO_USER").is_ok() {
        let _ = std::process::Command::new("chown")
            .args([
                &format!("{}:staff", user.username),
                plist_path.to_str().expect("plist path to str"),
            ])
            .status();
    }

    let domain_target = launchctl_domain_target(&user);
    let service_target = format!("{}/com.xrenew.timer", domain_target);
    let plist_path_string = plist_path.to_string_lossy().to_string();

    let _ = run_launchctl(&["bootout", &domain_target, &plist_path_string]);
    if !run_launchctl(&["bootstrap", &domain_target, &plist_path_string]) {
        println!("Could not enable service: launchctl bootstrap failed");
        return;
    }
    if !run_launchctl(&["enable", &service_target]) {
        println!("Could not enable service: launchctl enable failed");
        return;
    }

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
    let Some(user) = macos_launchd_user() else {
        println!("Could not determine the macOS user for launchd");
        return;
    };
    let plist_path = user
        .home_dir
        .join("Library/LaunchAgents/com.xrenew.timer.plist");

    let domain_target = launchctl_domain_target(&user);
    let service_target = format!("{}/com.xrenew.timer", domain_target);
    let _ = run_launchctl(&["disable", &service_target]);

    if let Some(plist_path) = plist_path.to_str() {
        let _ = run_launchctl(&["bootout", &domain_target, plist_path]);
    } else {
        println!("Warning: failed to convert plist path for launchctl");
    }

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
