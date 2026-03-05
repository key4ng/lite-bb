use std::process::Command;

use anyhow::{bail, Result};

use super::RepoArgs;
use crate::context::CmdContext;

pub async fn run(number: u64, repo: RepoArgs) -> Result<()> {
    let ctx = CmdContext::new(repo.repo.as_deref())?;

    let pr = ctx
        .client
        .get_pr(ctx.workspace(), ctx.repo_slug(), number)
        .await?;

    let branch = &pr.source.branch.name;

    let fetch = Command::new("git")
        .args(["fetch", "origin", branch])
        .status()?;

    if !fetch.success() {
        bail!("failed to fetch branch {branch}");
    }

    let checkout = Command::new("git")
        .args(["checkout", branch])
        .status()?;

    if !checkout.success() {
        bail!("failed to checkout branch {branch}");
    }

    println!("Checked out PR #{number} branch: {branch}");

    Ok(())
}
