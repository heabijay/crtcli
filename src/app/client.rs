use crate::app::app_installer::AppInstallerService;
use crate::app::auth::AuthService;
use crate::app::credentials::CrtCredentials;
use crate::app::package::PackageService;
use crate::app::package_installer::PackageInstallerService;
use crate::app::session::CrtSession;
use crate::app::session_cache::{
    create_default_session_cache, create_memory_session_cache, CrtSessionCache,
};
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

const MAX_REDIRECTS: usize = 10;

const DEFAULT_TIMEOUT_SECONDS: u64 = 1800;

#[derive(Debug, Default, Clone)]
pub struct CrtClientFlags {
    net_framework: bool,

    insecure: bool,
}

pub struct CrtClientBuilder {
    credentials: CrtCredentials,
    flags: CrtClientFlags,
    session_cache: Box<dyn CrtSessionCache>,
    inner_client_builder: reqwest::blocking::ClientBuilder,
}

impl CrtClientBuilder {
    pub fn new(credentials: CrtCredentials) -> Self {
        Self {
            credentials,
            flags: Default::default(),
            session_cache: create_default_session_cache(),
            inner_client_builder: reqwest::blocking::ClientBuilder::new()
                .user_agent(CRTCLI_CLIENT_USER_AGENT)
                .timeout(std::time::Duration::from_secs(DEFAULT_TIMEOUT_SECONDS))
                .redirect(reqwest::redirect::Policy::custom(|attempt| {
                    Self::custom_redirect_policy(attempt)
                })),
        }
    }

