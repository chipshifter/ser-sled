use std::ops::RangeBounds;

use serde::{Deserialize, Serialize};

use crate::{error::SerSledError, SerSledTree};

/// Sled is optimised to work with big-endian bytes
pub const BINCODE_CONFIG: bincode::config::Configuration<bincode::config::BigEndian> =
    bincode::config::standard().with_big_endian();

pub struct BincodeSledTree {
    inner_tree: sled::Tree,
}

impl SerSledTree for BincodeSledTree {
    fn new(sled_tree: sled::Tree) -> Self {
        Self {
            inner_tree: sled_tree,
        }
    }

    /// Retrieve value from table.
    fn get<K: Serialize, V: for<'de> Deserialize<'de>>(
        &self,
        key: &K,
    ) -> Result<Option<V>, SerSledError> {
        let bytes = bincode::serde::encode_to_vec(key, BINCODE_CONFIG)?;

        match self.inner_tree.get(bytes)? {
            Some(res_ivec) => {
                let deser =
                    bincode::serde::decode_borrowed_from_slice::<V, _>(&res_ivec, BINCODE_CONFIG)?;

                Ok(Some(deser))
            }
            None => Ok(None),
        }
    }

    /// Insert value into table.
    fn insert<K: Serialize, V: Serialize + for<'de> Deserialize<'de>>(
        &self,
        key: &K,
        value: &V,
    ) -> Result<Option<V>, SerSledError> {
        let key_bytes = bincode::serde::encode_to_vec(key, BINCODE_CONFIG)?;
        let value_bytes = bincode::serde::encode_to_vec(value, BINCODE_CONFIG)?;

        match self.inner_tree.insert(key_bytes, value_bytes)? {
            Some(ivec) => {
                let old_value =
                    bincode::serde::decode_borrowed_from_slice::<V, _>(&ivec, BINCODE_CONFIG)?;

                Ok(Some(old_value))
            }
            None => Ok(None),
        }
    }

    fn first<K: for<'de> Deserialize<'de>, V: for<'de> Deserialize<'de>>(
        &self,
    ) -> Result<Option<(K, V)>, SerSledError> {
        match self.inner_tree.first()? {
            Some((key_ivec, value_ivec)) => {
                let key =
                    bincode::serde::decode_borrowed_from_slice::<K, _>(&key_ivec, BINCODE_CONFIG)?;

                let value = bincode::serde::decode_borrowed_from_slice::<V, _>(
                    &value_ivec,
                    BINCODE_CONFIG,
                )?;

                Ok(Some((key, value)))
            }
            None => Ok(None),
        }
    }

    fn last<K: for<'de> Deserialize<'de>, V: for<'de> Deserialize<'de>>(
        &self,
    ) -> Result<Option<(K, V)>, SerSledError> {
        match self.inner_tree.last()? {
            Some((key_ivec, value_ivec)) => {
                let key =
                    bincode::serde::decode_borrowed_from_slice::<K, _>(&key_ivec, BINCODE_CONFIG)?;

                let value = bincode::serde::decode_borrowed_from_slice::<V, _>(
                    &value_ivec,
                    BINCODE_CONFIG,
                )?;

                Ok(Some((key, value)))
            }
            None => Ok(None),
        }
    }

    fn iter<K: for<'de> Deserialize<'de>, V: for<'de> Deserialize<'de>>(
        &self,
    ) -> impl Iterator<Item = (K, V)> {
        self.inner_tree
            .into_iter()
            .filter(|res| res.is_ok())
            .filter_map(|res| {
                let (key_ivec, value_ivec) = res.expect("previous filter checked that res is Ok()");

                let key =
                    bincode::serde::decode_borrowed_from_slice::<K, _>(&key_ivec, BINCODE_CONFIG)
                        .ok();

                let value =
                    bincode::serde::decode_borrowed_from_slice::<V, _>(&value_ivec, BINCODE_CONFIG)
                        .ok();

                if key.is_some() && value.is_some() {
                    Some((key.expect("key is Some"), value.expect("value is Some")))
                } else {
                    None
                }
            })
    }

    fn range_key_bytes<'a, K: AsRef<[u8]>, R: RangeBounds<K>, V: for<'de> Deserialize<'de>>(
        &self,
        range: R,
    ) -> impl Iterator<Item = (Vec<u8>, V)> {
        self.inner_tree
            .range(range)
            .filter(|res| res.is_ok())
            .filter_map(|res| {
                let (key_ivec, value_ivec) = res.expect("previous filter checked that res is Ok()");

                let key = key_ivec.to_vec();

                let value =
                    bincode::serde::decode_borrowed_from_slice::<V, _>(&value_ivec, BINCODE_CONFIG)
                        .ok();

                if value.is_some() {
                    Some((key, value.expect("value is Some")))
                } else {
                    None
                }
            })
    }
}
