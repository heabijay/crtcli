use crate::app::app_installer::AppInstallerService;
use crate::app::auth::AuthService;
use crate::app::cookie_cache::{get_cookie_cache_entry, set_cookie_cache_entry};
use crate::app::credentials::CrtCredentials;
use crate::app::package::PackageService;
use crate::app::package_installer::PackageInstallerService;
use crate::app::session::CrtSession;
use crate::app::utils::iter_set_cookies;
use crate::app::workspace_explorer::WorkspaceExplorerService;
use crate::app::{auth, sql};
use reqwest::blocking::{RequestBuilder, Response};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::sync::RwLock;
use thiserror::Error;

const CRTCLI_CLIENT_USER_AGENT: &str =
    concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

#[derive(Debug, Default)]
pub struct CrtClientFlags {
    net_framework: bool,
}

pub struct CrtClientBuilder {
    credentials: CrtCredentials,
    flags: CrtClientFlags,
    session: Option<CrtSession>,
    inner_client_builder: reqwest::blocking::ClientBuilder,
}

impl CrtClientBuilder {
    pub fn new(credentials: CrtCredentials) -> Self {
        Self {
            credentials,
            flags: Default::default(),
            session: None,
            inner_client_builder: reqwest::blocking::ClientBuilder::new()
                .user_agent(CRTCLI_CLIENT_USER_AGENT)
                .timeout(std::time::Duration::from_secs(1800))
                .redirect(reqwest::redirect::Policy::custom(
                    CrtClientBuilder::custom_redirect_policy,
                )),
        }
    }

    fn custom_redirect_policy(attempt: reqwest::redirect::Attempt) -> reqwest::redirect::Action {
        let previous = attempt.previous();

        if previous.len() > 10 {
            let is_same_url_all_times = previous.iter().skip(1).all(|x| x == attempt.url());

            if is_same_url_all_times {
                attempt.error(CrtClientRedirectError::Unauthorized)
            } else {
                attempt.error("too many redirects")
            }
        } else if (attempt.url().path() == "/Login/Login.html"
            || attempt.url().path() == "/Login/NuiLogin.aspx")
            && attempt.url().query_pairs().any(|(k, _)| k == "ReturnUrl")
        {
            attempt.error(CrtClientRedirectError::Unauthorized)
        } else {
            attempt.follow()
        }
    }

    pub fn use_net_framework_mode(mut self, value: bool) -> Self {
        self.flags.net_framework = value;
        self
    }

    pub fn danger_accept_invalid_certs(mut self, value: bool) -> Self {
        self.inner_client_builder = self.inner_client_builder.danger_accept_invalid_certs(value);
        self
    }

    pub fn build(self) -> Result<CrtClient, CrtClientGenericError> {
        Ok(CrtClient {
            credentials: self.credentials,
            flags: self.flags,
            inner_client: self.inner_client_builder.build()?,
            session: RwLock::new(self.session),
            sql_runner: RwLock::new(None),
            db_type: RwLock::new(None),
        })
    }
}

pub struct CrtClient {
    credentials: CrtCredentials,
    flags: CrtClientFlags,
    inner_client: reqwest::blocking::Client,
    session: RwLock<Option<CrtSession>>,
    sql_runner: RwLock<Option<Box<dyn sql::SqlRunner>>>,
    db_type: RwLock<Option<CrtDbType>>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Error)]
enum CrtClientRedirectError {
    #[error("unauthorized")]
    Unauthorized,
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum CrtDbType {
    MsSql,
    Oracle,
    Postgres,
}

impl CrtClient {
    pub fn builder(credentials: CrtCredentials) -> CrtClientBuilder {
        CrtClientBuilder::new(credentials)
    }

    pub fn base_url(&self) -> &str {
        self.credentials.url()
    }

    pub fn is_net_framework(&self) -> bool {
        self.flags.net_framework
    }

    pub fn request(&self, method: reqwest::Method, relative_url: &str) -> RequestBuilder {
        self.inner_client
            .request(method, format!("{}/{}", self.base_url(), relative_url))
    }

