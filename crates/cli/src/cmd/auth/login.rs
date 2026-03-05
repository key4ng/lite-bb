use anyhow::Result;
use dialoguer::{Input, Password, Select};

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
            let url: String = Input::new()
                .with_prompt("Server URL (e.g. https://bitbucket.company.com)")
                .interact_text()?;
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

    config.save()?;
    println!(
        "Logged in. Credentials saved to {}",
        Config::config_path().display()
    );
    Ok(())
}
