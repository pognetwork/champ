use thiserror::Error;

#[derive(Error, Debug)]
pub enum CLIError {
    #[error("unknown error")]
    Unknown(String),
    #[error("error generating salt")]
    Salt,
}
