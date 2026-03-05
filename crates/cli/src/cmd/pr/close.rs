use anyhow::Result;

use super::RepoArgs;
use crate::context::CmdContext;

pub async fn run(number: u64, repo: RepoArgs) -> Result<()> {
    let ctx = CmdContext::new(repo.repo.as_deref())?;

    let pr = ctx
        .client
        .decline_pr(ctx.workspace(), ctx.repo_slug(), number)
        .await?;

    println!("Closed PR #{}: {}", pr.id, pr.title);

    Ok(())
}
