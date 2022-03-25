use cosmwasm_std::Timestamp;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;
use cw_storage_plus::Item;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub withdraw_delay: u64,
    pub target_address: Addr,
}

pub const CONFIG: Item<Config> = Item::new("config");

pub const WITHDRAWAL_READY: Item<Timestamp> = Item::new("withdrawal_ready");
