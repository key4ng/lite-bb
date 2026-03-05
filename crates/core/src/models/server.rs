use serde::Deserialize;

use super::*;

// Bitbucket Data Center / Server response types
// These map to the DC REST API v1.0 and are converted to canonical models.

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DcPaginated<T> {
    pub size: Option<u32>,
    pub limit: u32,
    pub is_last_page: bool,
    pub start: u32,
    pub next_page_start: Option<u32>,
    pub values: Vec<T>,
}

impl<T> DcPaginated<T> {
    pub fn into_paginated<U>(self, convert: impl Fn(T) -> U) -> Paginated<U> {
        Paginated {
            pagelen: self.limit,
            size: self.size,
            page: Some(self.start / self.limit + 1),
            next: self.next_page_start.map(|s| format!("start={s}")),
            previous: None,
            values: self.values.into_iter().map(convert).collect(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DcUser {
    pub name: String,
    pub display_name: String,
    pub slug: Option<String>,
    pub email_address: Option<String>,
}

impl From<DcUser> for User {
    fn from(u: DcUser) -> Self {
        User {
            display_name: u.display_name,
            uuid: None,
            nickname: Some(u.name),
            user_type: None,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DcParticipant {
    pub user: DcUser,
    pub role: Option<String>,
    pub approved: Option<bool>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DcRef {
    pub id: String,
    pub display_id: String,
    pub latest_commit: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DcPullRequest {
    pub id: u64,
    pub version: u32,
    pub title: String,
    pub description: Option<String>,
    pub state: String,
    pub open: Option<bool>,
    pub closed: Option<bool>,
    pub created_date: i64,
    pub updated_date: i64,
    pub from_ref: DcRef,
    pub to_ref: DcRef,
    pub author: DcParticipant,
    pub reviewers: Option<Vec<DcParticipant>>,
    pub closed_date: Option<i64>,
}

fn millis_to_iso(millis: i64) -> String {
    let secs = millis / 1000;
    let nanos = ((millis % 1000) * 1_000_000) as u32;
    let dt = chrono::DateTime::from_timestamp(secs, nanos)
        .unwrap_or_default();
    dt.to_rfc3339()
}

impl From<DcPullRequest> for PullRequest {
    fn from(pr: DcPullRequest) -> Self {
        PullRequest {
            id: pr.id,
            title: pr.title,
            description: pr.description,
            state: pr.state,
            author: pr.author.user.into(),
            source: Destination {
                branch: Branch {
                    name: pr.from_ref.display_id,
                },
                repository: None,
            },
            destination: Destination {
                branch: Branch {
                    name: pr.to_ref.display_id,
                },
                repository: None,
            },
            created_on: millis_to_iso(pr.created_date),
            updated_on: millis_to_iso(pr.updated_date),
            close_source_branch: None,
            merge_commit: None,
            reviewers: pr.reviewers.map(|rs| {
                rs.into_iter()
                    .map(|r| r.user.into())
                    .collect()
            }),
            comment_count: None,
            task_count: None,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DcComment {
    pub id: u64,
    pub version: Option<u32>,
    pub text: String,
    pub author: DcUser,
    pub created_date: i64,
    pub updated_date: i64,
    pub severity: Option<String>,
    pub state: Option<String>,
    pub anchor: Option<DcAnchor>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DcAnchor {
    pub path: Option<String>,
    pub line: Option<u32>,
    pub line_type: Option<String>,
    pub file_type: Option<String>,
}

impl From<DcComment> for Comment {
    fn from(c: DcComment) -> Self {
        let inline = c.anchor.and_then(|a| {
            a.path.map(|path| InlineComment {
                from: None,
                to: a.line,
                path,
            })
        });
        Comment {
            id: c.id,
            content: CommentContent {
                raw: Some(c.text),
                markup: None,
                html: None,
            },
            user: c.author.into(),
            created_on: millis_to_iso(c.created_date),
            updated_on: Some(millis_to_iso(c.updated_date)),
            inline,
            parent: None,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DcActivity {
    pub id: u64,
    pub action: String,
    pub comment: Option<DcComment>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DcBuildStatus {
    pub state: String,
    pub key: Option<String>,
    pub name: Option<String>,
    pub url: Option<String>,
    pub description: Option<String>,
    pub date_added: Option<i64>,
}

impl From<DcBuildStatus> for BuildStatus {
    fn from(s: DcBuildStatus) -> Self {
        BuildStatus {
            state: s.state,
            name: s.name,
            description: s.description,
            url: s.url,
            key: s.key,
            created_on: s.date_added.map(millis_to_iso),
            updated_on: None,
        }
    }
}

// DC-specific diff response types
#[derive(Debug, Deserialize)]
pub struct DcDiffResponse {
    pub diffs: Vec<DcFileDiff>,
}

#[derive(Debug, Deserialize)]
pub struct DcFileDiff {
    pub source: Option<DcDiffPath>,
    pub destination: Option<DcDiffPath>,
    pub hunks: Option<Vec<DcHunk>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DcDiffPath {
    pub to_string: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DcHunk {
    pub source_line: u32,
    pub source_span: u32,
    pub destination_line: u32,
    pub destination_span: u32,
    pub segments: Vec<DcSegment>,
}

#[derive(Debug, Deserialize)]
pub struct DcSegment {
    #[serde(rename = "type")]
    pub seg_type: String,
    pub lines: Vec<DcLine>,
}

#[derive(Debug, Deserialize)]
pub struct DcLine {
    pub line: String,
}

impl DcDiffResponse {
    pub fn to_unified_diff(&self) -> String {
        let mut output = String::new();
        for file_diff in &self.diffs {
            let src = file_diff
                .source
                .as_ref()
                .map(|p| p.to_string.as_str())
                .unwrap_or("/dev/null");
            let dst = file_diff
                .destination
                .as_ref()
                .map(|p| p.to_string.as_str())
                .unwrap_or("/dev/null");

            output.push_str(&format!("--- a/{src}\n"));
            output.push_str(&format!("+++ b/{dst}\n"));

            if let Some(hunks) = &file_diff.hunks {
                for hunk in hunks {
                    output.push_str(&format!(
                        "@@ -{},{} +{},{} @@\n",
                        hunk.source_line,
                        hunk.source_span,
                        hunk.destination_line,
                        hunk.destination_span
                    ));
                    for segment in &hunk.segments {
                        let prefix = match segment.seg_type.as_str() {
                            "ADDED" => "+",
                            "REMOVED" => "-",
                            _ => " ",
                        };
                        for line in &segment.lines {
                            output.push_str(&format!("{prefix}{}\n", line.line));
                        }
                    }
                }
            }
        }
        output
    }
}

// DC-specific request types
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DcCreatePullRequest {
    pub title: String,
    pub description: Option<String>,
    pub from_ref: DcRefRequest,
    pub to_ref: Option<DcRefRequest>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reviewers: Option<Vec<DcReviewerRequest>>,
}

#[derive(Debug, serde::Serialize)]
pub struct DcRefRequest {
    pub id: String,
}

#[derive(Debug, serde::Serialize)]
pub struct DcReviewerRequest {
    pub user: DcUserRef,
}

#[derive(Debug, serde::Serialize)]
pub struct DcUserRef {
    pub name: String,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DcUpdatePullRequest {
    pub version: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to_ref: Option<DcRefRequest>,
}

#[derive(Debug, serde::Serialize)]
pub struct DcMergeRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}
