use anyhow::{bail, Result};

use super::RepoArgs;
use crate::context::CmdContext;

pub async fn run(
    number: u64,
    repo: RepoArgs,
    approve: bool,
    request_changes: bool,
) -> Result<()> {
    if !approve && !request_changes {
        bail!("specify --approve or --request-changes");
    }
    if approve && request_changes {
        bail!("cannot use both --approve and --request-changes");
    }

    let ctx = CmdContext::new(repo.repo.as_deref())?;

    if approve {
        ctx.client
            .approve_pr(ctx.workspace(), ctx.repo_slug(), number)
            .await?;
        println!("Approved PR #{number}");
    } else {
        ctx.client
            .unapprove_pr(ctx.workspace(), ctx.repo_slug(), number)
            .await?;
        println!("Unapproved PR #{number}");
    }

    Ok(())
}
