use crate::api::client::{ApiError, Client};
use crate::models::*;

impl Client {
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
        self.get(&url).await
    }

    pub async fn get_pr(
        &self,
        workspace: &str,
        repo: &str,
        id: u64,
    ) -> Result<PullRequest, ApiError> {
        let url = self.repo_url(workspace, repo, &format!("/pullrequests/{id}"));
        self.get(&url).await
    }

    pub async fn create_pr(
        &self,
        workspace: &str,
        repo: &str,
        body: &CreatePullRequest,
    ) -> Result<PullRequest, ApiError> {
        let url = self.repo_url(workspace, repo, "/pullrequests");
        self.post(&url, body).await
    }

    pub async fn update_pr(
        &self,
        workspace: &str,
        repo: &str,
        id: u64,
        body: &UpdatePullRequest,
    ) -> Result<PullRequest, ApiError> {
        let url = self.repo_url(workspace, repo, &format!("/pullrequests/{id}"));
        self.put(&url, body).await
    }

    pub async fn merge_pr(
        &self,
        workspace: &str,
        repo: &str,
        id: u64,
        body: &MergeRequest,
    ) -> Result<PullRequest, ApiError> {
        let url = self.repo_url(workspace, repo, &format!("/pullrequests/{id}/merge"));
        self.post(&url, body).await
    }

    pub async fn decline_pr(
        &self,
        workspace: &str,
        repo: &str,
        id: u64,
    ) -> Result<PullRequest, ApiError> {
        let url = self.repo_url(workspace, repo, &format!("/pullrequests/{id}/decline"));
        self.post(&url, &serde_json::json!({})).await
    }

    pub async fn approve_pr(
        &self,
        workspace: &str,
        repo: &str,
        id: u64,
    ) -> Result<(), ApiError> {
        let url = self.repo_url(workspace, repo, &format!("/pullrequests/{id}/approve"));
        self.post_empty(&url).await
    }

    pub async fn unapprove_pr(
        &self,
        workspace: &str,
        repo: &str,
        id: u64,
    ) -> Result<(), ApiError> {
        let url = self.repo_url(workspace, repo, &format!("/pullrequests/{id}/approve"));
        self.delete(&url).await
    }

    pub async fn get_diff(
        &self,
        workspace: &str,
        repo: &str,
        id: u64,
    ) -> Result<String, ApiError> {
        let url = self.repo_url(workspace, repo, &format!("/pullrequests/{id}/diff"));
        self.get_text(&url).await
    }

    pub async fn list_comments(
        &self,
        workspace: &str,
        repo: &str,
        id: u64,
    ) -> Result<Paginated<Comment>, ApiError> {
        let url = self.repo_url(workspace, repo, &format!("/pullrequests/{id}/comments"));
        self.get(&url).await
    }

    pub async fn add_comment(
        &self,
        workspace: &str,
        repo: &str,
        id: u64,
        body: &CreateComment,
    ) -> Result<Comment, ApiError> {
        let url = self.repo_url(workspace, repo, &format!("/pullrequests/{id}/comments"));
        self.post(&url, body).await
    }

    pub async fn get_statuses(
        &self,
        workspace: &str,
        repo: &str,
        id: u64,
    ) -> Result<Paginated<BuildStatus>, ApiError> {
        let url = self.repo_url(workspace, repo, &format!("/pullrequests/{id}/statuses"));
        self.get(&url).await
    }
}
