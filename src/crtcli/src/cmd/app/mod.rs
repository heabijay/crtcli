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
use std::process::ExitCode;

mod flush_redis;
mod fs;
mod install_log;
mod pkg;
mod pkgs;
mod request;
mod restart;
mod sql;
mod tunnel;

use crate::CommandHandledError;
use crate::app::{CrtClient, CrtClientError, CrtCredentials, CrtSession};
use crate::cmd::cli::{CommandDynError, CommandResult};
use crate::cmd::workspace_config::{WorkspaceAppConfig, WorkspaceConfig};
use anstyle::{AnsiColor, Color, Style};
use async_trait::async_trait;
use clap::{Args, Subcommand};
use std::sync::Arc;

const DEFAULT_APP_USERNAME: &str = "Supervisor";
const DEFAULT_APP_PASSWORD: &str = "Supervisor";

#[derive(Debug, Args, Clone)]
pub struct AppCommandArgs {
    /// Creatio Base URL or App Alias
    ///
    /// Check workspace.crtcli.toml in docs for more information about app aliases
    #[arg(value_name = "URL/APP", value_hint = clap::ValueHint::Url, env = "CRTCLI_APP_URL")]
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

#[async_trait]
pub trait AppCommand {
    async fn run(&self, client: Arc<CrtClient>) -> CommandResult;
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
    ///
    /// This command requires any compatible SQL runner package to be installed.
    ///
    /// For more information, please check docs: https://github.com/heabijay/crtcli
    Sql(sql::SqlCommand),

    /// Establishes TCP tunnels via the Creatio instance to access internal services
    ///
    /// This command requires the crtcli.tunneling package to be installed.
    ///
    /// For more information, please check docs: https://github.com/heabijay/crtcli
    Tunnel(tunnel::TunnelCommand),
}

impl AppCommands {
    pub async fn run(&self, args: AppCommandArgs) -> CommandResult {
        rustls::crypto::ring::default_provider()
            .install_default()
            .expect("failed to install rustls crypto provider");

        let args = Self::load_and_apply_workspace_config(args)?;
        let client = Arc::new(Self::build_client(&args)?);

        match self {
            AppCommands::Compile(command) => command.run(client).await,
            AppCommands::FlushRedis(command) => command.run(client).await,
            AppCommands::Fs { command } => command.run(client).await,
            AppCommands::InstallLog(command) => command.run(client).await,
            AppCommands::Pkg { command } => command.run(client).await,
            AppCommands::Pkgs(command) => command.run(client).await,
            AppCommands::Restart(command) => command.run(client).await,
            AppCommands::Request(command) => command.run(client).await,
            AppCommands::Sql(command) => command.run(client).await,
            AppCommands::Tunnel(command) => command.run(client).await,
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
                    style = Style::new()
                        .fg_color(Some(Color::Ansi(AnsiColor::BrightYellow)))
                        .dimmed(),
                    italic = Style::new()
                        .fg_color(Some(Color::Ansi(AnsiColor::BrightYellow)))
                        .dimmed()
                        .italic(),
                );
            }

            if args.password.is_none() {
                eprintln!(
                    "{style}warning: Creatio password is not specified, using default:{style:#} {italic}{DEFAULT_APP_USERNAME}{italic:#}",
                    style = Style::new()
                        .fg_color(Some(Color::Ansi(AnsiColor::BrightYellow)))
                        .dimmed(),
                    italic = Style::new()
                        .fg_color(Some(Color::Ansi(AnsiColor::BrightYellow)))
                        .dimmed()
                        .italic(),
                );
            }

            None
        }
    }

    fn is_http_url_address(url: &str) -> bool {
        let url_lowercase = url.to_lowercase();

        url_lowercase.starts_with("http://") || url_lowercase.starts_with("https://")
    }

    fn load_and_apply_workspace_config(
        mut args: AppCommandArgs,
    ) -> Result<AppCommandArgs, CommandDynError> {
        if Self::is_http_url_address(&args.url) {
            return Ok(args);
        }

        let workspace_config = WorkspaceConfig::load_from_current_dir()?;
        let workspace_app_config = workspace_config.apps().get(&args.url);

        return if let Some(app_config) = workspace_app_config {
            args.merge_from_workspace_app_config(app_config.to_owned());

            Ok(args)
        } else {
            print_app_aliases_not_found(workspace_config, &args.url);

            Err(CommandHandledError(ExitCode::FAILURE).into())
        };

        fn print_app_aliases_not_found(workspace_config: WorkspaceConfig, alias: &str) {
            let bold = Style::new().bold();
            let bold_underline = Style::new().bold().underline();
            let max_key_len = workspace_config
                .apps()
                .keys()
                .map(|k| k.len())
                .max()
                .unwrap_or(0);

            eprintln!(
                "{red_bold}error:{red_bold:#} unrecognized app alias '{orange}{alias}{orange:#}' or it is not valid http(s) Creatio Base URL",
                red_bold = Style::new()
                    .fg_color(Some(Color::Ansi(AnsiColor::Red)))
                    .bold(),
                orange = Style::new().fg_color(Some(Color::Ansi(AnsiColor::BrightYellow))),
            );

            eprintln!();

            let sorted_apps = {
                let mut apps: Vec<_> = workspace_config.apps().iter().collect();
                apps.sort_by(|k1, k2| k1.0.cmp(k2.0));
                apps
            };

            eprintln!("{bold_underline}Apps:{bold_underline:#}");

            for app in &sorted_apps {
                eprintln!(
                    "  {bold}{alias:<max_key_len$}{bold:#}  {url}",
                    alias = app.0,
                    url = app.1.url,
                );
            }

            if sorted_apps.is_empty() {
                eprintln!(
                    "  {italic}[No apps defined along workspace.crtcli.toml files]{italic:#}",
                    italic = Style::new().italic(),
                );
            }

            eprintln!();
            eprintln!(
                "{bold_underline}Usage:{bold_underline:#} {bold}crtcli app{bold:#} <URL/APP> [COMMAND]"
            );
            eprintln!();
            eprintln!("For more information, try '{bold}crtcli app --help{bold:#}'.");
        }
    }
}

impl AppCommandArgs {
    pub fn merge_from_workspace_app_config(&mut self, app_config: WorkspaceAppConfig) {
        self.url = app_config.url;
        self.username = app_config.username;
        self.password = app_config.password;
        self.insecure = app_config.insecure.unwrap_or_default();
        self.net_framework = app_config.net_framework.unwrap_or_default();
    }
}
