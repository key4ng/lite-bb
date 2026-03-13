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
    line_type: Option<&str>,
) -> Result<()> {
    let inline = match (path, line) {
        (Some(p), Some(l)) => {
            let lt = line_type.unwrap_or("context");
            let (from, to) = match lt {
                "added" => (None, Some(l)),
                "removed" => (Some(l), None),
                "context" => (Some(l), None),
                _ => bail!("--line-type must be 'added', 'removed', or 'context'"),
            };
            Some(InlineComment {
                from,
                to,
                path: p.to_string(),
                line_type: Some(lt.to_string()),
                file_type: Some(match lt {
                    "added" => "to".to_string(),
                    _ => "from".to_string(),
                }),
            })
        }
        (None, None) => None,
        _ => bail!("--path and --line must both be specified for inline comments"),
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
