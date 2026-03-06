use anyhow::Result;
use bb_core::api::ApiClient;
use bb_core::config::Config;

use crate::context::CmdContext;

pub async fn run(repo: Option<String>, path: String, ref_: Option<String>) -> Result<()> {
    let config = Config::load()?;
    let credentials = config.credentials()?;
    let provider = config.provider();
    let client = ApiClient::new(&credentials, &provider)?;

    let (workspace, slug) = if let Some(r) = &repo {
        let parts: Vec<&str> = r.splitn(2, '/').collect();
        anyhow::ensure!(
            parts.len() == 2 && !parts[0].is_empty() && !parts[1].is_empty(),
            "invalid repo format — expected WORKSPACE/REPO or PROJECT/REPO, got: {r}"
        );
        (parts[0].to_string(), parts[1].to_string())
    } else {
        let ctx = CmdContext::new(None)?;
        (ctx.workspace().to_string(), ctx.repo_slug().to_string())
    };

    let content = client
        .get_file(&workspace, &slug, &path, ref_.as_deref())
        .await?;

    print!("{content}");
    Ok(())
}
