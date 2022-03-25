use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;
use cw_storage_plus::Item;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub withdraw_delay: i32,
    pub admin_address: Addr,
}

pub const CONFIG: Item<Config> = Item::new("config");

// wrong type for now, needs timestamp
pub const WITHDRAWAL_READY: Item<String> = Item::new("withdrawal_ready");
