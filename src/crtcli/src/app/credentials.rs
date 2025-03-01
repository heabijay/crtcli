use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Serialize, Deserialize, Hash)]
pub struct CrtCredentials {
    url: String,
    username: String,
    password: String,
}

impl CrtCredentials {
    pub fn new(url: &str, username: &str, password: &str) -> CrtCredentials {
        CrtCredentials {
            url: url.trim_end_matches("/").to_owned(),
            username: username.to_owned(),
            password: password.to_owned(),
        }
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn password(&self) -> &str {
        &self.password
    }
}
