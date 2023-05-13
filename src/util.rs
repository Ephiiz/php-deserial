
use thiserror::Error;

#[derive(Error, Debug, Clone, Copy)]
pub enum CerealError {
    #[error("failed to parse string after lexing")]
    StringParseFailure,
    #[error("unknown error")]
    Unknown,
}