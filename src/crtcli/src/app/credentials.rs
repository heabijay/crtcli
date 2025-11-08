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
    pub fn new(
        url: impl AsRef<str>,
        username: impl AsRef<str>,
        password: impl AsRef<str>,
    ) -> CrtCredentials {
        CrtCredentials::Basic {
            url: url.as_ref().trim_end_matches("/").to_owned(),
            username: username.as_ref().to_owned(),
            password: password.as_ref().to_owned(),
        }
    }

    pub fn new_oauth(
        url: impl AsRef<str>,
        oauth_url: impl AsRef<str>,
        oauth_client_id: impl AsRef<str>,
        oauth_client_secret: impl AsRef<str>,
    ) -> CrtCredentials {
        CrtCredentials::OAuth {
            url: url.as_ref().trim_end_matches("/").to_owned(),
            oauth_url: oauth_url
                .as_ref()
                .trim_end_matches("/")
                .trim_end_matches("/connect/token")
                .to_owned(),
            oauth_client_id: oauth_client_id.as_ref().to_owned(),
            oauth_client_secret: oauth_client_secret.as_ref().to_owned(),
        }
    }

    pub fn url(&self) -> &str {
        match &self {
            CrtCredentials::Basic { url, .. } => url,
            CrtCredentials::OAuth { url, .. } => url,
        }
    }
}
