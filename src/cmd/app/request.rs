use crate::app::{CrtClientGenericError, CrtRequestBuilderReauthorize};
use crate::cmd::app::{AppCommand, AppCommandArgs};
use anstream::{stderr, stdout};
use anstyle::Style;
use clap::builder::{ValueParser, ValueParserFactory};
use clap::Args;
use reqwest::blocking::Response;
use reqwest::header::{HeaderName, HeaderValue};
use reqwest::Method;
use std::error::Error;
use std::fs::File;
use std::io::{stdin, BufRead, ErrorKind, Read, Write};
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

        print_response_headers(&response);

        match &self.output_file {
            Some(output_file) => {
                std::io::copy(&mut response, &mut File::create(output_file)?)?;
            }
            None => try_read_response_to_stdout(&mut response)?,
        }

        return Ok(());

        fn read_data_from_stdin() -> Result<String, std::io::Error> {
            let dimmed = Style::new().dimmed();
            let italic = Style::new().italic();

            eprintln!("Please enter request data (body) below: ");
            eprintln!("{dimmed}-=-=- -=-=- -=-=- -=-=- -=-=-{dimmed:#}");
            eprintln!("{italic}");

            let mut data = String::new();

            loop {
                if stdin()
                    .lock()
                    .read_line(&mut data)
                    .inspect_err(|_| eprint!("{italic:#}"))?
                    == 1
                {
                    break;
                }
            }

            data.truncate(data.len() - 2);

            eprintln!("{dimmed}-=-=- -=-=- -=-=- -=-=- -=-=-{dimmed:#}");
            eprintln!();

            Ok(data)
        }

        fn print_response_headers(response: &Response) {
            let key_style = Style::new().bold();
            let header_style = Style::new().bold().underline();
            let mut stderr = stderr().lock();

            writeln!(
                stderr,
                "{header_style}{version:?} {status_code} {status_reason}{header_style:#}",
                version = response.version(),
                status_code = response.status().as_str(),
                status_reason = response.status().canonical_reason().unwrap_or_default(),
            )
            .unwrap();

            const PRINT_HEADERS: [&str; 3] = ["content-length", "content-type", "location"];

            let mut headers: Vec<(&HeaderName, &HeaderValue)> = response
                .headers()
                .iter()
                .filter(|(name, _)| PRINT_HEADERS.contains(&name.as_str()))
                .collect();

            headers.sort_by_key(|(name, _)| name.as_str());

            let mut name_buf = String::with_capacity(64);

            for (name, value) in headers {
                writeln!(
                    stderr,
                    "{key_style}{name}{key_style:#}: {value}",
                    name = titlecase_header(name, &mut name_buf),
                    value = value.to_str().unwrap_or("<not an ascii str>")
                )
                .unwrap();
            }

            return;

            // Source: https://github.com/ducaale/xh/blob/master/src/formatting/headers.rs#L216C1-L232C2
            fn titlecase_header<'b>(name: &HeaderName, buffer: &'b mut String) -> &'b str {
                let name = name.as_str();

                buffer.clear();
                buffer.reserve(name.len());

                // Ought to be equivalent to how hyper does it
                // https://github.com/hyperium/hyper/blob/f46b175bf71b202fbb907c4970b5743881b891e1/src/proto/h1/role.rs#L1332
                // Header names are ASCII so operating on char or u8 is equivalent
                let mut prev = '-';

                for mut c in name.chars() {
                    if prev == '-' {
                        c.make_ascii_uppercase();
                    }
                    buffer.push(c);
                    prev = c;
                }

                buffer
            }
        }

        fn try_read_response_to_stdout(response: &mut Response) -> Result<(), Box<dyn Error>> {
            let mut response_str = String::new();

            match response.read_to_string(&mut response_str) {
                Ok(_) => {},
                Err(err) if err.kind() == ErrorKind::InvalidData => return Err("response body seems like not valid utf8 string, consider to use --output-file <path> parameter to save response body to file".into()),
                Err(err) => return Err(err.into()),
            }

            if response_str.is_empty() {
                return Ok(());
            }

            eprintln!();

            let mut stdout = stdout().lock();

            match serde_json::from_str::<serde_json::Value>(&response_str) {
                Ok(json) => {
                    serde_json::to_writer_pretty(&mut stdout, &json)?;

                    writeln!(stdout).unwrap();
                }
                _ => writeln!(stdout, "{response_str}").unwrap(),
            }

            Ok(())
        }
    }
}
