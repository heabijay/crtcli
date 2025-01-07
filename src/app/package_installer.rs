use crate::app::client::{CrtClient, CrtClientGenericError};
use crate::app::{CrtRequestBuilderReauthorize, StandardServiceResponse};
use reqwest::header::HeaderMap;
use reqwest::Method;
use serde::Serialize;
use serde_json::json;
use std::io::{Read, Seek};

pub struct PackageInstallerService<'c>(&'c CrtClient);

impl<'c> PackageInstallerService<'c> {
    pub fn new(client: &'c CrtClient) -> Self {
        Self(client)
    }

    pub fn get_log_file(&self) -> Result<String, CrtClientGenericError> {
        Ok(self
            .0
            .request(
                Method::GET,
                "0/ServiceModel/PackageInstallerService.svc/GetLogFile",
            )
            .send_with_session(self.0)?
            .error_for_status()?
            .text()?)
    }

    pub fn get_zip_packages<StrArr, Str>(
        &self,
        package_names: StrArr,
    ) -> Result<impl Read, CrtClientGenericError>
    where
        StrArr: AsRef<[Str]> + Serialize,
        Str: AsRef<str>,
    {
        let response = self
            .0
            .request(
                Method::POST,
                "0/ServiceModel/PackageInstallerService.svc/GetZipPackages",
            )
            .json(&json!(package_names))
            .send_with_session(self.0)?
            .error_for_status()?;

        Ok(response)
    }

    pub fn upload_package(
        &self,
        mut package_reader: impl Read + Send + Seek + 'static,
        package_filename: String,
    ) -> Result<(), CrtClientGenericError> {
        let mut file_header_map = HeaderMap::new();

        let content_type =
            match crate::pkg::utils::is_gzip_stream(&mut package_reader).unwrap_or(false) {
                true => "application/x-gzip".parse().unwrap(),
                false => "application/x-zip-compressed".parse().unwrap(),
            };

        file_header_map.insert("Content-Type", content_type);

        let response = self
            .0
            .request(
                Method::POST,
                "0/ServiceModel/PackageInstallerService.svc/UploadPackage",
            )
            .multipart(
                reqwest::blocking::multipart::Form::new().part(
                    "files",
                    reqwest::blocking::multipart::Part::reader(package_reader)
                        .file_name(package_filename)
                        .headers(file_header_map),
                ),
            )
            .send_with_session(self.0)?
            .error_for_status()?;

        response.json::<StandardServiceResponse>()?.into_result()?;

        Ok(())
    }

    pub fn install_package(&self, package_filename: &str) -> Result<(), CrtClientGenericError> {
        let response = self
            .0
            .request(
                Method::POST,
                "0/ServiceModel/PackageInstallerService.svc/InstallPackage",
            )
            .json(&json!(package_filename))
            .send_with_session(self.0)?;

        response.json::<StandardServiceResponse>()?.into_result()?;

        Ok(())
    }

    #[allow(dead_code)]
    pub fn validate_package(
        &self,
        code: &str,
        package_filename: &str,
    ) -> Result<(), CrtClientGenericError> {
        let response = self
            .0
            .request(
                Method::POST,
                "0/ServiceModel/PackageInstallerService.svc/Validate",
            )
            .json(&json!({
                "Code": code,
                "ZipPackageName": package_filename
            }))
            .send_with_session(self.0)?
            .error_for_status()?;

        response.json::<StandardServiceResponse>()?.into_result()?;

        Ok(())
    }
}
