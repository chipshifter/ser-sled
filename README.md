# ser-sled: A `sled` database wrapper for `Serialize`/`Deserialize` objects

*WARNING*: This crate is still a work-in-progress and its API is not complete or stabilised.

`sled` is a key-value store which has an API similar to `BTreeMap<[u8], [u8]>`.
This means that if you want to work with serializable objects, you have to convert
them into bytes yourself.

`ser-sled` is a basic wrapper around `sled` that aims to add an easy API which
does all this work for you. Serialization into bytes is done using `bincode`.

The structs must both derive `Serialize` and `Deserialize` for them to be used with
this crate.

There are two types of trees you can use:

- `StrictTree<K, V>`: The type-strict tree. If you know your tree will only contain 
specific types (`K` as the key, `V` as the value) and shouldn't contain anything else, then you should use this.
- `RelaxedTree`: The not-type-stript tree. You can store any type that you want. Typically,
you will have to specify that type you're trying to `get` or `insert` in the code.

Note that in both cases you cannot guarantee that the bytes stored in the database will result
in proper serialization/deserialization, and nothing prevents the user from having two instances of `StrictTree`
on the same tree but with different types.

## Example

```rust
// Initialise sled database
let db = sled::Config::new().temporary(true).open()?;
let ser_db: ser_sled::Db = db.into();


// Open "strict" tree
let tree = ser_db   // <key type, value type>
    .open_bincode_tree::<u16, Vec<u8>>("example")?;

// Or open "relaxed" tree
let tree = ser_db
    .open_relaxed_bincode_tree("example")?;

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