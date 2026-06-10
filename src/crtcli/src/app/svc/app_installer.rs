use crate::app::client::{CrtClient, CrtClientError};
use crate::app::{CrtRequestBuilderExt, StandardServiceError, StandardServiceResponse};
use reqwest::Method;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fmt::{Debug, Display};

pub struct AppInstallerService<'c>(&'c CrtClient);

impl<'c> AppInstallerService<'c> {
    pub fn new(client: &'c CrtClient) -> Self {
        Self(client)
    }

    pub async fn restart_app(&self) -> Result<(), CrtClientError> {
        let response = self
            .0
            .request(
                Method::POST,
                match self.0.is_net_framework() {
                    true => "0/ServiceModel/AppInstallerService.svc/UnloadAppDomain",
                    false => "0/ServiceModel/AppInstallerService.svc/RestartApp",
                },
            )
            .header(reqwest::header::CONTENT_LENGTH, "0")
            .send_with_session(self.0)
            .await?
            .error_for_status()?;

        Ok(response
            .json::<StandardServiceResponse>()
            .await?
            .into_result()?)
    }

    pub async fn clear_redis_db(&self) -> Result<(), CrtClientError> {
        let response = self
            .0
            .request(
                Method::POST,
                "0/ServiceModel/AppInstallerService.svc/ClearRedisDb",
            )
            .header(reqwest::header::CONTENT_LENGTH, "0")
            .send_with_session(self.0)
            .await?
            .error_for_status()?;

        Ok(response
            .json::<StandardServiceResponse>()
            .await?
            .into_result()?)
    }

    #[allow(dead_code)]
    pub async fn install_app_from_file(
        &self,
        code: &str,
        name: &str,
        package_filename: &str,
    ) -> Result<(), CrtClientError> {
        let response = self
            .0
            .request(
                Method::POST,
                "0/ServiceModel/AppInstallerService.svc/InstallAppFromFile",
            )
            .json(&json!({
                "Code": code,
                "Name": name,
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

    pub async fn load_packages_to_db<StrArr, Str>(
        &self,
        package_names: Option<StrArr>,
    ) -> Result<FileSystemSynchronizationResultResponse, CrtClientError>
    where
        Str: AsRef<str>,
        StrArr: AsRef<[Str]> + Serialize,
    {
        let response = self
            .0
            .request(
                Method::POST,
                "0/ServiceModel/AppInstallerService.svc/LoadPackagesToDB",
            )
            .json(&json!(package_names))
            .send_with_session(self.0)
            .await?
            .error_for_status()?;

        Ok(response
            .json::<FileSystemSynchronizationResultResponse>()
            .await?
            .into_result()?)
    }

    pub async fn load_packages_to_fs<StrArr, Str>(
        &self,
        package_names: Option<StrArr>,
    ) -> Result<FileSystemSynchronizationResultResponse, CrtClientError>
    where
        Str: AsRef<str>,
        StrArr: AsRef<[Str]> + Serialize,
    {
        let response = self
            .0
            .request(
                Method::POST,
                "0/ServiceModel/AppInstallerService.svc/LoadPackagesToFileSystem",
            )
            .json(&json!(package_names))
            .send_with_session(self.0)
            .await?
            .error_for_status()?;

        Ok(response
            .json::<FileSystemSynchronizationResultResponse>()
            .await?
            .into_result()?)
    }
}

#[derive(Debug, Deserialize)]
pub struct FileSystemSynchronizationResultResponse {
    pub changes: Vec<FileSystemSynchronizationPackage>,

    pub errors: Vec<FileSystemSynchronizationError>,

    #[serde(flatten)]
    pub base: StandardServiceResponse,
}

impl FileSystemSynchronizationResultResponse {
    pub fn into_result(
        self,
    ) -> Result<FileSystemSynchronizationResultResponse, StandardServiceError> {
        if !self.base.success {
            let err = self.base.error_info.expect(
                "self.base.success = true, but self.base.error_info is None, which is unexpected",
            );

            return Err(err);
        }

        Ok(self)
    }
}

#[derive(Debug, Deserialize)]
pub struct FileSystemSynchronizationWorkspaceItem {
    pub name: String,

    pub state: FileSystemSynchronizationObjectState,

    #[serde(rename = "type")]
    pub object_type: FileSystemSynchronizationObjectType,

    #[allow(dead_code)]
    #[serde(rename = "uId")]
    pub uid: String,

    #[serde(rename = "cultureName")]
    pub culture_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct FileSystemSynchronizationPackage {
    #[serde(flatten)]
    pub workspace_item: FileSystemSynchronizationWorkspaceItem,

    pub items: Vec<FileSystemSynchronizationWorkspaceItem>,
}

#[derive(Debug, Deserialize)]
pub struct FileSystemSynchronizationError {
    #[serde(rename = "workspaceItem")]
    pub workspace_item: FileSystemSynchronizationWorkspaceItem,

    #[serde(rename = "errorInfo")]
    pub error_info: StandardServiceError,
}

#[derive(Debug)]
pub enum FileSystemSynchronizationObjectState {
    NotChanged = 0,
    New = 1,
    Changed = 2,
    Deleted = 3,
    Reverted = 4,
    Conflicted = 5,
}

impl Display for FileSystemSynchronizationObjectState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::NotChanged => write!(f, "Not changed"),
            _ => Debug::fmt(&self, f),
        }
    }
}

#[derive(Debug)]
pub enum FileSystemSynchronizationObjectType {
    Package = 0,
    Schema = 1,
    Assembly = 2,
    SqlScript = 3,
    SchemaData = 4,
    CoreResource = 5,
    SchemaResource = 6,
    FileContent = 7,
}

impl Display for FileSystemSynchronizationObjectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self, f)
    }
}

