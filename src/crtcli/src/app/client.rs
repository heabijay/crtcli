use crate::app::app_installer::AppInstallerService;
use crate::app::auth::AuthService;
use crate::app::credentials::CrtCredentials;
use crate::app::package::PackageService;
use crate::app::package_installer::PackageInstallerService;
use crate::app::session::CrtSession;
use crate::app::session_cache::{
    CrtSessionCache, create_default_session_cache, create_memory_session_cache,
};
use crate::app::utils::{iter_set_cookies, iter_set_cookies_in_websocket_response};
use crate::app::workspace_explorer::WorkspaceExplorerService;
use crate::app::{auth, oauth, sql, tunneling};
use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::pki_types::*;
use rustls::{ClientConfig, DigitallySignedStruct, SignatureScheme};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use std::sync::{Arc, RwLock};
use thiserror::Error;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::ClientRequestBuilder;
use tokio_tungstenite::tungstenite::handshake::client::Response;
use tokio_tungstenite::tungstenite::http::Uri;
use tokio_tungstenite::{Connector, MaybeTlsStream, WebSocketStream};

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
    session: Option<CrtSession>,
    session_cache: Box<dyn CrtSessionCache>,
}

impl CrtClientBuilder {
    pub fn new(credentials: CrtCredentials) -> Self {
        Self {
            credentials,
            flags: Default::default(),
            session: None,
            session_cache: create_default_session_cache(),
        }
    }

    pub fn use_net_framework_mode(mut self, value: bool) -> Self {
        self.flags.net_framework = value;
        self
    }

    pub fn danger_accept_invalid_certs(mut self, value: bool) -> Self {
        self.flags.insecure = value;
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

    pub fn with_session(mut self, session: Option<CrtSession>) -> Self {
        self.session = session;
        self
    }

    pub fn build(self) -> Result<CrtClient, CrtClientError> {
        let client_builder = reqwest::ClientBuilder::new()
            .user_agent(CRTCLI_CLIENT_USER_AGENT)
            .timeout(std::time::Duration::from_secs(DEFAULT_TIMEOUT_SECONDS))
            .redirect(reqwest::redirect::Policy::custom(|attempt| {
                Self::custom_redirect_policy(attempt)
            }))
            .danger_accept_invalid_certs(self.flags.insecure);

        let ws_connector = if self.flags.insecure {
            let mut client_config = ClientConfig::builder()
                .with_root_certificates(rustls::RootCertStore::empty())
                .with_no_client_auth();

            client_config
                .dangerous()
                .set_certificate_verifier(Arc::new(NoCertVerifier));

            Some(Connector::Rustls(Arc::new(client_config)))
        } else {
            None
        };

        Ok(CrtClient {
            inner_client: client_builder.build()?,
            inner_ws_connector: ws_connector,
            credentials: self.credentials,
            flags: self.flags,
            session: RwLock::new(self.session),
            session_cache: self.session_cache,
            sql_runner: RwLock::new(None),
            db_type: RwLock::new(None),
        })
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
}

pub struct CrtClient {
    credentials: CrtCredentials,
    flags: CrtClientFlags,
    inner_client: reqwest::Client,
    session: RwLock<Option<CrtSession>>,
    session_cache: Box<dyn CrtSessionCache>,
    sql_runner: RwLock<Option<Arc<Box<dyn sql::SqlRunner>>>>,
    db_type: RwLock<Option<CrtDbType>>,
    inner_ws_connector: Option<Connector>,
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

    pub fn base_websocket_url(&self) -> String {
        let base_url_lower = self.base_url().to_lowercase();

        if base_url_lower.starts_with("http://") {
            format!("ws://{}", &self.base_url()[7..])
        } else if base_url_lower.starts_with("https://") {
            format!("wss://{}", &self.base_url()[8..])
        } else {
            self.base_url().to_owned()
        }
    }

    pub fn is_net_framework(&self) -> bool {
        self.flags.net_framework
    }

    pub fn is_insecure(&self) -> bool {
        self.flags.insecure
    }

    pub fn request(&self, method: reqwest::Method, relative_url: &str) -> reqwest::RequestBuilder {
        self.inner_client
            .request(method, format!("{}/{}", self.base_url(), relative_url))
    }

    pub fn auth_service(&self) -> AuthService<'_> {
        AuthService::new(self)
    }

    pub fn oauth_service(&self) -> oauth::OAuthService<'_> {
        if let CrtCredentials::OAuth { oauth_url, .. } = &self.credentials {
            oauth::OAuthService::new(&self.inner_client, oauth_url)
        } else {
            panic!("cannot use oauth service with non-oauth credentials");
        }
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

    pub fn crtcli_tunneling_service(&self) -> tunneling::CrtCliTunnelingService<'_> {
        tunneling::CrtCliTunnelingService::new(self)
    }

    pub async fn db_type(&self) -> Result<CrtDbType, CrtClientError> {
        if let Some(db_type) = *self.db_type.read().unwrap() {
            Ok(db_type)
        } else {
            let db_type = sql::detect_db_type(self).await?;

            self.db_type.write().unwrap().replace(db_type);

            Ok(db_type)
        }
    }

