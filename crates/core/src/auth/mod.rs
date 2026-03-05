use base64::prelude::*;

use crate::config::Provider;

#[derive(Debug, Clone)]
pub enum Credentials {
    Token(String),
    AppPassword { username: String, app_password: String },
}

impl Credentials {
    pub fn auth_header(&self, provider: &Provider) -> String {
        match (self, provider) {
            (Credentials::Token(token), Provider::Cloud) => {
                let encoded = BASE64_STANDARD.encode(format!("x-token-auth:{token}"));
                format!("Basic {encoded}")
            }
            (Credentials::Token(token), Provider::Server { .. }) => {
                format!("Bearer {token}")
            }
            (Credentials::AppPassword { username, app_password }, _) => {
                let encoded = BASE64_STANDARD.encode(format!("{username}:{app_password}"));
                format!("Basic {encoded}")
            }
        }
    }
}
