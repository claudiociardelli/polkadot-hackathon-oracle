use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Invalid Response")]
    InvalidResponse,
    #[error("Invalid Exchange Rate")]
    InvalidExchangeRate
}
