use std::time::Duration;
use tokio::time::sleep;

use crate::cli::{Cli, Commands};
use clap::Parser;

use crate::{
    captcha::solve_captcha,
    data::DATA,
    login::LoginStatus,
    server::{CaptchaResponse, ExtendResponse, get_server_id},
};

mod account;
mod captcha;
mod cli;
mod client;
mod data;
mod form;
mod logger;
mod login;
mod ops;
mod server;
mod task;
mod update;
mod webhook;

use ops::{clear_data, set_webhook, show_status};
use task::{disable_auto, enable_auto, should_run};
use update::update;

#[derive(Debug)]
enum ExtendError {
    CaptchaFailure(String),
    Other(String),
}

impl std::fmt::Display for ExtendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExtendError::CaptchaFailure(msg) | ExtendError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for ExtendError {}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Login => login_flow().await,
        Commands::Extend { auto } => extend_flow(auto).await,
        Commands::Status => {
            show_status();
            return;
        }
        Commands::Enable => enable_auto(),
        Commands::Disable => disable_auto(),
        Commands::Clear => clear_data(),
        Commands::Webhook { url } => set_webhook(url),
        Commands::Update { auto } => update(auto).await,
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
    match do_login_and_extend_with_retry(&client, true).await {
        Ok(msg) => {
            logger::log_message(&format!("SUCCESS {}", msg));
            webhook::send(&format!("Extend successful: {}", msg)).await;
        }
        Err(e) => {
            logger::log_message(&format!("FAILURE {}", e));
            webhook::send(&format!("Extend failed: {}", e)).await;
        }
    }
}

async fn extend_flow(auto: bool) {
    if auto && !should_run() {
        let msg = "Skip: last success within 23h";
        logger::log_message(msg);
        webhook::send(msg).await;
        return;
    }
    let client = create_client().await;
    match do_login_and_extend_with_retry(&client, false).await {
        Ok(msg) => {
            logger::log_message(&format!("SUCCESS {}", msg));
            webhook::send(&format!("Extend successful: {}", msg)).await;
        }
        Err(e) => {
            logger::log_message(&format!("FAILURE {}", e));
            webhook::send(&format!("Extend failed: {}", e)).await;
        }
    }
}

async fn do_login_and_extend_with_retry(
    client: &client::Client,
    interactive: bool,
) -> Result<String, ExtendError> {
    let mut attempts = 0;
    loop {
        match do_login_and_extend(client, interactive).await {
            Ok(msg) => return Ok(msg),
            Err(ExtendError::CaptchaFailure(_msg)) if attempts < 2 => {
                attempts += 1;
                println!(
                    "Captcha failed. Retrying in 60 seconds... ({}/{})",
                    attempts + 1,
                    3
                );
                sleep(Duration::from_secs(60)).await;
            }
            Err(e) => return Err(e),
        }
    }
}

async fn do_login_and_extend(
    client: &client::Client,
    interactive: bool,
) -> Result<String, ExtendError> {
    let form = client
        .login_page()
        .await
        .map_err(|e| ExtendError::Other(format!("login page: {}", e)))?;
    let account = {
        let data = DATA.lock().unwrap();
        data.unwrap().get_account().clone()
    };
    let login_res = client
        .try_login(&form, &account)
        .await
        .map_err(|e| ExtendError::Other(format!("login: {}", e)))?;
    let html;
    match login_res {
        LoginStatus::Success(text) => html = text,
        LoginStatus::Failure(msg) => return Err(ExtendError::Other(msg)),
        LoginStatus::TowWayAuthRequired(form, email) => {
            if !interactive {
                return Err(ExtendError::Other("Two-way authentication required".into()));
            }
            if let Some(email) = email {
                println!("Two-way authentication required. Email: {}", email);
            } else {
                println!("Two-way authentication required.");
            }
            let form = client
                .two_way_select_email(&form)
                .await
                .map_err(|e| ExtendError::Other(format!("auth select: {}", e)))?;
            let code = {
                let mut buf = String::new();
                println!("Please enter the authentication code sent to your email:");
                std::io::stdin().read_line(&mut buf).unwrap();
                buf.trim().to_string()
            };
            match client
                .two_way_auth(&form, &code)
                .await
                .map_err(|e| ExtendError::Other(format!("two-way auth: {}", e)))?
            {
                LoginStatus::Success(text) => {
                    html = text;
                }
                _ => return Err(ExtendError::Other("Two-way authentication failed".into())),
            }
        }
    }

    {
        let cookie = client.get_cookie();
        let mut data = DATA.lock().unwrap();
        data.save_cookie(cookie);
    }

    let vps = get_server_id(&html).ok_or_else(|| ExtendError::Other("No VPS found".to_string()))?;
    let extend_form = client
        .extend_vps(&vps)
        .await
        .map_err(|e| ExtendError::Other(format!("extend vps: {}", e)))?;
    match client
        .submit_extend_form(&extend_form)
        .await
        .map_err(|e| ExtendError::Other(format!("submit extend: {}", e)))?
    {
        ExtendResponse::Success(msg) => {
            println!("Extend successful: {}", msg);
            Ok(msg)
        }
        ExtendResponse::Failure(msg) => {
            println!("Extend failed: {}", msg);
            Err(ExtendError::Other(msg))
        }
        ExtendResponse::CaptchaRequired(captcha) => {
            println!("Captcha required (Solving...)");
            let res = solve_captcha(&captcha)
                .await
                .map_err(|e| ExtendError::CaptchaFailure(format!("Captcha solve: {}", e)))?;
            let res = client
                .submit_captcha(&captcha, res)
                .await
                .map_err(|e| ExtendError::CaptchaFailure(format!("Captcha submit: {}", e)))?;
            match res {
                CaptchaResponse::Success(msg) => {
                    println!("Extend successful(with captcha): {}", msg);
                    Ok(msg)
                }
                CaptchaResponse::Failure(msg) => {
                    println!("Extend failed(with captcha): {}", msg);
                    Err(ExtendError::CaptchaFailure(msg))
                }
            }
        }
    }
}
