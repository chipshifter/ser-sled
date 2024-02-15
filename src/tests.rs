#[cfg(feature = "bincode")]
#[cfg(test)]
mod bincode_tests {
    use sled::IVec;

    use crate::{SerSledDb, SerSledTree};

    #[test]
    fn insert_and_get() {
        let db = sled::Config::new().temporary(true).open().unwrap();
        let ser_db = SerSledDb::new_from_config_or_else(db, crate::SerialiserMode::BINCODE)
            .expect("db should open");
        let tree = ser_db
            .open_tree_impl("insert_and_get")
            .expect("tree should open");

        let bytes = vec![2, 3, 5, 7, 9, 11];
        tree.insert(b"w", &bytes).unwrap();
        assert_eq!(tree.get(b"w").unwrap(), Some(bytes.clone()));

        let same_tree = ser_db
            .open_tree_impl("insert_and_get")
            .expect("tree should open");

        let other_bytes = vec![2, 3, 11];
        assert_eq!(same_tree.insert(b"w", &other_bytes).unwrap(), Some(bytes));
    }

    #[test]
    fn get_or_init() {
        let db = sled::Config::new().temporary(true).open().unwrap();
        let ser_db = SerSledDb::new_from_config_or_else(db, crate::SerialiserMode::BINCODE)
            .expect("db should open");
        let tree = ser_db
            .open_tree_impl("get_or_init")
            .expect("tree should open");

        let other_bytes = vec![2, 3, 11];
        assert_eq!(
            tree.get_or_init(b"angel", || { other_bytes.clone() })
                .unwrap(),
            Some(other_bytes)
        );
    }

    #[test]
    fn first_and_last() {
        let db = sled::Config::new().temporary(true).open().unwrap();
        let ser_db = SerSledDb::new_from_config_or_else(db, crate::SerialiserMode::BINCODE)
            .expect("db should open");
        let tree = ser_db
            .open_tree_impl("first_and_last")
            .expect("tree should open");

        let bytes = vec![2, 3];
        let bytes_2 = vec![3, 3];

        tree.insert(&[1u8], &bytes).unwrap();
        tree.insert(&[2u8], &bytes_2).unwrap();

        assert_eq!(tree.first().unwrap(), Some(([1u8], bytes)));
        assert_eq!(tree.last().unwrap(), Some(([2u8], bytes_2)));
    }

    #[test]
    fn load_config() {
        use crate::CONFIGUATION_TREE_KEY;
        let db = sled::Config::new().temporary(true).open().unwrap();
        let ser_db = SerSledDb::new_from_config_or_else(db, crate::SerialiserMode::BINCODE)
            .expect("db should open");

        // Is config properly stored?
        assert_eq!(
            ser_db.inner_db.get(CONFIGUATION_TREE_KEY).unwrap(),
            Some(IVec::from(&[0]))
        );
    }

    #[test]
    fn iter() {
        let db = sled::Config::new().temporary(true).open().unwrap();
        let ser_db = SerSledDb::new_from_config_or_else(db, crate::SerialiserMode::BINCODE)
            .expect("db should open");
        let tree = ser_db.open_tree_impl("iter").expect("tree should open");

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
        let ser_db = SerSledDb::new_from_config_or_else(db, crate::SerialiserMode::BINCODE)
            .expect("db should open");
        let tree = ser_db.open_tree_impl("range").expect("tree should open");

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
    fn is_binary_order_preserved() {
        let db = sled::Config::new().temporary(true).open().unwrap();
        let ser_db = SerSledDb::new_from_config_or_else(db, crate::SerialiserMode::BINCODE)
            .expect("db should open");
        let tree = ser_db
            .open_tree_impl("is_binary_order_preserved")
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
        let ser_db = SerSledDb::new_from_config_or_else(db, crate::SerialiserMode::BINCODE)
            .expect("db should open");
        let tree = ser_db
            .open_tree_impl("is_binary_order_preserved")
            .expect("tree should open");

        tree.insert(&[1u8], &[1u8]).unwrap();
        tree.insert(&[4u8], &[4u8]).unwrap();
        tree.insert(&[3u8], &[3u8]).unwrap();
        tree.insert(&[2u8], &[2u8]).unwrap();

        tree.clear().unwrap();

        assert!(tree.iter::<[u8; 1], [u8; 1]>().next().is_none());
    }

    #[test]
    fn contains_key() {
        let db = sled::Config::new().temporary(true).open().unwrap();
        let ser_db = SerSledDb::new_from_config_or_else(db, crate::SerialiserMode::BINCODE)
            .expect("db should open");
        let tree = ser_db
            .open_tree_impl("is_binary_order_preserved")
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
        let ser_db = SerSledDb::new_from_config_or_else(db, crate::SerialiserMode::BINCODE)
            .expect("db should open");
        let tree = ser_db
            .open_tree_impl("is_binary_order_preserved")
            .expect("tree should open");

        tree.insert(&[1u8], &[1u8]).unwrap();
        tree.insert(&[4u8], &[4u8]).unwrap();
        tree.insert(&[2u8], &[2u8]).unwrap();

        assert_eq!(tree.pop_max().unwrap(), Some(([4u8], [4u8])));
    }

    #[test]
    fn remove() {
        let db = sled::Config::new().temporary(true).open().unwrap();
        let ser_db = SerSledDb::new_from_config_or_else(db, crate::SerialiserMode::BINCODE)
            .expect("db should open");
        let tree = ser_db
            .open_tree_impl("is_binary_order_preserved")
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
