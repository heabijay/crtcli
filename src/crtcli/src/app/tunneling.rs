use crate::app::{CrtClient, CrtClientError, CrtRequestBuilderExt};
use indexmap::IndexMap;
use reqwest::Method;
use serde::Deserialize;
use std::ops::Deref;
use tokio::net::TcpStream;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

pub struct CrtCliTunnelingService<'c>(&'c CrtClient);

impl<'c> CrtCliTunnelingService<'c> {
    pub fn new(client: &'c CrtClient) -> Self {
        Self(client)
    }

    pub async fn get_status(&self) -> Result<TunnelingInformationResponse, CrtClientError> {
        let response = self
            .0
            .request(Method::GET, "0/rest/crtcli/tunneling/")
            .send_with_session(self.0)
            .await?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(CrtClientError::CrtCliTunnelingPackageNotInstalled);
        }

        Ok(response
            .error_for_status()?
            .json::<TunnelingInformationResponse>()
            .await?)
    }

    pub async fn get_connection_strings(&self) -> Result<IndexMap<String, String>, CrtClientError> {
        let response = self
            .0
            .request(Method::GET, "0/rest/crtcli/tunneling/connection-strings")
            .send_with_session(self.0)
            .await?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(CrtClientError::CrtCliTunnelingPackageNotInstalled);
        }

        Ok(response
            .error_for_status()?
            .json::<IndexMap<String, String>>()
            .await?)
    }

    pub async fn connect(
        &self,
        host: &str,
        port: u16,
    ) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>, CrtClientError> {
        let result = self
            .0
            .send_websocket_request_with_session(&format!(
                "0/rest/crtcli/tunneling/connect?host={host}&port={port}",
                host = urlencoding::encode(host),
                port = port
            ))
            .await;

        if let Err(CrtClientError::WebSocket(websocket_err)) = &result {
            if let tokio_tungstenite::tungstenite::Error::Http(response) = websocket_err.deref() {
                if response.status().as_u16() == 404 {
                    return Err(CrtClientError::CrtCliTunnelingPackageNotInstalled);
                }
            }
        }

        let (stream, _) = result?;

        Ok(stream)
    }
}

#[derive(Debug, Deserialize)]
pub struct TunnelingInformationResponse {
    pub allowed: bool,
    pub error: Option<TunnelingInformationResponseError>,
}

#[derive(Debug, Deserialize)]
pub struct TunnelingInformationResponseError {
    pub message: String,
}
