use crate::app::CrtRequestBuilderExt;
use crate::app::client::{CrtClient, CrtClientError};
use reqwest::Method;

pub struct ProcessSchemaManagerService<'c>(&'c CrtClient);

impl<'c> ProcessSchemaManagerService<'c> {
    pub fn new(client: &'c CrtClient) -> Self {
        Self(client)
    }

    pub async fn ping(&self) -> Result<bool, CrtClientError> {
        let response = self
            .0
            .request(
                Method::POST,
                "0/ServiceModel/ProcessSchemaManagerService.svc/PingApplication",
            )
            .header(reqwest::header::CONTENT_LENGTH, "0")
            .send_with_session(self.0)
            .await?
            .error_for_status()?;

        Ok(response.json().await?)
    }
}
