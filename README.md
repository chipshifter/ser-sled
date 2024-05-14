# ser-sled: A `sled` database wrapper for `Serialize`/`Deserialize` objects

*WARNING*: This crate is still a work-in-progress and its API is not complete or stabilised.

`sled` is a key-value store which has an API similar to `BTreeMap<[u8], [u8]>`.
This means that if you want to work with it, you have to convert everything into bytes yourself.

`ser-sled` is a wrapper that aims to solve that problem, using the `bincode` serializer.
You can use this crate with `serde` to use `sled` with objects implementing `serde::Serialize`/`serde::Deserialize`.
It is also possible to not use `serde` and instead use `bincode::Encode`/`bincode::Decode`.


There are four types of trees you can use:

### With `serde`:
- `serde_tree::SerdeTree<K: Serialize + DeserializeOwned, V: Serialize + DeserializeOwned>`
- `serde_tree::RelaxedTree`

### With `bincode`:
- `bincode_tree::BincodeTree<K: Encode + Decode, V: Encode + Decode>`
- `bincode_tree::RelaxedTree`

## Difference between "relaxed" tree and regular tree

You cannot guarantee that the bytes stored in the database will result in proper serialization/deserialization. It is possible that for instance a `u64` was stored in the database tree at some point, but that you're attempting to deserialize it as a `String`.

While "relaxed" trees allow you to use any type you want with `get`, `insert`, etc., we also provide wrapper around the relaxed tree to enforce one type for the key, and one type for the value.

For instance, `SerdeTree<u64, String>` will only allow you to use `u64` as keys and `String` as values. Note that this is only a best effort attempt at type strictness: nothing prevents you from having two different instances of `SerdeTree` pointing to the same tree in the database itself. But this type strictness helps simplify the API and ensure that you're not accidentally serialising/deserializing an incorrect type.

The types are defined when creating the table. Both the key and the value must implement serializing AND deserializing.


## Example

```rust
// Initialise sled database
let db = sled::Config::new().temporary(true).open()?;
let ser_db: ser_sled::Db = db.into();


// Open "strict" tree
let tree = ser_db   // <key type, value type>
    .open_serde_tree::<u16, Vec<u8>>("example")?;

// Or open "relaxed" tree
let tree = ser_db
    .open_relaxed_serde_tree("example")?;

// Either way, the API is the exact same:
let value = vec![2, 3, 5, 7, 9, 11];
tree.insert(&1, &value);
assert_eq!(tree.get(&1)?, value);
```

## API

This crate has an API similar to `sled` but does not implement everything as of yet.

- [ ] `apply_batch`
- [ ] `checksum`
- [x] `clear`
- [ ] `compare_and_swap`
- [x] `contains_key`
- [ ] `fetch_and_update`
- [x] `first`
- [ ] `flush`
- [ ] `flush_async`
- [x] `get`
- [ ] `get_gt`
- [ ] `get_lt`
- [x] `insert`
- [ ] `is_empty`
- [x] `iter`
- [x] `last`
- [x] `len`
- [ ] `merge`
- [ ] `name`
- [x] `pop_max`
- [ ] `pop_min`
- [x] `range`
- [x] `remove`
- [ ] `scan_prefix`
- [ ] `transaction`
- [ ] `update_and_fetch`
- [ ] `watch_prefix`

#### Extra things

- [x] `get_or_init`
- [x] `range_key_bytes` if your want your key to be raw bytes