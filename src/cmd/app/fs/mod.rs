mod check_fs;
pub mod pull_fs;
pub mod push_fs;

use crate::app::{FileSystemSynchronizationObjectState, FileSystemSynchronizationResultResponse};
use crate::cmd::app::{AppCommand, AppCommandArgs};
use clap::Subcommand;
use owo_colors::OwoColorize;
use std::error::Error;

#[derive(Debug, Subcommand)]
pub enum FsCommands {
    /// Check is File System Development mode is enabled for the Creatio instance
    Check(check_fs::CheckFsCommand),

    /// Unload packages from Creatio database into filesystem
    Pull(pull_fs::PullFsCommand),

    /// Load packages from filesystem into Creatio database
    Push(push_fs::PushFsCommand),
}

impl AppCommand for FsCommands {
    fn run(&self, app: &AppCommandArgs) -> Result<(), Box<dyn Error>> {
        match self {
            FsCommands::Check(command) => command.run(app),
            FsCommands::Pull(command) => command.run(app),
            FsCommands::Push(command) => command.run(app),
        }
    }
}

fn print_fs_sync_result(result: &FileSystemSynchronizationResultResponse) {
    if !result.changes.is_empty() {
        for package in &result.changes {
            eprintln!(
                "Package {} - {:?}",
                package.workspace_item.name.bold(),
                package.workspace_item.state
            );

            let mut sorted_items_refs = package.items.iter().collect::<Vec<_>>();

            sorted_items_refs.sort_by(|i1, i2| {
                i1.object_type
                    .get_fs_order_index()
                    .cmp(&i2.object_type.get_fs_order_index())
                    .then(
                        i1.name
                            .cmp(&i2.name)
                            .then(i1.culture_name.cmp(&i2.culture_name)),
                    )
            });

            for item in sorted_items_refs {
                match item.state {
                    FileSystemSynchronizationObjectState::NotChanged => {}
                    FileSystemSynchronizationObjectState::New => eprintln!(
                        "\t{status}\t{object_type:?} {arrow} {name}{culture}",
                        status = "created:".green(),
                        object_type = item.object_type.green(),
                        arrow = "->".green(),
                        name = item.name.green(),
                        culture = item
                            .culture_name
                            .as_ref()
                            .map(|x| format!(", {}", x))
                            .unwrap_or_default()
                            .green()
                    ),
                    FileSystemSynchronizationObjectState::Deleted => eprintln!(
                        "\t{status}\t{object_type:?} {arrow} {name}{culture}",
                        status = "deleted:".red(),
                        object_type = item.object_type.red(),
                        arrow = "->".red(),
                        name = item.name.red(),
                        culture = item
                            .culture_name
                            .as_ref()
                            .map(|x| format!(", {}", x))
                            .unwrap_or_default()
                            .red()
                    ),
                    FileSystemSynchronizationObjectState::Changed => eprintln!(
                        "\t{status}\t{object_type:?} -> {name}{culture}",
                        status = "modified:",
                        object_type = item.object_type,
                        name = item.name,
                        culture = item
                            .culture_name
                            .as_ref()
                            .map(|x| format!(", {}", x))
                            .unwrap_or_default(),
                    ),
                    _ => eprintln!(
                        "\t{status:?}{colon}\t{object_type:?} -> {name}{culture}",
                        status = item.state,
                        colon = ":",
                        object_type = item.object_type,
                        name = item.name,
                        culture = item
                            .culture_name
                            .as_ref()
                            .map(|x| format!(", {}", x))
                            .unwrap_or_default(),
                    ),
                }
            }
        }
    }

    if !result.errors.is_empty() {
        if !result.changes.is_empty() {
            eprintln!();
            eprintln!();
        }

        eprintln!("Errors ({}):", result.errors.len());

        for error in &result.errors {
            eprintln!(
                "{}{} {}: {}",
                error.workspace_item.name.red(),
                error
                    .workspace_item
                    .culture_name
                    .as_ref()
                    .map(|x| format!(", {}", x))
                    .unwrap_or_default()
                    .red(),
                format!("({:?})", error.workspace_item.object_type).red(),
                error.error_info.red()
            );
        }
    }

    // eprintln!("{}", "Done!".bold().green());
}
