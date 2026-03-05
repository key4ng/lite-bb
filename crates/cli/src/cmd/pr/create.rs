use anyhow::{bail, Result};

use super::RepoArgs;
use crate::context::CmdContext;
use bb_core::models::{Branch, CreatePullRequest, Destination};

pub async fn run(
    repo: RepoArgs,
    title: Option<String>,
    body: Option<String>,
    head: Option<String>,
    base: Option<String>,
) -> Result<()> {
    let ctx = CmdContext::new(repo.repo.as_deref())?;

    let source_branch = match head {
        Some(b) => b,
        None => bb_core::git::current_branch()?,
    };

    let title = match title {
        Some(t) => t,
        None => bail!("--title is required"),
    };

    let request = CreatePullRequest {
        title,
        source: Destination {
            branch: Branch {
                name: source_branch,
            },
            repository: None,
        },
        destination: base.map(|b| Destination {
            branch: Branch { name: b },
            repository: None,
        }),
        description: body,
        close_source_branch: None,
        reviewers: None,
    };

    let pr = ctx
        .client
        .create_pr(ctx.workspace(), ctx.repo_slug(), &request)
        .await?;

    println!("Created PR #{}: {}", pr.id, pr.title);
    println!(
        "{} -> {}",
        pr.source.branch.name, pr.destination.branch.name
    );

    Ok(())
}
