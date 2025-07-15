use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Interactive login and extend VPS
    Login,
    /// Extend VPS without interaction
    Extend {
        /// Run from systemd timer
        #[arg(long)]
        auto: bool,
    },
    /// Show stored account and run logs
    Status,
    /// Enable daily automatic extension
    Enable,
    /// Disable automatic extension
    Disable,
    /// Delete saved data
    Clear,
    /// Set Discord webhook URL
    Webhook { url: String },
    /// Update xrenew to the latest version
    Update {
        /// Run from systemd timer
        #[arg(long)]
        auto: bool,
    },
}
