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
use error::SerSledError;
use serde::{Deserialize, Serialize};

pub mod error;
pub mod tests;

/// Sled is optimised to work with big-endian bytes
#[cfg(feature = "bincode")]
pub const BINCODE_CONFIG: bincode::config::Configuration<bincode::config::BigEndian> =
    bincode::config::standard().with_big_endian();
pub const CONFIGUATION_TREE_KEY: &[u8] = b"_ser-sled_serialiser";

pub enum SerialiserMode {
    #[cfg(feature = "bincode")]
    BINCODE,
}

/// Convert the enum into "bytes" for inserting config into the sled tree.
impl AsRef<[u8]> for SerialiserMode {
    fn as_ref(&self) -> &[u8] {
        match self {
            #[cfg(feature = "bincode")]
            SerialiserMode::BINCODE => &[0u8],
        }
    }
}

pub struct SerSled {
    pub inner_tree: sled::Tree,
    ser_mode: SerialiserMode,
}

impl SerSled {
    /// Loads the tree without storing the serialiser used in the tree.
    /// It is up to you to remember/select the correct one if you want to read data.
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

    /// Loads the tree and attempts to load the serialiser mode stored in the tree.
    /// Otherwise, it will use `ser_mode` and store that in the tree configuration.
    pub fn new_from_config_or_else(
        sled_tree: sled::Tree,
        ser_mode: SerialiserMode,
    ) -> Result<Self, SerSledError> {
        let ser_config = match sled_tree.get(CONFIGUATION_TREE_KEY)? {
            Some(bytes) => {
                match bytes.first() {
                    // Bincode config
                    Some(0) => SerialiserMode::BINCODE,
                    // No readable config
                    Some(_) | None => {
                        // Found bytes, but couldn't read them
                        let _insert_config =
                            sled_tree.insert(CONFIGUATION_TREE_KEY, ser_mode.as_ref())?;

                        ser_mode
                    }
                }
            }
            None => {
                // No config found
                let _insert_config = sled_tree.insert(CONFIGUATION_TREE_KEY, ser_mode.as_ref())?;

                ser_mode
            }
        };

        Ok(SerSled {
            inner_tree: sled_tree,
            ser_mode: ser_config,
        })
    }

    pub fn get<K: Serialize, V: for<'de> Deserialize<'de>>(
        &self,
        key: &K,
    ) -> Result<Option<V>, SerSledError> {
        match self.ser_mode {
            #[cfg(feature = "bincode")]
            SerialiserMode::BINCODE => {
                let bytes = bincode::serde::encode_to_vec(key, BINCODE_CONFIG)?;
                match self.inner_tree.get(bytes)? {
                    Some(res_ivec) => {
                        let deser = bincode::serde::decode_borrowed_from_slice::<V, _>(
                            &res_ivec,
                            BINCODE_CONFIG,
                        )?;

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
                let key_bytes = bincode::serde::encode_to_vec(key, BINCODE_CONFIG)?;
                let value_bytes = bincode::serde::encode_to_vec(value, BINCODE_CONFIG)?;

                match self.inner_tree.insert(key_bytes, value_bytes)? {
                    Some(ivec) => {
                        let old_value = bincode::serde::decode_borrowed_from_slice::<V, _>(
                            &ivec,
                            BINCODE_CONFIG,
                        )?;

                        Ok(Some(old_value))
                    }
                    None => Ok(None),
                }
            }
        }
    }

    pub fn first<K: for<'de> Deserialize<'de>, V: for<'de> Deserialize<'de>>(
        &self,
    ) -> Result<Option<(K, V)>, SerSledError> {
        match self.ser_mode {
            #[cfg(feature = "bincode")]
            SerialiserMode::BINCODE => match self.inner_tree.first()? {
                Some((key_ivec, value_ivec)) => {
                    let key = bincode::serde::decode_borrowed_from_slice::<K, _>(
                        &key_ivec,
                        BINCODE_CONFIG,
                    )?;

                    let value = bincode::serde::decode_borrowed_from_slice::<V, _>(
                        &value_ivec,
                        BINCODE_CONFIG,
                    )?;

                    Ok(Some((key, value)))
                }
                None => Ok(None),
            },
        }
    }

    pub fn last<K: for<'de> Deserialize<'de>, V: for<'de> Deserialize<'de>>(
        &self,
    ) -> Result<Option<(K, V)>, SerSledError> {
        match self.ser_mode {
            #[cfg(feature = "bincode")]
            SerialiserMode::BINCODE => match self.inner_tree.last()? {
                Some((key_ivec, value_ivec)) => {
                    let key = bincode::serde::decode_borrowed_from_slice::<K, _>(
                        &key_ivec,
                        BINCODE_CONFIG,
                    )?;

                    let value = bincode::serde::decode_borrowed_from_slice::<V, _>(
                        &value_ivec,
                        BINCODE_CONFIG,
                    )?;

                    Ok(Some((key, value)))
                }
                None => Ok(None),
            },
        }
    }

    pub fn iter<K: for<'de> Deserialize<'de>, V: for<'de> Deserialize<'de>>(
        &self,
    ) -> impl Iterator<Item = (K, V)> {
        match self.ser_mode {
            #[cfg(feature = "bincode")]
            SerialiserMode::BINCODE => self
                .inner_tree
                .into_iter()
                .filter(|res| res.is_ok())
                .filter_map(|res| {
                    let (key_ivec, value_ivec) =
                        res.expect("previous filter checked that res is Ok()");

                    let key = bincode::serde::decode_borrowed_from_slice::<K, _>(
                        &key_ivec,
                        BINCODE_CONFIG,
                    )
                    .ok();

                    let value = bincode::serde::decode_borrowed_from_slice::<V, _>(
                        &value_ivec,
                        BINCODE_CONFIG,
                    )
                    .ok();

                    if key.is_some() && value.is_some() {
                        Some((key.expect("key is Some"), value.expect("value is Some")))
                    } else {
                        None
                    }
                }),
        }
    }
}
