use crate::api::client::{ApiError, HttpClient};
use crate::auth::Credentials;
use crate::config::Provider;
use crate::models::server::*;
use crate::models::*;

pub struct ServerClient {
    http: HttpClient,
    base_url: String,
}

impl ServerClient {
    pub fn new(credentials: &Credentials, server_url: &str) -> Result<Self, reqwest::Error> {
        let provider = Provider::Server {
            base_url: server_url.to_string(),
        };
        let http = HttpClient::new(credentials, &provider)?;
        Ok(Self {
            http,
            base_url: format!("{}/rest/api/1.0", server_url.trim_end_matches('/')),
        })
    }

    pub async fn verify(&self) -> Result<String, ApiError> {
        // DC: GET /rest/api/1.0/application-properties returns server info
        // If auth fails, we get 401. If it succeeds, we know credentials work.
        let url = format!("{}/application-properties", self.base_url);
        let status = self.http.check_status(&url).await?;
        if status == 401 || status == 403 {
            return Err(ApiError::Api {
                status,
                message: "authentication failed".to_string(),
            });
        }
        Ok("authenticated".to_string())
    }

    fn pr_url(&self, project: &str, repo: &str, path: &str) -> String {
        format!(
            "{}/projects/{}/repos/{}/pull-requests{}",
            self.base_url, project, repo, path
        )
    }

    pub async fn list_prs(
        &self,
        project: &str,
        repo: &str,
        state: Option<&str>,
        _page: Option<u32>,
        limit: Option<u32>,
    ) -> Result<Paginated<PullRequest>, ApiError> {
        let mut url = self.pr_url(project, repo, "");
        let mut params = Vec::new();
        if let Some(state) = state {
            params.push(format!("state={state}"));
        }
        if let Some(limit) = limit {
            params.push(format!("limit={limit}"));
        }
        if !params.is_empty() {
            url = format!("{}?{}", url, params.join("&"));
        }
        let result: DcPaginated<DcPullRequest> = self.http.get(&url).await?;
        Ok(result.into_paginated(PullRequest::from))
    }

    pub async fn get_pr(
        &self,
        project: &str,
        repo: &str,
        id: u64,
    ) -> Result<PullRequest, ApiError> {
        let url = self.pr_url(project, repo, &format!("/{id}"));
        let dc_pr: DcPullRequest = self.http.get(&url).await?;
        Ok(dc_pr.into())
    }

    async fn get_dc_pr(
        &self,
        project: &str,
        repo: &str,
        id: u64,
    ) -> Result<DcPullRequest, ApiError> {
        let url = self.pr_url(project, repo, &format!("/{id}"));
        self.http.get(&url).await
    }

    pub async fn create_pr(
        &self,
        project: &str,
        repo: &str,
        body: &CreatePullRequest,
    ) -> Result<PullRequest, ApiError> {
        let url = self.pr_url(project, repo, "");
        let dc_body = DcCreatePullRequest {
            title: body.title.clone(),
            description: body.description.clone(),
            from_ref: DcRefRequest {
                id: format!("refs/heads/{}", body.source.branch.name),
            },
            to_ref: body.destination.as_ref().map(|d| DcRefRequest {
                id: format!("refs/heads/{}", d.branch.name),
            }),
            reviewers: None,
        };
        let dc_pr: DcPullRequest = self.http.post(&url, &dc_body).await?;
        Ok(dc_pr.into())
    }

    pub async fn update_pr(
        &self,
        project: &str,
        repo: &str,
        id: u64,
        body: &UpdatePullRequest,
    ) -> Result<PullRequest, ApiError> {
        let dc_pr = self.get_dc_pr(project, repo, id).await?;
        let url = self.pr_url(project, repo, &format!("/{id}"));
        let dc_body = DcUpdatePullRequest {
            version: dc_pr.version,
            title: body.title.clone(),
            description: body.description.clone(),
            to_ref: body.destination.as_ref().map(|d| DcRefRequest {
                id: format!("refs/heads/{}", d.branch.name),
            }),
        };
        let updated: DcPullRequest = self.http.put(&url, &dc_body).await?;
        Ok(updated.into())
    }

