/// Copyright (C) 2024 Chipshifter
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
use error::Error;
use bincode::{Decode, Encode};
use bincode_tree::{BincodeTree, RelaxedTree};
#[cfg(feature = "serde")]
use serde::{de::DeserializeOwned, Serialize};
#[cfg(feature = "serde")]
use serde_tree::RelaxedBincodeSerdeTree;

/// Sled is optimised to work with big-endian bytes
/// See <https://github.com/spacejam/sled?tab=readme-ov-file#a-note-on-lexicographic-ordering-and-endianness>
pub const BINCODE_CONFIG: bincode::config::Configuration<bincode::config::BigEndian> =
    bincode::config::standard().with_big_endian();

use sled::IVec;
use std::ops::RangeBounds;

pub mod bincode_tree;
#[cfg(feature = "serde")]
pub mod serde_tree;
#[cfg(feature = "serde")]
use serde_tree::BincodeSerdeTree;
pub mod error;
pub mod tests;

impl From<sled::Db> for Db {
    fn from(value: sled::Db) -> Self {
        Self { inner_db: value }
    }
}

/// A wrapper for `T: Encode + Decode` to easily
/// convert into/from sled's `IVec` using `try_into()`/`try_from()`
#[derive(Encode, Decode)]
pub struct BincodeItem<T>(pub T);

impl<T: Encode + Decode> TryFrom<IVec> for BincodeItem<T> {
    type Error = error::BincodeError;

    fn try_from(value: IVec) -> Result<Self, Self::Error> {
        Ok(bincode::decode_from_slice(&value, BINCODE_CONFIG)?.0)
    }
}

impl<T: Encode + Decode> TryInto<IVec> for BincodeItem<T> {
    type Error = error::BincodeError;

    fn try_into(self) -> Result<IVec, Self::Error> {
        Ok(bincode::encode_to_vec(self.0, BINCODE_CONFIG)?.into())
    }
}

#[derive(Clone)]
pub struct Db {
    inner_db: sled::Db,
}

impl Db {
    pub fn open_relaxed_bincode_tree(&self, tree_name: &str) -> Result<RelaxedTree, Error> {
        let tree = self.inner_db.open_tree(tree_name)?;

        Ok(RelaxedTree::new(tree))
    }

    pub fn open_bincode_tree<K: Encode + Decode, V: Encode + Decode>(
        &self,
        tree_name: &str,
    ) -> Result<BincodeTree<K, V>, Error> {
        let tree = self.inner_db.open_tree(tree_name)?;

        Ok(BincodeTree::new(tree))
    }

    #[cfg(feature = "serde")]
    pub fn open_relaxed_serde_tree(
        &self,
        tree_name: &str,
    ) -> Result<RelaxedBincodeSerdeTree, Error> {
        let tree = self.inner_db.open_tree(tree_name)?;

        Ok(RelaxedBincodeSerdeTree::new(tree))
    }

    #[cfg(feature = "serde")]
    pub fn open_serde_tree<K: Serialize + DeserializeOwned, V: Serialize + DeserializeOwned>(
        &self,
        tree_name: &str,
    ) -> Result<BincodeSerdeTree<K, V>, Error> {
        let tree = self.inner_db.open_tree(tree_name)?;

        Ok(BincodeSerdeTree::new(tree))
    }
}

/// A type strict sled tree structure.
pub trait StrictTree<Key, Value> {
    fn new(tree: sled::Tree) -> Self;
    fn get(&self, key: &Key) -> Result<Option<Value>, Error>;
    fn get_or_init<F: FnOnce() -> Value>(
        &self,
        key: Key,
        init_func: F,
    ) -> Result<Option<Value>, Error>;
    fn insert(&self, key: &Key, value: &Value) -> Result<Option<Value>, Error>;
    fn first(&self) -> Result<Option<(Key, Value)>, Error>;
    fn last(&self) -> Result<Option<(Key, Value)>, Error>;
    fn pop_max(&self) -> Result<Option<(Key, Value)>, Error>;
    fn iter(&self) -> impl DoubleEndedIterator<Item = (Key, Value)>;
    fn range_key_bytes<K: AsRef<[u8]>, R: RangeBounds<K>>(
        &self,
        range: R,
    ) -> impl DoubleEndedIterator<Item = (Vec<u8>, Value)>;
    fn range<R: RangeBounds<Key>>(
        &self,
        range: R,
    ) -> Result<impl DoubleEndedIterator<Item = (Key, Value)>, Error>;
    fn clear(&self) -> Result<(), Error>;
    fn contains_key(&self, key: &Key) -> Result<bool, Error>;
    fn len(&self) -> usize;
    fn remove(&self, key: &Key) -> Result<Option<Value>, Error>;
}

