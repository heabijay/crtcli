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

use crate::app::{CrtClient, CrtClientError, CrtCredentials, CrtSession};
use anstyle::{AnsiColor, Color, Style};
use clap::{Args, Subcommand};
use std::error::Error;
use std::sync::Arc;

const DEFAULT_APP_USERNAME: &str = "Supervisor";
const DEFAULT_APP_PASSWORD: &str = "Supervisor";

#[derive(Debug, Args)]
pub struct AppCommandArgs {
    /// Creatio Base URL
    #[arg(value_hint = clap::ValueHint::Url, env = "CRTCLI_APP_URL")]
    url: String,

    /// Creatio Username [default: Supervisor]
    #[arg(value_hint = clap::ValueHint::Other, env = "CRTCLI_APP_USERNAME")]
    username: Option<String>,

    /// Creatio Password [default: Supervisor]
    #[arg(value_hint = clap::ValueHint::Other, env = "CRTCLI_APP_PASSWORD")]
    password: Option<String>,

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

    /// Executes SQL queries in the Creatio using a supported SQL runner installed package
    Sql(sql::SqlCommand),
}

impl AppCommands {
    pub fn run(&self, args: &AppCommandArgs) -> Result<(), Box<dyn Error>> {
        let client = Arc::new(Self::build_client(args)?);

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

    fn build_client(args: &AppCommandArgs) -> Result<CrtClient, CrtClientError> {
        let username = if let Some(username) = &args.username {
            username
        } else {
            DEFAULT_APP_USERNAME
        };

        let password = if let Some(password) = &args.password {
            password
        } else {
            DEFAULT_APP_PASSWORD
        };

        let credentials = CrtCredentials::new(&args.url, username, password);
        let session = check_default_credentials_in_cache(&credentials, args);

        return CrtClient::builder(credentials)
            .danger_accept_invalid_certs(args.insecure)
            .use_net_framework_mode(args.net_framework)
            .with_session(session)
            .build();

        fn check_default_credentials_in_cache(
            credentials: &CrtCredentials,
            args: &AppCommandArgs,
        ) -> Option<CrtSession> {
            let session =
                crate::app::session_cache::create_default_session_cache().get_entry(credentials);

            if let Some(session) = session {
                return Some(session);
            }

            if args.username.is_none() {
                eprintln!(
                    "{style}warning: Creatio username is not specified, using default:{style:#} {italic}{DEFAULT_APP_USERNAME}{italic:#}",
                    style=Style::new().fg_color(Some(Color::Ansi(AnsiColor::BrightYellow))).dimmed(),
                    italic=Style::new().fg_color(Some(Color::Ansi(AnsiColor::BrightYellow))).dimmed().italic(),
                );
            }

            if args.password.is_none() {
                eprintln!(
                    "{style}warning: Creatio password is not specified, using default:{style:#} {italic}{DEFAULT_APP_USERNAME}{italic:#}",
                    style=Style::new().fg_color(Some(Color::Ansi(AnsiColor::BrightYellow))).dimmed(),
                    italic=Style::new().fg_color(Some(Color::Ansi(AnsiColor::BrightYellow))).dimmed().italic(),
                );
            }

            None
        }
    }
}
