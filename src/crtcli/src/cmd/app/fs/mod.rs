mod check_fs;
pub mod pull_fs;
pub mod push_fs;

use crate::app::{
    CrtClient, FileSystemSynchronizationObjectState, FileSystemSynchronizationResultResponse,
};
use crate::cmd::app::AppCommand;
use crate::cmd::cli::CommandResult;
use anstream::stdout;
use anstyle::{AnsiColor, Color, Style};
use async_trait::async_trait;
use clap::Subcommand;
use std::io::Write;
use std::sync::Arc;

#[derive(Debug, Subcommand)]
pub enum FsCommands {
    /// Check is File System Development mode is enabled for the Creatio instance
    Check(check_fs::CheckFsCommand),

    /// Unload packages from Creatio database into filesystem
    Pull(pull_fs::PullFsCommand),

    /// Load packages from filesystem into Creatio database
    Push(push_fs::PushFsCommand),
}

#[async_trait]
impl AppCommand for FsCommands {
    async fn run(&self, client: Arc<CrtClient>) -> CommandResult {
        match self {
            FsCommands::Check(command) => command.run(client).await,
            FsCommands::Pull(command) => command.run(client).await,
            FsCommands::Push(command) => command.run(client).await,
        }
    }
}

fn print_fs_sync_result(result: &FileSystemSynchronizationResultResponse) {
    let mut stdout = stdout().lock();
    let bold = Style::new().bold();
    let green = Style::new().fg_color(Some(Color::Ansi(AnsiColor::Green)));
    let red = Style::new().fg_color(Some(Color::Ansi(AnsiColor::Red)));

    if !result.changes.is_empty() {
        for package in &result.changes {
            writeln!(
                stdout,
                "Package {bold}{name}{bold:#} - {state:?}:",
                name = package.workspace_item.name,
                state = package.workspace_item.state
            )
            .unwrap();

            let mut sorted_items_refs = package.items.iter().collect::<Vec<_>>();

            sorted_items_refs.sort_by(|i1, i2| {
                i1.object_type
                    .get_fs_order_index()
                    .cmp(&i2.object_type.get_fs_order_index())
                    .then(
                        i1.name
                            .to_lowercase()
                            .cmp(&i2.name.to_lowercase())
                            .then(i1.culture_name.cmp(&i2.culture_name)),
                    )
            });

            for item in sorted_items_refs {
                let object_type = &item.object_type;
                let item_name = &item.name;
                let culture_suffix = item
                    .culture_name
                    .as_ref()
                    .map(|x| format!(", {x}"))
                    .unwrap_or_default();

                match item.state {
                    FileSystemSynchronizationObjectState::NotChanged => {}
                    FileSystemSynchronizationObjectState::New => writeln!(
                        stdout,
                        "{green}\tcreated:\t{object_type:?} -> {item_name}{culture_suffix}{green:#}",
                    ).unwrap(),
                    FileSystemSynchronizationObjectState::Deleted => writeln!(
                        stdout,
                        "{red}\tdeleted:\t{object_type:?} -> {item_name}{culture_suffix}{red:#}",
                    ).unwrap(),
                    FileSystemSynchronizationObjectState::Changed => writeln!(
                        stdout,
                        "\tmodified:\t{object_type:?} -> {item_name}{culture_suffix}",
                    ).unwrap(),
                    _ => writeln!(
                        stdout,
                        "\t{status:?}:\t{object_type:?} -> {item_name}{culture_suffix}",
                        status = item.state,
                    ).unwrap(),
                }
            }
        }
    }

    if !result.errors.is_empty() {
        writeln!(stdout, "{red}Errors ({}):{red:#}", result.errors.len()).unwrap();

        for error in &result.errors {
            let culture_suffix = error
                .workspace_item
                .culture_name
                .as_ref()
                .map(|x| format!(", {x}"))
                .unwrap_or_default();

            writeln!(
                stdout,
                "{red}{item_name}{culture_suffix} ({object_type:?}): {error_info}{red:#}",
                item_name = error.workspace_item.name,
                object_type = error.workspace_item.object_type,
                error_info = error.error_info,
            )
            .unwrap();
        }
    }
}