    pub async fn sql(&self, sql: &str) -> Result<sql::SqlRunnerResult, CrtClientError> {
        let result = self.sql_runner.read().unwrap().clone();

        if let Some(result) = result {
            Ok(result.sql(self, sql).await?)
        } else {
            let (sql_runner, result) = sql::AutodetectSqlRunner::detect_and_run_sql(self, sql)
                .await
                .ok_or(sql::SqlRunnerError::NotFound)?;

            self.sql_runner
                .write()
                .unwrap()
                .replace(Arc::new(sql_runner));

            Ok(result?)
        }
    }

    async fn send_request_with_session(
        &self,
        builder: reqwest::RequestBuilder,
    ) -> Result<reqwest::Response, CrtClientError> {
        self.ensure_session().await?;

        let builder = apply_session_to_request(self, builder);

        let result = builder.send().await;

        return match result {
            result if Self::is_unauthorized_response_result(&result) => {
                self.authenticate_and_store_session().await?;

                Err(CrtClientError::Unauthorized)
            }
            Ok(response) => {
                update_session_from_response(self, &response);

                Ok(response)
            }
            Err(err) => Err(CrtClientError::Reqwest(err)),
        };

        fn apply_session_to_request(
            _self: &CrtClient,
            builder: reqwest::RequestBuilder,
        ) -> reqwest::RequestBuilder {
            let session_lock = _self.session.read().unwrap();
            let session = session_lock.as_ref().unwrap();

            match session {
                CrtSession::Cookie(cookie) => builder
                    .header("Cookie", cookie.to_cookie_value())
                    .header("BPMCSRF", cookie.bpmcsrf()),
                CrtSession::OAuthSession(oauth_session) => builder.header(
                    reqwest::header::AUTHORIZATION,
                    format!(
                        "{} {}",
                        oauth_session.token_type(),
                        oauth_session.access_token()
                    ),
                ),
            }
        }

        fn update_session_from_response(_self: &CrtClient, response: &reqwest::Response) {
            let set_session_id = iter_set_cookies(response)
                .filter_map(|x| x.ok())
                .find(|&x| x.0 == "BPMSESSIONID");

            if let Some((_, set_session_id)) = set_session_id {
                _self.update_session_bpmsessionid(set_session_id);
            }
        }
    }

    pub async fn send_websocket_request_with_session(
        &self,
        relative_url: &str,
    ) -> Result<(WebSocketStream<MaybeTlsStream<TcpStream>>, Response), CrtClientError> {
        self.ensure_session().await?;

        for attempt in 0..2 {
            let uri = Uri::from_str(&format!("{}/{relative_url}", self.base_websocket_url()))
                .map_err(|_| CrtClientError::InvalidBaseUrl(self.base_websocket_url()))?;

            let builder = apply_session_to_websocket_request(self, ClientRequestBuilder::new(uri));

            let (stream, response) = {
                let result = tokio_tungstenite::connect_async_tls_with_config(
                    builder,
                    None,
                    false,
                    self.inner_ws_connector.clone(),
                )
                .await;

                if Self::is_unauthorized_websocket_response(&result) {
                    self.authenticate_and_store_session().await?;

                    if attempt < 1 {
                        continue;
                    }

                    return Err(CrtClientError::Unauthorized);
                }

                result?
            };

            update_session_from_websocket_response(self, &response);

            return Ok((stream, response));
        }

        panic!("too many login attempts");

        fn apply_session_to_websocket_request(
            _self: &CrtClient,
            builder: ClientRequestBuilder,
        ) -> ClientRequestBuilder {
            let session_lock = _self.session.read().unwrap();
            let session = session_lock.as_ref().unwrap();

            match session {
                CrtSession::Cookie(cookie) => builder
                    .with_header("Cookie", cookie.to_cookie_value())
                    .with_header("BPMCSRF", cookie.bpmcsrf()),
                CrtSession::OAuthSession(oauth_session) => builder.with_header(
                    reqwest::header::AUTHORIZATION.as_str(),
                    format!(
                        "{} {}",
                        oauth_session.token_type(),
                        oauth_session.access_token()
                    ),
                ),
            }
        }

        fn update_session_from_websocket_response(_self: &CrtClient, response: &Response) {
            let set_session_id = iter_set_cookies_in_websocket_response(response)
                .filter_map(|x| x.ok())
                .find(|&x| x.0 == "BPMSESSIONID");

            if let Some((_, set_session_id)) = set_session_id {
                _self.update_session_bpmsessionid(set_session_id);
            }
        }
    }

    async fn ensure_session(&self) -> Result<(), CrtClientError> {
        if self.session.read().unwrap().is_none() {
            if let Some(session) = self.session_cache.get_entry(&self.credentials) {
                self.session.write().unwrap().replace(session);
            } else {
                self.authenticate_and_store_session().await?;
            }
        }

        Ok(())
    }

