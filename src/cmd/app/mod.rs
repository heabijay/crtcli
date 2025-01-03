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
        detect_target_package_name!(
            $specified_package_name,
            &std::env::current_dir()
                .map_err(crate::cmd::app::pkg::DetectTargetPackageNameError::GetCurrentDirError)?
        )
    };
    () => {
        crate::pkg::utils::get_package_name_from_folder(
            &std::env::current_dir()
                .map_err(crate::cmd::app::pkg::DetectTargetPackageNameError::GetCurrentDirError)?,
        )
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

use crate::app::{CrtClient, CrtClientGenericError, CrtCredentials};
use clap::{Args, Subcommand};
use std::error::Error;

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
    pub fn build_client(&self) -> Result<CrtClient, CrtClientGenericError> {
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
    fn run(&self, app: &AppCommandArgs) -> Result<(), Box<dyn Error>>;
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

impl AppCommand for AppCommands {
    fn run(&self, app: &AppCommandArgs) -> Result<(), Box<dyn Error>> {
        match self {
            AppCommands::Compile(command) => command.run(app),
            AppCommands::FlushRedis(command) => command.run(app),
            AppCommands::Fs { command } => command.run(app),
            AppCommands::InstallLog(command) => command.run(app),
            AppCommands::Pkg { command } => command.run(app),
            AppCommands::Pkgs(command) => command.run(app),
            AppCommands::Restart(command) => command.run(app),
            AppCommands::Request(command) => command.run(app),
            AppCommands::Sql(command) => command.run(app),
        }
    }
}
