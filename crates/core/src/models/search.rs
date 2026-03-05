use serde::{Deserialize, Serialize};

fn decode_html_entities(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#x27;", "'")
        .replace("&#x2F;", "/")
        .replace("&apos;", "'")
        .replace("<em>", "")
        .replace("</em>", "")
}

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
    pub next_start: Option<u32>,
}

/// A single search hit from Bitbucket Server/DC.
/// `hit_contexts` is a Vec of context windows; each window is a Vec of lines.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerCodeResult {
    pub hit_contexts: Vec<Vec<ServerContextLine>>,
    /// Plain path string, e.g. "src/main.rs"
    pub file: String,
    pub repository: Option<ServerRepository>,
}

#[derive(Debug, Deserialize)]
pub struct ServerContextLine {
    pub line: u32,
    pub text: String,
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
                let project_key = r.project.as_ref().map(|p| p.key.as_str()).unwrap_or("~");
                format!("{}/{}", project_key, r.slug)
            }
            None => "unknown".to_string(),
        };

        let mut matches = Vec::new();
        for window in &self.hit_contexts {
            let mid = window.len() / 2;
            for (j, line) in window.iter().enumerate() {
                matches.push(MatchLine {
                    line: line.line,
                    content: decode_html_entities(&line.text),
                    // Middle line of each context window is the actual match
                    highlighted: j == mid,
                });
            }
        }

        CodeResult {
            path: self.file.clone(),
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
