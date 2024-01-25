use thiserror::Error;

#[derive(Error, Debug)]
pub enum SerSledError {
    #[error("Sled error")]
    SledError(#[from] sled::Error),
    #[error("Serialiser error")]
    SerialiserError(#[from] SerialiserError),
    #[error("This operation is not allowed")]
    IllegalOperation,
}

#[derive(Error, Debug)]
pub enum SerialiserError {
    #[cfg(feature = "bincode")]
    #[error("Bincode error")]
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

impl From<bincode::error::DecodeError> for SerSledError {
    fn from(value: bincode::error::DecodeError) -> Self {
        Self::SerialiserError(SerialiserError::BincodeError(BincodeError::DecodeError(
            value,
        )))
    }
}

impl From<bincode::error::EncodeError> for SerSledError {
    fn from(value: bincode::error::EncodeError) -> Self {
        Self::SerialiserError(SerialiserError::BincodeError(BincodeError::EncodeError(
            value,
        )))
    }
}
