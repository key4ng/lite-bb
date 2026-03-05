use anyhow::Result;

use super::{JsonFlag, RepoArgs};
use crate::context::CmdContext;

pub async fn run(number: u64, repo: RepoArgs, json: JsonFlag) -> Result<()> {
    let ctx = CmdContext::new(repo.repo.as_deref())?;

    let result = ctx
        .client
        .get_statuses(ctx.workspace(), ctx.repo_slug(), number)
        .await?;

    if json.json {
        println!("{}", serde_json::to_string_pretty(&result.values)?);
        return Ok(());
    }

    if result.values.is_empty() {
        println!("No status checks found for PR #{number}");
        return Ok(());
    }

    for status in &result.values {
        let name = status.name.as_deref().unwrap_or("unnamed");
        let icon = match status.state.as_str() {
            "SUCCESSFUL" => "✓",
            "FAILED" => "✗",
            "INPROGRESS" => "○",
            "STOPPED" => "■",
            _ => "?",
        };
        println!("{icon}\t{}\t{name}", status.state);
    }

    Ok(())
}
