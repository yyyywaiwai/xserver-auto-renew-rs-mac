use std::time::Duration;
use tokio::time::sleep;

use crate::{
    cli::{Cli, Commands},
    client::{
        Account, CaptchaResponse, DEFAULT_CLIENT, ExtendResponse, LoginStatus, get_server_id,
        save_default_client,
    },
    data::{
        initialize_db,
        value::{get_account, set_account},
    },
    external::send_webhook,
};
use clap::Parser;

use crate::external::solve_captcha;

mod cli;
mod client;
mod data;
mod external;
mod logger;
mod ops;
mod task;
mod update;

use ops::{clear_data, set_webhook, show_status};
use task::{disable_auto, enable_auto, refresh_auto, should_run};
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
    if !matches!(cli.command, Commands::Refresh) {
        initialize_db();
    }
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
        Commands::Webhook { url } => set_webhook(&url),
        Commands::Update { auto } => update(auto).await,
        Commands::Refresh => refresh_auto(),
    }
}

async fn login_flow() {
    // handle account input/update
    {
        if let Some(account) = get_account() {
            println!("Current account: {}", account.email);
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
                let acc = Account { email, password };
                set_account(&acc);
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
            let acc = Account { email, password };
            set_account(&acc);
        }
    }

    match do_login_and_extend_with_retry(&DEFAULT_CLIENT, true).await {
        Ok(msg) => {
            logger::log_message(&format!("SUCCESS {}", msg)).await;
            send_webhook(&format!("Extend successful: {}", msg)).await;
        }
        Err(e) => {
            logger::log_message(&format!("FAILURE {}", e)).await;
            send_webhook(&format!("Extend failed: {}", e)).await;
        }
    }
}

async fn extend_flow(auto: bool) {
    if auto && !should_run() {
        let msg = "Skip: last success within 23h";
        logger::log_message(msg).await;
        send_webhook(msg).await;
        return;
    }

    match do_login_and_extend_with_retry(&DEFAULT_CLIENT, false).await {
        Ok(msg) => {
            logger::log_message(&format!("SUCCESS {}", msg)).await;
            send_webhook(&format!("Extend successful: {}", msg)).await;
        }
        Err(e) => {
            logger::log_message(&format!("FAILURE {}", e)).await;
            send_webhook(&format!("Extend failed: {}", e)).await;
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
            Err(ExtendError::CaptchaFailure(msg)) if attempts < 2 => {
                println!("Captcha failed. ({})", msg);
                attempts += 1;
                println!("Retrying in 60 seconds... ({}/{})", attempts + 1, 3);
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
    let account =
        get_account().ok_or_else(|| ExtendError::Other("No account found".to_string()))?;
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

    save_default_client();

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