    fn custom_redirect_policy(attempt: reqwest::redirect::Attempt) -> reqwest::redirect::Action {
        let previous = attempt.previous();

        if previous.len() > MAX_REDIRECTS {
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
        self.flags.insecure = value;
        self.inner_client_builder = self.inner_client_builder.danger_accept_invalid_certs(value);
        self
    }

    #[allow(dead_code)]
    pub fn with_default_cache(mut self) -> Self {
        self.session_cache = create_default_session_cache();
        self
    }

    #[allow(dead_code)]
    pub fn with_new_memory_cache(mut self) -> Self {
        self.session_cache = create_memory_session_cache();
        self
    }

    pub fn build(self) -> Result<CrtClient, CrtClientError> {
        Ok(CrtClient {
            credentials: self.credentials,
            flags: self.flags,
            inner_client: self.inner_client_builder.build()?,
            session: RwLock::new(None),
            session_cache: self.session_cache,
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
    session_cache: Box<dyn CrtSessionCache>,
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

    pub fn credentials(&self) -> &CrtCredentials {
        &self.credentials
    }

    pub fn base_url(&self) -> &str {
        self.credentials.url()
    }

    pub fn is_net_framework(&self) -> bool {
        self.flags.net_framework
    }

    pub fn is_insecure(&self) -> bool {
        self.flags.insecure
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

    pub fn db_type(&self) -> Result<CrtDbType, CrtClientError> {
        let db_type = *self.db_type.read().unwrap();

        if let Some(db_type) = db_type {
            Ok(db_type)
        } else {
            let db_type = sql::detect_db_type(self)?;

            self.db_type.write().unwrap().replace(db_type);

            Ok(db_type)
        }
    }

    pub fn sql(&self, sql: &str) -> Result<sql::SqlRunnerResult, CrtClientError> {
        let result = self
            .sql_runner
            .read()
            .unwrap()
            .as_ref()
            .map(|sql_runner| sql_runner.sql(self, sql));

        if let Some(result) = result {
            Ok(result?)
        } else {
            let (sql_runner, result) = sql::AutodetectSqlRunner::detect_and_run_sql(self, sql)
                .ok_or(sql::SqlRunnerError::NotFound)?;

            self.sql_runner.write().unwrap().replace(sql_runner);

            Ok(result?)
        }
    }

    fn send_request_with_session(
        &self,
        builder: RequestBuilder,
    ) -> Result<Response, CrtClientError> {
        self.ensure_session()?;

        let builder = {
            let session = self.session.read().unwrap();

            builder
                .header("Cookie", session.as_ref().unwrap().to_cookie_value())
                .header("BPMCSRF", session.as_ref().unwrap().bpmcsrf())
        };

        let result = builder.send();

        match result {
            result if Self::is_unauthorized_response_result(&result) => {
                self.authenticate_and_store_session()?;

                Err(CrtClientError::Unauthorized)
            }
            Ok(response) => {
                self.update_session_from_response(&response);

                Ok(response)
            }
            Err(err) => Err(CrtClientError::ReqwestError(err)),
        }
    }

    fn ensure_session(&self) -> Result<(), CrtClientError> {
        if self.session.read().unwrap().is_none() {
            if let Some(session) = self.session_cache.get_entry(&self.credentials) {
                self.session.write().unwrap().replace(session);
            } else {
                self.authenticate_and_store_session()?;
            }
        }

        Ok(())
    }

    fn authenticate_and_store_session(&self) -> Result<(), CrtClientError> {
        let new_session = self
            .auth_service()
            .login(self.credentials.username(), self.credentials.password())?;

        self.session.write().unwrap().replace(new_session.clone());
        self.session_cache.set_entry(&self.credentials, new_session);

        Ok(())
    }

    fn update_session_from_response(&self, response: &Response) {
        let set_session_id = iter_set_cookies(response)
            .filter_map(|x| x.ok())
            .find(|&x| x.0 == "BPMSESSIONID");

        if let Some((_, set_session_id)) = set_session_id {
            let mut session_lock = self.session.write().unwrap();
            let session = session_lock.as_mut().unwrap();

            if session.bpmsessionid() != Some(set_session_id) {
                session.set_bpmsessionid(Some(set_session_id.to_owned()));
            }

            self.session_cache
                .set_entry(&self.credentials, session.clone());
        }
    }

    fn is_unauthorized_redirect_error(err: &reqwest::Error) -> bool {
        err.is_redirect()
            && err.source().is_some_and(|x| {
                x.downcast_ref::<CrtClientRedirectError>()
                    .is_some_and(|x| matches!(x, CrtClientRedirectError::Unauthorized))
            })
    }

    fn is_unauthorized_response_result(result: &Result<Response, reqwest::Error>) -> bool {
        result
            .as_ref()
            .is_ok_and(|r| r.status() == reqwest::StatusCode::UNAUTHORIZED)
            || result
                .as_ref()
                .is_err_and(Self::is_unauthorized_redirect_error)
    }
}

pub trait CrtRequestBuilderExt {
    fn send_with_session(self, client: &CrtClient) -> Result<Response, CrtClientError>;
}

impl CrtRequestBuilderExt for RequestBuilder {
    fn send_with_session(self, client: &CrtClient) -> Result<Response, CrtClientError> {
        client.send_request_with_session(self)
    }
}

#[derive(Debug, Error)]
pub enum CrtClientError {
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

impl From<auth::LoginError> for CrtClientError {
    fn from(value: auth::LoginError) -> Self {
        CrtClientError::LoginFailed(Box::new(value))
    }
}

impl From<sql::SqlRunnerError> for CrtClientError {
    fn from(value: sql::SqlRunnerError) -> Self {
        CrtClientError::SqlRunner(Box::new(value))
    }
}

impl From<reqwest::Error> for CrtClientError {
    fn from(value: reqwest::Error) -> Self {
        if value.is_request() {
            if let Some(source) = value.source() {
                if let Some(hyper_error) =
                    source.downcast_ref::<hyper_util::client::legacy::Error>()
                {
                    if hyper_error.is_connect() {
                        if let Some(inner_error) = hyper_error.source() {
                            return CrtClientError::ConnectionFailed {
                                inner_message: inner_error.to_string(),
                                source: value,
                            };
                        }
                    }
                }
            }
        }

        CrtClientError::ReqwestError(value)
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
