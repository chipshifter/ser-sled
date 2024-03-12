#[cfg(feature = "bincode")]
#[cfg(test)]
mod relaxed_bincode_tests {
    use crate::{RelaxedTree, SerSledDb};

    #[test]
    fn insert_and_get() {
        let db = sled::Config::new().temporary(true).open().unwrap();
        let ser_db: SerSledDb = db.into();
        let tree = ser_db
            .open_relaxed_bincode_tree("insert_and_get")
            .expect("tree should open");

        let bytes = vec![2, 3, 5, 7, 9, 11];
        tree.insert(b"wa", &bytes).unwrap();
        assert_eq!(tree.get(b"wa").unwrap(), Some(bytes.clone()));

        let same_tree = ser_db
            .open_relaxed_bincode_tree("insert_and_get")
            .expect("tree should open");

        let other_bytes = vec![2, 3, 11];
        assert_eq!(same_tree.insert(&b"wa", &other_bytes).unwrap(), Some(bytes));
    }

    #[test]
    fn get_or_init() {
        let db = sled::Config::new().temporary(true).open().unwrap();
        let ser_db: SerSledDb = db.into();
        let tree = ser_db
            .open_relaxed_bincode_tree("get_or_init")
            .expect("tree should open");

        let other_bytes = vec![2, 3, 11];
        assert_eq!(
            tree.get_or_init(b"angel".to_vec(), || { other_bytes.clone() })
                .unwrap(),
            Some(other_bytes)
        );
    }

    #[test]
    fn first_and_last() {
        let db = sled::Config::new().temporary(true).open().unwrap();
        let ser_db: SerSledDb = db.into();
        let tree = ser_db
            .open_relaxed_bincode_tree("first_and_last")
            .expect("tree should open");

        let bytes = vec![2, 3];
        let bytes_2 = vec![3, 3];

        tree.insert(&[1u8], &bytes).unwrap();
        tree.insert(&[2u8], &bytes_2).unwrap();

        assert_eq!(tree.first().unwrap(), Some(([1u8], bytes)));
        assert_eq!(tree.last().unwrap(), Some(([2u8], bytes_2)));
    }

