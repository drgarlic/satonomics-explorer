mod _trait;
mod address_index_to_address_data;
mod counters;
mod date_data_vec;
mod tx_index_to_tx_data;
mod txout_index_to_address_index;
mod txout_index_to_sats;

use std::thread;

use _trait::*;
use address_index_to_address_data::*;
use counters::*;
use date_data_vec::*;
use tx_index_to_tx_data::*;
use txout_index_to_address_index::*;
use txout_index_to_sats::*;

#[derive(Default)]
pub struct States {
    pub address_index_to_address_data: AddressIndexToAddressData,
    pub counters: Counters,
    pub date_data_vec: DateDataVec,
    pub tx_index_to_tx_data: TxIndexToTxData,
    pub txout_index_to_address_index: TxoutIndexToAddressIndex,
    pub txout_index_to_sats: TxoutIndexToSats,
}

impl States {
    pub fn import() -> color_eyre::Result<Self> {
        let address_index_to_address_data_handle = thread::spawn(AddressIndexToAddressData::import);

        let tx_index_to_tx_data_handle = thread::spawn(TxIndexToTxData::import);

        let txout_index_to_sats_handle = thread::spawn(TxoutIndexToSats::import);

        let txout_index_to_address_index_handle = thread::spawn(TxoutIndexToAddressIndex::import);

        let date_data_vec_handle = thread::spawn(DateDataVec::import);

        let counters = Counters::import()?;

        let date_data_vec = date_data_vec_handle.join().unwrap()?;

        let txout_index_to_address_index = txout_index_to_address_index_handle.join().unwrap()?;

        let txout_index_to_sats = txout_index_to_sats_handle.join().unwrap()?;

        let tx_index_to_tx_data = tx_index_to_tx_data_handle.join().unwrap()?;

        let address_index_to_address_data = address_index_to_address_data_handle.join().unwrap()?;

        Ok(Self {
            date_data_vec,
            counters,
            tx_index_to_tx_data,
            txout_index_to_address_index,
            txout_index_to_sats,
            address_index_to_address_data,
        })
    }

    pub fn reset(&mut self) -> color_eyre::Result<()> {
        self.address_index_to_address_data.reset()?;
        self.counters.reset()?;
        self.date_data_vec.reset()?;
        self.tx_index_to_tx_data.reset()?;
        self.txout_index_to_address_index.reset()?;
        self.txout_index_to_sats.reset()?;

        Ok(())
    }

    pub fn export(&self) -> color_eyre::Result<()> {
        thread::scope(|s| {
            s.spawn(|| self.address_index_to_address_data.export().unwrap());
            s.spawn(|| self.counters.export().unwrap());
            s.spawn(|| self.date_data_vec.export().unwrap());
            s.spawn(|| self.tx_index_to_tx_data.export().unwrap());
            s.spawn(|| self.txout_index_to_address_index.export().unwrap());
            s.spawn(|| self.txout_index_to_sats.export().unwrap());
        });

        Ok(())
    }
}