    pub fn auth_service(&self) -> AuthService<'_> {
        AuthService::new(self)
    }

    pub fn app_installer_service(&self) -> AppInstallerService<'_> {
        AppInstallerService::new(self)
    }

    pub fn workspace_explorer_service(&self) -> WorkspaceExplorerService<'_> {
        WorkspaceExplorerService::new(self)
    }

    pub fn package_service(&self) -> PackageService<'_> {
        PackageService::new(self)
    }

    pub fn package_installer_service(&self) -> PackageInstallerService<'_> {
        PackageInstallerService::new(self)
    }

    pub fn sql_scripts(&self) -> sql::SqlScripts<'_> {
        sql::SqlScripts::new(self)
    }

    pub fn db_type(&self) -> Result<CrtDbType, CrtClientGenericError> {
        if self.db_type.read().unwrap().is_none() {
            let db_type = sql::detect_db_type(self)?;

            self.db_type.write().unwrap().replace(db_type);

            return Ok(db_type);
        }

        Ok(*self.db_type.read().unwrap().as_ref().unwrap())
    }

    pub fn sql(&self, sql: &str) -> Result<sql::SqlRunnerResult, CrtClientGenericError> {
        if self.sql_runner.read().unwrap().is_none() {
            let (executor, result) = sql::AutodetectSqlRunner::detect_and_run_sql(self, sql)
                .ok_or(sql::SqlRunnerError::NotFound)?;

            self.sql_runner.write().unwrap().replace(executor);

            return Ok(result?);
        }

        Ok(self
            .sql_runner
            .read()
            .unwrap()
            .as_ref()
            .unwrap()
            .sql(self, sql)?)
    }
}

pub trait CrtRequestBuilderReauthorize {
    fn send_with_session(self, client: &CrtClient) -> Result<Response, CrtClientGenericError>;
}

impl CrtRequestBuilderReauthorize for RequestBuilder {
    fn send_with_session(self, client: &CrtClient) -> Result<Response, CrtClientGenericError> {
        if client.session.read().unwrap().is_none() {
            setup_session_with_cache(client)?
        }

        let response = self
            .header(
                "Cookie",
                client
                    .session
                    .read()
                    .unwrap()
                    .as_ref()
                    .unwrap()
                    .to_cookie_value(),
            )
            .header(
                "BPMCSRF",
                client.session.read().unwrap().as_ref().unwrap().bpmcsrf(),
            )
            .send();

        return match response {
            Ok(response) => {
                try_enrich_bpmsessionid_from_response(client, &response);

                Ok(response)
            }
            Err(err)
                if err.status() == Some(reqwest::StatusCode::UNAUTHORIZED)
                    || err.is_redirect()
                        && err.source().is_some_and(|x| {
                            x.downcast_ref::<CrtClientRedirectError>()
                                .is_some_and(|x| matches!(x, CrtClientRedirectError::Unauthorized))
                        }) =>
            {
                login_and_store(client)?;

                Err(CrtClientGenericError::Unauthorized)
            }
            Err(err) => return Err(CrtClientGenericError::ReqwestError(err)),
        };

        fn setup_session_with_cache(client: &CrtClient) -> Result<(), CrtClientGenericError> {
            match get_cookie_cache_entry(&client.credentials) {
                Some(new_session) => {
                    client.session.write().unwrap().replace(new_session);

                    Ok(())
                }
                None => login_and_store(client),
            }
        }

        fn try_enrich_bpmsessionid_from_response(client: &CrtClient, response: &Response) {
            let bmpsessionid = iter_set_cookies(response)
                .find(|x| x.as_ref().is_ok_and(|x| x.0 == "BPMSESSIONID"));

            if let Some(bmpsessionid) = bmpsessionid {
                let bmpsessionid = bmpsessionid.unwrap();
                let mut session = client.session.read().unwrap().as_ref().unwrap().to_owned();

                if session.bpmsessionid() != Some(bmpsessionid.1) {
                    session.set_bpmsessionid(Some(bmpsessionid.1.to_owned()));

                    set_and_save_to_cache_session(client, session)
                }
            }
        }

        fn login_and_store(client: &CrtClient) -> Result<(), CrtClientGenericError> {
            let new_session = client
                .auth_service()
                .login(client.credentials.username(), client.credentials.password())
                .map_err(|err| CrtClientGenericError::LoginFailed(Box::new(err)))?;

            set_and_save_to_cache_session(client, new_session);

            Ok(())
        }

        fn set_and_save_to_cache_session(client: &CrtClient, new_session: CrtSession) {
            set_cookie_cache_entry(&client.credentials, new_session.clone());

            client.session.write().unwrap().replace(new_session);
        }

        // fn verify_session_valid(client: &CrtClient, session: &CrtSession) -> bool {
        //     let response = client
        //         .request(reqwest::Method::HEAD, "0/DataService/json/SyncReply/PostClientLog")
        //         .header("Cookie", session.to_cookie_value())
        //         .header("BPMCSRF", session.bpmcsrf())
        //         .body(r#"{"LogItems":[]}"#)
        //         .send();
        //
        //     match response {
        //         Ok(r) if { r.status() == reqwest::StatusCode::METHOD_NOT_ALLOWED } => true,
        //         _ => false,
        //     }
        // }
    }
}

