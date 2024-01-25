#[cfg(feature = "bincode")]
#[cfg(test)]
mod bincode_tests {
    use crate::SerSledDb;

    #[test]
    fn insert_and_get() {
        let db = sled::Config::new().temporary(true).open().unwrap();
        let ser_db = SerSledDb::new_from_config_or_else(db, crate::SerialiserMode::BINCODE)
            .expect("db should open");
        let tree = ser_db
            .open_tree("insert_and_get")
            .expect("tree should open");

        let bytes = vec![2, 3, 5, 7, 9, 11];
        tree.insert(b"w", &bytes).unwrap();
        assert_eq!(tree.get(b"w").unwrap(), Some(bytes));
    }

    #[test]
    fn first_and_last() {
        let db = sled::Config::new().temporary(true).open().unwrap();
        let ser_db = SerSledDb::new_from_config_or_else(db, crate::SerialiserMode::BINCODE)
            .expect("db should open");
        let tree = ser_db
            .open_tree("first_and_last")
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
        use sled::IVec;

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
        let tree = ser_db.open_tree("iter").expect("tree should open");

        let bytes = vec![2, 3];
        let bytes_2 = vec![3, 3];

        tree.insert(&[1u8], &bytes).unwrap();
        tree.insert(&[2u8], &bytes_2).unwrap();

        let mut iter = tree.iter();
        assert_eq!(iter.next(), Some(([1u8], bytes)));
        assert_eq!(iter.next(), Some(([2u8], bytes_2)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn range() {
        let db = sled::Config::new().temporary(true).open().unwrap();
        let ser_db = SerSledDb::new_from_config_or_else(db, crate::SerialiserMode::BINCODE)
            .expect("db should open");
        let tree = ser_db.open_tree("range").expect("tree should open");

        let bytes = vec![2, 3];
        let bytes_2 = vec![3, 3];

        tree.insert(&[1u8], &bytes).unwrap();
        tree.insert(&[2u8], &bytes_2).unwrap();

        let mut range = tree.range_key_bytes(..[2u8]);
        assert_eq!(range.next(), Some((vec![1], bytes.clone())));
        assert_eq!(range.next(), None);

        let mut range = tree.range_key_bytes([1u8]..);
        assert_eq!(range.next(), Some((vec![1], bytes)));
        assert_eq!(range.next(), Some((vec![2], bytes_2.clone())));
        assert_eq!(range.next(), None);

        let mut range = tree.range_key_bytes([2u8]..);
        assert_eq!(range.next(), Some((vec![2], bytes_2)));
        assert_eq!(range.next(), None);
    }
}
