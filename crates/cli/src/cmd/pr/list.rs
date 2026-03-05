use anyhow::Result;

use super::{JsonFlag, RepoArgs};
use crate::context::CmdContext;

pub async fn run(repo: RepoArgs, json: JsonFlag, state: &str, limit: u32) -> Result<()> {
    let ctx = CmdContext::new(repo.repo.as_deref())?;

    let result = ctx
        .client
        .list_prs(ctx.workspace(), ctx.repo_slug(), Some(state), None, Some(limit))
        .await?;

    if json.json {
        println!("{}", serde_json::to_string_pretty(&result.values)?);
        return Ok(());
    }

    if result.values.is_empty() {
        println!("No pull requests match the search");
        return Ok(());
    }

    for pr in &result.values {
        println!(
            "#{}\t{}\t{} -> {}\t{}",
            pr.id,
            pr.title,
            pr.source.branch.name,
            pr.destination.branch.name,
            pr.author.display_name
        );
    }

    Ok(())
}
