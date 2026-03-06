use serde::{Deserialize, Serialize};

/// Canonical repository info, normalized from Cloud and Server/DC responses.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RepoInfo {
    pub slug: String,
    pub full_name: String,
    pub name: String,
    pub description: Option<String>,
    pub is_private: bool,
    pub scm: String,
    pub clone_url_https: Option<String>,
    pub clone_url_ssh: Option<String>,
    pub default_branch: Option<String>,
    pub created_on: Option<String>,
    pub updated_on: Option<String>,
    pub language: Option<String>,
    pub size: Option<u64>,
    pub forks_count: Option<u32>,
    pub has_wiki: Option<bool>,
    pub has_issues: Option<bool>,
    pub web_url: Option<String>,
    pub parent: Option<String>,
}

// --- Bitbucket Cloud types ---

#[derive(Debug, Deserialize)]
pub struct CloudRepo {
    pub slug: Option<String>,
    pub full_name: String,
    pub name: String,
    pub description: Option<String>,
    pub is_private: bool,
    pub scm: Option<String>,
    pub created_on: Option<String>,
    pub updated_on: Option<String>,
    pub language: Option<String>,
    pub size: Option<u64>,
    pub has_wiki: Option<bool>,
    pub has_issues: Option<bool>,
    pub mainbranch: Option<CloudBranch>,
    pub links: Option<CloudRepoLinks>,
    pub parent: Option<CloudRepoRef>,
}

#[derive(Debug, Deserialize)]
pub struct CloudBranch {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct CloudRepoLinks {
    pub clone: Option<Vec<CloudCloneLink>>,
    pub html: Option<CloudHref>,
}

#[derive(Debug, Deserialize)]
pub struct CloudCloneLink {
    pub href: String,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct CloudHref {
    pub href: String,
}

#[derive(Debug, Deserialize)]
pub struct CloudRepoRef {
    pub full_name: String,
}

impl CloudRepo {
    pub fn into_repo_info(self) -> RepoInfo {
        let slug = self
            .slug
            .clone()
            .unwrap_or_else(|| self.full_name.split('/').last().unwrap_or("").to_string());

        let clone_url_https = self.links.as_ref().and_then(|l| {
            l.clone.as_ref().and_then(|links| {
                links
                    .iter()
                    .find(|c| c.name == "https")
                    .map(|c| c.href.clone())
            })
        });
        let clone_url_ssh = self.links.as_ref().and_then(|l| {
            l.clone.as_ref().and_then(|links| {
                links
                    .iter()
                    .find(|c| c.name == "ssh")
                    .map(|c| c.href.clone())
            })
        });
        let web_url = self
            .links
            .as_ref()
            .and_then(|l| l.html.as_ref().map(|h| h.href.clone()));

        RepoInfo {
            slug,
            full_name: self.full_name,
            name: self.name,
            description: self.description,
            is_private: self.is_private,
            scm: self.scm.unwrap_or_else(|| "git".to_string()),
            clone_url_https,
            clone_url_ssh,
            default_branch: self.mainbranch.map(|b| b.name),
            created_on: self.created_on,
            updated_on: self.updated_on,
            language: self.language,
            size: self.size,
            forks_count: None,
            has_wiki: self.has_wiki,
            has_issues: self.has_issues,
            web_url,
            parent: self.parent.map(|p| p.full_name),
        }
    }
}

// --- Bitbucket Server/DC types ---

#[derive(Debug, Deserialize)]
pub struct DcRepo {
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub public: Option<bool>,
    pub forkable: Option<bool>,
    pub project: Option<DcRepoProject>,
    pub links: Option<DcRepoLinks>,
    pub origin: Option<Box<DcRepo>>,
}

#[derive(Debug, Deserialize)]
pub struct DcRepoProject {
    pub key: String,
    pub name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DcRepoLinks {
    pub clone: Option<Vec<DcCloneLink>>,
    #[serde(rename = "self")]
    pub self_links: Option<Vec<DcHref>>,
}

#[derive(Debug, Deserialize)]
pub struct DcCloneLink {
    pub href: String,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct DcHref {
    pub href: String,
}

impl DcRepo {
    pub fn into_repo_info(self, default_branch: Option<String>) -> RepoInfo {
        let project_key = self
            .project
            .as_ref()
            .map(|p| p.key.as_str())
            .unwrap_or("UNKNOWN");
        let full_name = format!("{}/{}", project_key, self.slug);

        let clone_url_https = self.links.as_ref().and_then(|l| {
            l.clone.as_ref().and_then(|links| {
                links
                    .iter()
                    .find(|c| c.name == "http" || c.name == "https")
                    .map(|c| c.href.clone())
            })
        });
        let clone_url_ssh = self.links.as_ref().and_then(|l| {
            l.clone.as_ref().and_then(|links| {
                links
                    .iter()
                    .find(|c| c.name == "ssh")
                    .map(|c| c.href.clone())
            })
        });
        let web_url = self.links.as_ref().and_then(|l| {
            l.self_links
                .as_ref()
                .and_then(|s| s.first().map(|h| h.href.clone()))
        });

        let parent = self
            .origin
            .map(|o| format!("{}/{}", o.project.as_ref().map(|p| p.key.as_str()).unwrap_or("?"), o.slug));

        RepoInfo {
            slug: self.slug,
            full_name,
            name: self.name,
            description: self.description,
            is_private: !self.public.unwrap_or(false),
            scm: "git".to_string(),
            clone_url_https,
            clone_url_ssh,
            default_branch,
            created_on: None,
            updated_on: None,
            language: None,
            size: None,
            forks_count: None,
            has_wiki: None,
            has_issues: None,
            web_url,
            parent,
        }
    }
}

// --- Create request types ---

#[derive(Debug, Serialize)]
pub struct CreateRepoCloud {
    pub scm: String,
    pub is_private: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CreateRepoServer {
    pub name: String,
    #[serde(rename = "scmId")]
    pub scm_id: String,
    pub forkable: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}
