use bincode_tree::RelaxedBincodeTree;
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
use serde::{Deserialize, Serialize};
use std::ops::RangeBounds;

#[cfg(feature = "bincode")]
pub mod bincode_tree;
#[cfg(feature = "bincode")]
use bincode_tree::BincodeTree;
pub mod error;
pub mod tests;

impl From<sled::Db> for Db {
    fn from(value: sled::Db) -> Self {
        Self { inner_db: value }
    }
}

#[derive(Clone)]
pub struct Db {
    inner_db: sled::Db,
}

impl Db {
    #[cfg(feature = "bincode")]
    pub fn open_relaxed_bincode_tree(
        &self,
        tree_name: &str,
    ) -> Result<RelaxedBincodeTree, Error> {
        let tree = self.inner_db.open_tree(tree_name)?;

        Ok(RelaxedBincodeTree::new(tree))
    }

    #[cfg(feature = "bincode")]
    pub fn open_bincode_tree<
        K: Serialize + for<'de> Deserialize<'de>,
        V: Serialize + for<'de> Deserialize<'de>,
    >(
        &self,
        tree_name: &str,
    ) -> Result<BincodeTree<K, V>, Error> {
        let tree = self.inner_db.open_tree(tree_name)?;

        Ok(BincodeTree::new(tree))
    }
}

/// A type strict sled tree structure.
pub trait StrictTree {
    type Key: Serialize + for<'de> Deserialize<'de>;
    type Value: Serialize + for<'de> Deserialize<'de>;

    fn new(tree: sled::Tree) -> Self;
    fn get(&self, key: &Self::Key) -> Result<Option<Self::Value>, Error>;
    fn get_or_init<F: FnOnce() -> Self::Value>(
        &self,
        key: Self::Key,
        init_func: F,
    ) -> Result<Option<Self::Value>, Error>;
    fn insert(
        &self,
        key: &Self::Key,
        value: &Self::Value,
    ) -> Result<Option<Self::Value>, Error>;
    fn first(&self) -> Result<Option<(Self::Key, Self::Value)>, Error>;
    fn last(&self) -> Result<Option<(Self::Key, Self::Value)>, Error>;
    fn pop_max(&self) -> Result<Option<(Self::Key, Self::Value)>, Error>;
    fn iter(&self) -> impl DoubleEndedIterator<Item = (Self::Key, Self::Value)>;
    fn range_key_bytes<K: AsRef<[u8]>, R: RangeBounds<K>>(
        &self,
        range: R,
    ) -> impl DoubleEndedIterator<Item = (Vec<u8>, Self::Value)>;
    fn range<R: RangeBounds<Self::Key>>(
        &self,
        range: R,
    ) -> Result<impl DoubleEndedIterator<Item = (Self::Key, Self::Value)>, Error>;
    fn clear(&self) -> Result<(), Error>;
    fn contains_key(&self, key: &Self::Key) -> Result<bool, Error>;
    fn len(&self) -> usize;
    fn remove(&self, key: &Self::Key) -> Result<Option<Self::Value>, Error>;
}

pub trait RelaxedTree {
    fn new(tree: sled::Tree) -> Self;
    fn get<K: Serialize, V: for<'de> Deserialize<'de>>(
        &self,
        key: &K,
    ) -> Result<Option<V>, Error>;
    fn get_or_init<F: FnOnce() -> T, K: Serialize, T: Serialize + for<'wa> Deserialize<'wa>>(
        &self,
        key: K,
        init_func: F,
    ) -> Result<Option<T>, Error>;
    fn insert<K: Serialize, V: Serialize + for<'de> Deserialize<'de>>(
        &self,
        key: &K,
        value: &V,
    ) -> Result<Option<V>, Error>;
    fn first<K: for<'de> Deserialize<'de>, V: for<'de> Deserialize<'de>>(
        &self,
    ) -> Result<Option<(K, V)>, Error>;
    fn last<K: for<'de> Deserialize<'de>, V: for<'de> Deserialize<'de>>(
        &self,
    ) -> Result<Option<(K, V)>, Error>;
    fn pop_max<K: for<'de> Deserialize<'de>, V: for<'de> Deserialize<'de>>(
        &self,
    ) -> Result<Option<(K, V)>, Error>;
    fn iter<K: for<'de> Deserialize<'de>, V: for<'de> Deserialize<'de>>(
        &self,
    ) -> impl DoubleEndedIterator<Item = (K, V)>;
    fn range_key_bytes<K: AsRef<[u8]>, R: RangeBounds<K>, V: for<'de> Deserialize<'de>>(
        &self,
        range: R,
    ) -> impl DoubleEndedIterator<Item = (Vec<u8>, V)>;
    fn range<
        K: Serialize + for<'de> Deserialize<'de>,
        R: RangeBounds<K>,
        V: for<'de> Deserialize<'de>,
    >(
        &self,
        range: R,
    ) -> Result<impl DoubleEndedIterator<Item = (K, V)>, Error>;
    fn clear(&self) -> Result<(), Error>;
    fn contains_key<K: Serialize>(&self, key: &K) -> Result<bool, Error>;
    fn len(&self) -> usize;
    fn remove<K: Serialize, V: for<'de> Deserialize<'de>>(
        &self,
        key: &K,
    ) -> Result<Option<V>, Error>;
}
