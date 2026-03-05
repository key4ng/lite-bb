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

pub fn get_remote_url() -> Result<String, GitError> {
    let output = Command::new("git")
        .args(["remote", "get-url", "origin"])
        .output()
        .map_err(GitError::Exec)?;

    if !output.status.success() {
        return Err(GitError::NoBitbucketRemote);
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Extract the server base URL from a git remote URL.
/// e.g. "ssh://git@bitbucket.company.com:7999/PROJECT/repo.git" → "https://bitbucket.company.com"
/// e.g. "https://bitbucket.company.com/scm/PROJECT/repo.git" → "https://bitbucket.company.com"
pub fn server_url_from_remote() -> Option<String> {
    let url = get_remote_url().ok()?;
    extract_server_url(&url)
}

fn extract_server_url(url: &str) -> Option<String> {
    // HTTPS: https://host/... or https://user@host/...
    if url.starts_with("https://") || url.starts_with("http://") {
        let scheme_end = url.find("://").unwrap() + 3;
        let rest = &url[scheme_end..];
        // Strip user@ if present
        let rest = if let Some(at_idx) = rest.find('@') {
            if rest[..at_idx].contains('/') {
                rest // no user@ — the @ is in the path
            } else {
                &rest[at_idx + 1..]
            }
        } else {
            rest
        };
        // Take just the host (before first /)
        let host = rest.split('/').next()?;
        let scheme = &url[..url.find("://").unwrap()];
        return Some(format!("{scheme}://{host}"));
    }

    // SSH: ssh://git@host:port/... → https://host
    if url.starts_with("ssh://") {
        let rest = &url["ssh://".len()..];
        let rest = rest.split('@').last()?;
        // host:port or host
        let host = rest.split(':').next()?.split('/').next()?;
        return Some(format!("https://{host}"));
    }

    // SCP: git@host:path or git@host:port/path → https://host
    if let Some(at_idx) = url.find('@') {
        let after_at = &url[at_idx + 1..];
        let host = after_at.split(':').next()?.split('/').next()?;
        return Some(format!("https://{host}"));
    }

    None
}

/// Parse repo context from git remote, for Bitbucket Cloud.
pub fn repo_context_from_remote() -> Result<RepoContext, GitError> {
    let url = get_remote_url()?;
    parse_bitbucket_url(&url)
}

/// Parse repo context from any git remote URL (for Bitbucket Server/DC).
/// Extracts project/repo from the URL path.
pub fn repo_context_from_any_remote() -> Result<RepoContext, GitError> {
    let url = get_remote_url()?;
    parse_generic_url(&url)
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

/// Parse any git remote URL to extract project/repo.
/// Handles DC patterns:
///   ssh://git@host:port/PROJECT/repo.git
///   https://host/scm/PROJECT/repo.git
///   git@host:PROJECT/repo.git
fn parse_generic_url(url: &str) -> Result<RepoContext, GitError> {
    // SSH URL: ssh://git@host:port/PROJECT/repo.git
    if url.starts_with("ssh://") {
        let without_scheme = &url["ssh://".len()..];
        // Skip user@host:port, find the path
        let path = if let Some(idx) = without_scheme.find('/') {
            &without_scheme[idx + 1..]
        } else {
            return Err(GitError::Parse(url.to_string()));
        };
        return parse_path(path);
    }

    // HTTPS: https://host/scm/PROJECT/repo.git or https://host/PROJECT/repo.git
    if url.starts_with("https://") || url.starts_with("http://") {
        let without_scheme = url.split("://").nth(1).unwrap_or("");
        // Skip host (and optional user@)
        let path = if let Some(idx) = without_scheme.find('/') {
            &without_scheme[idx + 1..]
        } else {
            return Err(GitError::Parse(url.to_string()));
        };
        // Strip /scm/ prefix used by DC HTTPS URLs
        let path = path.strip_prefix("scm/").unwrap_or(path);
        return parse_path(path);
    }

    // SCP-style: git@host:PROJECT/repo.git
    if let Some(colon_idx) = url.find(':') {
        if url[..colon_idx].contains('@') {
            let path = &url[colon_idx + 1..];
            // Skip port number if present (e.g., git@host:7999/PROJECT/repo.git)
            let path = if path.starts_with(|c: char| c.is_ascii_digit()) {
                if let Some(slash_idx) = path.find('/') {
                    &path[slash_idx + 1..]
                } else {
                    return Err(GitError::Parse(url.to_string()));
                }
            } else {
                path
            };
            return parse_path(path);
        }
    }

    Err(GitError::Parse(url.to_string()))
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

    // Server/DC URL tests
    #[test]
    fn test_parse_dc_ssh_url() {
        let ctx =
            parse_generic_url("ssh://git@bitbucket.company.com:7999/PROJECT/myrepo.git").unwrap();
        assert_eq!(ctx.workspace, "PROJECT");
        assert_eq!(ctx.repo_slug, "myrepo");
    }

    #[test]
    fn test_parse_dc_https_scm_url() {
        let ctx =
            parse_generic_url("https://bitbucket.company.com/scm/PROJECT/myrepo.git").unwrap();
        assert_eq!(ctx.workspace, "PROJECT");
        assert_eq!(ctx.repo_slug, "myrepo");
    }

    #[test]
    fn test_parse_dc_scp_style() {
        let ctx = parse_generic_url("git@bitbucket.company.com:PROJECT/myrepo.git").unwrap();
        assert_eq!(ctx.workspace, "PROJECT");
        assert_eq!(ctx.repo_slug, "myrepo");
    }

    #[test]
    fn test_parse_dc_scp_with_port() {
        let ctx =
            parse_generic_url("git@bitbucket.company.com:7999/PROJECT/myrepo.git").unwrap();
        assert_eq!(ctx.workspace, "PROJECT");
        assert_eq!(ctx.repo_slug, "myrepo");
    }

    // Server URL extraction tests
    #[test]
    fn test_extract_server_url_https() {
        assert_eq!(
            extract_server_url("https://bitbucket.company.com/scm/PROJECT/repo.git"),
            Some("https://bitbucket.company.com".to_string())
        );
    }

    #[test]
    fn test_extract_server_url_https_with_user() {
        assert_eq!(
            extract_server_url("https://user@bitbucket.company.com/scm/PROJECT/repo.git"),
            Some("https://bitbucket.company.com".to_string())
        );
    }

    #[test]
    fn test_extract_server_url_ssh() {
        assert_eq!(
            extract_server_url("ssh://git@bitbucket.company.com:7999/PROJECT/repo.git"),
            Some("https://bitbucket.company.com".to_string())
        );
    }

    #[test]
    fn test_extract_server_url_scp() {
        assert_eq!(
            extract_server_url("git@bitbucket.company.com:PROJECT/repo.git"),
            Some("https://bitbucket.company.com".to_string())
        );
    }

    #[test]
    fn test_parse_dc_https_no_scm() {
        let ctx =
            parse_generic_url("https://bitbucket.company.com/PROJECT/myrepo.git").unwrap();
        assert_eq!(ctx.workspace, "PROJECT");
        assert_eq!(ctx.repo_slug, "myrepo");
    }
}
