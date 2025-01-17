use crate::app::{CrtClient, CrtClientError, CrtRequestBuilderExt};
use reqwest::StatusCode;
use serde::Deserialize;
use serde_json::json;
use thiserror::Error;

pub trait SqlRunner: Send + Sync {
    fn sql(&self, client: &CrtClient, sql: &str) -> Result<SqlRunnerResult, SqlRunnerError>;
}

#[derive(Debug)]
pub struct SqlRunnerResult {
    pub rows_affected: u64,
    pub table: Option<Vec<serde_json::Map<String, serde_json::Value>>>,
}

#[derive(Debug, Error)]
pub enum SqlRunnerError {
    #[error("cannot detect db type: {err}")]
    DbTypeDetection { err: String },

    #[error("sql request error: {0}")]
    Request(#[from] CrtClientError),

    #[error("sql runner not found on target server")]
    NotFound,

    #[error("sql execution returned error: {err}")]
    Execution { err: String },

    #[error("failed to execute sql: {0}")]
    Other(#[source] Box<dyn std::error::Error>),
}

pub struct ClioGateSqlRunner;

impl SqlRunner for ClioGateSqlRunner {
    fn sql(&self, client: &CrtClient, sql: &str) -> Result<SqlRunnerResult, SqlRunnerError> {
        let response = client
            .request(
                reqwest::Method::POST,
                "0/rest/CreatioApiGateway/ExecuteSqlScript",
            )
            .json(&json!({
                "script": sql
            }))
            .send_with_session(client)?;

        if response.status() == StatusCode::NOT_FOUND {
            return Err(SqlRunnerError::NotFound);
        }

        if response.status() == StatusCode::BAD_REQUEST {
            let response_text = response.text().map_err(CrtClientError::ReqwestError)?;

            return Err(SqlRunnerError::Execution { err: response_text });
        }

        let response_body: serde_json::Value = response
            .error_for_status()
            .map_err(CrtClientError::ReqwestError)?
            .json()
            .map_err(CrtClientError::ReqwestError)?;

        let response_body = response_body.as_str().ok_or_else(|| {
            SqlRunnerError::Other("failed to parse response body as json string".into())
        })?;

        let rows_affected: Result<u64, _> = response_body.parse();
        if let Ok(rows_affected) = rows_affected {
            return Ok(SqlRunnerResult {
                rows_affected,
                table: None,
            });
        }

        let response_body: Vec<serde_json::Map<String, serde_json::Value>> =
            serde_json::from_str(response_body)
                .map_err(|err| SqlRunnerError::Other(Box::new(err)))?;

        Ok(SqlRunnerResult {
            rows_affected: 0,
            table: Some(response_body),
        })
    }
}

pub struct SqlConsoleSqlRunner;

impl SqlRunner for SqlConsoleSqlRunner {
    fn sql(&self, client: &CrtClient, sql: &str) -> Result<SqlRunnerResult, SqlRunnerError> {
        let response = client
            .request(
                reqwest::Method::POST,
                "0/rest/SqlConsoleService/ExecuteSqlScript",
            )
            .json(&json!({
                "sqlScript": sql
            }))
            .send_with_session(client)?;

        if response.status() == StatusCode::NOT_FOUND {
            return Err(SqlRunnerError::NotFound);
        }

        let response_body: SqlConsoleResponse = response
            .error_for_status()
            .map_err(CrtClientError::ReqwestError)?
            .json()
            .map_err(CrtClientError::ReqwestError)?;

        let response_body = response_body.execute_sql_script_result_root;

        if !response_body.success {
            return Err(SqlRunnerError::Execution {
                err: response_body
                    .error_message
                    .unwrap_or("unknown error".to_owned()),
            });
        }

        Ok(SqlRunnerResult {
            rows_affected: response_body.rows_affected,
            table: match response_body.query_results {
                None => None,
                Some(query_result) => {
                    match query_result.len() {
                        0 => Some(Vec::new()),
                        len => {
                            if len > 1 {
                                eprintln!("more than one table returned, this currently unsupported, the first table will out");
                            }

                            todo!()

                            //todo column strings should be separated

                            // let table = query_result.remove(0);
                            //
                            // table.rows
                            //     .into_iter()
                            //     .map(|r| {
                            //         let map = HashMap::new();
                            //
                            //         for (i, rv) in r.into_iter().enumerate() {
                            //
                            //         }
                            //
                            //
                            //     })
                            //     .collect()
                        }
                    }
                }
            },
        })
    }
}

#[derive(Debug, Deserialize)]
struct SqlConsoleResponse {
    #[serde(rename = "ExecuteSqlScriptResult")]
    execute_sql_script_result_root: SqlConsoleRootResponse,
}

#[derive(Debug, Deserialize)]
struct SqlConsoleRootResponse {
    #[serde(rename = "Success")]
    success: bool,

    #[serde(rename = "ErrorMessage")]
    error_message: Option<String>,

    // #[serde(rename = "SecurityError")]
    // security_error: bool,
    #[serde(rename = "RowsAffected")]
    rows_affected: u64,

    #[serde(rename = "QueryResults")]
    query_results: Option<Vec<SqlConsoleQueryResult>>,
}

#[derive(Debug, Deserialize)]
struct SqlConsoleQueryResult {
    #[serde(rename = "Columns")]
    columns: Vec<String>,

    #[serde(rename = "Rows")]
    rows: Vec<Vec<String>>,
}

pub struct AutodetectSqlRunner;

macro_rules! next_if_not_found {
    ($client:expr, $sql: expr, $left_runner: expr, $next_runner: expr) => {
        match $left_runner.sql($client, $sql) {
            Err(SqlRunnerError::NotFound) => $next_runner,
            r => return Some((Box::new($left_runner), r)),
        }
    };
    ($client:expr, $sql: expr, $left_runner: expr) => {
        match $left_runner.sql($client, $sql) {
            Err(SqlRunnerError::NotFound) => return None,
            r => return Some((Box::new($left_runner), r)),
        }
    };
}

impl AutodetectSqlRunner {
    pub fn detect_and_run_sql(
        client: &CrtClient,
        sql: &str,
    ) -> Option<(Box<dyn SqlRunner>, Result<SqlRunnerResult, SqlRunnerError>)> {
        let next = next_if_not_found!(client, sql, ClioGateSqlRunner, SqlConsoleSqlRunner);

        next_if_not_found!(client, sql, next);
    }
}
