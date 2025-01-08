macro_rules! detect_target_package_name {
    ($specified_package_name: expr, $destination_folder: expr) => {
        match &$specified_package_name {
            Some(p) => p,
            None => &crate::pkg::utils::get_package_name_from_folder($destination_folder).map_err(
                crate::cmd::app::pkg::DetectTargetPackageNameError::GetPackageNameFromFolder,
            )?,
        }
    };
    ($specified_package_name:expr) => {
        detect_target_package_name!($specified_package_name, &std::path::PathBuf::from("."))
    };
    () => {
        crate::pkg::utils::get_package_name_from_folder(&std::path::PathBuf::from("."))
            .map_err(crate::cmd::app::pkg::DetectTargetPackageNameError::GetPackageNameFromFolder)?
    };
}

mod compile;
pub use compile::print_build_response;

mod flush_redis;
mod fs;
mod install_log;
mod pkg;
mod pkgs;
mod request;
mod restart;
mod sql;

use crate::app::{CrtClient, CrtClientError, CrtCredentials};
use clap::{Args, Subcommand};
use std::error::Error;
use std::sync::Arc;

#[derive(Debug, Args)]
pub struct AppCommandArgs {
    /// Creatio Base URL
    #[arg(value_hint = clap::ValueHint::Url, env = "CRTCLI_APP_URL")]
    url: String,

    /// Creatio Username
    #[arg(value_hint = clap::ValueHint::Other, env = "CRTCLI_APP_USERNAME")]
    username: String,

    /// Creatio Password
    #[arg(value_hint = clap::ValueHint::Other, env = "CRTCLI_APP_PASSWORD")]
    password: String,

    /// Ignore SSL certificate errors
    #[arg(long, short, env = "CRTCLI_APP_INSECURE")]
    insecure: bool,

    /// Use .NET Framework (IIS) Creatio compatibility
    ///
    /// By default, crtcli primary uses .NET Core / .NET (Kestrel) API routes to operate with remote.
    /// However, some features like "app restart" works by different API routes in both platforms.
    #[arg(long = "net-framework", env = "CRTCLI_APP_NETFRAMEWORK")]
    net_framework: bool,
}

impl AppCommandArgs {
    pub fn build_client(&self) -> Result<CrtClient, CrtClientError> {
        let client = CrtClient::builder(CrtCredentials::new(
            &self.url,
            &self.username,
            &self.password,
        ))
        .danger_accept_invalid_certs(self.insecure)
        .use_net_framework_mode(self.net_framework)
        .build()?;

        Ok(client)
    }
}

pub trait AppCommand {
    fn run(&self, client: Arc<CrtClient>) -> Result<(), Box<dyn Error>>;
}

#[derive(Debug, Subcommand)]
pub enum AppCommands {
    /// Compiles the Creatio application
    Compile(compile::CompileCommand),

    /// Clears the Redis cache associated with the Creatio instance
    FlushRedis(flush_redis::FlushRedisCommand),

    /// Commands for interacting with Creatio's File System Development (FSD) mode
    Fs {
        #[command(subcommand)]
        command: fs::FsCommands,
    },

    /// Print last package installation log
    InstallLog(install_log::InstallLogCommand),

    /// Commands to manipulate with packages in Creatio
    Pkg {
        #[command(subcommand)]
        command: pkg::PkgCommands,
    },

    /// Lists the installed packages in the Creatio instance
    Pkgs(pkgs::PkgsCommand),

    /// Restarts the Creatio application
    Restart(restart::RestartCommand),

    /// Sends authenticated HTTP requests to the Creatio instance, similar to curl
    Request(request::RequestCommand),

    /// Executes SQL queries in the Creatio database using a supported SQL runner package installed in Creatio
    Sql(sql::SqlCommand),
}

impl AppCommands {
    pub fn run(&self, app: &AppCommandArgs) -> Result<(), Box<dyn Error>> {
        let client = Arc::new(app.build_client()?);

        match self {
            AppCommands::Compile(command) => command.run(client),
            AppCommands::FlushRedis(command) => command.run(client),
            AppCommands::Fs { command } => command.run(client),
            AppCommands::InstallLog(command) => command.run(client),
            AppCommands::Pkg { command } => command.run(client),
            AppCommands::Pkgs(command) => command.run(client),
            AppCommands::Restart(command) => command.run(client),
            AppCommands::Request(command) => command.run(client),
            AppCommands::Sql(command) => command.run(client),
        }
    }
}
