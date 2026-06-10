use crate::app::CrtSessionOAuth;
use reqwest::Client;
use serde::Deserialize;
use thiserror::Error;

pub struct OAuthService<'c> {
    reqwest_client: &'c Client,
    oauth_base_url: &'c str,
}

impl<'c> OAuthService<'c> {
    pub fn new(reqwest_client: &'c Client, oauth_base_url: &'c str) -> Self {
        OAuthService {
            reqwest_client,
            oauth_base_url,
        }
    }

    pub async fn connect_token(
        &self,
        client_id: String,
        client_secret: String,
    ) -> Result<CrtSessionOAuth, OAuthLoginError> {
        let response = self
            .reqwest_client
            .post(format!("{}/connect/token", self.oauth_base_url))
            .form(&[
                ("client_id", client_id),
                ("client_secret", client_secret),
                ("grant_type", "client_credentials".to_owned()),
            ])
            .send()
            .await?;

        if response.status() == reqwest::StatusCode::BAD_REQUEST {
            let error = response
                .json::<OAuthErrorContract>()
                .await
                .map_err(OAuthLoginError::ResponseRead)?;

            return Err(OAuthLoginError::Remote(error.error));
        }

        response
            .error_for_status()?
            .json::<CrtSessionOAuth>()
            .await
            .map_err(OAuthLoginError::ResponseRead)
    }
}

#[derive(Error, Debug)]
pub enum OAuthLoginError {
    #[error("request error: {0}")]
    Request(#[from] reqwest::Error),

    #[error("response read error: {0}")]
    ResponseRead(#[source] reqwest::Error),

    #[error("{0}")]
    Remote(String),
}

#[derive(Debug, Deserialize)]
struct OAuthErrorContract {
    error: String,
}
