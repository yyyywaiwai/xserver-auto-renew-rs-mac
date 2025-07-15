### Repository Description: xserver-auto-renew-rs

#### **Objective**

This program is a command-line tool designed to automate the process of renewing contracts for services on Xserver, a Japanese hosting provider. It handles logging in, solving CAPTCHAs, and performing the renewal actions automatically.

#### **Project Structure (Key Files)**

- `src/main.rs`: The main entry point for the command-line application. It parses arguments and orchestrates the renewal process.
- `src/client.rs`: A module responsible for handling all HTTP communications. It manages a `reqwest` client instance with a cookie store to maintain session state across requests.
- `src/login.rs`: Contains the logic for the user login process, including handling two-factor authentication if required.
- `src/data/`: This directory contains modules for managing the application's persistent data, such as user account credentials, webhook configurations, and other settings stored locally.
- `src/external/`: Encapsulates logic for interacting with third-party services, such as a CAPTCHA solving service and sending notifications via webhooks.
- `Cargo.toml`: The Rust project manifest. It defines project metadata and lists all dependencies.

#### **Development Guidelines**

1.  **Dependencies**: Always use `cargo add <crate-name>` to add a new library. This ensures the latest version is added to `Cargo.toml`.
2.  **Code Formatting**: Run `cargo fmt` to format the code before finalizing any changes.
3.  **Rust Edition**: This project is locked to the `2024` edition. Do not downgrade it.