    pub async fn merge_pr(
        &self,
        project: &str,
        repo: &str,
        id: u64,
        body: &MergeRequest,
    ) -> Result<PullRequest, ApiError> {
        let dc_pr = self.get_dc_pr(project, repo, id).await?;
        let url = format!(
            "{}?version={}",
            self.pr_url(project, repo, &format!("/{id}/merge")),
            dc_pr.version
        );
        let dc_body = DcMergeRequest {
            message: body.message.clone(),
        };
        let merged: DcPullRequest = self.http.post(&url, &dc_body).await?;
        Ok(merged.into())
    }

    pub async fn decline_pr(
        &self,
        project: &str,
        repo: &str,
        id: u64,
    ) -> Result<PullRequest, ApiError> {
        let dc_pr = self.get_dc_pr(project, repo, id).await?;
        let url = format!(
            "{}?version={}",
            self.pr_url(project, repo, &format!("/{id}/decline")),
            dc_pr.version
        );
        let declined: DcPullRequest = self.http.post(&url, &serde_json::json!({})).await?;
        Ok(declined.into())
    }

    pub async fn approve_pr(
        &self,
        project: &str,
        repo: &str,
        id: u64,
    ) -> Result<(), ApiError> {
        let url = self.pr_url(project, repo, &format!("/{id}/approve"));
        self.http.post_empty(&url).await
    }

    pub async fn unapprove_pr(
        &self,
        project: &str,
        repo: &str,
        id: u64,
    ) -> Result<(), ApiError> {
        let url = self.pr_url(project, repo, &format!("/{id}/approve"));
        self.http.delete(&url).await
    }

    pub async fn get_diff(
        &self,
        project: &str,
        repo: &str,
        id: u64,
    ) -> Result<String, ApiError> {
        let url = self.pr_url(project, repo, &format!("/{id}/diff"));
        let diff_response: DcDiffResponse = self.http.get(&url).await?;
        Ok(diff_response.to_unified_diff())
    }

    pub async fn list_comments(
        &self,
        project: &str,
        repo: &str,
        id: u64,
    ) -> Result<Paginated<Comment>, ApiError> {
        let url = format!("{}?limit=100", self.pr_url(project, repo, &format!("/{id}/activities")));
        let result: DcPaginated<DcActivity> = self.http.get(&url).await?;
        let comments: Vec<Comment> = result
            .values
            .into_iter()
            .filter(|a| a.action == "COMMENTED")
            .filter_map(|a| a.comment)
            .map(Comment::from)
            .collect();
        let size = comments.len() as u32;
        Ok(Paginated {
            pagelen: size,
            size: Some(size),
            page: Some(1),
            next: None,
            previous: None,
            values: comments,
        })
    }

    pub async fn add_comment(
        &self,
        project: &str,
        repo: &str,
        id: u64,
        body: &CreateComment,
    ) -> Result<Comment, ApiError> {
        let url = self.pr_url(project, repo, &format!("/{id}/comments"));
        let dc_body = serde_json::json!({
            "text": body.content.raw.as_deref().unwrap_or("")
        });
        let dc_comment: DcComment = self.http.post(&url, &dc_body).await?;
        Ok(dc_comment.into())
    }

    pub async fn get_statuses(
        &self,
        project: &str,
        repo: &str,
        id: u64,
    ) -> Result<Paginated<BuildStatus>, ApiError> {
        // DC requires the commit hash for build statuses
        let dc_pr = self.get_dc_pr(project, repo, id).await?;
        let commit = dc_pr
            .from_ref
            .latest_commit
            .unwrap_or_default();

        if commit.is_empty() {
            return Ok(Paginated {
                pagelen: 0,
                size: Some(0),
                page: Some(1),
                next: None,
                previous: None,
                values: vec![],
            });
        }

        let url = format!(
            "{}/rest/build-status/1.0/commits/{}",
            self.base_url.trim_end_matches("/rest/api/1.0"),
            commit
        );
        let result: DcPaginated<DcBuildStatus> = self.http.get(&url).await?;
        Ok(result.into_paginated(BuildStatus::from))
    }
}
