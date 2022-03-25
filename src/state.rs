use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Timestamp};
use cw_storage_plus::Item;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub withdraw_address: Addr,
    pub withdraw_delay_in_days: u64,
    pub native_denom: String,
}

pub const CONFIG: Item<Config> = Item::new("config");

pub const WITHDRAWAL_READY: Item<Timestamp> = Item::new("withdrawal_ready");
