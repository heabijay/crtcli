use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Serialize, Deserialize, Hash)]
pub enum CrtCredentials {
    Basic {
        url: String,
        username: String,
        password: String,
    },
    OAuth {
        url: String,
        oauth_url: String,
        oauth_client_id: String,
        oauth_client_secret: String,
    },
}

impl CrtCredentials {
    pub fn new(url: &str, username: &str, password: &str) -> CrtCredentials {
        CrtCredentials::Basic {
            url: url.trim_end_matches("/").to_owned(),
            username: username.to_owned(),
            password: password.to_owned(),
        }
    }

    pub fn new_oauth(
        url: &str,
        oauth_url: &str,
        oauth_client_id: &str,
        oauth_client_secret: &str,
    ) -> CrtCredentials {
        CrtCredentials::OAuth {
            url: url.trim_end_matches("/").to_owned(),
            oauth_url: oauth_url
                .trim_end_matches("/")
                .trim_end_matches("/connect/token")
                .to_owned(),
            oauth_client_id: oauth_client_id.to_owned(),
            oauth_client_secret: oauth_client_secret.to_owned(),
        }
    }

    pub fn url(&self) -> &str {
        match &self {
            CrtCredentials::Basic { url, .. } => url,
            CrtCredentials::OAuth { url, .. } => url,
        }
    }
}
