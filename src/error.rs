pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("I/O Error")]
    Io(#[from] std::io::Error),
    #[error("No device")]
    Missing,
    #[error("Error Parsing Integer")]
    ParseInt(#[from] std::num::ParseIntError),
}
