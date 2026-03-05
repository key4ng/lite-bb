use crate::api::client::{ApiError, HttpClient};
use crate::auth::Credentials;
use crate::config::Provider;
use crate::models::*;

const BASE_URL: &str = "https://api.bitbucket.org/2.0";

pub struct CloudClient {
    http: HttpClient,
    base_url: String,
}

impl CloudClient {
    pub fn new(credentials: &Credentials) -> Result<Self, reqwest::Error> {
        let http = HttpClient::new(credentials, &Provider::Cloud)?;
        Ok(Self {
            http,
            base_url: BASE_URL.to_string(),
        })
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
}