/// A relaxed tree structure that allows any serde key or value type
/// as long as they implement `Serialize` and/or `Deserialize`.
/// This trait is not compatible with bincode's `Encode`/`Decode`.
#[cfg(feature = "serde")]
pub trait RelaxedSerdeTree {
    fn new(tree: sled::Tree) -> Self;
    fn get<K: Serialize, V: DeserializeOwned>(&self, key: &K) -> Result<Option<V>, Error>;
    fn get_or_init<F: FnOnce() -> T, K: Serialize, T: Serialize + DeserializeOwned>(
        &self,
        key: K,
        init_func: F,
    ) -> Result<Option<T>, Error>;
    fn insert<K: Serialize, V: Serialize + DeserializeOwned>(
        &self,
        key: &K,
        value: &V,
    ) -> Result<Option<V>, Error>;
    fn first<K: DeserializeOwned, V: DeserializeOwned>(&self) -> Result<Option<(K, V)>, Error>;
    fn last<K: DeserializeOwned, V: DeserializeOwned>(&self) -> Result<Option<(K, V)>, Error>;
    fn pop_max<K: DeserializeOwned, V: DeserializeOwned>(&self) -> Result<Option<(K, V)>, Error>;
    fn iter<K: DeserializeOwned, V: DeserializeOwned>(
        &self,
    ) -> impl DoubleEndedIterator<Item = (K, V)>;
    fn range_key_bytes<K: AsRef<[u8]>, R: RangeBounds<K>, V: DeserializeOwned>(
        &self,
        range: R,
    ) -> impl DoubleEndedIterator<Item = (Vec<u8>, V)>;
    fn range<K: Serialize + DeserializeOwned, R: RangeBounds<K>, V: DeserializeOwned>(
        &self,
        range: R,
    ) -> Result<impl DoubleEndedIterator<Item = (K, V)>, Error>;
    fn clear(&self) -> Result<(), Error>;
    fn contains_key<K: Serialize>(&self, key: &K) -> Result<bool, Error>;
    fn len(&self) -> usize;
    fn remove<K: Serialize, V: DeserializeOwned>(&self, key: &K) -> Result<Option<V>, Error>;
}

/// A relaxed tree structure that allows any bincode key or value type
/// as long as they implement `Encode` and/or `Decode`.
/// This trait is not compatible with serde's `Serialize`/`Deserialize`.
pub trait RelaxedBincodeTree {
    fn new(tree: sled::Tree) -> Self;
    fn get<K: Encode, V: Decode>(&self, key: &K) -> Result<Option<V>, Error>;
    fn get_or_init<F: FnOnce() -> T, K: Encode, T: Encode + Decode>(
        &self,
        key: K,
        init_func: F,
    ) -> Result<Option<T>, Error>;
    fn insert<K: Encode, V: Encode + Decode>(&self, key: &K, value: &V)
        -> Result<Option<V>, Error>;
    fn first<K: Decode, V: Decode>(&self) -> Result<Option<(K, V)>, Error>;
    fn last<K: Decode, V: Decode>(&self) -> Result<Option<(K, V)>, Error>;
    fn pop_max<K: Decode, V: Decode>(&self) -> Result<Option<(K, V)>, Error>;
    fn iter<K: Decode, V: Decode>(&self) -> impl DoubleEndedIterator<Item = (K, V)>;
    fn range_key_bytes<K: AsRef<[u8]>, R: RangeBounds<K>, V: Decode>(
        &self,
        range: R,
    ) -> impl DoubleEndedIterator<Item = (Vec<u8>, V)>;
    fn range<K: Encode + Decode, R: RangeBounds<K>, V: Decode>(
        &self,
        range: R,
    ) -> Result<impl DoubleEndedIterator<Item = (K, V)>, Error>;
    fn clear(&self) -> Result<(), Error>;
    fn contains_key<K: Encode>(&self, key: &K) -> Result<bool, Error>;
    fn len(&self) -> usize;
    fn remove<K: Encode, V: Decode>(&self, key: &K) -> Result<Option<V>, Error>;
}
