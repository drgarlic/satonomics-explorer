use itertools::Itertools;

use crate::{
    datasets::AnyDataset,
    structs::{AnyHeightMap, BiMap},
};

use super::ProcessedBlockData;

pub struct RewardsDataset {
    pub fees_sumed: BiMap<u64>,
    pub subsidy: BiMap<u64>,
}

impl RewardsDataset {
    pub fn import(parent_path: &str) -> color_eyre::Result<Self> {
        let folder_path = format!("{parent_path}/rewards");
        let f = |s: &str| format!("{folder_path}/{s}");

        Ok(Self {
            fees_sumed: BiMap::new_on_disk_bin(&f("fees/sumed")),
            subsidy: BiMap::new_on_disk_bin(&f("subsidy")),
        })
    }
}

impl AnyDataset for RewardsDataset {
    fn insert_block_data(
        &self,
        &ProcessedBlockData {
            height,
            date,
            coinbase_vec,
            fees_vec,
            is_date_last_block,
            ..
        }: &ProcessedBlockData,
    ) {
        let coinbase = coinbase_vec.last().unwrap();
        let fees_sumed = fees_vec.last().unwrap().iter().sum();

        let subsidy = coinbase - fees_sumed;

        self.fees_sumed.height.insert(height, fees_sumed);
        self.subsidy.height.insert(height, subsidy);

        if is_date_last_block {
            let fees_sumed = fees_vec
                .iter()
                .map(|vec| vec.iter().sum::<u64>())
                .collect_vec();

            self.fees_sumed.date.insert(date, fees_sumed.iter().sum());

            self.subsidy.date.insert(
                date,
                coinbase_vec
                    .iter()
                    .enumerate()
                    .map(|(index, coinbase)| coinbase - fees_sumed.get(index).unwrap())
                    .sum(),
            )
        }
    }

    fn to_any_height_map_vec(&self) -> Vec<&(dyn AnyHeightMap + Send + Sync)> {
        vec![&self.fees_sumed.height, &self.subsidy.height]
    }

    fn to_any_date_map_vec(&self) -> Vec<&(dyn crate::structs::AnyDateMap + Send + Sync)> {
        vec![&self.fees_sumed.date, &self.subsidy.date]
    }
}