use anyhow::Result;
use bb_core::api::ApiClient;
use bb_core::config::Config;
use bb_core::git;

pub async fn run(
    owner: Option<String>,
    limit: u32,
    visibility: Option<String>,
    json: bool,
) -> Result<()> {
    let config = Config::load()?;
    let credentials = config.credentials()?;
    let provider = config.provider();
    let client = ApiClient::new(&credentials, &provider)?;

    let workspace = if let Some(o) = owner {
        o
    } else if let Some(ws) = &config.workspace {
        ws.clone()
    } else {
        git::repo_context_from_remote()
            .or_else(|_| git::repo_context_from_any_remote())
            .map(|c| c.workspace)
            .map_err(|_| anyhow::anyhow!("could not detect workspace — pass an owner argument or set a default workspace in config"))?
    };

    let repos = client
        .list_repos(&workspace, limit, visibility.as_deref())
        .await?;

    if json {
        println!("{}", serde_json::to_string_pretty(&repos)?);
        return Ok(());
    }

    if repos.is_empty() {
        println!("No repositories found in {workspace}");
        return Ok(());
    }

    // Column widths
    let name_width = repos
        .iter()
        .map(|r| r.full_name.len())
        .max()
        .unwrap_or(4)
        .max(4);

    println!(
        "{:<name_width$}  {:<8}  {}",
        "REPO", "VIS", "UPDATED"
    );
    for repo in &repos {
        let vis = if repo.is_private { "private" } else { "public " };
        let updated = repo
            .updated_on
            .as_deref()
            .and_then(|s| s.get(..10))
            .unwrap_or("-");
        println!("{:<name_width$}  {:<8}  {}", repo.full_name, vis, updated);
    }

    Ok(())
}
