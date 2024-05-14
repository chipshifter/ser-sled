use bincode::{Decode, Encode};
use std::ops::Bound::{Excluded, Included, Unbounded};
use std::{marker::PhantomData, ops::RangeBounds};

use crate::BINCODE_CONFIG;
use crate::{error::Error, RelaxedBincodeTree, StrictTree};

/// A tree that allows you to pass any key or value as long as they
/// are serialisable and deserialisable.
/// This is NOT type strict, and as such can fail if you expect a different
/// structure than what is actually in the database. [`BincodeTree`] is recommended instead.
#[derive(Clone)]
pub struct RelaxedTree {
    inner_tree: sled::Tree,
}

/// Type strict bincode tree.
/// It is a wrapper of RelaxedBincodeTree, but with a type-strict property.
/// It is recommended to use this instead of [`RelaxedBincodeTree`] if
/// you don't plan on mixing different types in the same database tree.
/// While this should prevent type errors, it is only a best effort:
/// [`sled`] stores everything as bytes, and therefore it is never a guarantee
/// that the things stored in the tree are of the type you expect.
#[derive(Clone)]
pub struct BincodeTree<K: Encode + Decode, V: Encode + Decode> {
    inner_tree: RelaxedTree,
    key_type: PhantomData<K>,
    value_type: PhantomData<V>,
}

impl RelaxedBincodeTree for RelaxedTree {
    fn new(sled_tree: sled::Tree) -> Self {
        Self {
            inner_tree: sled_tree,
        }
    }

    /// Retrieve value from table.
    fn get<K: Encode, V: Decode>(&self, key: &K) -> Result<Option<V>, Error> {
        let bytes = bincode::encode_to_vec(key, BINCODE_CONFIG)?;

        match self.inner_tree.get(bytes)? {
            Some(res_ivec) => {
                let (deser, _size) = bincode::decode_from_slice::<V, _>(&res_ivec, BINCODE_CONFIG)?;

                Ok(Some(deser))
            }
            None => Ok(None),
        }
    }

    /// Insert value into table.
    fn insert<K: Encode, V: Encode + Decode>(
        &self,
        key: &K,
        value: &V,
    ) -> Result<Option<V>, Error> {
        let key_bytes = bincode::encode_to_vec(key, BINCODE_CONFIG)?;
        let value_bytes = bincode::encode_to_vec(value, BINCODE_CONFIG)?;

        match self.inner_tree.insert(key_bytes, value_bytes)? {
            Some(ivec) => {
                let (old_value, _size) = bincode::decode_from_slice::<V, _>(&ivec, BINCODE_CONFIG)?;

                Ok(Some(old_value))
            }
            None => Ok(None),
        }
    }

    fn first<K: Decode, V: Decode>(&self) -> Result<Option<(K, V)>, Error> {
        match self.inner_tree.first()? {
            Some((key_ivec, value_ivec)) => {
                let (key, _size) = bincode::decode_from_slice::<K, _>(&key_ivec, BINCODE_CONFIG)?;

                let (value, _size) =
                    bincode::decode_from_slice::<V, _>(&value_ivec, BINCODE_CONFIG)?;

                Ok(Some((key, value)))
            }
            None => Ok(None),
        }
    }

    fn last<K: Decode, V: Decode>(&self) -> Result<Option<(K, V)>, Error> {
        match self.inner_tree.last()? {
            Some((key_ivec, value_ivec)) => {
                let (key, _size) = bincode::decode_from_slice::<K, _>(&key_ivec, BINCODE_CONFIG)?;

                let (value, _size) =
                    bincode::decode_from_slice::<V, _>(&value_ivec, BINCODE_CONFIG)?;

                Ok(Some((key, value)))
            }
            None => Ok(None),
        }
    }

    fn iter<K: Decode, V: Decode>(&self) -> impl DoubleEndedIterator<Item = (K, V)> {
        self.inner_tree.into_iter().filter_map(|res| match res {
            Ok((key_ivec, value_ivec)) => {
                let key = bincode::decode_from_slice::<K, _>(&key_ivec, BINCODE_CONFIG).ok();

                let value = bincode::decode_from_slice::<V, _>(&value_ivec, BINCODE_CONFIG).ok();

                if let Some((key, _size)) = key {
                    if let Some((value, _size)) = value {
                        return Some((key, value));
                    }
                }

                return None;
            }
            Err(_) => None,
        })
    }

    fn range_key_bytes<K: AsRef<[u8]>, R: RangeBounds<K>, V: Decode>(
        &self,
        range: R,
    ) -> impl DoubleEndedIterator<Item = (Vec<u8>, V)> {
        self.inner_tree.range(range).filter_map(|res| match res {
            Ok((key_ivec, value_ivec)) => {
                let key = key_ivec.to_vec();

                let value = bincode::decode_from_slice::<V, _>(&value_ivec, BINCODE_CONFIG).ok();

                if let Some((value, _)) = value {
                    Some((key, value))
                } else {
                    None
                }
            }
            Err(_) => None,
        })
    }

    fn clear(&self) -> Result<(), Error> {
        Ok(self.inner_tree.clear()?)
    }

    fn contains_key<K: Encode>(&self, key: &K) -> Result<bool, Error> {
        let key_bytes = bincode::encode_to_vec(key, BINCODE_CONFIG)?;

        Ok(self.inner_tree.contains_key(key_bytes)?)
    }

