use crate::app::client::{CrtClient, CrtClientError};
use crate::app::{CrtRequestBuilderExt, StandardServiceResponse};
use futures::TryStreamExt;
use reqwest::header::HeaderMap;
use reqwest::{Body, Method};
use serde::Serialize;
use serde_json::json;
use tokio::io::{AsyncReadExt, AsyncSeekExt};
use tokio_util::io::StreamReader;

pub struct PackageInstallerService<'c>(&'c CrtClient);

impl<'c> PackageInstallerService<'c> {
    pub fn new(client: &'c CrtClient) -> Self {
        Self(client)
    }

    pub async fn get_log_file(&self) -> Result<String, CrtClientError> {
        Ok(self
            .0
            .request(
                Method::GET,
                "0/ServiceModel/PackageInstallerService.svc/GetLogFile",
            )
            .send_with_session(self.0)
            .await?
            .error_for_status()?
            .text()
            .await?)
    }

    pub async fn get_zip_packages<StrArr, Str>(
        &self,
        package_names: StrArr,
    ) -> Result<impl AsyncReadExt + 'static, CrtClientError>
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
            .send_with_session(self.0)
            .await?
            .error_for_status()?;

        Ok(StreamReader::new(response.bytes_stream().map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::Other, e)
        })))
    }

    pub async fn upload_package<R>(
        &self,
        mut package_reader: R,
        package_filename: String,
    ) -> Result<(), CrtClientError>
    where
        R: AsyncReadExt + AsyncSeekExt + Send + 'static + Unpin,
    {
        let mut file_header_map = HeaderMap::new();

        let content_type = match crate::pkg::utils::is_gzip_async_stream(&mut package_reader)
            .await
            .unwrap_or(false)
        {
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
                reqwest::multipart::Form::new().part(
                    "files",
                    reqwest::multipart::Part::stream(Body::wrap_stream(
                        tokio_util::io::ReaderStream::new(package_reader),
                    ))
                    .file_name(package_filename)
                    .headers(file_header_map),
                ),
            )
            .send_with_session(self.0)
            .await?
            .error_for_status()?;

        Ok(response
            .json::<StandardServiceResponse>()
            .await?
            .into_result()?)
    }

    pub async fn install_package(&self, package_filename: &str) -> Result<(), CrtClientError> {
        let response = self
            .0
            .request(
                Method::POST,
                "0/ServiceModel/PackageInstallerService.svc/InstallPackage",
            )
            .json(&json!(package_filename))
            .send_with_session(self.0)
            .await?;

        Ok(response
            .json::<StandardServiceResponse>()
            .await?
            .into_result()?)
    }

    #[allow(dead_code)]
    pub async fn validate_package(
        &self,
        code: &str,
        package_filename: &str,
    ) -> Result<(), CrtClientError> {
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
            .send_with_session(self.0)
            .await?
            .error_for_status()?;

        Ok(response
            .json::<StandardServiceResponse>()
            .await?
            .into_result()?)
    }
}
