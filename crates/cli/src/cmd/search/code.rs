use anyhow::Result;
use bb_core::api::ApiClient;
use bb_core::config::Config;
use bb_core::git;

use super::{JsonFlag, RepoArgs};

pub async fn run(
    query: String,
    repo: RepoArgs,
    json: JsonFlag,
    limit: u32,
    extension: Option<String>,
    filename: Option<String>,
) -> Result<()> {
    let config = Config::load()?;
    let credentials = config.credentials()?;
    let provider = config.provider();
    let client = ApiClient::new(&credentials, &provider)?;

    // Resolve workspace and repo slug from -R flag, config, or git remote.
    // For search, repo is optional — omitting it searches the whole workspace (Cloud)
    // or the whole server (DC).
    let (workspace, repo_slug) = if let Some(r) = &repo.repo {
        let parts: Vec<&str> = r.splitn(2, '/').collect();
        anyhow::ensure!(
            parts.len() == 2 && !parts[0].is_empty() && !parts[1].is_empty(),
            "invalid repo format — expected WORKSPACE/REPO or ~username/REPO, got: {r}"
        );
        (parts[0].to_string(), Some(parts[1].to_string()))
    } else if let (Some(ws), Some(slug)) = (&config.workspace, &config.default_repo) {
        (ws.clone(), Some(slug.clone()))
    } else {
        match git::repo_context_from_remote()
            .or_else(|_| git::repo_context_from_any_remote())
        {
            Ok(ctx) => (ctx.workspace, Some(ctx.repo_slug)),
            Err(_) => {
                // No repo context — use config workspace for Cloud, empty for DC
                (config.workspace.unwrap_or_default(), None)
            }
        }
    };

    let results = client
        .search_code(
            &workspace,
            repo_slug.as_deref(),
            &query,
            limit,
            extension.as_deref(),
            filename.as_deref(),
        )
        .await?;

    if json.json {
        println!("{}", serde_json::to_string_pretty(&results)?);
        return Ok(());
    }

    if results.is_empty() {
        println!("No results found for \"{}\"", query);
        return Ok(());
    }

    for result in &results {
        println!("{}\t{}", result.path, result.repo);
        for m in &result.matches {
            let prefix = if m.highlighted { ">" } else { " " };
            println!("  {} {:>4}: {}", prefix, m.line, m.content.trim_end());
        }
        println!();
    }

    Ok(())
}
