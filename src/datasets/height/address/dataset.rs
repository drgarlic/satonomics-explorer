use std::fs;

use itertools::Itertools;
use ordered_float::OrderedFloat;

use crate::{
    bitcoin::{btc_to_sats, sats_to_btc},
    datasets::{
        height::{RealizedDataset, UTXOsMetadataDataset, UnrealizedDataset},
        AnyHeightDataset, ProcessedBlockData,
    },
    structs::{AnyHeightMap, HeightMap},
};

use super::{AddressFilter, LiquidityClassification};

pub struct AddressDataset {
    filter: AddressFilter,

    full_dataset: AddressSubDataset,
    illiquid_dataset: AddressSubDataset,
    liquid_dataset: AddressSubDataset,
    highly_liquid_dataset: AddressSubDataset,
}

impl AddressDataset {
    pub fn import(path: &str, filter: AddressFilter) -> color_eyre::Result<Self> {
        let f = |s: &str| format!("{path}/{s}");

        Ok(Self {
            filter,
            full_dataset: AddressSubDataset::import(&f("full"))?,
            illiquid_dataset: AddressSubDataset::import(&f("illiquid"))?,
            liquid_dataset: AddressSubDataset::import(&f("liquid"))?,
            highly_liquid_dataset: AddressSubDataset::import(&f("highly_liquid"))?,
        })
    }
}

impl AnyHeightDataset for AddressDataset {
    fn insert(
        &self,
        &ProcessedBlockData {
            states,
            price,
            height,
            address_index_to_address_realized_data,
            address_index_to_removed_address_data,
            ..
        }: &ProcessedBlockData,
    ) {
        let address_index_to_address_data = &states.address_index_to_address_data;

        let mut full_realized_profit = 0.0;
        let mut illiquid_realized_profit = 0.0;
        let mut liquid_realized_profit = 0.0;
        let mut highly_liquid_realized_profit = 0.0;

        let mut full_realized_loss = 0.0;
        let mut illiquid_realized_loss = 0.0;
        let mut liquid_realized_loss = 0.0;
        let mut highly_liquid_realized_loss = 0.0;

        address_index_to_address_realized_data
            .iter()
            .map(|(address_index, address_realized_data)| {
                let address_data = address_index_to_address_data
                    .get(address_index)
                    .unwrap_or_else(|| {
                        address_index_to_removed_address_data
                            .get(address_index)
                            .unwrap()
                    });

                let previous_amount = address_data.amount + address_realized_data.sent
                    - address_realized_data.received;

                (address_data, previous_amount, address_realized_data)
            })
            .filter(|(address_data, previous_amount, _)| {
                self.filter
                    .check(previous_amount, &address_data.address_type)
            })
            .for_each(|(address_data, _, address_realized_data)| {
                full_realized_profit += address_realized_data.profit;
                full_realized_loss += address_realized_data.loss;

                // Realized == previous amount
                // If a whale sent all its sats to another address at a loss, it's the whale that realized the loss not the empty adress
                let previous_sent = address_data.sent - address_realized_data.sent;
                let previous_received = address_data.received - address_realized_data.received;

                let split_profit = LiquidityClassification::new(previous_sent, previous_received)
                    .split(address_realized_data.profit as f64);

                illiquid_realized_profit += split_profit.illiquid as f32;
                liquid_realized_profit += split_profit.liquid as f32;
                highly_liquid_realized_profit += split_profit.highly_liquid as f32;

                let split_loss = LiquidityClassification::new(previous_sent, previous_received)
                    .split(address_realized_data.loss as f64);

                illiquid_realized_loss += split_loss.illiquid as f32;
                liquid_realized_loss += split_loss.liquid as f32;
                highly_liquid_realized_loss += split_loss.highly_liquid as f32;
            });

        let mut full_total_supply = 0;
        let mut illiquid_total_supply = 0;
        let mut liquid_total_supply = 0;
        let mut highly_liquid_total_supply = 0;

        let mut full_utxo_count = 0;
        let mut illiquid_utxo_count = 0;
        let mut liquid_utxo_count = 0;
        let mut highly_liquid_utxo_count = 0;

        let vec = address_index_to_address_data
            .values()
            .filter(|address_data| {
                self.filter
                    .check(&address_data.amount, &address_data.address_type)
            })
            .map(|address_data| {
                let amount = address_data.amount;
                let utxo_count = address_data.outputs_len as usize;
                let mean_price_paid = OrderedFloat(address_data.mean_price_paid);

                let liquidity_classification =
                    LiquidityClassification::new(address_data.sent, address_data.received);

                let split_utxo_count = liquidity_classification.split(utxo_count as f64);

                full_utxo_count += utxo_count;
                illiquid_utxo_count += split_utxo_count.illiquid.round() as usize;
                liquid_utxo_count += split_utxo_count.liquid.round() as usize;
                highly_liquid_utxo_count += split_utxo_count.highly_liquid.round() as usize;

                let split_amount = liquidity_classification.split(sats_to_btc(amount));

                let illiquid_sats = btc_to_sats(split_amount.illiquid);
                let liquid_sats = btc_to_sats(split_amount.liquid);
                let highly_liquid_sats = btc_to_sats(split_amount.highly_liquid);

                full_total_supply += amount;
                illiquid_total_supply += illiquid_sats;
                liquid_total_supply += liquid_sats;
                highly_liquid_total_supply += highly_liquid_sats;

                (
                    mean_price_paid,
                    (amount, illiquid_sats, liquid_sats, highly_liquid_sats),
                )
            })
            .sorted_unstable_by(|a, b| Ord::cmp(&a.0, &b.0))
            .collect_vec();

        let len = vec.len();

        self.full_dataset.insert(
            height,
            price,
            full_realized_loss,
            full_realized_profit,
            full_total_supply,
            full_utxo_count,
            len,
            vec.iter().map(|(price, (full, _, _, _))| (price, full)),
        );

        self.illiquid_dataset.insert(
            height,
            price,
            illiquid_realized_loss,
            illiquid_realized_profit,
            illiquid_total_supply,
            illiquid_utxo_count,
            len,
            vec.iter()
                .map(|(price, (_, illiquid, _, _))| (price, illiquid)),
        );

        self.liquid_dataset.insert(
            height,
            price,
            liquid_realized_loss,
            liquid_realized_profit,
            liquid_total_supply,
            liquid_utxo_count,
            len,
            vec.iter().map(|(price, (_, _, liquid, _))| (price, liquid)),
        );

        self.highly_liquid_dataset.insert(
            height,
            price,
            highly_liquid_realized_loss,
            highly_liquid_realized_profit,
            highly_liquid_total_supply,
            highly_liquid_utxo_count,
            len,
            vec.iter()
                .map(|(price, (_, _, _, highly_liquid))| (price, highly_liquid)),
        );
    }

