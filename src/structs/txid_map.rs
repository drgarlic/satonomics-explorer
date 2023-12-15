use std::{
    collections::BTreeMap,
    ops::{Deref, DerefMut},
};

use bincode::{
    de::{BorrowDecoder, Decoder},
    enc::Encoder,
    error::{DecodeError, EncodeError},
    BorrowDecode, Decode, Encode,
};
use bitcoin_explorer::Txid;
use bitcoin_hashes::{sha256d::Hash, Hash as HashTrait};

pub type TxidMap<T> = BTreeMap<WTxid, T>;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct WTxid(Txid);

impl WTxid {
    pub fn wrap(txid: Txid) -> Self {
        Self(txid)
    }

    fn from_slice(slice: Vec<u8>) -> Self {
        Self(Txid::from_hash(Hash::from_slice(&slice).unwrap()))
    }
}

impl Encode for WTxid {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        Encode::encode(&self.to_vec(), encoder)
    }
}

impl Decode for WTxid {
    fn decode<D: Decoder>(decoder: &mut D) -> core::result::Result<Self, DecodeError> {
        Ok(Self::from_slice(bincode::Decode::decode(decoder)?))
    }
}

impl<'de> BorrowDecode<'de> for WTxid {
    fn borrow_decode<D: BorrowDecoder<'de>>(decoder: &mut D) -> Result<Self, DecodeError> {
        Ok(Self::from_slice(bincode::Decode::decode(decoder)?))
    }
}

impl Deref for WTxid {
    type Target = Txid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for WTxid {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
