use anyhow::Result;

use bb_core::auth::Credentials;
use bb_core::config::{Config, Provider};

pub async fn run() -> Result<()> {
    let config = Config::load()?;

    match config.provider() {
        Provider::Cloud => println!("  Provider: Bitbucket Cloud"),
        Provider::Server { base_url } => println!("  Provider: Bitbucket Server ({base_url})"),
    }

    match config.credentials() {
        Ok(creds) => {
            match &creds {
                Credentials::Token(token) => {
                    let masked = mask(token);
                    println!("  Logged in with access token: {masked}");
                }
                Credentials::AppPassword { username, app_password } => {
                    let masked = mask(app_password);
                    println!("  Logged in as {username} (app password: {masked})");
                }
            }
            if let Some(workspace) = &config.workspace {
                println!("  Workspace: {workspace}");
            }
            if let Some(repo) = &config.default_repo {
                println!("  Default repo: {repo}");
            }
        }
        Err(e) => {
            println!("  {e}");
        }
    }

    Ok(())
}

fn mask(s: &str) -> String {
    if s.len() <= 4 {
        "****".to_string()
    } else {
        format!("{}****", &s[..4])
    }
}
