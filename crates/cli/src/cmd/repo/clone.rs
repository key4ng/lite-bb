use anyhow::{Context, Result};
use bb_core::api::ApiClient;
use bb_core::config::Config;

pub async fn run(repo: String, directory: Option<String>, git_args: Vec<String>) -> Result<()> {
    let parts: Vec<&str> = repo.splitn(2, '/').collect();
    anyhow::ensure!(
        parts.len() == 2 && !parts[0].is_empty() && !parts[1].is_empty(),
        "invalid repo format — expected WORKSPACE/REPO or PROJECT/REPO, got: {repo}"
    );
    let workspace = parts[0];
    let slug = parts[1];

    let config = Config::load()?;
    let credentials = config.credentials()?;
    let provider = config.provider();
    let client = ApiClient::new(&credentials, &provider)?;

    let info = client.get_repo(workspace, slug).await?;

    // Prefer SSH, fall back to HTTPS
    let clone_url = info
        .clone_url_ssh
        .or(info.clone_url_https)
        .with_context(|| format!("no clone URL found for {workspace}/{slug}"))?;

    let dir = directory.unwrap_or_else(|| slug.to_string());

    println!("Cloning {workspace}/{slug} into {dir}...");

    let mut cmd = std::process::Command::new("git");
    cmd.arg("clone").arg(&clone_url).arg(&dir);
    for arg in &git_args {
        cmd.arg(arg);
    }

    let status = cmd.status().context("failed to run git clone")?;
    anyhow::ensure!(status.success(), "git clone failed");

    Ok(())
}
