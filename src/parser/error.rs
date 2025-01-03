pub(crate) type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum Error {
    #[error("The time format is wrong: {0}")]
    TimeFormat(String),
    #[error("The following input is not a command(Syntax Error)")]
    NotACommand,
    #[error("The comment is not mentioning to the bot")]
    NotAMention,
}
