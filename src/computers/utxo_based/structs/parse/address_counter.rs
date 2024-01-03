use bincode::{Decode, Encode};
use derive_deref::{Deref, DerefMut};

use crate::{structs::Counter, traits::Snapshot};

#[derive(Encode, Decode, Default, Deref, DerefMut, Debug)]
pub struct AddressCounter(Counter);

impl Snapshot for AddressCounter {
    fn name<'a>() -> &'a str {
        "address_counter"
    }
}
