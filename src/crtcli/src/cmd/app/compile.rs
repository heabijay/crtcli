use crate::app::CrtClient;
use crate::app::workspace_explorer::{BaseResponse, BuildPackageError};
use crate::cmd::app;
use crate::cmd::app::AppCommand;
use crate::cmd::cli::{CommandDynError, CommandResult};
use anstream::stdout;
use anstyle::{AnsiColor, Color, Style};
use async_trait::async_trait;
use clap::Args;
use std::io::Write;
use std::sync::Arc;
use thiserror::Error;

#[derive(Args, Debug)]
pub struct CompileCommand {
    /// Use Rebuild method instead of just Build
    #[arg(short = 'f', long)]
    pub force_rebuild: bool,

    /// Restart application after successful compilation
    #[arg(short, long)]
    pub restart: bool,
}

#[derive(Debug, Error)]
pub enum CompileCommandError {
    #[error("App restart error: {0}")]
    AppRestart(#[source] CommandDynError),
}

#[async_trait]
impl AppCommand for CompileCommand {
    async fn run(&self, client: Arc<CrtClient>) -> CommandResult {
        let progress = spinner_precise!(
            "{operation_str} Creatio application at {bold}{url}{bold:#}",
            bold = Style::new().bold(),
            operation_str = if self.force_rebuild {
                "Rebuilding"
            } else {
                "Compiling"
            },
            url = client.base_url()
        );

        let response = if self.force_rebuild {
            client.workspace_explorer_service().rebuild().await?
        } else {
            client.workspace_explorer_service().build().await?
        };

        progress.suspend(|| print_build_response(&response))?;

        progress.finish_with_message(format!(
            "{green}Creatio application {operation_str} successfully at {green_bold}{url}{green_bold:#}{green}!{green:#}",
            green = Style::new().fg_color(Some(Color::Ansi(AnsiColor::Green))),
            green_bold = Style::new().fg_color(Some(Color::Ansi(AnsiColor::Green))).bold(),
            operation_str = if self.force_rebuild {
                "rebuilt" 
            } else {
                "compiled" 
            },
            url = client.base_url(),
        ));

        if self.restart {
            app::restart::RestartCommand
                .run(client)
                .await
                .map_err(CompileCommandError::AppRestart)?;
        }

        Ok(())
    }
}

pub fn print_build_response(response: &BaseResponse) -> CommandResult {
    let warn_printer = BuildPackageErrorPrinter::new_for_warning();
    let error_printer = BuildPackageErrorPrinter::new_for_error();

    let mut stdout = stdout().lock();

    if let Some(errors) = &response.errors {
        for error in errors {
            match error.warning {
                true => warn_printer.print(&mut stdout, error),
                false => error_printer.print(&mut stdout, error),
            }
        }
    }

    if let Some(error_info) = &response.error_info {
        writeln!(
            stdout,
            "{style}Error message -> {}{style:#}",
            error_info.message,
            style = error_printer.error_style
        )
        .unwrap();
    }

    if let Some(message) = &response.message {
        writeln!(stdout, "> {message}").unwrap();
    }

    match (
        response.success,
        response.has_any_error(),
        &response.error_info,
    ) {
        (true, _, _) => {}
        (false, false, None) => {}
        _ => return Err("compilation finished with errors".into()),
    }

    Ok(())
}

struct BuildPackageErrorPrinter {
    type_style: Style,
    file_style: Style,
    error_style: Style,
}

impl BuildPackageErrorPrinter {
    fn new_for_warning() -> Self {
        Self::new_from_style(Style::new().fg_color(Some(Color::Ansi(AnsiColor::Yellow))))
    }

    fn new_for_error() -> Self {
        Self::new_from_style(Style::new().fg_color(Some(Color::Ansi(AnsiColor::Red))))
    }

    fn new_from_style(base_style: Style) -> Self {
        Self {
            type_style: base_style.bold(),
            file_style: base_style.italic(),
            error_style: base_style,
        }
    }

    fn print(&self, mut w: impl Write, e: &BuildPackageError) {
        match e.warning {
            true => write!(w, "{style}WARN{style:#}", style = self.type_style).unwrap(),
            false => write!(w, "{style}ERROR{style:#}", style = self.type_style).unwrap(),
        };

        match e.filename.as_str() {
            "" => (),
            _ => write!(
                w,
                " {style}{}({}:{}){style:#}",
                e.filename,
                e.line,
                e.column,
                style = self.file_style
            )
            .unwrap(),
        };

        writeln!(
            w,
            " {style}{}: {}{style:#}",
            e.error_number,
            e.error_text,
            style = self.error_style
        )
        .unwrap();
    }
}
