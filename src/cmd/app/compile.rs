use crate::app::{BuildResponse, CrtClientGenericError};
use crate::cmd::app::{AppCommand, AppCommandArgs};
use clap::Args;
use owo_colors::OwoColorize;
use std::error::Error;
use thiserror::Error;

#[derive(Args, Debug)]
pub struct CompileCommand {
    /// Use Rebuild method instead of just Build
    #[arg(short = 'f', long)]
    force_rebuild: bool,

    /// Restart application after successful compilation
    #[arg(short, long)]
    restart: bool,
}

#[derive(Debug, Error)]
pub enum CompileCommandError {
    #[error("App restart error: {0}")]
    AppRestart(#[source] CrtClientGenericError),
}

impl AppCommand for CompileCommand {
    fn run(&self, app: &AppCommandArgs) -> Result<(), Box<dyn Error>> {
        let client = app.build_client()?;

        let response = match self.force_rebuild {
            true => client.workspace_explorer_service().rebuild()?,
            false => client.workspace_explorer_service().build()?,
        };

        print_build_response(&response)?;

        if self.restart {
            client
                .app_installer_service()
                .restart_app()
                .map_err(CompileCommandError::AppRestart)?;

            eprintln!("Application restart has been requested");

            if !client.is_net_framework() {
                eprintln!("Note: if restart does not work, please check if you need to use --net-framework flag");
            }
        }

        Ok(())
    }
}

pub fn print_build_response(response: &BuildResponse) -> Result<(), Box<dyn Error>> {
    if let Some(errors) = &response.errors {
        for error in errors {
            println!("{error}");
        }
    }

    if let Some(error_info) = &response.error_info {
        println!("{}", error_info.message)
    }

    if let Some(message) = &response.message {
        println!("--> {message}")
    }

    match (
        response.success,
        response.has_any_error(),
        &response.error_info,
    ) {
        (true, _, _) => {}
        (false, false, None) => {}
        _ => return Err("compile was finished with errors".into()),
    }

    eprintln!("{}", "Compiled successfully!".green());

    Ok(())
}