impl FileSystemSynchronizationObjectType {
    pub fn get_fs_order_index(&self) -> i8 {
        match self {
            FileSystemSynchronizationObjectType::Package => 0,
            FileSystemSynchronizationObjectType::CoreResource => 1,
            FileSystemSynchronizationObjectType::Assembly => 2,
            FileSystemSynchronizationObjectType::SchemaData => 3,
            FileSystemSynchronizationObjectType::FileContent => 4,
            FileSystemSynchronizationObjectType::SchemaResource => 5,
            FileSystemSynchronizationObjectType::Schema => 6,
            FileSystemSynchronizationObjectType::SqlScript => 7,
        }
    }
}

impl<'de> Deserialize<'de> for FileSystemSynchronizationObjectState {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        match i8::deserialize(deserializer)? {
            0 => Ok(FileSystemSynchronizationObjectState::NotChanged),
            1 => Ok(FileSystemSynchronizationObjectState::New),
            2 => Ok(FileSystemSynchronizationObjectState::Changed),
            3 => Ok(FileSystemSynchronizationObjectState::Deleted),
            4 => Ok(FileSystemSynchronizationObjectState::Reverted),
            5 => Ok(FileSystemSynchronizationObjectState::Conflicted),
            _ => Err(serde::de::Error::custom("Expected 0-5 for state")),
        }
    }
}

impl<'de> Deserialize<'de> for FileSystemSynchronizationObjectType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        match i8::deserialize(deserializer)? {
            0 => Ok(FileSystemSynchronizationObjectType::Package),
            1 => Ok(FileSystemSynchronizationObjectType::Schema),
            2 => Ok(FileSystemSynchronizationObjectType::Assembly),
            3 => Ok(FileSystemSynchronizationObjectType::SqlScript),
            4 => Ok(FileSystemSynchronizationObjectType::SchemaData),
            5 => Ok(FileSystemSynchronizationObjectType::CoreResource),
            6 => Ok(FileSystemSynchronizationObjectType::SchemaResource),
            7 => Ok(FileSystemSynchronizationObjectType::FileContent),
            _ => Err(serde::de::Error::custom("Expected 0-7 for type")),
        }
    }
}
