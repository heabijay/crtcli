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

    pub fn login(&self, username: &str, password: &str) -> Result<CrtSession, LoginError> {
        let mut response = self
            .0
            .request(Method::POST, "ServiceModel/AuthService.svc/Login")
            .header("ForceUseSession", "true")
            .json(&json!({
                "UserName": username,
                "UserPassword": password
            }))
            .send()?
            .error_for_status()?;

        read_login_response(&mut response)?.into_result()?;

        let set_cookies = collect_set_cookies(&response)?;

        let aspxauth = find_cookie_by_name(&set_cookies, ".ASPXAUTH")
            .ok_or_else(|| LoginError::CookieNotFound(".ASPXAUTH"))?;

        let bpmcrsf = find_cookie_by_name(&set_cookies, "BPMCSRF")
            .ok_or_else(|| LoginError::CookieNotFound("BPMCSRF"))?;

        let csrftoken = find_cookie_by_name(&set_cookies, "CsrfToken");

        return Ok(CrtSession::new(aspxauth, bpmcrsf, csrftoken, None));

        fn read_login_response(
            response: &mut impl std::io::Read,
        ) -> Result<LoginResponse, LoginError> {
            let mut body = vec![];

            response.read_to_end(&mut body)?;

            serde_json::from_slice(&body)
                .map_err(|err| LoginError::ResponseParse { body, source: err })
        }
    }
}

#[derive(Error, Debug)]
pub enum LoginError {
    #[error("{0}")]
    Request(#[from] CrtClientError),

    #[error("response read error: {0}")]
    ResponseRead(#[from] std::io::Error),

    #[error("response parse error: {source}")]
    ResponseParse {
        body: Vec<u8>,

        #[source]
        source: serde_json::Error,
    },

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
