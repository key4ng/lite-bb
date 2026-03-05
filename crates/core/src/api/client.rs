use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use thiserror::Error;

use crate::auth::Credentials;

const BASE_URL: &str = "https://api.bitbucket.org/2.0";

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("{status}: {message}")]
    Api { status: u16, message: String },
}

pub struct Client {
    http: reqwest::Client,
    base_url: String,
}

impl Client {
    pub fn new(credentials: &Credentials) -> Result<Self, reqwest::Error> {
        let mut headers = HeaderMap::new();
        let auth_value = credentials.basic_auth_header();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&auth_value).expect("invalid auth header"),
        );

        let http = reqwest::Client::builder()
            .default_headers(headers)
            .build()?;

        Ok(Self {
            http,
            base_url: BASE_URL.to_string(),
        })
    }

    pub fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    pub fn repo_url(&self, workspace: &str, repo: &str, path: &str) -> String {
        format!(
            "{}/repositories/{}/{}{}",
            self.base_url, workspace, repo, path
        )
    }

    pub async fn get<T: serde::de::DeserializeOwned>(&self, url: &str) -> Result<T, ApiError> {
        let resp = self.http.get(url).send().await?;
        Self::handle_response(resp).await
    }

    pub async fn get_text(&self, url: &str) -> Result<String, ApiError> {
        let resp = self.http.get(url).send().await?;
        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let message = resp.text().await.unwrap_or_default();
            return Err(ApiError::Api { status, message });
        }
        Ok(resp.text().await?)
    }

    pub async fn post<T, B>(&self, url: &str, body: &B) -> Result<T, ApiError>
    where
        T: serde::de::DeserializeOwned,
        B: serde::Serialize,
    {
        let resp = self.http.post(url).json(body).send().await?;
        Self::handle_response(resp).await
    }

    pub async fn post_empty(&self, url: &str) -> Result<(), ApiError> {
        let resp = self.http.post(url).send().await?;
        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let message = resp.text().await.unwrap_or_default();
            return Err(ApiError::Api { status, message });
        }
        Ok(())
    }

    pub async fn put<T, B>(&self, url: &str, body: &B) -> Result<T, ApiError>
    where
        T: serde::de::DeserializeOwned,
        B: serde::Serialize,
    {
        let resp = self.http.put(url).json(body).send().await?;
        Self::handle_response(resp).await
    }

    pub async fn delete(&self, url: &str) -> Result<(), ApiError> {
        let resp = self.http.delete(url).send().await?;
        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let message = resp.text().await.unwrap_or_default();
            return Err(ApiError::Api { status, message });
        }
        Ok(())
    }

    async fn handle_response<T: serde::de::DeserializeOwned>(
        resp: reqwest::Response,
    ) -> Result<T, ApiError> {
        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let message = resp.text().await.unwrap_or_default();
            return Err(ApiError::Api { status, message });
        }
        Ok(resp.json().await?)
    }
}
