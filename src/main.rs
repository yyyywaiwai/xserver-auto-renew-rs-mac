use clap::{Parser, Subcommand};

use crate::{
    data::DATA,
    login::LoginStatus,
    server::{ExtendResponse, get_server_id},
};

mod account;
mod client;
mod data;
mod form;
mod logger;
mod login;
mod server;

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Interactive login and extend VPS
    Login,
    /// Extend VPS without interaction
    Extend,
    /// Show stored account and run logs
    Status,
    /// Enable daily automatic extension
    Enable,
    /// Disable automatic extension
    Disable,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Login => login_flow().await,
        Commands::Extend => extend_flow().await,
        Commands::Status => {
            show_status();
            return;
        }
        Commands::Enable => enable_auto(),
        Commands::Disable => disable_auto(),
    }
}

async fn create_client() -> client::Client {
    let data = DATA.lock().expect("lock data");
    let data = data.unwrap();
    client::create_client(data.get_ua(), data.get_cookie())
}

async fn login_flow() {
    // handle account input/update
    {
        let mut data = DATA.lock().expect("lock data");
        if data.is_some() {
            let email = data.unwrap().get_account().email.clone();
            println!("Current account: {}", email);
            println!("Update credentials? (y/N)");
            let mut buf = String::new();
            std::io::stdin().read_line(&mut buf).unwrap();
            if buf.trim().eq_ignore_ascii_case("y") {
                buf.clear();
                println!("Please enter your email:");
                std::io::stdin().read_line(&mut buf).unwrap();
                let email = buf.trim().to_string();
                buf.clear();
                println!("Please enter your password:");
                std::io::stdin().read_line(&mut buf).unwrap();
                let password = buf.trim().to_string();
                let acc = account::Account { email, password };
                data.save_account(acc);
            }
        } else {
            let mut buf = String::new();
            println!("Please enter your email:");
            std::io::stdin().read_line(&mut buf).unwrap();
            let email = buf.trim().to_string();
            buf.clear();
            println!("Please enter your password:");
            std::io::stdin().read_line(&mut buf).unwrap();
            let password = buf.trim().to_string();
            let acc = account::Account { email, password };
            data.save_account(acc);
        }
    }

    let client = create_client().await;
    if let Err(e) = do_login_and_extend(&client, true).await {
        logger::log_message(&format!("FAILURE {}", e));
    }
}

async fn extend_flow() {
    let client = create_client().await;
    if let Err(e) = do_login_and_extend(&client, false).await {
        logger::log_message(&format!("FAILURE {}", e));
    }
}

async fn do_login_and_extend(client: &client::Client, interactive: bool) -> Result<(), String> {
    let form = client
        .login_page()
        .await
        .map_err(|e| format!("login page: {}", e))?;
    let account = {
        let data = DATA.lock().unwrap();
        data.unwrap().get_account().clone()
    };
    let login_res = client
        .try_login(&form, &account)
        .await
        .map_err(|e| format!("login: {}", e))?;
    let html;
    match login_res {
        LoginStatus::Success(text) => html = text,
        LoginStatus::Failure(msg) => return Err(msg),
        LoginStatus::TowWayAuthRequired(form, email) => {
            if !interactive {
                return Err("Two-way authentication required".into());
            }
            if let Some(email) = email {
                println!("Two-way authentication required. Email: {}", email);
            } else {
                println!("Two-way authentication required.");
            }
            let form = client
                .two_way_select_email(&form)
                .await
                .map_err(|e| format!("auth select: {}", e))?;
            let code = {
                let mut buf = String::new();
                println!("Please enter the authentication code sent to your email:");
                std::io::stdin().read_line(&mut buf).unwrap();
                buf.trim().to_string()
            };
            match client
                .two_way_auth(&form, &code)
                .await
                .map_err(|e| format!("two-way auth: {}", e))?
            {
                LoginStatus::Success(text) => {
                    html = text;
                }
                _ => return Err("Two-way authentication failed".into()),
            }
        }
    }

    {
        let cookie = client.get_cookie();
        let mut data = DATA.lock().unwrap();
        data.save_cookie(cookie);
    }

    let vps = get_server_id(&html).ok_or_else(|| "No VPS found".to_string())?;
    let extend_form = client
        .extend_vps(&vps)
        .await
        .map_err(|e| format!("extend vps: {}", e))?;
    match client
        .submit_extend_form(&extend_form)
        .await
        .map_err(|e| format!("submit extend: {}", e))?
    {
        ExtendResponse::Success(msg) => {
            println!("Extend successful: {}", msg);
            logger::log_message(&format!("SUCCESS {}", msg));
            Ok(())
        }
        ExtendResponse::Failure(msg) => {
            println!("Extend failed: {}", msg);
            Err(msg)
        }
    }
}

fn show_status() {
    let data = DATA.lock().unwrap();
    if data.is_some() {
        println!("Account: {}", data.unwrap().get_account().email);
    } else {
        println!("No account configured");
    }
    let logs = logger::read_logs();
    if let Some((ts, msg)) = logs.last() {
        println!("Last run: {} - {}", ts.format("%Y-%m-%d %H:%M:%S"), msg);
    }
    if let Some((ts, _)) = logs.iter().rev().find(|(_, m)| m.starts_with("SUCCESS")) {
        println!("Last success: {}", ts.format("%Y-%m-%d %H:%M:%S"));
    }
}

fn enable_auto() {
    {
        let data = DATA.lock().unwrap();
        if !data.is_some() {
            println!("No account configured. Run 'xrenew login' first.");
            return;
        }
    }

    let exe = std::env::current_exe().expect("get exe path");
    let service = include_str!("../systemd/xrenew.service")
        .replace("{{EXEC_PATH}}", exe.to_str().expect("exe path to str"));
    let timer = include_str!("../systemd/xrenew.timer");
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

fn disable_auto() {
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
