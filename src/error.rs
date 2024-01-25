use thiserror::Error;

#[derive(Error, Debug)]
pub enum SerSledError {
    #[error("Sled error")]
    SledError(#[from] sled::Error),
    #[error("Serialiser error")]
    SerialiserError(#[from] SerialiserError),
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