#[derive(Debug, Error)]
pub enum CrtClientGenericError {
    #[error("request error: {0}")]
    ReqwestError(#[source] reqwest::Error),

    #[error("login failed: {0}")]
    LoginFailed(#[from] Box<auth::LoginError>),

    #[error("request connection error: {inner_message}")]
    ConnectionFailed {
        #[source]
        source: reqwest::Error,

        inner_message: String,
    },

    #[error("unauthorized, please try send request again")]
    Unauthorized,

    #[error("service returned error: {0}")]
    ServiceReturnedErrorInfo(#[from] StandardServiceError),

    #[error("sql runner error: {0}")]
    SqlRunner(#[from] Box<sql::SqlRunnerError>),
    // #[error("failed to access the cache: {0}")]
    // AccessCache(#[from] AccessCacheError)
}

impl From<auth::LoginError> for CrtClientGenericError {
    fn from(value: auth::LoginError) -> Self {
        CrtClientGenericError::LoginFailed(Box::new(value))
    }
}

impl From<sql::SqlRunnerError> for CrtClientGenericError {
    fn from(value: sql::SqlRunnerError) -> Self {
        CrtClientGenericError::SqlRunner(Box::new(value))
    }
}

impl From<reqwest::Error> for CrtClientGenericError {
    fn from(value: reqwest::Error) -> Self {
        if value.is_request() {
            if let Some(source) = value.source() {
                if let Some(hyper_error) =
                    source.downcast_ref::<hyper_util::client::legacy::Error>()
                {
                    if hyper_error.is_connect() {
                        if let Some(inner_error) = hyper_error.source() {
                            return CrtClientGenericError::ConnectionFailed {
                                inner_message: inner_error.to_string(),
                                source: value,
                            };
                        }
                    }
                }
            }
        }

        CrtClientGenericError::ReqwestError(value)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct StandardServiceResponse {
    pub success: bool,

    #[serde(rename = "errorInfo")]
    pub error_info: Option<StandardServiceError>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct StandardServiceError {
    pub message: String,

    #[serde(rename = "errorCode")]
    pub error_code: String,

    #[serde(rename = "stackTrace")]
    pub stack_trace: Option<String>,
}

impl Display for StandardServiceError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.error_code, self.message)
    }
}

impl std::error::Error for StandardServiceError {}

impl StandardServiceResponse {
    pub fn into_result(self) -> Result<(), StandardServiceError> {
        if !self.success {
            if let Some(err) = self.error_info {
                return Err(err);
            }
        }

        Ok(())
    }
}
