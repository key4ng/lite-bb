use anyhow::Result;
use dialoguer::{Input, Password, Select};

use bb_core::api::ApiClient;
use bb_core::config::Config;

pub async fn run() -> Result<()> {
    let mut config = Config::load().unwrap_or_default();

    let server_types = &["Bitbucket Cloud", "Bitbucket Server / Data Center"];
    let server_selection = Select::new()
        .with_prompt("Which Bitbucket instance?")
        .items(server_types)
        .default(0)
        .interact()?;

    match server_selection {
        0 => {
            config.server_url = None;
        }
        1 => {
            let url: String = if let Some(default_url) = bb_core::git::server_url_from_remote() {
                Input::new()
                    .with_prompt("Server URL")
                    .default(default_url)
                    .interact_text()?
            } else {
                Input::new()
                    .with_prompt("Server URL (e.g. https://bitbucket.company.com)")
                    .interact_text()?
            };
            config.server_url = Some(url.trim_end_matches('/').to_string());
        }
        _ => unreachable!(),
    }

    let auth_types = &["Access Token", "App Password"];
    let selection = Select::new()
        .with_prompt("What type of credentials?")
        .items(auth_types)
        .default(0)
        .interact()?;

    match selection {
        0 => {
            let token = Password::new()
                .with_prompt("Paste your access token")
                .interact()?;
            config.token = Some(token);
            config.username = None;
            config.app_password = None;
        }
        1 => {
            let username: String = Input::new()
                .with_prompt("Username")
                .interact_text()?;
            let app_password = Password::new()
                .with_prompt("App password")
                .interact()?;
            config.token = None;
            config.username = Some(username);
            config.app_password = Some(app_password);
        }
        _ => unreachable!(),
    }

    // Verify credentials before saving
    let provider = config.provider();
    let credentials = config.credentials()?;
    let client = ApiClient::new(&credentials, &provider)?;

    print!("Verifying credentials... ");
    match client.verify().await {
        Ok(info) => {
            println!("OK ({info})");
            config.save()?;
            println!(
                "Credentials saved to {}",
                Config::config_path().display()
            );
        }
        Err(e) => {
            println!("FAILED");
            anyhow::bail!("authentication failed: {e}");
        }
    }

    Ok(())
}
