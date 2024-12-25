pub(crate) type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("the time format is wrong: {0}")]
    TimeFormat(String),
}
