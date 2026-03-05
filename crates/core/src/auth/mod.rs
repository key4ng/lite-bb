use base64::prelude::*;

#[derive(Debug, Clone)]
pub enum Credentials {
    Token(String),
    AppPassword { username: String, app_password: String },
}

impl Credentials {
    pub fn basic_auth_header(&self) -> String {
        let encoded = match self {
            Credentials::Token(token) => {
                BASE64_STANDARD.encode(format!("x-token-auth:{token}"))
            }
            Credentials::AppPassword { username, app_password } => {
                BASE64_STANDARD.encode(format!("{username}:{app_password}"))
            }
        };
        format!("Basic {encoded}")
    }
}
