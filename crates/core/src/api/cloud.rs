use crate::api::client::{ApiError, HttpClient};
use crate::auth::Credentials;
use crate::config::Provider;
use crate::models::search::{CloudSearchResponse, CodeResult};
use crate::models::repo::{CloudRepo, CreateRepoCloud, RepoInfo};
use crate::models::*;

const BASE_URL: &str = "https://api.bitbucket.org/2.0";

pub struct CloudClient {
    pub(crate) http: HttpClient,
    pub(crate) base_url: String,
}

impl CloudClient {
    pub fn new(credentials: &Credentials) -> Result<Self, reqwest::Error> {
        let http = HttpClient::new(credentials, &Provider::Cloud)?;
        Ok(Self {
            http,
            base_url: BASE_URL.to_string(),
        })
    }

    pub async fn verify(&self) -> Result<String, ApiError> {
        let url = format!("{}/user", self.base_url);
        let user: User = self.http.get(&url).await?;
        Ok(user.display_name)
    }

    fn repo_url(&self, workspace: &str, repo: &str, path: &str) -> String {
        format!(
            "{}/repositories/{}/{}{}",
            self.base_url, workspace, repo, path
        )
    }

    pub async fn list_prs(
        &self,
        workspace: &str,
        repo: &str,
        state: Option<&str>,
        page: Option<u32>,
        pagelen: Option<u32>,
    ) -> Result<Paginated<PullRequest>, ApiError> {
        let mut url = self.repo_url(workspace, repo, "/pullrequests");
        let mut params = Vec::new();
        if let Some(state) = state {
            params.push(format!("state={state}"));
        }
        if let Some(page) = page {
            params.push(format!("page={page}"));
        }
        if let Some(pagelen) = pagelen {
            params.push(format!("pagelen={pagelen}"));
        }
        if !params.is_empty() {
            url = format!("{}?{}", url, params.join("&"));
        }
        self.http.get(&url).await
    }

    pub async fn get_pr(
        &self,
        workspace: &str,
        repo: &str,
        id: u64,
    ) -> Result<PullRequest, ApiError> {
        let url = self.repo_url(workspace, repo, &format!("/pullrequests/{id}"));
        self.http.get(&url).await
    }

    pub async fn create_pr(
        &self,
        workspace: &str,
        repo: &str,
        body: &CreatePullRequest,
    ) -> Result<PullRequest, ApiError> {
        let url = self.repo_url(workspace, repo, "/pullrequests");
        self.http.post(&url, body).await
    }

    pub async fn update_pr(
        &self,
        workspace: &str,
        repo: &str,
        id: u64,
        body: &UpdatePullRequest,
    ) -> Result<PullRequest, ApiError> {
        let url = self.repo_url(workspace, repo, &format!("/pullrequests/{id}"));
        self.http.put(&url, body).await
    }

    pub async fn merge_pr(
        &self,
        workspace: &str,
        repo: &str,
        id: u64,
        body: &MergeRequest,
    ) -> Result<PullRequest, ApiError> {
        let url = self.repo_url(workspace, repo, &format!("/pullrequests/{id}/merge"));
        self.http.post(&url, body).await
    }

    pub async fn decline_pr(
        &self,
        workspace: &str,
        repo: &str,
        id: u64,
    ) -> Result<PullRequest, ApiError> {
        let url = self.repo_url(workspace, repo, &format!("/pullrequests/{id}/decline"));
        self.http.post(&url, &serde_json::json!({})).await
    }

    pub async fn approve_pr(
        &self,
        workspace: &str,
        repo: &str,
        id: u64,
    ) -> Result<(), ApiError> {
        let url = self.repo_url(workspace, repo, &format!("/pullrequests/{id}/approve"));
        self.http.post_empty(&url).await
    }

    pub async fn unapprove_pr(
        &self,
        workspace: &str,
        repo: &str,
        id: u64,
    ) -> Result<(), ApiError> {
        let url = self.repo_url(workspace, repo, &format!("/pullrequests/{id}/approve"));
        self.http.delete(&url).await
    }

