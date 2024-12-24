use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum Error {}

impl From<Error> for worker::Error {
    fn from(val: Error) -> Self {
        worker::Error::RustError(format!("{}", val))
    }
}
