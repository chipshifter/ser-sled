#[cfg(feature = "bincode")]
mod bincode_tests {

    #[test]
    fn test_insert_and_get() {
        use crate::SerSled;
        use std::ops::Deref;

        let db = sled::Config::new().temporary(true).open().unwrap();
        let ser_db = SerSled::new(db.deref().clone(), crate::SerialiserMode::BINCODE);

        let bytes = vec![2, 3, 5, 7, 9, 11];
        ser_db.insert(b"w", &bytes).unwrap();
        assert_eq!(ser_db.get(b"w").unwrap(), Some(bytes));

        drop(ser_db);
        drop(db);
    }
}
