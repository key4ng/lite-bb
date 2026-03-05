use anyhow::Result;

use super::RepoArgs;
use crate::context::CmdContext;
use bb_core::models::MergeRequest;

pub async fn run(
    number: u64,
    repo: RepoArgs,
    strategy: Option<String>,
    message: Option<String>,
) -> Result<()> {
    let ctx = CmdContext::new(repo.repo.as_deref())?;

    let request = MergeRequest {
        merge_strategy: strategy,
        close_source_branch: Some(true),
        message,
    };

    let pr = ctx
        .client
        .merge_pr(ctx.workspace(), ctx.repo_slug(), number, &request)
        .await?;

    println!("Merged PR #{}: {}", pr.id, pr.title);

    Ok(())
}
