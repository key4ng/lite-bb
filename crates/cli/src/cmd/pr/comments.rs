use anyhow::Result;

use super::{JsonFlag, RepoArgs};
use crate::context::CmdContext;

pub async fn run(number: u64, repo: RepoArgs, json: JsonFlag) -> Result<()> {
    let ctx = CmdContext::new(repo.repo.as_deref())?;

    let page = ctx
        .client
        .list_comments(ctx.workspace(), ctx.repo_slug(), number)
        .await?;

    if json.json {
        println!("{}", serde_json::to_string_pretty(&page.values)?);
        return Ok(());
    }

    if page.values.is_empty() {
        println!("No comments on PR #{number}");
        return Ok(());
    }

    for comment in &page.values {
        let author = comment
            .user
            .nickname
            .as_deref()
            .unwrap_or(&comment.user.display_name);
        let date = &comment.created_on;

        let inline_info = comment
            .inline
            .as_ref()
            .map(|i| {
                let line = i.to.map(|l| l.to_string()).unwrap_or_default();
                format!("  ({}:{})", i.path, line)
            })
            .unwrap_or_default();

        println!("#{:<4} @{:<12} {}{}", comment.id, author, date, inline_info);
        if let Some(raw) = &comment.content.raw {
            for line in raw.lines() {
                println!("    {line}");
            }
        }
        println!();
    }

    Ok(())
}
