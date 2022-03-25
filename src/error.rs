use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Withdrawal not ready - wait until after timeout has passed")]
    WithdrawalNotReady {},

    #[error("Custom Error val: {val:?}")]
    CustomError { val: String },
}
