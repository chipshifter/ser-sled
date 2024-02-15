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

#[cfg(feature = "bincode")]
pub mod bincode_tree;
#[cfg(feature = "bincode")]
use bincode_tree::BincodeSledTree;
pub mod error;
pub mod tests;

pub const CONFIGUATION_TREE_KEY: &[u8] = b"_ser-sled_serialiser";

#[derive(Debug, Clone, Copy)]
pub enum SerialiserMode {
    #[cfg(feature = "bincode")]
    BINCODE,
}

impl AsRef<[u8]> for SerialiserMode {
    fn as_ref(&self) -> &[u8] {
        match self {
            #[cfg(feature = "bincode")]
            SerialiserMode::BINCODE => &[0u8],
        }
    }
}

#[derive(Clone)]
pub struct SerSledDb {
    inner_db: sled::Db,
    pub ser_mode: SerialiserMode,
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
                    #[cfg(feature = "bincode")]
                    Some(0) => SerialiserMode::BINCODE,
                    // No readable config
                    Some(_) | None => {
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

    pub fn open_tree_impl(&self, tree_name: &str) -> Result<impl SerSledTree, SerSledError> {
        let tree = self.inner_db.open_tree(tree_name)?;
        match self.ser_mode {
            SerialiserMode::BINCODE => Ok(BincodeSledTree::new(tree)),
        }
    }

    pub fn open_tree<T: SerSledTree>(&self, tree_name: &str) -> Result<T, SerSledError> {
        let tree = self.inner_db.open_tree(tree_name)?;

        Ok(T::new(tree))
    }
}

pub trait SerSledTree {
    fn new(tree: sled::Tree) -> Self;
    fn get<K: Serialize, V: for<'de> Deserialize<'de>>(
        &self,
        key: &K,
    ) -> Result<Option<V>, SerSledError>;
    fn get_or_init<F: FnOnce() -> T, K: Serialize, T: Serialize + for<'wa> Deserialize<'wa>>(
        &self,
        key: K,
        init_func: F,
    ) -> Result<Option<T>, SerSledError>;
    fn insert<K: Serialize, V: Serialize + for<'de> Deserialize<'de>>(
        &self,
        key: &K,
        value: &V,
    ) -> Result<Option<V>, SerSledError>;
    fn first<K: for<'de> Deserialize<'de>, V: for<'de> Deserialize<'de>>(
        &self,
    ) -> Result<Option<(K, V)>, SerSledError>;
    fn last<K: for<'de> Deserialize<'de>, V: for<'de> Deserialize<'de>>(
        &self,
    ) -> Result<Option<(K, V)>, SerSledError>;
    fn pop_max<K: for<'de> Deserialize<'de>, V: for<'de> Deserialize<'de>>(
        &self,
    ) -> Result<Option<(K, V)>, SerSledError>;
    fn iter<K: for<'de> Deserialize<'de>, V: for<'de> Deserialize<'de>>(
        &self,
    ) -> impl DoubleEndedIterator<Item = (K, V)>;
    fn range_key_bytes<K: AsRef<[u8]>, R: RangeBounds<K>, V: for<'de> Deserialize<'de>>(
        &self,
        range: R,
    ) -> impl Iterator<Item = (Vec<u8>, V)>;
    fn clear(&self) -> Result<(), SerSledError>;
    fn contains_key<K: Serialize>(&self, key: &K) -> Result<bool, SerSledError>;
    fn len(&self) -> usize;
    fn remove<K: Serialize, V: for<'de> Deserialize<'de>>(
        &self,
        key: &K,
    ) -> Result<Option<V>, SerSledError>;
}
