use anyhow::Result;
use bb_core::api::ApiClient;
use bb_core::config::Config;
use bb_core::git;
use std::io::{self, Write};

pub async fn run(
    name: Option<String>,
    description: Option<String>,
    workspace: Option<String>,
    public: bool,
    clone: bool,
    json: bool,
) -> Result<()> {
    let config = Config::load()?;
    let credentials = config.credentials()?;
    let provider = config.provider();
    let client = ApiClient::new(&credentials, &provider)?;

    // Resolve workspace
    let ws = if let Some(w) = workspace {
        w
    } else if let Some(w) = &config.workspace {
        w.clone()
    } else {
        git::repo_context_from_remote()
            .or_else(|_| git::repo_context_from_any_remote())
            .map(|c| c.workspace)
            .unwrap_or_default()
    };
    anyhow::ensure!(!ws.is_empty(), "workspace/project is required — pass --workspace or set a default in config");

    // Resolve name interactively if not passed
    let slug = if let Some(n) = name {
        n
    } else {
        print!("Repository name: ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let trimmed = input.trim().to_string();
        anyhow::ensure!(!trimmed.is_empty(), "repository name is required");
        trimmed
    };

    let is_private = !public;

    println!("Creating repository {ws}/{slug}...");
    let info = client.create_repo(&ws, &slug, description, is_private).await?;

    if json {
        println!("{}", serde_json::to_string_pretty(&info)?);
    } else {
        println!("Created {}", info.full_name);
        let vis = if info.is_private { "private" } else { "public" };
        println!("  Visibility:  {vis}");
        if let Some(url) = &info.clone_url_https {
            println!("  Clone HTTPS: {url}");
        }
        if let Some(url) = &info.clone_url_ssh {
            println!("  Clone SSH:   {url}");
        }
        if let Some(url) = &info.web_url {
            println!("  Web:         {url}");
        }
    }

    if clone {
        let clone_url = info
            .clone_url_ssh
            .or(info.clone_url_https)
            .ok_or_else(|| anyhow::anyhow!("no clone URL returned"))?;
        println!("\nCloning into {slug}...");
        let status = std::process::Command::new("git")
            .args(["clone", &clone_url, &slug])
            .status()?;
        anyhow::ensure!(status.success(), "git clone failed");
    }

    Ok(())
}
