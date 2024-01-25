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
use std::ops::RangeBounds;

pub mod error;
pub mod tests;

/// Sled is optimised to work with big-endian bytes
#[cfg(feature = "bincode")]
pub const BINCODE_CONFIG: bincode::config::Configuration<bincode::config::BigEndian> =
    bincode::config::standard().with_big_endian();
pub const CONFIGUATION_TREE_KEY: &[u8] = b"_ser-sled_serialiser";

#[derive(Debug, Clone, Copy)]
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

pub struct SerSledDb {
    inner_db: sled::Db,
    ser_mode: SerialiserMode,
}

impl SerSledDb {
    /// Loads the tree and attempts to load the serialiser mode stored in the tree.
    /// Otherwise, it will use `ser_mode` and store that in the tree configuration.
    pub fn new_from_config_or_else(
        sled_db: sled::Db,
        ser_mode: SerialiserMode,
    ) -> Result<Self, SerSledError> {
        let ser_config = match sled_db.get(CONFIGUATION_TREE_KEY)? {
            Some(bytes) => {
                match bytes.first() {
                    // Bincode config
                    Some(0) => SerialiserMode::BINCODE,
                    // No readable config
                    Some(_) | None => {
                        // Found bytes, but couldn't read them
                        let _insert_config =
                            sled_db.insert(CONFIGUATION_TREE_KEY, ser_mode.as_ref())?;

                        ser_mode
                    }
                }
            }
            None => {
                // No config found
                let _insert_config = sled_db.insert(CONFIGUATION_TREE_KEY, ser_mode.as_ref())?;

                ser_mode
            }
        };

        Ok(Self {
            inner_db: sled_db,
            ser_mode: ser_config,
        })
    }

    pub fn open_tree(&self, tree_name: &str) -> Result<SerSledTree, SerSledError> {
        let tree = self.inner_db.open_tree(tree_name)?;

        Ok(SerSledTree {
            inner_tree: tree,
            ser_mode: self.ser_mode,
        })
    }
}

pub struct SerSledTree {
    pub inner_tree: sled::Tree,
    ser_mode: SerialiserMode,
}

impl SerSledTree {
    /// Loads the tree without storing the serialiser used in the tree.
    /// It is up to you to remember/select the correct one if you want to read data.
    pub fn new(
        // Also works with sled::Db since it implements Deref<sled::Tree>
        sled_tree: sled::Tree,
        ser_mode: SerialiserMode,
    ) -> Self {
        Self {
            inner_tree: sled_tree,
            ser_mode,
        }
    }

    /// Retrieve value from table.
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

    /// Retrieve value from table using raw key bytes.
    pub fn get_key_bytes<V: for<'de> Deserialize<'de>>(
        &self,
        key: &[u8],
    ) -> Result<Option<V>, SerSledError> {
        match self.inner_tree.get(key)? {
            Some(res_ivec) => match self.ser_mode {
                #[cfg(feature = "bincode")]
                SerialiserMode::BINCODE => {
                    let deser = bincode::serde::decode_borrowed_from_slice::<V, _>(
                        &res_ivec,
                        BINCODE_CONFIG,
                    )?;

                    Ok(Some(deser))
                }
            },
            None => Ok(None),
        }
    }

    /// Insert value into table.
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

    /// Insert value into table using raw key bytes.
    pub fn insert_key_bytes<V: Serialize + for<'de> Deserialize<'de>>(
        &self,
        key: &[u8],
        value: &V,
    ) -> Result<Option<V>, SerSledError> {
        match self.ser_mode {
            #[cfg(feature = "bincode")]
            SerialiserMode::BINCODE => {
                let value_bytes = bincode::serde::encode_to_vec(value, BINCODE_CONFIG)?;

                match self.inner_tree.insert(key, value_bytes)? {
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

    pub fn range_key_bytes<'a, K: AsRef<[u8]>, R: RangeBounds<K>, V: for<'de> Deserialize<'de>>(
        &self,
        range: R,
    ) -> impl Iterator<Item = (Vec<u8>, V)> {
        match self.ser_mode {
            #[cfg(feature = "bincode")]
            SerialiserMode::BINCODE => self
                .inner_tree
                .range(range)
                .filter(|res| res.is_ok())
                .filter_map(|res| {
                    let (key_ivec, value_ivec) =
                        res.expect("previous filter checked that res is Ok()");

                    let key = key_ivec.to_vec();

                    let value = bincode::serde::decode_borrowed_from_slice::<V, _>(
                        &value_ivec,
                        BINCODE_CONFIG,
                    )
                    .ok();

                    if value.is_some() {
                        Some((key, value.expect("value is Some")))
                    } else {
                        None
                    }
                }),
        }
    }
}
