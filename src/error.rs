use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Sled error")]
    SledError(#[from] sled::Error),
    #[error("Bincode serialiser error")]
    BincodeError(#[from] BincodeError),
    #[error("This operation is not allowed")]
    IllegalOperation,
}

#[derive(Error, Debug)]
pub enum BincodeError {
    #[error("Encode error")]
    EncodeError(#[from] bincode::error::EncodeError),
    #[error("Decode error")]
    DecodeError(#[from] bincode::error::DecodeError),
}

impl From<bincode::error::DecodeError> for Error {
    fn from(value: bincode::error::DecodeError) -> Self {
        Self::BincodeError(BincodeError::DecodeError(value))
    }
}

impl From<bincode::error::EncodeError> for Error {
    fn from(value: bincode::error::EncodeError) -> Self {
        Self::BincodeError(BincodeError::EncodeError(value))
    }
}

impl From<Error> for std::io::Error {
    fn from(value: Error) -> Self {
        match value {
            Error::SledError(e) => e.into(),
            Error::BincodeError(_) => {
                std::io::Error::new::<Error>(std::io::ErrorKind::InvalidData, value)
            }
            Error::IllegalOperation => {
                std::io::Error::new::<Error>(std::io::ErrorKind::InvalidInput, value)
            }
        }
    }
}
