use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Withdrawal not ready - wait until after timeout has passed")]
    WithdrawalNotReady {},

    #[error("Contract balance is too small to execute")]
    InsufficientContractBalance {},

    #[error("A native balance was not found in the Contract balances")]
    NoNativeBalance {},

    #[error("Custom Error val: {val:?}")]
    CustomError { val: String },
}
