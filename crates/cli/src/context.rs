use anyhow::{Context, Result};

use bb_core::api::ApiClient;
use bb_core::config::{Config, Provider};
use bb_core::git::{self, RepoContext};

pub struct CmdContext {
    pub client: ApiClient,
    pub repo: RepoContext,
}

impl CmdContext {
    pub fn new(repo_override: Option<&str>) -> Result<Self> {
        let config = Config::load().context("failed to load config")?;
        let credentials = config.credentials()?;
        let provider = config.provider();
        let client =
            ApiClient::new(&credentials, &provider).context("failed to create API client")?;

        let repo = if let Some(r) = repo_override {
            parse_repo_flag(r)?
        } else {
            // Try config default, then git remote
            if let (Some(ws), Some(slug)) = (&config.workspace, &config.default_repo) {
                RepoContext {
                    workspace: ws.clone(),
                    repo_slug: slug.clone(),
                }
            } else {
                match &provider {
                    Provider::Cloud => git::repo_context_from_remote()
                        .context("could not detect repo — use --repo WORKSPACE/REPO or set defaults in config")?,
                    Provider::Server { .. } => git::repo_context_from_any_remote()
                        .context("could not detect repo — use --repo PROJECT/REPO or set defaults in config")?,
                }
            }
        };

        Ok(Self { client, repo })
    }

    pub fn workspace(&self) -> &str {
        &self.repo.workspace
    }

    pub fn repo_slug(&self) -> &str {
        &self.repo.repo_slug
    }
}

fn parse_repo_flag(s: &str) -> Result<RepoContext> {
    let parts: Vec<&str> = s.splitn(2, '/').collect();
    anyhow::ensure!(
        parts.len() == 2 && !parts[0].is_empty() && !parts[1].is_empty(),
        "invalid repo format — expected WORKSPACE/REPO, got: {s}"
    );
    Ok(RepoContext {
        workspace: parts[0].to_string(),
        repo_slug: parts[1].to_string(),
    })
}
