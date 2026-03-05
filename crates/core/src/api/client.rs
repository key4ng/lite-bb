use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use crate::auth::Credentials;

const BASE_URL: &str = "https://api.bitbucket.org/2.0";

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

    pub fn http(&self) -> &reqwest::Client {
        &self.http
    }

    pub fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }
}
