use crate::{
    data::DATA,
    login::LoginStatus,
    server::{ExtendResponse, get_server_id},
};

mod account;
mod client;
mod data;
mod form;
mod login;
mod server;

#[tokio::main]
async fn main() {
    let client = {
        let mut data = DATA.lock().expect("Failed to lock data");
        if !data.is_some() {
            let mut buf = String::new();
            println!("Please enter your email:");
            std::io::stdin()
                .read_line(&mut buf)
                .expect("Failed to read email");
            let email = buf.trim().to_string();
            buf.clear();
            println!("Please enter your password:");
            std::io::stdin()
                .read_line(&mut buf)
                .expect("Failed to read password");
            let password = buf.trim().to_string();
            let account = account::Account { email, password };
            data.save_account(account);
        }
        let data = data.unwrap();
        client::create_client(data.get_ua(), data.get_cookie())
    };

    let form = client.login_page().await.unwrap();

    let res = client
        .try_login(
            &form,
            DATA.lock()
                .expect("Failed to lock data")
                .unwrap()
                .get_account(),
        )
        .await
        .unwrap();

    let html = match res {
        LoginStatus::Success(text) => {
            println!("Login successful!");
            text
        }
        LoginStatus::Failure(msg) => {
            panic!("{:?}", msg);
        }
        LoginStatus::TowWayAuthRequired(form, email) => {
            if let Some(email) = email {
                println!("Two-way authentication required. Email: {}", email);
            } else {
                println!("Two-way authentication required.");
            }
            let form = client
                .two_way_select_email(&form)
                .await
                .expect("Failed to select email for two-way authentication");
            let code = {
                let mut buf = String::new();
                println!("Please enter the authentication code sent to your email:");
                std::io::stdin()
                    .read_line(&mut buf)
                    .expect("Failed to read authentication code");
                buf.trim().to_string()
            };
            let res = client
                .two_way_auth(&form, &code)
                .await
                .expect("Failed to complete two-way authentication");
            if let LoginStatus::Success(text) = res {
                println!("Login successful!");
                text
            } else {
                panic!("Two-way authentication failed.");
            }
        }
    };

    {
        let cookie = client.get_cookie();
        let mut data = DATA.lock().expect("Failed to lock data");
        data.save_cookie(cookie);
    }

    let vps = get_server_id(html.as_str());
    println!("VPS: {:?}", vps);

    if let Some(id) = vps {
        let extend_res = client.extend_vps(&id).await.expect("Failed to extend VPS");
        let res = client
            .submit_extend_form(&extend_res)
            .await
            .expect("Failed to submit extend form");
        match res {
            ExtendResponse::Success(msg) => {
                println!("Extend successful: {}", msg);
            }
            ExtendResponse::Failure(msg) => {
                println!("Extend failed: {}", msg);
            }
        }
    } else {
        println!("No VPS found.");
    }
}
