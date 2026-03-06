pub mod client;
pub mod cloud;
pub mod server;

use client::ApiError;
use cloud::CloudClient;
use server::ServerClient;

use crate::auth::Credentials;
use crate::config::Provider;
use crate::models::repo::RepoInfo;
use crate::models::search::CodeResult;
use crate::models::*;

pub enum ApiClient {
    Cloud(CloudClient),
    Server(ServerClient),
}

impl ApiClient {
    pub fn new(credentials: &Credentials, provider: &Provider) -> Result<Self, reqwest::Error> {
        match provider {
            Provider::Cloud => Ok(Self::Cloud(CloudClient::new(credentials)?)),
            Provider::Server { base_url } => {
                Ok(Self::Server(ServerClient::new(credentials, base_url)?))
            }
        }
    }

    pub async fn verify(&self) -> Result<String, ApiError> {
        match self {
            Self::Cloud(c) => c.verify().await,
            Self::Server(s) => s.verify().await,
        }
    }

    pub async fn list_prs(
        &self,
        workspace: &str,
        repo: &str,
        state: Option<&str>,
        page: Option<u32>,
        pagelen: Option<u32>,
    ) -> Result<Paginated<PullRequest>, ApiError> {
        match self {
            Self::Cloud(c) => c.list_prs(workspace, repo, state, page, pagelen).await,
            Self::Server(s) => s.list_prs(workspace, repo, state, page, pagelen).await,
        }
    }

    pub async fn get_pr(
        &self,
        workspace: &str,
        repo: &str,
        id: u64,
    ) -> Result<PullRequest, ApiError> {
        match self {
            Self::Cloud(c) => c.get_pr(workspace, repo, id).await,
            Self::Server(s) => s.get_pr(workspace, repo, id).await,
        }
    }

    pub async fn create_pr(
        &self,
        workspace: &str,
        repo: &str,
        body: &CreatePullRequest,
    ) -> Result<PullRequest, ApiError> {
        match self {
            Self::Cloud(c) => c.create_pr(workspace, repo, body).await,
            Self::Server(s) => s.create_pr(workspace, repo, body).await,
        }
    }

    pub async fn update_pr(
        &self,
        workspace: &str,
        repo: &str,
        id: u64,
        body: &UpdatePullRequest,
    ) -> Result<PullRequest, ApiError> {
        match self {
            Self::Cloud(c) => c.update_pr(workspace, repo, id, body).await,
            Self::Server(s) => s.update_pr(workspace, repo, id, body).await,
        }
    }

    pub async fn merge_pr(
        &self,
        workspace: &str,
        repo: &str,
        id: u64,
        body: &MergeRequest,
    ) -> Result<PullRequest, ApiError> {
        match self {
            Self::Cloud(c) => c.merge_pr(workspace, repo, id, body).await,
            Self::Server(s) => s.merge_pr(workspace, repo, id, body).await,
        }
    }

    pub async fn decline_pr(
        &self,
        workspace: &str,
        repo: &str,
        id: u64,
    ) -> Result<PullRequest, ApiError> {
        match self {
            Self::Cloud(c) => c.decline_pr(workspace, repo, id).await,
            Self::Server(s) => s.decline_pr(workspace, repo, id).await,
        }
    }

    pub async fn approve_pr(
        &self,
        workspace: &str,
        repo: &str,
        id: u64,
    ) -> Result<(), ApiError> {
        match self {
            Self::Cloud(c) => c.approve_pr(workspace, repo, id).await,
            Self::Server(s) => s.approve_pr(workspace, repo, id).await,
        }
    }

    pub async fn unapprove_pr(
        &self,
        workspace: &str,
        repo: &str,
        id: u64,
    ) -> Result<(), ApiError> {
        match self {
            Self::Cloud(c) => c.unapprove_pr(workspace, repo, id).await,
            Self::Server(s) => s.unapprove_pr(workspace, repo, id).await,
        }
    }

    pub async fn get_diff(
        &self,
        workspace: &str,
        repo: &str,
        id: u64,
    ) -> Result<String, ApiError> {
        match self {
            Self::Cloud(c) => c.get_diff(workspace, repo, id).await,
            Self::Server(s) => s.get_diff(workspace, repo, id).await,
        }
    }

    pub async fn list_comments(
        &self,
        workspace: &str,
        repo: &str,
        id: u64,
    ) -> Result<Paginated<Comment>, ApiError> {
        match self {
            Self::Cloud(c) => c.list_comments(workspace, repo, id).await,
            Self::Server(s) => s.list_comments(workspace, repo, id).await,
        }
    }

    pub async fn add_comment(
        &self,
        workspace: &str,
        repo: &str,
        id: u64,
        body: &CreateComment,
    ) -> Result<Comment, ApiError> {
        match self {
            Self::Cloud(c) => c.add_comment(workspace, repo, id, body).await,
            Self::Server(s) => s.add_comment(workspace, repo, id, body).await,
        }
    }

    pub async fn get_statuses(
        &self,
        workspace: &str,
        repo: &str,
        id: u64,
    ) -> Result<Paginated<BuildStatus>, ApiError> {
        match self {
            Self::Cloud(c) => c.get_statuses(workspace, repo, id).await,
            Self::Server(s) => s.get_statuses(workspace, repo, id).await,
        }
    }

    pub async fn search_code(
        &self,
        workspace: &str,
        repo: Option<&str>,
        query: &str,
        limit: u32,
        extension: Option<&str>,
        filename: Option<&str>,
    ) -> Result<Vec<CodeResult>, ApiError> {
        match self {
            Self::Cloud(c) => {
                c.search_code(workspace, repo, query, limit, extension, filename)
                    .await
            }
            Self::Server(s) => {
                s.search_code(repo, query, limit, extension, filename).await
            }
        }
    }

    pub async fn list_repos(
        &self,
        workspace: &str,
        limit: u32,
        visibility: Option<&str>,
    ) -> Result<Vec<RepoInfo>, ApiError> {
        match self {
            Self::Cloud(c) => c.list_repos(workspace, limit, visibility).await,
            Self::Server(s) => s.list_repos(workspace, limit, visibility).await,
        }
    }

    pub async fn get_repo(&self, workspace: &str, repo: &str) -> Result<RepoInfo, ApiError> {
        match self {
            Self::Cloud(c) => c.get_repo(workspace, repo).await,
            Self::Server(s) => s.get_repo(workspace, repo).await,
        }
    }

    pub async fn create_repo(
        &self,
        workspace: &str,
        slug: &str,
        description: Option<String>,
        is_private: bool,
    ) -> Result<RepoInfo, ApiError> {
        match self {
            Self::Cloud(c) => c.create_repo(workspace, slug, description, is_private).await,
            Self::Server(s) => s.create_repo(workspace, slug, description, is_private).await,
        }
    }
}
