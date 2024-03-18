use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Sled error")]
    SledError(#[from] sled::Error),
    #[error("Serialiser error")]
    SerialiserError(#[from] SerialiserError),
    #[error("This operation is not allowed")]
    IllegalOperation,
}

#[derive(Error, Debug)]
pub enum SerialiserError {
    #[cfg_attr(feature = "bincode", error("Bincode error"))]
    BincodeError(#[from] BincodeError),
}

#[cfg(feature = "bincode")]
#[derive(Error, Debug)]
pub enum BincodeError {
    #[error("Encode error")]
    EncodeError(#[from] bincode::error::EncodeError),
    #[error("Decode error")]
    DecodeError(#[from] bincode::error::DecodeError),
}

#[cfg(feature = "bincode")]
impl From<bincode::error::DecodeError> for Error {
    fn from(value: bincode::error::DecodeError) -> Self {
        Self::SerialiserError(SerialiserError::BincodeError(BincodeError::DecodeError(
            value,
        )))
    }
}

#[cfg(feature = "bincode")]
impl From<bincode::error::EncodeError> for Error {
    fn from(value: bincode::error::EncodeError) -> Self {
        Self::SerialiserError(SerialiserError::BincodeError(BincodeError::EncodeError(
            value,
        )))
    }
}

impl From<Error> for std::io::Error {
    fn from(value: Error) -> Self {
        match value {
            Error::SledError(e) => e.into(),
            Error::SerialiserError(_) => {
                std::io::Error::new::<Error>(std::io::ErrorKind::InvalidData, value)
            }
            Error::IllegalOperation => {
                std::io::Error::new::<Error>(std::io::ErrorKind::InvalidInput, value)
            }
        }
    }
}
