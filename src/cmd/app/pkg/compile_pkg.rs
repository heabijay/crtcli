use crate::app::CrtClientGenericError;
use crate::cmd::app::{print_build_response, AppCommand, AppCommandArgs};
use clap::Args;
use std::error::Error;
use thiserror::Error;

#[derive(Args, Debug)]
pub struct CompilePkgCommand {
    /// Name of package to compile (default: package name from ./descriptor.json)
    #[arg(value_hint = clap::ValueHint::Other)]
    pub package_name: Option<String>,

    /// Restart the Creatio application after successful package compilation
    #[arg(short, long)]
    pub restart: bool,
}

#[derive(Debug, Error)]
pub enum CompilePkgCommandError {
    #[error("App restart error: {0}")]
    AppRestart(#[source] CrtClientGenericError),
}

impl AppCommand for CompilePkgCommand {
    fn run(&self, app: &AppCommandArgs) -> Result<(), Box<dyn Error>> {
        let package_name = detect_target_package_name!(&self.package_name);
        let client = app.build_client()?;

        let result = client
            .workspace_explorer_service()
            .build_package(package_name)?;

        print_build_response(&result)?;

        if self.restart {
            client
                .app_installer_service()
                .restart_app()
                .map_err(CompilePkgCommandError::AppRestart)?;

            eprintln!("Application restart has been requested");

            if !client.is_net_framework() {
                eprintln!("Note: if restart does not work, please check if you need to use --net-framework flag");
            }
        }

        Ok(())
    }
}
