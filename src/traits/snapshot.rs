use std::fmt::Debug;

use bincode::{
    config::{standard, Configuration},
    Decode, Encode,
};

use crate::utils::Binary;

pub const SNAPSHOTS_FOLDER: &str = "./snapshots";

// https://github.com/djkoloski/rust_serialization_benchmark
pub trait Snapshot
where
    Self: Encode + Decode + Debug,
{
    fn name<'a>() -> &'a str;

    fn format_path_str() -> String {
        let name = Self::name();

        format!("{SNAPSHOTS_FOLDER}/{name}.bin")
    }

    fn config() -> Configuration {
        standard()
    }

    fn import() -> color_eyre::Result<Self> {
        Binary::import(Self::format_path_str())
    }

    fn export(&self) -> color_eyre::Result<()> {
        Binary::export(Self::format_path_str(), self)
    }
}
