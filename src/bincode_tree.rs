use std::{marker::PhantomData, ops::RangeBounds};

use serde::{Deserialize, Serialize};
use std::ops::Bound::{Excluded, Included, Unbounded};

use crate::{error::SerSledError, SerSledTree};

/// Sled is optimised to work with big-endian bytes
pub const BINCODE_CONFIG: bincode::config::Configuration<bincode::config::BigEndian> =
    bincode::config::standard().with_big_endian();

/// Type strict bincode tree.
#[derive(Clone)]
pub struct BincodeTree<
    K: Serialize + for<'de> Deserialize<'de>,
    V: Serialize + for<'de> Deserialize<'de>,
> {
    inner_tree: sled::Tree,
    key_type: PhantomData<K>,
    value_type: PhantomData<V>,
}

impl<K, V> SerSledTree for BincodeTree<K, V>
where
    K: Serialize + for<'de> Deserialize<'de>,
    V: Serialize + for<'de> Deserialize<'de>,
{
    type Key = K;
    type Value = V;

    fn new(tree: sled::Tree) -> Self {
        Self {
            inner_tree: tree,
            key_type: PhantomData,
            value_type: PhantomData,
        }
    }

    fn get(&self, key: &Self::Key) -> Result<Option<Self::Value>, SerSledError> {
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

    fn get_or_init<F: FnOnce() -> Self::Value>(
        &self,
        key: Self::Key,
        init_func: F,
    ) -> Result<Option<Self::Value>, SerSledError> {
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

    fn insert(
        &self,
        key: &Self::Key,
        value: &Self::Value,
    ) -> Result<Option<Self::Value>, SerSledError> {
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

    fn first(&self) -> Result<Option<(Self::Key, Self::Value)>, SerSledError> {
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

    fn last(&self) -> Result<Option<(Self::Key, Self::Value)>, SerSledError> {
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

    fn pop_max(&self) -> Result<Option<(Self::Key, Self::Value)>, SerSledError> {
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

    fn iter(&self) -> impl DoubleEndedIterator<Item = (Self::Key, Self::Value)> {
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

    fn range_key_bytes<KeyBytes: AsRef<[u8]>, R: RangeBounds<KeyBytes>>(
        &self,
        range: R,
    ) -> impl DoubleEndedIterator<Item = (Vec<u8>, Self::Value)> {
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

    fn range<R: RangeBounds<Self::Key>>(
        &self,
        range: R,
    ) -> Result<impl DoubleEndedIterator<Item = (Self::Key, Self::Value)>, SerSledError> {
        let start_bound_bytes = match range.start_bound() {
            Included(r) => Included(bincode::serde::encode_to_vec(r, BINCODE_CONFIG)?),
            Excluded(r) => Excluded(bincode::serde::encode_to_vec(r, BINCODE_CONFIG)?),
            Unbounded => Unbounded,
        };
        let end_bound_bytes = match range.end_bound() {
            Included(r) => Included(bincode::serde::encode_to_vec(r, BINCODE_CONFIG)?),
            Excluded(r) => Excluded(bincode::serde::encode_to_vec(r, BINCODE_CONFIG)?),
            Unbounded => Unbounded,
        };

        Ok(self
            .inner_tree
            .range((start_bound_bytes, end_bound_bytes))
            .filter_map(|res| match res {
                Ok((key_ivec, value_ivec)) => {
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
                }
                Err(_) => None,
            }))
    }

    fn clear(&self) -> Result<(), SerSledError> {
        Ok(self.inner_tree.clear()?)
    }

    fn contains_key(&self, key: &Self::Key) -> Result<bool, SerSledError> {
        let key_bytes = bincode::serde::encode_to_vec(key, BINCODE_CONFIG)?;

        Ok(self.inner_tree.contains_key(key_bytes)?)
    }

    fn len(&self) -> usize {
        self.inner_tree.len()
    }

    fn remove(&self, key: &Self::Key) -> Result<Option<Self::Value>, SerSledError> {
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
}