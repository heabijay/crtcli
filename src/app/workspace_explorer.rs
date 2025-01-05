use crate::app::client::{CrtClient, CrtClientGenericError};
use crate::app::{CrtRequestBuilderReauthorize, StandardServiceResponse};
use reqwest::Method;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fmt::{Debug, Display, Formatter};

pub struct WorkspaceExplorerService<'c>(&'c CrtClient);

impl<'c> WorkspaceExplorerService<'c> {
    pub fn new(client: &'c CrtClient) -> Self {
        Self(client)
    }

    pub fn build(&self) -> Result<BuildResponse, CrtClientGenericError> {
        let response = self
            .0
            .request(
                Method::POST,
                "0/ServiceModel/WorkspaceExplorerService.svc/Build",
            )
            .header(reqwest::header::CONTENT_LENGTH, "0")
            .send_with_session(self.0)?
            .error_for_status()?;

        Ok(response.json()?)
    }

    pub fn rebuild(&self) -> Result<BuildResponse, CrtClientGenericError> {
        let response = self
            .0
            .request(
                Method::POST,
                "0/ServiceModel/WorkspaceExplorerService.svc/Rebuild",
            )
            .header(reqwest::header::CONTENT_LENGTH, "0")
            .send_with_session(self.0)?
            .error_for_status()?;

        Ok(response.json()?)
    }

    pub fn build_package(
        &self,
        package_name: &str,
    ) -> Result<BuildResponse, CrtClientGenericError> {
        let response = self
            .0
            .request(
                Method::POST,
                "0/ServiceModel/WorkspaceExplorerService.svc/BuildPackage",
            )
            .json(&json!({
                "packageName": package_name
            }))
            .send_with_session(self.0)?
            .error_for_status()?;

        Ok(response.json()?)
    }

    pub fn get_packages(&self) -> Result<Vec<GetPackagesResponseItem>, CrtClientGenericError> {
        let response = self
            .0
            .request(
                Method::POST,
                "0/ServiceModel/WorkspaceExplorerService.svc/GetPackages",
            )
            .header(reqwest::header::CONTENT_LENGTH, "0")
            .send_with_session(self.0)?
            .error_for_status()?;

        let response: GetPackagesResponse = response.json()?;

        Ok(response.packages)
    }

    pub fn get_is_file_system_development_mode(&self) -> Result<bool, CrtClientGenericError> {
        let response = self
            .0
            .request(
                Method::POST,
                "0/ServiceModel/WorkspaceExplorerService.svc/GetIsFileDesignMode",
            )
            .header(reqwest::header::CONTENT_LENGTH, "0")
            .send_with_session(self.0)?
            .error_for_status()?;

        let response = response.json::<GetIsFileDesignModeResponse>()?;

        response.base.into_result()?;

        Ok(response.value)
    }
}

#[derive(Deserialize, Debug)]
pub struct BuildResponse {
    pub success: bool,

    // #[serde(rename = "buildResult")]
    // pub build_result: u32,
    pub message: Option<String>,

    #[serde(rename = "errorInfo")]
    pub error_info: Option<BuildPackageErrorInfo>,

    pub errors: Option<Vec<BuildPackageError>>,
}

impl BuildResponse {
    pub fn has_any_error(&self) -> bool {
        self.errors
            .as_ref()
            .is_some_and(|x| x.iter().any(|x| !x.warning))
    }
}

#[derive(Deserialize, Debug)]
pub struct BuildPackageError {
    pub line: u32,
    pub column: u32,
    pub warning: bool,

    #[serde(rename = "fileName")]
    pub filename: String,

    #[serde(rename = "errorNumber")]
    pub error_number: String,

    #[serde(rename = "errorText")]
    pub error_text: String,
}

#[derive(Deserialize, Debug)]
pub struct BuildPackageErrorInfo {
    pub message: String,
}

#[derive(Deserialize, Debug)]
struct GetPackagesResponse {
    packages: Vec<GetPackagesResponseItem>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetPackagesResponseItem {
    #[serde(rename = "uId")]
    uid: String,

    name: String,
}

impl Display for GetPackagesResponseItem {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} (UId: {})", self.name, self.uid)
    }
}

#[derive(Debug, Deserialize)]
struct GetIsFileDesignModeResponse {
    #[serde(flatten)]
    base: StandardServiceResponse,

    value: bool,
}
