use anyhow::Result;

use super::{JsonFlag, RepoArgs};
use crate::context::CmdContext;

pub async fn run(number: u64, repo: RepoArgs, json: JsonFlag) -> Result<()> {
    let ctx = CmdContext::new(repo.repo.as_deref())?;

    let pr = ctx
        .client
        .get_pr(ctx.workspace(), ctx.repo_slug(), number)
        .await?;

    if json.json {
        println!("{}", serde_json::to_string_pretty(&pr)?);
        return Ok(());
    }

    println!("#{} {}", pr.id, pr.title);
    println!("State:  {}", pr.state);
    println!("Author: {}", pr.author.display_name);
    println!(
        "Branch: {} -> {}",
        pr.source.branch.name, pr.destination.branch.name
    );
    if let Some(desc) = &pr.description {
        if !desc.is_empty() {
            println!("\n{desc}");
        }
    }
    if let Some(reviewers) = &pr.reviewers {
        if !reviewers.is_empty() {
            let names: Vec<&str> = reviewers.iter().map(|r| r.display_name.as_str()).collect();
            println!("Reviewers: {}", names.join(", "));
        }
    }

    Ok(())
}
