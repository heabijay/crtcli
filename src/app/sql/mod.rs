mod runner;
pub use runner::*;

mod scripts;
pub use scripts::*;

use crate::app::{CrtClient, CrtClientError, CrtDbType};
use std::ops::Deref;

pub fn detect_db_type(client: &CrtClient) -> Result<CrtDbType, CrtClientError> {
    return match client.sql("SELECT version();") {
        Ok(r) => {
            let output = r.table.ok_or_else(get_unexpected_output_error)?;

            let output_str = output
                .first()
                .ok_or_else(get_unexpected_output_error)?
                .iter()
                .next()
                .ok_or_else(get_unexpected_output_error)?
                .1
                .as_str()
                .ok_or_else(get_unexpected_output_error)?;

            return match output_str
                .to_lowercase()
                .starts_with(&"PostgreSQL".to_lowercase())
            {
                true => Ok(CrtDbType::Postgres),
                false => Ok(CrtDbType::Oracle),
            };
        }
        Err(CrtClientError::SqlRunner(sql_err))
            if { matches!(sql_err.deref(), SqlRunnerError::NotFound) } =>
        {
            Ok(CrtDbType::MsSql)
        }
        Err(err) => Err(err),
    };

    fn get_unexpected_output_error() -> CrtClientError {
        CrtClientError::SqlRunner(Box::new(SqlRunnerError::DbTypeDetection {
            err: "unexpected empty sql output".into(),
        }))
    }
}