    fn pop_max<K: Decode, V: Decode>(&self) -> Result<Option<(K, V)>, Error> {
        match self.inner_tree.pop_max()? {
            Some((key_ivec, value_ivec)) => {
                let (key, _size) = bincode::decode_from_slice::<K, _>(&key_ivec, BINCODE_CONFIG)?;

                let (value, _size) =
                    bincode::decode_from_slice::<V, _>(&value_ivec, BINCODE_CONFIG)?;

                Ok(Some((key, value)))
            }
            None => Ok(None),
        }
    }

    fn len(&self) -> usize {
        self.inner_tree.len()
    }

    fn remove<K: Encode, V: Decode>(&self, key: &K) -> Result<Option<V>, Error> {
        let bytes = bincode::encode_to_vec(key, BINCODE_CONFIG)?;

        match self.inner_tree.remove(bytes)? {
            Some(res_ivec) => {
                let (deser, _size) = bincode::decode_from_slice::<V, _>(&res_ivec, BINCODE_CONFIG)?;

                Ok(Some(deser))
            }
            None => Ok(None),
        }
    }

    fn get_or_init<F: FnOnce() -> T, K: Encode, T: Encode + Decode>(
        &self,
        key: K,
        init_func: F,
    ) -> Result<Option<T>, Error> {
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

    fn range<K: Encode + Decode, R: RangeBounds<K>, V: Decode>(
        &self,
        range: R,
    ) -> Result<impl DoubleEndedIterator<Item = (K, V)>, Error> {
        let start_bound_bytes = match range.start_bound() {
            Included(r) => Included(bincode::encode_to_vec(r, BINCODE_CONFIG)?),
            Excluded(r) => Excluded(bincode::encode_to_vec(r, BINCODE_CONFIG)?),
            Unbounded => Unbounded,
        };
        let end_bound_bytes = match range.end_bound() {
            Included(r) => Included(bincode::encode_to_vec(r, BINCODE_CONFIG)?),
            Excluded(r) => Excluded(bincode::encode_to_vec(r, BINCODE_CONFIG)?),
            Unbounded => Unbounded,
        };

        Ok(self
            .inner_tree
            .range((start_bound_bytes, end_bound_bytes))
            .filter_map(|res| match res {
                Ok((key_ivec, value_ivec)) => {
                    let key = bincode::decode_from_slice::<K, _>(&key_ivec, BINCODE_CONFIG).ok();

                    let value =
                        bincode::decode_from_slice::<V, _>(&value_ivec, BINCODE_CONFIG).ok();

                    if let Some((key, _size)) = key {
                        if let Some((value, _size)) = value {
                            return Some((key, value));
                        }
                    }

                    return None;
                }
                Err(_) => None,
            }))
    }
}

impl<KeyItem, ValueItem> StrictTree<KeyItem, ValueItem> for BincodeTree<KeyItem, ValueItem>
where
    KeyItem: Encode + Decode,
    ValueItem: Encode + Decode,
{
    fn new(tree: sled::Tree) -> Self {
        Self {
            inner_tree: RelaxedBincodeTree::new(tree),
            key_type: PhantomData,
            value_type: PhantomData,
        }
    }

    fn get(&self, key: &KeyItem) -> Result<Option<ValueItem>, Error> {
        self.inner_tree.get(key)
    }

    fn get_or_init<F: FnOnce() -> ValueItem>(
        &self,
        key: KeyItem,
        init_func: F,
    ) -> Result<Option<ValueItem>, Error> {
        self.inner_tree.get_or_init(key, init_func)
    }

    fn insert(&self, key: &KeyItem, value: &ValueItem) -> Result<Option<ValueItem>, Error> {
        self.inner_tree.insert(key, value)
    }

    fn first(&self) -> Result<Option<(KeyItem, ValueItem)>, Error> {
        self.inner_tree.first()
    }

    fn last(&self) -> Result<Option<(KeyItem, ValueItem)>, Error> {
        self.inner_tree.last()
    }

    fn pop_max(&self) -> Result<Option<(KeyItem, ValueItem)>, Error> {
        self.inner_tree.pop_max()
    }

    fn iter(&self) -> impl DoubleEndedIterator<Item = (KeyItem, ValueItem)> {
        self.inner_tree.iter()
    }

    fn range_key_bytes<KeyBytes: AsRef<[u8]>, R: RangeBounds<KeyBytes>>(
        &self,
        range: R,
    ) -> impl DoubleEndedIterator<Item = (Vec<u8>, ValueItem)> {
        self.inner_tree.range_key_bytes(range)
    }

    fn range<R: RangeBounds<KeyItem>>(
        &self,
        range: R,
    ) -> Result<impl DoubleEndedIterator<Item = (KeyItem, ValueItem)>, Error> {
        self.inner_tree.range(range)
    }

    fn clear(&self) -> Result<(), Error> {
        self.inner_tree.clear()
    }

    fn contains_key(&self, key: &KeyItem) -> Result<bool, Error> {
        self.inner_tree.contains_key(key)
    }

    fn len(&self) -> usize {
        self.inner_tree.len()
    }

    fn remove(&self, key: &KeyItem) -> Result<Option<ValueItem>, Error> {
        self.inner_tree.remove(key)
    }
}
