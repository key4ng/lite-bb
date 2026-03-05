use serde::{Deserialize, Serialize};

/// Canonical code search result, normalized from both Cloud and Server responses.
#[derive(Debug, Serialize, Deserialize)]
pub struct CodeResult {
    pub path: String,
    pub repo: String,
    pub matches: Vec<MatchLine>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MatchLine {
    pub line: u32,
    pub content: String,
    pub highlighted: bool,
}

// --- Bitbucket Cloud response types ---

#[derive(Debug, Deserialize)]
pub struct CloudSearchResponse {
    pub values: Vec<CloudSearchResult>,
    pub next: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CloudSearchResult {
    pub file: CloudFile,
    pub content_matches: Vec<CloudContentMatch>,
}

#[derive(Debug, Deserialize)]
pub struct CloudFile {
    pub path: String,
    pub type_: Option<String>,
    pub links: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct CloudContentMatch {
    pub lines: Vec<CloudMatchLine>,
}

#[derive(Debug, Deserialize)]
pub struct CloudMatchLine {
    pub line: u32,
    pub segments: Vec<CloudSegment>,
}

#[derive(Debug, Deserialize)]
pub struct CloudSegment {
    pub text: String,
    #[serde(default)]
    pub match_: bool,
}

impl CloudSearchResult {
    pub fn into_code_result(self, repo: String) -> CodeResult {
        let mut matches = Vec::new();
        for content_match in self.content_matches {
            for line in content_match.lines {
                let content: String = line.segments.iter().map(|s| s.text.as_str()).collect();
                let highlighted = line.segments.iter().any(|s| s.match_);
                matches.push(MatchLine {
                    line: line.line,
                    content,
                    highlighted,
                });
            }
        }
        CodeResult {
            path: self.file.path,
            repo,
            matches,
        }
    }
}

// --- Bitbucket Server/DC response types ---

#[derive(Debug, Deserialize)]
pub struct ServerSearchResponse {
    pub code: Option<ServerCodePage>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerCodePage {
    pub values: Vec<ServerCodeResult>,
    pub is_last_page: bool,
    pub start: u32,
    pub next_page_start: Option<u32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerCodeResult {
    pub path_matches: Option<Vec<ServerHitContext>>,
    pub hit_contexts: Vec<ServerHitContext>,
    pub file: ServerFile,
    pub repository: Option<ServerRepository>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerHitContext {
    pub context: Vec<ServerContextLine>,
    pub lines: Option<Vec<ServerContextLine>>,
}

#[derive(Debug, Deserialize)]
pub struct ServerContextLine {
    pub text: String,
    #[serde(rename = "truncated", default)]
    pub truncated: bool,
}

#[derive(Debug, Deserialize)]
pub struct ServerFile {
    pub path: ServerPath,
    pub node: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerPath {
    #[serde(rename = "toString")]
    pub to_string: String,
}

#[derive(Debug, Deserialize)]
pub struct ServerRepository {
    pub slug: String,
    pub project: Option<ServerProject>,
}

#[derive(Debug, Deserialize)]
pub struct ServerProject {
    pub key: String,
}

impl ServerCodeResult {
    pub fn into_code_result(self) -> CodeResult {
        let repo = match &self.repository {
            Some(r) => {
                let project_key = r
                    .project
                    .as_ref()
                    .map(|p| p.key.as_str())
                    .unwrap_or("~");
                format!("{}/{}", project_key, r.slug)
            }
            None => "unknown".to_string(),
        };

        let mut matches = Vec::new();
        for (i, ctx) in self.hit_contexts.iter().enumerate() {
            let lines = ctx.context.iter().enumerate();
            for (j, line) in lines {
                // The middle line(s) in a context window are the matches
                let is_hit = j == self.hit_contexts.len().min(ctx.context.len() / 2);
                matches.push(MatchLine {
                    line: (i * ctx.context.len() + j + 1) as u32,
                    content: line.text.clone(),
                    highlighted: is_hit,
                });
            }
        }

        CodeResult {
            path: self.file.path.to_string,
            repo,
            matches,
        }
    }
}

// --- Server search request ---

#[derive(Debug, Serialize)]
pub struct ServerSearchRequest {
    pub query: String,
    pub entities: ServerSearchEntities,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scopes: Option<Vec<ServerSearchScope>>,
}

#[derive(Debug, Serialize)]
pub struct ServerSearchEntities {
    pub code: ServerCodeEntity,
}

#[derive(Debug, Serialize)]
pub struct ServerCodeEntity {
    pub start: u32,
    pub limit: u32,
}

#[derive(Debug, Serialize)]
pub struct ServerSearchScope {
    #[serde(rename = "type")]
    pub scope_type: String,
    pub key: String,
}