    async fn authenticate_and_store_session(&self) -> Result<(), CrtClientError> {
        let session = match &self.credentials {
            CrtCredentials::Basic {
                url: _,
                username,
                password,
            } => CrtSession::Cookie(self.auth_service().login(username, password).await?),
            CrtCredentials::OAuth {
                oauth_client_id,
                oauth_client_secret,
                ..
            } => CrtSession::OAuthSession(
                self.oauth_service()
                    .connect_token(oauth_client_id.to_owned(), oauth_client_secret.to_owned())
                    .await?,
            ),
        };

        self.session.write().unwrap().replace(session.clone());
        self.session_cache.set_entry(&self.credentials, session);

        Ok(())
    }

    fn update_session_bpmsessionid(&self, bpmsessionid: &str) {
        if has_same_bpmsessionid(self, bpmsessionid) {
            return;
        }

        let mut session_lock = self.session.write().unwrap();
        let session = session_lock.as_mut().unwrap();

        if let CrtSession::Cookie(cookie_session) = session {
            cookie_session.set_bpmsessionid(Some(bpmsessionid.to_owned()));

            self.session_cache
                .set_entry(&self.credentials, session.clone());
        }

        fn has_same_bpmsessionid(_self: &CrtClient, bpmsessionid: &str) -> bool {
            let session_lock = _self.session.read().unwrap();

            if let Some(CrtSession::Cookie(cookie_session)) = session_lock.as_ref() {
                return cookie_session.bpmsessionid() == Some(bpmsessionid);
            }

            false
        }
    }

    fn is_unauthorized_redirect_error(err: &reqwest::Error) -> bool {
        err.is_redirect()
            && err.source().is_some_and(|x| {
                x.downcast_ref::<CrtClientRedirectError>()
                    .is_some_and(|x| matches!(x, CrtClientRedirectError::Unauthorized))
            })
    }

    fn is_unauthorized_response_result(result: &Result<reqwest::Response, reqwest::Error>) -> bool {
        result
            .as_ref()
            .is_ok_and(|r| r.status() == reqwest::StatusCode::UNAUTHORIZED)
            || result
                .as_ref()
                .is_err_and(Self::is_unauthorized_redirect_error)
    }

    fn is_unauthorized_websocket_response(
        result: &Result<
            (WebSocketStream<MaybeTlsStream<TcpStream>>, Response),
            tokio_tungstenite::tungstenite::Error,
        >,
    ) -> bool {
        match result {
            Ok((_, response)) => {
                response.status().as_u16() == 401 || response.status().is_redirection()
            }
            Err(err) => match err {
                tokio_tungstenite::tungstenite::Error::Http(response) => {
                    response.status().as_u16() == 401 || response.status().is_redirection()
                }
                _ => false,
            },
        }
    }
}

pub trait CrtRequestBuilderExt {
    async fn send_with_session(
        self,
        client: &CrtClient,
    ) -> Result<reqwest::Response, CrtClientError>;
}

impl CrtRequestBuilderExt for reqwest::RequestBuilder {
    async fn send_with_session(
        self,
        client: &CrtClient,
    ) -> Result<reqwest::Response, CrtClientError> {
        client.send_request_with_session(self).await
    }
}

#[derive(Debug, Error)]
pub enum CrtClientError {
    #[error("request error: {0}")]
    Reqwest(#[source] reqwest::Error),

    #[error("websocket error: {0}")]
    WebSocket(#[from] tokio_tungstenite::tungstenite::Error),

    #[error("login failed: {0}")]
    Login(#[from] Box<auth::LoginError>),

    #[error("oauth login failed: {0}")]
    OAuthLogin(#[from] Box<oauth::OAuthLoginError>),

    #[error("request connection error: {inner_message}")]
    Connection {
        #[source]
        source: reqwest::Error,

        inner_message: String,
    },

    #[error("invalid base url: {0}")]
    InvalidBaseUrl(String),

    #[error("unauthorized, please try send request again")]
    Unauthorized,

    #[error("service returned error: {0}")]
    ServiceReturnedErrorInfo(#[from] StandardServiceError),

    #[error("sql runner error: {0}")]
    SqlRunner(#[from] Box<sql::SqlRunnerError>),

    #[error("crtcli tunneling package not installed, please check docs for more information")]
    CrtCliTunnelingPackageNotInstalled,
}

impl From<auth::LoginError> for CrtClientError {
    fn from(value: auth::LoginError) -> Self {
        CrtClientError::Login(Box::new(value))
    }
}

impl From<oauth::OAuthLoginError> for CrtClientError {
    fn from(value: oauth::OAuthLoginError) -> Self {
        CrtClientError::OAuthLogin(Box::new(value))
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
                            return CrtClientError::Connection {
                                inner_message: inner_error.to_string(),
                                source: value,
                            };
                        }
                    }
                }
            }
        }

        CrtClientError::Reqwest(value)
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

#[derive(Debug)]
struct NoCertVerifier;

impl ServerCertVerifier for NoCertVerifier {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: UnixTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        rustls::crypto::ring::default_provider()
            .signature_verification_algorithms
            .supported_schemes()
    }
}
