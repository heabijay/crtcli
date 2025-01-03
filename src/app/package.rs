use crate::app::{
    CrtClient, CrtClientGenericError, CrtRequestBuilderReauthorize, StandardServiceResponse,
};
use reqwest::Method;
use serde::{Deserialize, Serialize};
use serde_json::json;

pub struct PackageService<'c>(&'c CrtClient);

impl<'c> PackageService<'c> {
    pub fn new(client: &'c CrtClient) -> Self {
        Self(client)
    }

    pub fn get_package_properties(
        &self,
        package_uid: &str,
    ) -> Result<GetPackagePropertiesModel, CrtClientGenericError> {
        let response = self
            .0
            .request(
                Method::POST,
                "0/ServiceModel/PackageService.svc/GetPackageProperties",
            )
            .json(&json!(package_uid))
            .send_with_session(self.0)?
            .error_for_status()
            .map_err(CrtClientGenericError::from)?;

        let response: GetPackagePropertiesResponse =
            response.json().map_err(CrtClientGenericError::from)?;

        response.into_result()
    }
}

#[derive(Debug, Deserialize)]
struct GetPackagePropertiesResponse {
    package: Option<GetPackagePropertiesModel>,

    #[serde(flatten)]
    base: StandardServiceResponse,
}

impl GetPackagePropertiesResponse {
    pub fn into_result(self) -> Result<GetPackagePropertiesModel, CrtClientGenericError> {
        if self.base.success {
            Ok(self.package.expect(
                "get_package_properties response success, but package info is not received",
            ))
        } else {
            Err(CrtClientGenericError::from(self.base.error_info.expect(
                "get_package_properties response not success, but error is not received",
            )))
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GetPackagePropertiesModel {
    pub id: String,

    #[serde(rename = "uId")]
    pub uid: String,

    pub name: String,

    #[serde(rename = "type")]
    pub package_type: u32,

    pub maintainer: String,

    #[serde(rename = "createdOn")]
    pub created_on: String,

    #[serde(rename = "modifiedOn")]
    pub modified_on: String,
}
