pub async fn update(auto: bool) {
    let current = semver::Version::parse(env!("CARGO_PKG_VERSION")).unwrap();
    let client = reqwest::Client::new();
    let res = client
        .get("https://api.github.com/repos/h-sumiya/xserver-auto-renew-rs/releases/latest")
        .header(reqwest::header::USER_AGENT, "xrenew")
        .send()
        .await;
    let Ok(resp) = res else {
        if !auto {
            eprintln!("Failed to check latest version");
        }
        return;
    };
    let json: serde_json::Value = match resp.json().await {
        Ok(j) => j,
        Err(e) => {
            if !auto {
                eprintln!("Failed to parse version info: {}", e);
            }
            return;
        }
    };
    let tag = json.get("tag_name").and_then(|v| v.as_str());
    let Some(tag) = tag else {
        return;
    };
    let latest_str = tag.trim_start_matches('v');
    let Ok(latest) = semver::Version::parse(latest_str) else {
        return;
    };
    if latest > current {
        if !auto {
            println!("Updating from {} to {}", current, latest);
        }
        let cmd = format!(
            "curl -sSf https://raw.githubusercontent.com/h-sumiya/xserver-auto-renew-rs/main/install.sh | VERSION={} bash",
            tag
        );
        let status = std::process::Command::new("bash")
            .arg("-c")
            .arg(cmd)
            .status();
        if status.map(|s| s.success()).unwrap_or(false) {
            if !auto {
                println!("Update complete");
            }
        } else if !auto {
            eprintln!("Update failed");
        }
    } else if !auto {
        println!("xrenew is up to date ({}).", current);
    }
}
