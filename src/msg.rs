use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Basic configuration for the contract
/// The contract will have no admin so this will need to be set correctly
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub admin_address: String, // CCN
    pub withdraw_delay: i32,   // Withdraw delay in days
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Can be run by the admin_address
    /// Starts the withdraw process and creates a timestamp
    /// of when the funds will be ready for claim
    StartWithdraw {},
    /// When the funds are ready to be claimed,
    /// this allows them to actually be claimed
    ExecuteWithdraw {},
    /// This should only be sudo-callable by the governance
    /// module of the chain.
    /// Executes an immediate burn of any funds held by the contract
    ExecuteBurn {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// This returns the configured contract info
    GetConfig {},
    /// If a withdrawal has been initiated, this gets
    /// the timestamp that it will be ready to claim
    GetWithdrawalReadyTime {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct WithdrawalReadyResponse {
    pub withdrawal_ready_timestamp: String,
}
