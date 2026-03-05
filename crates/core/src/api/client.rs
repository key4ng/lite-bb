use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use thiserror::Error;

use crate::auth::Credentials;
use crate::config::Provider;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("{status}: {message}")]
    Api { status: u16, message: String },
}

pub struct HttpClient {
    pub(crate) http: reqwest::Client,
}

impl HttpClient {
    pub fn new(credentials: &Credentials, provider: &Provider) -> Result<Self, reqwest::Error> {
        let mut headers = HeaderMap::new();
        let auth_value = credentials.auth_header(provider);
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&auth_value).expect("invalid auth header"),
        );

        let http = reqwest::Client::builder()
            .default_headers(headers)
            .build()?;

        Ok(Self { http })
    }

    pub async fn check_status(&self, url: &str) -> Result<u16, ApiError> {
        let resp = self.http.get(url).send().await?;
        Ok(resp.status().as_u16())
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
