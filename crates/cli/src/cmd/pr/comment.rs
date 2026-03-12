use anyhow::Result;

use super::RepoArgs;
use crate::context::CmdContext;
use bb_core::models::{CommentContent, CreateComment};

pub async fn run(number: u64, repo: RepoArgs, body: &str) -> Result<()> {
    let ctx = CmdContext::new(repo.repo.as_deref())?;

    let comment = ctx
        .client
        .add_comment(
            ctx.workspace(),
            ctx.repo_slug(),
            number,
            &CreateComment {
                content: CommentContent {
                    raw: Some(body.to_string()),
                    markup: Some("markdown".to_string()),
                    html: None,
                },
                inline: None,
            },
        )
        .await?;

    println!("Added comment #{} to PR #{}", comment.id, number);

    Ok(())
}
