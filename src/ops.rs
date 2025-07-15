use crate::{data::DATA, logger};

pub fn show_status() {
    let data = DATA.lock().unwrap();
    if data.is_some() {
        let d = data.unwrap();
        println!("Account: {}", d.get_account().email);
        if d.get_webhook().is_some() {
            println!("Webhook: set");
        }
    } else {
        println!("No account configured");
    }
    let timer_enabled = std::process::Command::new("systemctl")
        .args(["--user", "is-enabled", "xrenew.timer"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    println!(
        "Auto update: {}",
        if timer_enabled { "enabled" } else { "disabled" }
    );
    let logs = logger::read_logs();
    if let Some((ts, msg)) = logs.last() {
        println!("Last run: {} - {}", ts.format("%Y-%m-%d %H:%M:%S"), msg);
    }
    if let Some((ts, _)) = logs.iter().rev().find(|(_, m)| m.starts_with("SUCCESS")) {
        println!("Last success: {}", ts.format("%Y-%m-%d %H:%M:%S"));
    }
}

pub fn clear_data() {
    let mut data = DATA.lock().unwrap();
    if data.is_some() {
        data.clear();
        println!("Saved data deleted");
    } else {
        println!("No saved data");
    }
}

pub fn set_webhook(url: String) {
    let mut data = DATA.lock().unwrap();
    if data.is_some() {
        data.save_webhook(Some(url));
        println!("Webhook set");
    } else {
        println!("No account configured. Run 'xrenew login' first.");
    }
}
