use crate::app::sql::SqlRunner;
use crate::app::CrtClient;
use crate::cmd::app::AppCommand;
use anstyle::Style;
use clap::{Args, ValueEnum};
use serde::Serialize;
use std::error::Error;
use std::io::{stdin, IsTerminal, Read};
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Args, Debug)]
pub struct SqlCommand {
    /// SQL query to execute
    #[arg(value_hint = clap::ValueHint::Other)]
    sql: Option<String>,

    /// Read the SQL query from a file
    #[arg(short, long, value_hint = clap::ValueHint::FilePath)]
    file: Option<PathBuf>,

    /// Specify the SQL runner to use (default: autodetect)
    #[arg(long, value_enum)]
    runner: Option<SqlRunnerSelect>,

    /// Display the results in JSON format
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, ValueEnum)]
enum SqlRunnerSelect {
    Cliogate,
    SqlConsole,
}

impl AppCommand for SqlCommand {
    fn run(&self, client: Arc<CrtClient>) -> Result<(), Box<dyn Error>> {
        let sql = match (self.sql.as_ref(), self.file.as_ref()) {
            (Some(_), Some(_)) => return Err("sql command and --file argument cannot be specified at the same time, consider to remove one of them".into()),
            (Some(sql), None) => sql,
            (None, Some(file)) => &std::fs::read_to_string(file)?,
            (None, None) => &read_data_from_stdin()?,
        };

        let process = spinner!(
            "Executing SQL query at {bold}{url}{bold:#}",
            bold = Style::new().bold(),
            url = client.base_url()
        );

        let result = match &self.runner {
            None => client.sql(sql)?,
            Some(SqlRunnerSelect::Cliogate) => {
                crate::app::sql::ClioGateSqlRunner.sql(&client, sql)?
            }
            Some(SqlRunnerSelect::SqlConsole) => {
                crate::app::sql::SqlConsoleSqlRunner.sql(&client, sql)?
            }
        };

        process.finish_and_clear();

        if let Some(table) = result.table {
            let mut buf = vec![];

            table.serialize(&mut serde_json::Serializer::pretty(&mut buf))?;

            println!("{}", String::from_utf8(buf)?);
        } else {
            println!("Rows affected: {}", result.rows_affected);
        }

        return Ok(());

        fn read_data_from_stdin() -> Result<String, std::io::Error> {
            let dimmed = Style::new().dimmed();
            let italic = Style::new().italic();
            let stdin_terminal = stdin().is_terminal();

            if stdin_terminal {
                eprintln!("Enter SQL query below: (Press Ctrl+D twice to finish)");
                eprintln!("{dimmed}-=-=- -=-=- -=-=- -=-=- -=-=-{dimmed:#}");
                eprintln!("{italic}");
            }

            let mut data = String::new();

            stdin().lock().read_to_string(&mut data).inspect_err(|_| {
                if stdin_terminal {
                    eprint!("{italic:#}")
                }
            })?;

            if stdin_terminal {
                eprintln!();
                eprintln!();
                eprintln!("{dimmed}-=-=- -=-=- -=-=- -=-=- -=-=-{dimmed:#}");
                eprintln!();
            }

            Ok(data)
        }
    }
}
