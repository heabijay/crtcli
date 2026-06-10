use crate::app::client::{CrtClient, CrtClientError};
use crate::app::{CrtRequestBuilderExt, StandardServiceResponse};
use futures::{FutureExt, TryStreamExt};
use reqwest::Method;
use reqwest::header::HeaderMap;
use serde::Serialize;
use serde_json::json;
use std::borrow::Cow;
use tokio::io::AsyncReadExt;
use tokio_util::bytes::Bytes;
use tokio_util::io::StreamReader;

pub const UPLOAD_PACKAGE_CHUNK_SIZE_DEFAULT: usize = 10 * 1024 * 1024; // 10 MB (Kestrel: ~25MB; IIS: ~10MB) 
pub const UPLOAD_PACKAGE_CHUNK_SIZE_ENV_KEY: &str = "CRTCLI_APP_PKG_UPLOAD_CHUNK_SIZE";

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

        Ok(StreamReader::new(
            response.bytes_stream().map_err(std::io::Error::other),
        ))
    }

    pub async fn upload_package(
        &self,
        package_bytes: impl Into<Bytes>,
        package_filename: impl Into<Cow<'static, str>>,
    ) -> Result<(), CrtClientError> {
        let package_bytes = package_bytes.into();
        let mut file_header_map = HeaderMap::new();

        let content_type =
            match crate::pkg::utils::is_gzip_bytes(package_bytes.as_ref()).unwrap_or(false) {
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
                    reqwest::multipart::Part::stream(package_bytes)
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

    async fn upload_package_chunk(
        &self,
        package_filename: &str,
        package_bytes_chunk: Bytes,
        package_bytes_current: usize,
        package_bytes_total: usize,
    ) -> Result<(), CrtClientError> {
        let chunk_size = package_bytes_chunk.len();

        self.0
            .request(
                Method::POST,
                "0/ServiceModel/PackageInstallerService.svc/UploadPackage",
            )
            .query(&[("fileName", package_filename)])
            .body(package_bytes_chunk)
            .header(
                "Content-Range",
                format!(
                    "bytes {current}-{last}/{total}",
                    current = package_bytes_current,
                    last = package_bytes_current + chunk_size - 1,
                    total = package_bytes_total,
                ),
            )
            .send_with_session(self.0)
            .await?
            .error_for_status()?
            .json::<StandardServiceResponse>()
            .await?
            .into_result()?;

        Ok(())
    }

    pub fn start_upload_package_chunked(
        &self,
        package_bytes: impl Into<Bytes>,
        package_filename: impl Into<Cow<'static, str>>,
    ) -> UploadPackageChunkIter<'_> {
        let chunk_size = std::env::var(UPLOAD_PACKAGE_CHUNK_SIZE_ENV_KEY)
            .ok()
            .and_then(|x| x.parse::<usize>().ok())
            .unwrap_or(UPLOAD_PACKAGE_CHUNK_SIZE_DEFAULT);

        UploadPackageChunkIter {
            package_installer_service: self,
            chunk_size,
            current_offset: 0,
            package_bytes: package_bytes.into(),
            package_filename: package_filename.into(),
        }
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

pub struct UploadPackageChunkIter<'a> {
    package_installer_service: &'a PackageInstallerService<'a>,
    package_filename: Cow<'static, str>,
    package_bytes: Bytes,
    current_offset: usize,
    chunk_size: usize,
}

impl UploadPackageChunkIter<'_> {
    pub fn next(&mut self) -> Option<impl Future<Output = Result<(), CrtClientError>>> {
        let current = self.current_offset;
        let total = self.package_bytes.len();
        let chunk_size = self.chunk_size;

        if current >= total {
            return None;
        }

        if self.chunk_size == 0 {
            self.current_offset += total;

            Some(
                self.package_installer_service
                    .upload_package(self.package_bytes.slice(..), self.package_filename.clone())
                    .boxed(),
            )
        } else {
            let chunk = self
                .package_bytes
                .slice(current..std::cmp::min(current + chunk_size, total));

            self.current_offset += chunk.len();

            Some(
                self.package_installer_service
                    .upload_package_chunk(
                        &self.package_filename,
                        chunk,
                        current,
                        self.package_bytes.len(),
                    )
                    .boxed(),
            )
        }
    }
}
