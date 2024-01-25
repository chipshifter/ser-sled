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

    #[test]
    fn test_first_and_last() {
        use crate::SerSled;
        use std::ops::Deref;

        let db = sled::Config::new().temporary(true).open().unwrap();
        let ser_db = SerSled::new(db.deref().clone(), crate::SerialiserMode::BINCODE);

        let bytes = vec![2, 3];
        let bytes_2 = vec![3, 3];

        ser_db.insert(&[1u8], &bytes).unwrap();
        ser_db.insert(&[2u8], &bytes_2).unwrap();

        assert_eq!(ser_db.first().unwrap(), Some(([1u8], bytes)));
        assert_eq!(ser_db.last().unwrap(), Some(([2u8], bytes_2)));

        drop(ser_db);
        drop(db);
    }
}