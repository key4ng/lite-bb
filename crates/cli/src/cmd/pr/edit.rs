use anyhow::{bail, Result};

use super::RepoArgs;
use crate::context::CmdContext;
use bb_core::models::{Branch, Destination, UpdatePullRequest};

pub async fn run(
    number: u64,
    repo: RepoArgs,
    title: Option<String>,
    body: Option<String>,
    base: Option<String>,
) -> Result<()> {
    if title.is_none() && body.is_none() && base.is_none() {
        bail!("specify at least one of --title, --body, or --base");
    }

    let ctx = CmdContext::new(repo.repo.as_deref())?;

    let update = UpdatePullRequest {
        title,
        description: body,
        destination: base.map(|b| Destination {
            branch: Branch { name: b },
            repository: None,
            commit: None,
        }),
    };

    let pr = ctx
        .client
        .update_pr(ctx.workspace(), ctx.repo_slug(), number, &update)
        .await?;

    println!("Updated PR #{}: {}", pr.id, pr.title);

    Ok(())
}
