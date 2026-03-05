use std::process::Command;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GitError {
    #[error("not a git repository")]
    NotARepo,
    #[error("no Bitbucket remote found")]
    NoBitbucketRemote,
    #[error("failed to run git: {0}")]
    Exec(#[from] std::io::Error),
    #[error("failed to parse remote URL: {0}")]
    Parse(String),
}

#[derive(Debug, Clone)]
pub struct RepoContext {
    pub workspace: String,
    pub repo_slug: String,
}

pub fn current_branch() -> Result<String, GitError> {
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .map_err(GitError::Exec)?;

    if !output.status.success() {
        return Err(GitError::NotARepo);
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub fn repo_context_from_remote() -> Result<RepoContext, GitError> {
    let output = Command::new("git")
        .args(["remote", "get-url", "origin"])
        .output()
        .map_err(GitError::Exec)?;

    if !output.status.success() {
        return Err(GitError::NoBitbucketRemote);
    }

    let url = String::from_utf8_lossy(&output.stdout).trim().to_string();
    parse_bitbucket_url(&url)
}

fn parse_bitbucket_url(url: &str) -> Result<RepoContext, GitError> {
    // SSH: git@bitbucket.org:workspace/repo.git
    if let Some(path) = url.strip_prefix("git@bitbucket.org:") {
        return parse_path(path);
    }

    // HTTPS: https://bitbucket.org/workspace/repo.git
    // Also: https://user@bitbucket.org/workspace/repo.git
    if url.contains("bitbucket.org") {
        if let Some(idx) = url.find("bitbucket.org/") {
            let path = &url[idx + "bitbucket.org/".len()..];
            return parse_path(path);
        }
    }

    Err(GitError::NoBitbucketRemote)
}

fn parse_path(path: &str) -> Result<RepoContext, GitError> {
    let path = path.trim_end_matches(".git").trim_end_matches('/');
    let parts: Vec<&str> = path.splitn(2, '/').collect();
    if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
        return Err(GitError::Parse(path.to_string()));
    }
    Ok(RepoContext {
        workspace: parts[0].to_string(),
        repo_slug: parts[1].to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ssh_url() {
        let ctx = parse_bitbucket_url("git@bitbucket.org:myteam/myrepo.git").unwrap();
        assert_eq!(ctx.workspace, "myteam");
        assert_eq!(ctx.repo_slug, "myrepo");
    }

    #[test]
    fn test_parse_https_url() {
        let ctx = parse_bitbucket_url("https://bitbucket.org/myteam/myrepo.git").unwrap();
        assert_eq!(ctx.workspace, "myteam");
        assert_eq!(ctx.repo_slug, "myrepo");
    }

    #[test]
    fn test_parse_https_url_with_user() {
        let ctx =
            parse_bitbucket_url("https://user@bitbucket.org/myteam/myrepo.git").unwrap();
        assert_eq!(ctx.workspace, "myteam");
        assert_eq!(ctx.repo_slug, "myrepo");
    }

    #[test]
    fn test_parse_no_git_suffix() {
        let ctx = parse_bitbucket_url("https://bitbucket.org/myteam/myrepo").unwrap();
        assert_eq!(ctx.workspace, "myteam");
        assert_eq!(ctx.repo_slug, "myrepo");
    }

    #[test]
    fn test_parse_github_url_fails() {
        assert!(parse_bitbucket_url("git@github.com:user/repo.git").is_err());
    }
}
