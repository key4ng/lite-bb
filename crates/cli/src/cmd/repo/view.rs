use anyhow::Result;
use bb_core::config::Config;
use crate::context::CmdContext;

pub async fn run(repo: Option<String>, web: bool, json: bool) -> Result<()> {
    let config = Config::load()?;
    let provider = config.provider();

    let (workspace, slug) = if let Some(r) = &repo {
        let parts: Vec<&str> = r.splitn(2, '/').collect();
        anyhow::ensure!(
            parts.len() == 2,
            "invalid repo format — expected WORKSPACE/REPO, got: {r}"
        );
        (parts[0].to_string(), parts[1].to_string())
    } else {
        let ctx = CmdContext::new(None)?;
        (ctx.workspace().to_string(), ctx.repo_slug().to_string())
    };

    let credentials = config.credentials()?;
    let client = bb_core::api::ApiClient::new(&credentials, &provider)?;
    let info = client.get_repo(&workspace, &slug).await?;

    if web {
        let url = info.web_url.as_deref().unwrap_or_else(|| {
            match &provider {
                bb_core::config::Provider::Cloud => {
                    // fallback constructed inline
                    ""
                }
                bb_core::config::Provider::Server { .. } => "",
            }
        });
        if url.is_empty() {
            anyhow::bail!("no web URL available for this repository");
        }
        open_browser(url)?;
        return Ok(());
    }

    if json {
        println!("{}", serde_json::to_string_pretty(&info)?);
        return Ok(());
    }

    // Human output
    println!("{}", info.full_name);
    if let Some(desc) = &info.description {
        if !desc.is_empty() {
            println!("\n{desc}");
        }
    }
    println!();
    println!("Visibility:  {}", if info.is_private { "private" } else { "public" });
    println!("SCM:         {}", info.scm);
    if let Some(branch) = &info.default_branch {
        println!("Default:     {branch}");
    }
    if let Some(lang) = &info.language {
        if !lang.is_empty() {
            println!("Language:    {lang}");
        }
    }
    if let Some(created) = &info.created_on {
        println!("Created:     {}", &created[..10.min(created.len())]);
    }
    if let Some(updated) = &info.updated_on {
        println!("Updated:     {}", &updated[..10.min(updated.len())]);
    }
    if let Some(url) = &info.clone_url_https {
        println!("Clone HTTPS: {url}");
    }
    if let Some(url) = &info.clone_url_ssh {
        println!("Clone SSH:   {url}");
    }
    if let Some(web) = &info.web_url {
        println!("Web:         {web}");
    }
    if let Some(parent) = &info.parent {
        println!("Forked from: {parent}");
    }

    Ok(())
}

fn open_browser(url: &str) -> Result<()> {
    #[cfg(target_os = "macos")]
    std::process::Command::new("open").arg(url).spawn()?;
    #[cfg(target_os = "linux")]
    std::process::Command::new("xdg-open").arg(url).spawn()?;
    #[cfg(target_os = "windows")]
    std::process::Command::new("cmd").args(["/c", "start", url]).spawn()?;
    Ok(())
}