    #[test]
    fn iter() {
        let db = sled::Config::new().temporary(true).open().unwrap();
        let ser_db: SerSledDb = db.into();
        let tree = ser_db
            .open_relaxed_bincode_tree("iter")
            .expect("tree should open");

        let bytes = vec![2, 3];
        let bytes_2 = vec![3, 3];

        tree.insert(&[1u8], &bytes).unwrap();
        tree.insert(&[2u8], &bytes_2).unwrap();

        let mut iter = tree.iter();
        assert_eq!(iter.next(), Some(([1u8], bytes.clone())));
        assert_eq!(iter.next(), Some(([2u8], bytes_2.clone())));
        assert_eq!(iter.next(), None);

        let mut iter = tree.iter().rev();
        assert_eq!(iter.next(), Some(([2u8], bytes_2)));
        assert_eq!(iter.next(), Some(([1u8], bytes)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn range_key_bytes() {
        let db = sled::Config::new().temporary(true).open().unwrap();

        let ser_db: SerSledDb = db.into();
        let tree = ser_db
            .open_relaxed_bincode_tree("range")
            .expect("tree should open");

        let bytes = vec![2, 3];
        let bytes_2 = vec![3, 3];

        tree.insert(&[1u8], &bytes).unwrap();
        tree.insert(&[2u8], &bytes_2).unwrap();

        let mut range = tree.range_key_bytes(..[2u8]);
        assert_eq!(range.next(), Some(([1u8].to_vec(), bytes.clone())));
        assert_eq!(range.next(), None);

        let mut range = tree.range_key_bytes([1u8]..);
        assert_eq!(range.next(), Some(([1u8].to_vec(), bytes)));
        assert_eq!(range.next(), Some(([2u8].to_vec(), bytes_2.clone())));
        assert_eq!(range.next(), None);

        let mut range = tree.range_key_bytes([2u8]..);
        assert_eq!(range.next(), Some(([2u8].to_vec(), bytes_2)));
        assert_eq!(range.next(), None);
    }

    #[test]
    fn range() {
        let db = sled::Config::new().temporary(true).open().unwrap();
        let ser_db: SerSledDb = db.into();
        let tree = ser_db
            .open_relaxed_bincode_tree("range")
            .expect("tree should open");

        let bytes = vec![2, 3];
        let bytes_2 = vec![3, 3];

        tree.insert(&1u64, &bytes).unwrap();
        tree.insert(&2u64, &bytes_2).unwrap();

        let mut range = tree.range(..2u64).expect("key should encode");
        assert_eq!(range.next(), Some((1u64, bytes.clone())));
        assert_eq!(range.next(), None);

        let mut range = tree.range(1u64..).expect("key should encode");
        assert_eq!(range.next(), Some((1u64, bytes)));
        assert_eq!(range.next(), Some((2u64, bytes_2.clone())));
        assert_eq!(range.next(), None);

        let mut range = tree.range(2u64..).expect("key should encode");
        assert_eq!(range.next(), Some((2u64, bytes_2)));
        assert_eq!(range.next(), None);
    }

    #[test]
    fn is_binary_order_preserved() {
        let db = sled::Config::new().temporary(true).open().unwrap();

        let ser_db: SerSledDb = db.into();
        let tree = ser_db
            .open_relaxed_bincode_tree("binary_order")
            .expect("tree should open");

        tree.insert(&[1u8], &[1u8]).unwrap();
        tree.insert(&[4u8], &[4u8]).unwrap();
        tree.insert(&[3u8], &[3u8]).unwrap();
        tree.insert(&[2u8], &[2u8]).unwrap();

        let mut iter = tree.iter();
        assert_eq!(iter.next(), Some(([1u8], [1u8])));
        assert_eq!(iter.next(), Some(([2u8], [2u8])));
        assert_eq!(iter.next(), Some(([3u8], [3u8])));
        assert_eq!(iter.next(), Some(([4u8], [4u8])));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn clear() {
        let db = sled::Config::new().temporary(true).open().unwrap();
        let ser_db: SerSledDb = db.into();
        let tree = ser_db
            .open_relaxed_bincode_tree("clear")
            .expect("tree should open");

        tree.insert(&[1u8], &[1u8]).unwrap();

        tree.clear().unwrap();

        assert!(tree.iter::<Vec<u8>, Vec<u8>>().next().is_none());
    }

    #[test]
    fn contains_key() {
        let db = sled::Config::new().temporary(true).open().unwrap();

        let ser_db: SerSledDb = db.into();
        let tree = ser_db
            .open_relaxed_bincode_tree("contains_key")
            .expect("tree should open");

        tree.insert(&[1u8], &[1u8]).unwrap();
        tree.insert(&[4u8], &[4u8]).unwrap();

        assert!(tree.contains_key(&[1u8]).unwrap());
        assert!(tree.contains_key(&[4u8]).unwrap());
        assert!(!tree.contains_key(&[2u8]).unwrap());
        assert!(!tree.contains_key(&[3u8]).unwrap());
    }

    #[test]
    fn pop_max() {
        let db = sled::Config::new().temporary(true).open().unwrap();
        let ser_db: SerSledDb = db.into();
        let tree = ser_db
            .open_relaxed_bincode_tree("pop_max")
            .expect("tree should open");

        tree.insert(&[1u8], &[1u8]).unwrap();
        tree.insert(&[4u8], &[4u8]).unwrap();
        tree.insert(&[2u8], &[2u8]).unwrap();

        assert_eq!(tree.pop_max().unwrap(), Some(([4u8], [4u8])));
    }

    #[test]
    fn remove() {
        let db = sled::Config::new().temporary(true).open().unwrap();
        let ser_db: SerSledDb = db.into();
        let tree = ser_db
            .open_relaxed_bincode_tree("remove")
            .expect("tree should open");

        tree.insert(&[1u8], &[1u8]).unwrap();
        tree.insert(&[4u8], &[4u8]).unwrap();
        tree.insert(&[3u8], &[3u8]).unwrap();
        tree.insert(&[2u8], &[2u8]).unwrap();

        assert_eq!(tree.remove(&[3u8]).unwrap(), Some([3u8]));

        let mut iter = tree.iter();
        assert_eq!(iter.next(), Some(([1u8], [1u8])));
        assert_eq!(iter.next(), Some(([2u8], [2u8])));
        assert_eq!(iter.next(), Some(([4u8], [4u8])));
        assert_eq!(iter.next(), None);
    }
}

#[cfg(feature = "bincode")]
#[cfg(test)]
mod strict_bincode_tests {
    use crate::{SerSledDb, SerSledTree};

    #[test]
    fn insert_and_get() {
        let db = sled::Config::new().temporary(true).open().unwrap();
        let ser_db: SerSledDb = db.into();
        let tree = ser_db
            .open_bincode_tree::<Vec<u8>, Vec<u8>>("insert_and_get")
            .expect("tree should open");

        let bytes = vec![2, 3, 5, 7, 9, 11];
        tree.insert(&vec![1], &bytes).unwrap();
        assert_eq!(tree.get(&vec![1]).unwrap(), Some(bytes.clone()));

        let same_tree = ser_db
            .open_bincode_tree::<Vec<u8>, Vec<u8>>("insert_and_get")
            .expect("tree should open");

        let other_bytes = vec![2, 3, 11];
        assert_eq!(
            same_tree.insert(&vec![1], &other_bytes).unwrap(),
            Some(bytes)
        );
    }

    #[test]
    fn get_or_init() {
        let db = sled::Config::new().temporary(true).open().unwrap();
        let ser_db: SerSledDb = db.into();
        let tree = ser_db
            .open_bincode_tree::<Vec<u8>, Vec<u8>>("get_or_init")
            .expect("tree should open");

        let other_bytes = vec![2, 3, 11];
        assert_eq!(
            tree.get_or_init(b"angel".to_vec(), || { other_bytes.clone() })
                .unwrap(),
            Some(other_bytes)
        );
    }

    #[test]
    fn first_and_last() {
        let db = sled::Config::new().temporary(true).open().unwrap();
        let ser_db: SerSledDb = db.into();
        let tree = ser_db
            .open_bincode_tree::<[u8; 1], Vec<u8>>("first_and_last")
            .expect("tree should open");

        let bytes = vec![2, 3];
        let bytes_2 = vec![3, 3];

        tree.insert(&[1u8], &bytes).unwrap();
        tree.insert(&[2u8], &bytes_2).unwrap();

        assert_eq!(tree.first().unwrap(), Some(([1u8], bytes)));
        assert_eq!(tree.last().unwrap(), Some(([2u8], bytes_2)));
    }

    #[test]
    fn iter() {
        let db = sled::Config::new().temporary(true).open().unwrap();
        let ser_db: SerSledDb = db.into();
        let tree = ser_db
            .open_bincode_tree::<[u8; 1], Vec<u8>>("iter")
            .expect("tree should open");

        let bytes = vec![2, 3];
        let bytes_2 = vec![3, 3];

        tree.insert(&[1u8], &bytes).unwrap();
        tree.insert(&[2u8], &bytes_2).unwrap();

        let mut iter = tree.iter();
        assert_eq!(iter.next(), Some(([1u8], bytes.clone())));
        assert_eq!(iter.next(), Some(([2u8], bytes_2.clone())));
        assert_eq!(iter.next(), None);

        let mut iter = tree.iter().rev();
        assert_eq!(iter.next(), Some(([2u8], bytes_2)));
        assert_eq!(iter.next(), Some(([1u8], bytes)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn range_key_bytes() {
        let db = sled::Config::new().temporary(true).open().unwrap();

        let ser_db: SerSledDb = db.into();
        let tree = ser_db
            .open_bincode_tree::<[u8; 1], Vec<u8>>("range")
            .expect("tree should open");

        let bytes = vec![2, 3];
        let bytes_2 = vec![3, 3];

        tree.insert(&[1u8], &bytes).unwrap();
        tree.insert(&[2u8], &bytes_2).unwrap();

        let mut range = tree.range_key_bytes(..[2u8]);
        assert_eq!(range.next(), Some(([1u8].to_vec(), bytes.clone())));
        assert_eq!(range.next(), None);

        let mut range = tree.range_key_bytes([1u8]..);
        assert_eq!(range.next(), Some(([1u8].to_vec(), bytes)));
        assert_eq!(range.next(), Some(([2u8].to_vec(), bytes_2.clone())));
        assert_eq!(range.next(), None);

        let mut range = tree.range_key_bytes([2u8]..);
        assert_eq!(range.next(), Some(([2u8].to_vec(), bytes_2)));
        assert_eq!(range.next(), None);
    }

    #[test]
    fn range() {
        let db = sled::Config::new().temporary(true).open().unwrap();
        let ser_db: SerSledDb = db.into();
        let tree = ser_db
            .open_bincode_tree::<u64, Vec<u8>>("range")
            .expect("tree should open");

        let bytes = vec![2, 3];
        let bytes_2 = vec![3, 3];

        tree.insert(&1u64, &bytes).unwrap();
        tree.insert(&2u64, &bytes_2).unwrap();

        let mut range = tree.range(..2u64).expect("key should encode");
        assert_eq!(range.next(), Some((1u64, bytes.clone())));
        assert_eq!(range.next(), None);

        let mut range = tree.range(1u64..).expect("key should encode");
        assert_eq!(range.next(), Some((1u64, bytes)));
        assert_eq!(range.next(), Some((2u64, bytes_2.clone())));
        assert_eq!(range.next(), None);

        let mut range = tree.range(2u64..).expect("key should encode");
        assert_eq!(range.next(), Some((2u64, bytes_2)));
        assert_eq!(range.next(), None);
    }

    #[test]
    fn is_binary_order_preserved() {
        let db = sled::Config::new().temporary(true).open().unwrap();

        let ser_db: SerSledDb = db.into();
        let tree = ser_db
            .open_bincode_tree::<[u8; 1], [u8; 1]>("binary_order")
            .expect("tree should open");

        tree.insert(&[1u8], &[1u8]).unwrap();
        tree.insert(&[4u8], &[4u8]).unwrap();
        tree.insert(&[3u8], &[3u8]).unwrap();
        tree.insert(&[2u8], &[2u8]).unwrap();

        let mut iter = tree.iter();
        assert_eq!(iter.next(), Some(([1u8], [1u8])));
        assert_eq!(iter.next(), Some(([2u8], [2u8])));
        assert_eq!(iter.next(), Some(([3u8], [3u8])));
        assert_eq!(iter.next(), Some(([4u8], [4u8])));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn clear() {
        let db = sled::Config::new().temporary(true).open().unwrap();
        let ser_db: SerSledDb = db.into();
        let tree = ser_db
            .open_bincode_tree::<[u8; 1], [u8; 1]>("clear")
            .expect("tree should open");

        tree.insert(&[1u8], &[1u8]).unwrap();

        tree.clear().unwrap();

        assert!(tree.iter().next().is_none());
    }

    #[test]
    fn contains_key() {
        let db = sled::Config::new().temporary(true).open().unwrap();

        let ser_db: SerSledDb = db.into();
        let tree = ser_db
            .open_bincode_tree::<[u8; 1], [u8; 1]>("contains_key")
            .expect("tree should open");

        tree.insert(&[1u8], &[1u8]).unwrap();
        tree.insert(&[4u8], &[4u8]).unwrap();

        assert!(tree.contains_key(&[1u8]).unwrap());
        assert!(tree.contains_key(&[4u8]).unwrap());
        assert!(!tree.contains_key(&[2u8]).unwrap());
        assert!(!tree.contains_key(&[3u8]).unwrap());
    }

    #[test]
    fn pop_max() {
        let db = sled::Config::new().temporary(true).open().unwrap();
        let ser_db: SerSledDb = db.into();
        let tree = ser_db
            .open_bincode_tree::<[u8; 1], [u8; 1]>("pop_max")
            .expect("tree should open");

        tree.insert(&[1u8], &[1u8]).unwrap();
        tree.insert(&[4u8], &[4u8]).unwrap();
        tree.insert(&[2u8], &[2u8]).unwrap();

        assert_eq!(tree.pop_max().unwrap(), Some(([4u8], [4u8])));
    }

    #[test]
    fn remove() {
        let db = sled::Config::new().temporary(true).open().unwrap();
        let ser_db: SerSledDb = db.into();
        let tree = ser_db
            .open_bincode_tree::<[u8; 1], [u8; 1]>("remove")
            .expect("tree should open");

        tree.insert(&[1u8], &[1u8]).unwrap();
        tree.insert(&[4u8], &[4u8]).unwrap();
        tree.insert(&[3u8], &[3u8]).unwrap();
        tree.insert(&[2u8], &[2u8]).unwrap();

        assert_eq!(tree.remove(&[3u8]).unwrap(), Some([3u8]));

        let mut iter = tree.iter();
        assert_eq!(iter.next(), Some(([1u8], [1u8])));
        assert_eq!(iter.next(), Some(([2u8], [2u8])));
        assert_eq!(iter.next(), Some(([4u8], [4u8])));
        assert_eq!(iter.next(), None);
    }
}
