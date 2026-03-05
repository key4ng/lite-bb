use anyhow::Result;

use super::RepoArgs;
use crate::context::CmdContext;

pub async fn run(number: u64, repo: RepoArgs) -> Result<()> {
    let ctx = CmdContext::new(repo.repo.as_deref())?;

    let diff = ctx
        .client
        .get_diff(ctx.workspace(), ctx.repo_slug(), number)
        .await?;

    print!("{diff}");

    Ok(())
}
