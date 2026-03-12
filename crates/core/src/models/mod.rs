pub mod repo;
pub mod search;
pub mod server;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Paginated<T> {
    pub pagelen: u32,
    pub size: Option<u32>,
    pub page: Option<u32>,
    pub next: Option<String>,
    pub previous: Option<String>,
    pub values: Vec<T>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub display_name: String,
    pub uuid: Option<String>,
    pub nickname: Option<String>,
    #[serde(rename = "type")]
    pub user_type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Branch {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Repository {
    pub full_name: String,
    pub name: String,
    pub uuid: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Destination {
    pub branch: Branch,
    pub repository: Option<Repository>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PullRequest {
    pub id: u64,
    pub title: String,
    pub description: Option<String>,
    pub state: String,
    pub author: User,
    pub source: Destination,
    pub destination: Destination,
    pub created_on: String,
    pub updated_on: String,
    pub close_source_branch: Option<bool>,
    pub merge_commit: Option<serde_json::Value>,
    pub reviewers: Option<Vec<User>>,
    pub comment_count: Option<u32>,
    pub task_count: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatePullRequest {
    pub title: String,
    pub source: Destination,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination: Option<Destination>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub close_source_branch: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reviewers: Option<Vec<ReviewerRef>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReviewerRef {
    pub uuid: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdatePullRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination: Option<Destination>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MergeRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub merge_strategy: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub close_source_branch: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Comment {
    pub id: u64,
    pub content: CommentContent,
    pub user: User,
    pub created_on: String,
    pub updated_on: Option<String>,
    pub inline: Option<InlineComment>,
    pub parent: Option<CommentRef>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommentContent {
    pub raw: Option<String>,
    pub markup: Option<String>,
    pub html: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InlineComment {
    pub from: Option<u32>,
    pub to: Option<u32>,
    pub path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommentRef {
    pub id: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateComment {
    pub content: CommentContent,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inline: Option<InlineComment>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BuildStatus {
    pub state: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub url: Option<String>,
    pub key: Option<String>,
    pub created_on: Option<String>,
    pub updated_on: Option<String>,
}
