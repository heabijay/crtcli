use clap::{Command, CommandFactory, Parser, Subcommand};
use regex::Regex;
use std::borrow::Cow;
use std::io::Write;
use std::process::exit;

#[derive(Parser, Debug)]
#[command(
    version,
    about,
    author = "heabijay (heabijay@gmail.com)",
    long_about = None)]
pub struct Cli {
    /// Print debug-view of exception if it is occurred
    #[arg(long, hide = true)]
    debug: bool,

    /// Generate terminal completions config for your shell
    #[arg(long, value_enum, value_name = "SHELL")]
    completions: Option<Option<clap_complete::Shell>>,

    #[command(subcommand)]
    command: Option<Commands>,
}

impl Cli {
    pub fn run(self) -> CommandResult {
        if let Some(completions) = self.completions {
            return run_completions_command(completions);
        }

        match self.command {
            None => {
                Self::command().print_help()?;
                exit(2);
            }
            Some(command) => command.run()?,
        }

        Ok(())
    }

    pub fn debug(&self) -> bool {
        self.debug
    }
}

pub trait CliCommand {
    fn run(self) -> CommandResult;
}

pub type CommandDynError = Box<dyn std::error::Error + Send + Sync>;

pub type CommandResult = Result<(), CommandDynError>;

#[derive(Debug, Subcommand)]
enum Commands {
    /// Commands to interact with Creatio application instance
    ///
    /// This is the collection of subcommands that are related to concrete Creatio instance.
    /// You should specify Creatio connection parameters like URL, USERNAME, PASSWORD through command arguments,
    /// or you could set ENV variables (as well as create .env file) for better UX.
    ///
    /// You can also use app aliases defined in `.crtcli.toml` config file by specifying
    /// the alias name as the URL parameter. For example: `crtcli app dev restart`
    ///
    /// Example use cases:
    /// `crtcli app https://localhost:5000 restart` -- Restarts Creatio instance.
    /// `crtcli app dev restart` -- Restarts Creatio instance using the 'dev' app alias.
    /// `crtcli app pkg download CrtBase,CrtCore` -- Downloads CrtBase and CrtCore packages from Creatio to single zip file.
    /// `crtcli app pkg push` -- Immediate packs current folder as package and installs it to Creatio instance.
    #[clap(verbatim_doc_comment, visible_alias = "a")]
    App {
        #[command(flatten)]
        args: crate::cmd::app::AppCommandArgs,

        #[command(subcommand)]
        command: crate::cmd::app::AppCommands,
    },

    /// Commands for working with Creatio package files (.zip, .gz) or package folders locally
    ///
    /// This is the collection of subcommands that are related to package files and not related to any Creatio instance.
    ///
    /// Example use cases:
    /// `crtcli pkg pack .` -- Packs current folder as package to single gzip/zip file.
    /// `crtcli pkg apply . --apply-localization-cleanup 'en-US'` -- Deletes all localization files in current folder as package except en-US.
    #[clap(verbatim_doc_comment, visible_alias = "p")]
    Pkg {
        #[command(subcommand)]
        command: crate::cmd::pkg::PkgCommands,
    },
}

impl CliCommand for Commands {
    fn run(self) -> CommandResult {
        match self {
            Commands::App { args, command } => run_app_command(args, command),
            Commands::Pkg { command } => command.run(),
        }
    }
}

#[tokio::main(flavor = "current_thread")]
async fn run_app_command(
    args: crate::cmd::app::AppCommandArgs,
    command: crate::cmd::app::AppCommands,
) -> CommandResult {
    command.run(args).await
}

fn print_completions(shell: clap_complete::Shell, cmd: &mut Command) {
    match shell {
        clap_complete::Shell::Fish => print_fish_completions(cmd),
        _ => clap_complete::generate(
            shell,
            cmd,
            cmd.get_name().to_string(),
            &mut std::io::stdout(),
        ),
    }
}

fn print_fish_completions(cmd: &mut Command) {
    let mut completions = vec![];

    clap_complete::generate(
        clap_complete::Shell::Fish,
        cmd,
        cmd.get_name().to_string(),
        &mut completions,
    );

    let completions_str = String::from_utf8_lossy(&completions);
    let completions_str = postprocess_fish_completions(&completions_str);

    std::io::stdout()
        .lock()
        .write_all(completions_str.as_bytes())
        .unwrap();

    return;

    fn postprocess_fish_completions(completions_str: &str) -> Cow<'_, str> {
        return fix_app_subcommand_completions(completions_str);

        /// Patches suggestions for `crtcli app ...` subcommands for fish shell.
        ///
        /// Due to some limitations in clap_complete crate:
        /// "fish completions currently only support named arguments (e.g. -o or –opt), not positional arguments."
        /// Source: https://docs.rs/clap/latest/clap/enum.ValueHint.html#fnref1 ↩
        ///
        /// This cause after use suggestions for `crtcli app ` you receive also file suggestions.
        /// After this patch, you will receive only suggestions for `crtcli app <subcommand>` for this.
        fn fix_app_subcommand_completions(completions_str: &str) -> Cow<'_, str> {
            let app_subcommand_completions_regex = Regex::new(
                r#"(complete -c crtcli -n "__fish_crtcli_using_subcommand app; and not __fish_seen_subcommand_from .+?") -a ""#
            )
            .unwrap(); // Due to Regex is called once per execution -- no need to make it static

            app_subcommand_completions_regex.replace_all(completions_str, "$1 -f -a \"")
        }
    }
}

fn run_completions_command(completions: Option<clap_complete::Shell>) -> CommandResult {
    let shell = completions.or_else(clap_complete::Shell::from_env);

    if let Some(shell) = shell {
        print_completions(shell, &mut Cli::command());

        Ok(())
    } else {
        Err(
            "failed to detect shell, please specify shell in --completions [SHELL] arguments"
                .into(),
        )
    }
}
