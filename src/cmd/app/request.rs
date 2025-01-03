use crate::app::{CrtClientGenericError, CrtRequestBuilderReauthorize};
use crate::cmd::app::{AppCommand, AppCommandArgs};
use clap::builder::{ValueParser, ValueParserFactory};
use clap::Args;
use reqwest::blocking::Response;
use reqwest::Method;
use std::error::Error;
use std::fs::File;
use std::io::{stdin, BufRead, ErrorKind, Read};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Args, Debug)]
pub struct RequestCommand {
    /// HTTP method (e.g., GET, POST, PUT, DELETE, etc.)
    #[arg(value_hint = clap::ValueHint::Other)]
    method: Method,

    /// URL to request (can be absolute or relative to the Creatio base URL)
    #[arg(value_hint = clap::ValueHint::Url)]
    url: String,

    /// Send the request without authentication
    #[arg(short, long)]
    anonymous: bool,

    /// Request body data (for methods like POST).
    #[arg(short, long, value_hint = clap::ValueHint::Other)]
    data: Option<String>,

    /// Read the request body data from standard input. Use a double Enter to signal the end of input
    #[arg(short = 'D', long)]
    data_stdin: bool,

    /// Add a custom header to the request (format: Key: Value). The default Content-Type is application/json
    #[arg(short = 'H', long, value_hint = clap::ValueHint::Other, value_delimiter = ',')]
    header: Vec<HeaderArg>,

    /// Save the response body to file
    #[arg(short, long = "output", value_name = "FILE", value_hint = clap::ValueHint::FilePath)]
    output_file: Option<PathBuf>,
}

#[derive(Debug, Clone)]
struct HeaderArg {
    key: String,
    value: String,
}

#[derive(Error, Debug)]
enum HeaderArgParsingError {
    #[error("value cannot be empty")]
    EmptyValue,

    #[error("expected format is \"Key: Value\"")]
    InvalidFormat,
}

impl TryFrom<&str> for HeaderArg {
    type Error = HeaderArgParsingError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Err(HeaderArgParsingError::EmptyValue);
        }

        let header = value
            .split_once(":")
            .ok_or(HeaderArgParsingError::InvalidFormat)?;

        Ok(Self {
            key: header.0.trim().to_owned(),
            value: header.1.trim().to_owned(),
        })
    }
}

impl ValueParserFactory for HeaderArg {
    type Parser = ValueParser;

    fn value_parser() -> Self::Parser {
        ValueParser::new(|s: &str| HeaderArg::try_from(s))
    }
}

impl AppCommand for RequestCommand {
    fn run(&self, app: &AppCommandArgs) -> Result<(), Box<dyn Error>> {
        let url = self
            .url
            .strip_prefix(&app.url)
            .unwrap_or(&self.url)
            .trim_start_matches('/');

        let data = match (&self.data, self.data_stdin) {
            (Some(_), true) => return Err("you cannot use --data and --data-stdin arguments together, please select one of them".into()),
            (None, false) => None,
            (Some(str), false) => Some(str.clone()),
            (None, true) => Some(read_data_from_stdin()?),
        };

        let client = app.build_client()?;
        let mut request = client.request(self.method.clone(), url);

        for header in &self.header {
            request = request.header(&header.key, &header.value);
        }

        if !self
            .header
            .iter()
            .any(|x| x.key.to_lowercase() == "content-type")
        {
            request = request.header("Content-Type", "application/json");
        }

        if let Some(data) = data {
            request = request.body(data);
        }

        let mut response = match self.anonymous {
            true => request.send().map_err(CrtClientGenericError::from)?,
            false => request
                .send_with_session(&client)
                .map_err(CrtClientGenericError::from)?,
        };

        eprintln!("Status: {}", response.status());

        if let Some(location) = response.headers().get(reqwest::header::LOCATION) {
            if let Ok(location) = location.to_str() {
                eprintln!("Location: {location}");
            }
        }

        match &self.output_file {
            Some(output_file) => {
                let mut file = File::create(output_file)?;
                let bytes = std::io::copy(&mut response, &mut file)?;

                eprintln!("Content: Written {bytes} bytes");
            }
            None => try_read_response_to_stdout(&mut response)?,
        }

        return Ok(());

        fn try_read_response_to_stdout(response: &mut Response) -> Result<(), Box<dyn Error>> {
            let mut response_str = String::new();

            match response.read_to_string(&mut response_str) {
                Ok(bytes) => {
                    eprintln!("Content: {bytes} bytes read");

                    if !response_str.is_empty() {
                        eprintln!();
                        println!("{response_str}");
                    }
                },
                Err(err) if err.kind() == ErrorKind::InvalidData => return Err("response body seems like not valid utf8 string, consider to use --output-file <path> parameter to save response body to file".into()),
                Err(err) => return Err(err.into()),
            }

            Ok(())
        }

        fn read_data_from_stdin() -> Result<String, std::io::Error> {
            eprintln!("Please enter request data (body) below: ");
            eprintln!("-=-=- -=-=- -=-=- -=-=- -=-=-");
            eprintln!();

            let mut data = String::new();

            loop {
                if stdin().lock().read_line(&mut data)? == 1 {
                    break;
                }
            }

            data.truncate(data.len() - 2);

            eprintln!("-=-=- -=-=- -=-=- -=-=- -=-=-");
            eprintln!();

            Ok(data)
        }
    }
}
