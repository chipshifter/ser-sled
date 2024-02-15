use std::ops::RangeBounds;

use serde::{Deserialize, Serialize};

use crate::{error::SerSledError, SerSledTree};

/// Sled is optimised to work with big-endian bytes
pub const BINCODE_CONFIG: bincode::config::Configuration<bincode::config::BigEndian> =
    bincode::config::standard().with_big_endian();

#[derive(Clone)]
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
    ) -> impl DoubleEndedIterator<Item = (K, V)> {
        self.inner_tree.into_iter().filter_map(|res| match res {
            Ok((key_ivec, value_ivec)) => {
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
            }
            Err(_) => None,
        })
    }

    fn range_key_bytes<K: AsRef<[u8]>, R: RangeBounds<K>, V: for<'de> Deserialize<'de>>(
        &self,
        range: R,
    ) -> impl DoubleEndedIterator<Item = (Vec<u8>, V)> {
        self.inner_tree.range(range).filter_map(|res| match res {
            Ok((key_ivec, value_ivec)) => {
                let key = key_ivec.to_vec();

                let value =
                    bincode::serde::decode_borrowed_from_slice::<V, _>(&value_ivec, BINCODE_CONFIG)
                        .ok();

                if value.is_some() {
                    Some((key, value.expect("value is Some")))
                } else {
                    None
                }
            }
            Err(_) => None,
        })
    }

    fn clear(&self) -> Result<(), SerSledError> {
        Ok(self.inner_tree.clear()?)
    }

    fn contains_key<K: Serialize>(&self, key: &K) -> Result<bool, SerSledError> {
        let key_bytes = bincode::serde::encode_to_vec(key, BINCODE_CONFIG)?;

        Ok(self.inner_tree.contains_key(key_bytes)?)
    }

    fn pop_max<K: for<'de> Deserialize<'de>, V: for<'de> Deserialize<'de>>(
        &self,
    ) -> Result<Option<(K, V)>, SerSledError> {
        match self.inner_tree.pop_max()? {
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

    fn len(&self) -> usize {
        self.inner_tree.len()
    }

    fn remove<K: Serialize, V: for<'de> Deserialize<'de>>(
        &self,
        key: &K,
    ) -> Result<Option<V>, SerSledError> {
        let bytes = bincode::serde::encode_to_vec(key, BINCODE_CONFIG)?;

        match self.inner_tree.remove(bytes)? {
            Some(res_ivec) => {
                let deser =
                    bincode::serde::decode_borrowed_from_slice::<V, _>(&res_ivec, BINCODE_CONFIG)?;

                Ok(Some(deser))
            }
            None => Ok(None),
        }
    }

    fn get_or_init<F: FnOnce() -> T, K: Serialize, T: Serialize + for<'wa> Deserialize<'wa>>(
        &self,
        key: K,
        init_func: F,
    ) -> Result<Option<T>, SerSledError> {
        let res = match self.get(&key)? {
            Some(v) => Some(v),
            None => {
                let value = init_func();
                let _ = self.insert(&key, &value)?;
                Some(value)
            }
        };

        Ok(res)
    }
}
