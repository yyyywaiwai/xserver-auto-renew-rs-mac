use crate::{
    data::{self, remove_all},
    logger,
};

pub fn show_status() {
    if let Some(account) = data::value::get_account() {
        println!("Current account: {}", account.email);
    } else {
        println!("No account configured");
    }
    if let Some(webhook) = data::value::get_webhook() {
        println!("Webhook: {}", webhook);
    }
    if let Some(ua) = data::value::get_ua() {
        println!("User-Agent: {}", ua);
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
    remove_all();
    println!("Saved data deleted");
}

pub fn set_webhook(url: &String) {
    data::value::set_webhook(url);
    println!("Webhook set");
}

pub fn set_two_captcha_key(key: &String) {
    data::value::set_two_captcha_key(key);
    println!("TwoCaptcha API key set");
}