    fn to_vec(&self) -> Vec<&(dyn AnyHeightMap + Send + Sync)> {
        [
            self.full_dataset.to_vec(),
            self.illiquid_dataset.to_vec(),
            self.liquid_dataset.to_vec(),
            self.highly_liquid_dataset.to_vec(),
        ]
        .into_iter()
        .flatten()
        .collect_vec()
    }
}

pub struct AddressSubDataset {
    price_point: PricePointDataset,
    realized: RealizedDataset,
    unrealized: UnrealizedDataset,
    utxos_metadata: UTXOsMetadataDataset,
    address_count: HeightMap<usize>,
}

impl AddressSubDataset {
    pub fn import(folder_path: &str) -> color_eyre::Result<Self> {
        fs::create_dir_all(folder_path)?;

        let f = |s: &str| format!("{folder_path}/{s}");

        Ok(Self {
            address_count: HeightMap::new(&f("address_count")),
            realized: RealizedDataset::import(&folder_path)?,
            unrealized: UnrealizedDataset::import(&folder_path)?,
            utxos_metadata: UTXOsMetadataDataset::import(&folder_path)?,
        })
    }

    pub fn insert<'a>(
        &self,
        height: usize,
        price: f32,
        realized_loss: f32,
        realized_profit: f32,
        total_supply: u64,
        utxo_count: usize,
        sorted_price_to_amount_len: usize,
        sorted_price_to_amount: impl Iterator<Item = (&'a OrderedFloat<f32>, &'a u64)>,
    ) {
        self.address_count
            .insert(height, sorted_price_to_amount_len);

        self.realized.insert(height, realized_loss, realized_profit);

        self.unrealized
            .insert(height, price, total_supply, sorted_price_to_amount);

        self.utxos_metadata.insert(height, utxo_count);
    }

    pub fn to_vec(&self) -> Vec<&(dyn AnyHeightMap + Send + Sync)> {
        [
            self.realized.to_vec(),
            self.unrealized.to_vec(),
            self.utxos_metadata.to_vec(),
            vec![&self.address_count],
        ]
        .into_iter()
        .flatten()
        .collect_vec()
    }
}
