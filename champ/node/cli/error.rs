use thiserror::Error;

#[derive(Error, Debug)]
pub enum CLIError {
    #[error("{0}")]
    Unknown(String),
    #[error("error generating salt")]
    Salt,
    #[error("user already exists")]
    UserExists,
    #[error("please generate a JWT key pair with '$ champ admin generate-key' ")]
    NoKeyPair,
    #[error("this command does not exist")]
    UnknownCommand,
}
