use std::ops::{Deref, DerefMut};

use heed::{
    byteorder::NativeEndian,
    types::{Bytes, U32},
    Database, Error, RwTxn,
};

use super::HeedEnv;

/// Address is the string version of either:
/// - mono
/// - multi (sorted and joined)
// type Key = &'static [u8];
type Key = Bytes;
type Value = U32<NativeEndian>;
type DB = Database<Key, Value>;

pub struct AddressToAddressIndex(DB);

impl AddressToAddressIndex {
    pub fn open(env: &HeedEnv, writer: &mut RwTxn) -> Result<Self, Error> {
        let db = env
            .create_database(writer, Some("address_index_to_address"))
            .unwrap();

        Ok(Self(db))
    }
}

impl Deref for AddressToAddressIndex {
    type Target = DB;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for AddressToAddressIndex {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
