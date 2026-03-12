use anyhow::{bail, Result};

use super::RepoArgs;
use crate::context::CmdContext;
use bb_core::models::{CommentContent, CreateComment, InlineComment};

pub async fn run(
    number: u64,
    repo: RepoArgs,
    body: &str,
    path: Option<&str>,
    line: Option<u32>,
) -> Result<()> {
    let inline = match (path, line) {
        (Some(p), Some(l)) => Some(InlineComment {
            from: None,
            to: Some(l),
            path: p.to_string(),
        }),
        (None, None) => None,
        _ => bail!("--path and --line must be used together"),
    };

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
                inline,
            },
        )
        .await?;

    println!("Added comment #{} to PR #{}", comment.id, number);

    Ok(())
}
