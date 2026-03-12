use anyhow::{bail, Result};

use super::RepoArgs;
use crate::context::CmdContext;
use bb_core::models::{CommentContent, CreateComment};

pub async fn run(
    number: u64,
    repo: RepoArgs,
    approve: bool,
    request_changes: bool,
    comment: bool,
    body: Option<String>,
    body_file: Option<String>,
) -> Result<()> {
    let action_count = [approve, request_changes, comment]
        .iter()
        .filter(|&&v| v)
        .count();
    if action_count == 0 {
        bail!("specify --approve, --request-changes, or --comment");
    }
    if action_count > 1 {
        bail!("cannot combine --approve, --request-changes, and --comment");
    }

    let body_text = resolve_body(body, body_file)?;

    if comment && body_text.is_none() {
        bail!("--comment requires --body or --body-file");
    }

    let ctx = CmdContext::new(repo.repo.as_deref())?;

    if approve {
        ctx.client
            .approve_pr(ctx.workspace(), ctx.repo_slug(), number)
            .await?;
        println!("Approved PR #{number}");
    } else if request_changes {
        ctx.client
            .unapprove_pr(ctx.workspace(), ctx.repo_slug(), number)
            .await?;
        println!("Requested changes on PR #{number}");
    }

    if let Some(text) = body_text {
        ctx.client
            .add_comment(
                ctx.workspace(),
                ctx.repo_slug(),
                number,
                &CreateComment {
                    content: CommentContent {
                        raw: Some(text),
                        markup: Some("markdown".to_string()),
                        html: None,
                    },
                    inline: None,
                },
            )
            .await?;
        println!("Added review comment to PR #{number}");
    }

    Ok(())
}

fn resolve_body(body: Option<String>, body_file: Option<String>) -> Result<Option<String>> {
    match (body, body_file) {
        (Some(_), Some(_)) => bail!("cannot use both --body and --body-file"),
        (Some(b), None) => Ok(Some(b)),
        (None, Some(path)) => {
            let text = if path == "-" {
                use std::io::Read;
                let mut buf = String::new();
                std::io::stdin().read_to_string(&mut buf)?;
                buf
            } else {
                std::fs::read_to_string(&path)?
            };
            Ok(Some(text))
        }
        (None, None) => Ok(None),
    }
}
