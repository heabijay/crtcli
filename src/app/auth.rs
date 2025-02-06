use crate::app::client::CrtClient;
use crate::app::session::CrtSession;
use crate::app::utils::{collect_set_cookies, find_cookie_by_name, CookieParsingError};
use crate::app::CrtClientError;
use reqwest::Method;
use serde::Deserialize;
use serde_json::json;
use thiserror::Error;

pub struct AuthService<'c>(&'c CrtClient);

impl<'c> AuthService<'c> {
    pub fn new(client: &'c CrtClient) -> Self {
        Self(client)
    }

    pub async fn login(&self, username: &str, password: &str) -> Result<CrtSession, LoginError> {
        let response = self
            .0
            .request(Method::POST, "ServiceModel/AuthService.svc/Login")
            .header("ForceUseSession", "true")
            .json(&json!({
                "UserName": username,
                "UserPassword": password
            }))
            .send()
            .await?
            .error_for_status()?;

        let set_cookies = collect_set_cookies(&response)?;

        response
            .json::<LoginResponse>()
            .await
            .map_err(LoginError::ResponseRead)?
            .into_result()?;

        let aspxauth = find_cookie_by_name(&set_cookies, ".ASPXAUTH")
            .ok_or_else(|| LoginError::CookieNotFound(".ASPXAUTH"))?;

        let bpmcrsf = find_cookie_by_name(&set_cookies, "BPMCSRF")
            .ok_or_else(|| LoginError::CookieNotFound("BPMCSRF"))?;

        let csrftoken = find_cookie_by_name(&set_cookies, "CsrfToken");

        Ok(CrtSession::new(aspxauth, bpmcrsf, csrftoken, None))
    }
}

#[derive(Error, Debug)]
pub enum LoginError {
    #[error("{0}")]
    Request(#[from] CrtClientError),

    #[error("response read error: {0}")]
    ResponseRead(#[source] reqwest::Error),

    #[error("login response error: {0}")]
    ResponseError(#[from] LoginResponse),

    #[error("{0}")]
    CookieParsingFailed(#[from] CookieParsingError),

    #[error("expected cookie {0} in response not found!")]
    CookieNotFound(&'static str),
}

impl From<reqwest::Error> for LoginError {
    fn from(value: reqwest::Error) -> Self {
        LoginError::Request(CrtClientError::from(value))
    }
}

#[derive(Debug, Deserialize, Error)]
#[error("{message} (code: {code})")]
pub struct LoginResponse {
    #[serde(rename = "Code")]
    code: i32,

    #[serde(rename = "Message")]
    message: String,
}

impl LoginResponse {
    pub fn into_result(self) -> Result<(), LoginResponse> {
        if self.code == 0 {
            Ok(())
        } else {
            Err(self)
        }
    }
}
