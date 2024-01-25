/// Copyright (C) 2024 Broward Apps
///
/// This program is free software: you can redistribute it and/or modify
/// it under the terms of the GNU General Public License as published by
/// the Free Software Foundation, either version 3 of the License, or
/// (at your option) any later version.
/// 
/// This program is distributed in the hope that it will be useful,
/// but WITHOUT ANY WARRANTY; without even the implied warranty of
/// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
/// GNU General Public License for more details.
///
/// You should have received a copy of the GNU General Public License
/// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use error::{SerSledError, SerialiserError};
use serde::{Deserialize, Serialize};

pub mod error;
pub mod tests;

/// Sled is optimised to work with big-endian bytes
#[cfg(feature = "bincode")]
pub const BINCODE_CONFIG: bincode::config::Configuration<bincode::config::BigEndian> =
    bincode::config::standard().with_big_endian();

pub enum SerialiserMode {
    #[cfg(feature = "bincode")]
    BINCODE,
}

pub struct SerSled {
    inner_tree: sled::Tree,
    ser_mode: SerialiserMode,
}

impl SerSled {
    pub fn new(
        // Also works with sled::Db since it implements Deref<sled::Tree>
        sled_tree: sled::Tree,
        ser_mode: SerialiserMode,
    ) -> Self {
        SerSled {
            inner_tree: sled_tree,
            ser_mode,
        }
    }

    pub fn get<K: Serialize, V: for<'de> Deserialize<'de>>(
        &self,
        key: &K,
    ) -> Result<Option<V>, SerSledError> {
        match self.ser_mode {
            #[cfg(feature = "bincode")]
            SerialiserMode::BINCODE => {
                let bytes = bincode::serde::encode_to_vec(key, BINCODE_CONFIG).map_err(|e| {
                    SerialiserError::BincodeError(error::BincodeError::EncodeError(e))
                })?;
                match self.inner_tree.get(bytes)? {
                    Some(res_ivec) => {
                        let deser = bincode::serde::decode_borrowed_from_slice::<V, _>(
                            &res_ivec,
                            BINCODE_CONFIG,
                        )
                        .map_err(|e| {
                            SerialiserError::BincodeError(error::BincodeError::DecodeError(e))
                        })?;

                        Ok(Some(deser))
                    }
                    None => Ok(None),
                }
            }
        }
    }

    pub fn insert<K: Serialize, V: Serialize + for<'de> Deserialize<'de>>(
        &self,
        key: &K,
        value: &V,
    ) -> Result<Option<V>, SerSledError> {
        match self.ser_mode {
            #[cfg(feature = "bincode")]
            SerialiserMode::BINCODE => {
                let key_bytes =
                    bincode::serde::encode_to_vec(key, BINCODE_CONFIG).map_err(|e| {
                        SerialiserError::BincodeError(error::BincodeError::EncodeError(e))
                    })?;
                let value_bytes =
                    bincode::serde::encode_to_vec(value, BINCODE_CONFIG).map_err(|e| {
                        SerialiserError::BincodeError(error::BincodeError::EncodeError(e))
                    })?;

                match self.inner_tree.insert(key_bytes, value_bytes)? {
                    Some(ivec) => {
                        let old_value = bincode::serde::decode_borrowed_from_slice::<V, _>(
                            &ivec,
                            BINCODE_CONFIG,
                        )
                        .map_err(|e| {
                            SerialiserError::BincodeError(error::BincodeError::DecodeError(e))
                        })?;

                        Ok(Some(old_value))
                    }
                    None => Ok(None),
                }
            }
        }
    }
}
