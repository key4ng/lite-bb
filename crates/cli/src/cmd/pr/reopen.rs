use anyhow::Result;

use super::RepoArgs;
use crate::context::CmdContext;
use bb_core::models::UpdatePullRequest;

pub async fn run(number: u64, repo: RepoArgs) -> Result<()> {
    let ctx = CmdContext::new(repo.repo.as_deref())?;

    // Bitbucket doesn't have a dedicated reopen endpoint.
    // Updating a declined PR via PUT can trigger a state change.
    let update = UpdatePullRequest {
        title: None,
        description: None,
        destination: None,
    };

    let pr = ctx
        .client
        .update_pr(ctx.workspace(), ctx.repo_slug(), number, &update)
        .await?;

    println!("Reopened PR #{}: {} (state: {})", pr.id, pr.title, pr.state);

    Ok(())
}
