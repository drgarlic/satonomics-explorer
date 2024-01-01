use bincode::{Decode, Encode};
use derive_deref::{Deref, DerefMut};

use crate::traits::Snapshot;

#[derive(Encode, Decode, Default, Deref, DerefMut)]
pub struct AddressCounter(u32);

impl AddressCounter {
    pub fn increment(&mut self) {
        self.0 += 1;
    }
}

impl Snapshot for AddressCounter {
    fn name<'a>() -> &'a str {
        "address_counter"
    }
}