    pub async fn get_diff(
        &self,
        workspace: &str,
        repo: &str,
        id: u64,
    ) -> Result<String, ApiError> {
        let url = self.repo_url(workspace, repo, &format!("/pullrequests/{id}/diff"));
        self.http.get_text(&url).await
    }

    pub async fn list_comments(
        &self,
        workspace: &str,
        repo: &str,
        id: u64,
    ) -> Result<Paginated<Comment>, ApiError> {
        let url = self.repo_url(workspace, repo, &format!("/pullrequests/{id}/comments"));
        self.http.get(&url).await
    }

    pub async fn add_comment(
        &self,
        workspace: &str,
        repo: &str,
        id: u64,
        body: &CreateComment,
    ) -> Result<Comment, ApiError> {
        let url = self.repo_url(workspace, repo, &format!("/pullrequests/{id}/comments"));
        self.http.post(&url, body).await
    }

    pub async fn get_statuses(
        &self,
        workspace: &str,
        repo: &str,
        id: u64,
    ) -> Result<Paginated<BuildStatus>, ApiError> {
        let url = self.repo_url(workspace, repo, &format!("/pullrequests/{id}/statuses"));
        self.http.get(&url).await
    }

    /// Search code within a workspace, optionally scoped to a repo.
    /// Cloud: GET /2.0/workspaces/{workspace}/search/code?search_query=...
    pub async fn search_code(
        &self,
        workspace: &str,
        repo: Option<&str>,
        query: &str,
        limit: u32,
        extension: Option<&str>,
        filename: Option<&str>,
    ) -> Result<Vec<CodeResult>, ApiError> {
        // Build search query with optional qualifiers
        let mut full_query = query.to_string();
        if let Some(ext) = extension {
            full_query.push_str(&format!(" extension:\"{ext}\""));
        }
        if let Some(fname) = filename {
            full_query.push_str(&format!(" file:\"{fname}\""));
        }

        let url = format!(
            "{}/workspaces/{}/search/code?search_query={}&pagelen={}",
            self.base_url,
            workspace,
            urlencoding::encode(&full_query),
            limit
        );

        let url = if let Some(r) = repo {
            format!("{}&scopes={}", url, urlencoding::encode(r))
        } else {
            url
        };

        let resp: CloudSearchResponse = self.http.get(&url).await?;
        let repo_label = repo.unwrap_or(workspace).to_string();
        Ok(resp
            .values
            .into_iter()
            .map(|r| r.into_code_result(repo_label.clone()))
            .collect())
    }

    // --- Repo methods ---

    pub async fn list_repos(
        &self,
        workspace: &str,
        limit: u32,
        visibility: Option<&str>,
    ) -> Result<Vec<RepoInfo>, ApiError> {
        let url = format!(
            "{}/repositories/{}?pagelen={}",
            self.base_url, workspace, limit
        );
        if let Some(v) = visibility {
            // Cloud uses role param + is_private filter; simplest approach: filter after
            let _ = v; // handled client-side below
        }
        let resp: Paginated<CloudRepo> = self.http.get(&url).await?;
        let mut repos: Vec<RepoInfo> = resp.values.into_iter().map(|r| r.into_repo_info()).collect();
        if let Some(v) = visibility {
            match v {
                "public" => repos.retain(|r| !r.is_private),
                "private" => repos.retain(|r| r.is_private),
                _ => {}
            }
        }
        Ok(repos)
    }

    pub async fn get_repo(&self, workspace: &str, repo: &str) -> Result<RepoInfo, ApiError> {
        let url = self.repo_url(workspace, repo, "");
        let r: CloudRepo = self.http.get(&url).await?;
        Ok(r.into_repo_info())
    }

    pub async fn create_repo(
        &self,
        workspace: &str,
        slug: &str,
        description: Option<String>,
        is_private: bool,
    ) -> Result<RepoInfo, ApiError> {
        let url = self.repo_url(workspace, slug, "");
        let body = CreateRepoCloud {
            scm: "git".to_string(),
            is_private,
            description,
        };
        let r: CloudRepo = self.http.post(&url, &body).await?;
        Ok(r.into_repo_info())
    }
}
