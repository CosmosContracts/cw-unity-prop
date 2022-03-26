#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coins, ensure_eq, to_binary, BankMsg, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo,
    Response, StdResult, Timestamp, Uint128,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{
    ExecuteMsg, InstantiateMsg, QueryMsg, SudoMsg, WithdrawalReadyResponse,
    WithdrawalTimestampResponse,
};
use crate::state::{Config, CONFIG, WITHDRAWAL_READY};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cw-unity-prop";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let withdraw_address = deps.api.addr_validate(&msg.withdraw_address)?;

    let config = Config {
        withdraw_address: withdraw_address.clone(),
        withdraw_delay_in_days: msg.withdraw_delay_in_days,
        native_denom: msg.native_denom,
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("withdraw_address", withdraw_address)
        .add_attribute("withdraw_delay", msg.withdraw_delay_in_days.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::StartWithdraw {} => start_withdraw(deps, env, info),
        ExecuteMsg::ExecuteWithdraw {} => execute_withdraw(deps, env, info),
    }
}

// this sets the withdraw delay
// note that it does not withdraw funds immediately
pub fn start_withdraw(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    // get config
    let config = CONFIG.load(deps.storage)?;
    let withdraw_address = config.withdraw_address;

    // before continuing, only withdraw_address can call this
    ensure_eq!(
        info.sender,
        withdraw_address,
        ContractError::Unauthorized {}
    );

    // get number of days delay
    let delay_in_days: u64 = config.withdraw_delay_in_days;

    // do some really simple maths
    let seconds_in_day = 86400u64;
    let delay_in_seconds = delay_in_days * seconds_in_day;

    // when is 'now'?
    let now: Timestamp = env.block.time;

    // calculate now + configured days (in seconds)
    let rewards_ready_at: Timestamp = now.plus_seconds(delay_in_seconds);

    WITHDRAWAL_READY.save(deps.storage, &rewards_ready_at)?;

    Ok(Response::new()
        .add_attribute("action", "start_withdraw")
        .add_attribute("withdrawal_ready_timestamp", rewards_ready_at.to_string()))
}

// this allows you to withdraw if the withdraw delay has passed
pub fn execute_withdraw(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    // get withdraw address
    let config = CONFIG.load(deps.storage)?;
    let withdraw_address = config.withdraw_address;

    // before continuing, only withdraw_address can call this
    ensure_eq!(
        info.sender,
        withdraw_address,
        ContractError::Unauthorized {}
    );

    // this returns Vec<Coin> for the contract's holdings
    let amount = deps.querier.query_all_balances(&env.contract.address)?;

    // get rewards ready timestamp
    let withdrawal_ready_timestamp = WITHDRAWAL_READY.load(deps.storage)?;

    // check if we are after that time
    let withdrawal_claimable = env.block.time > withdrawal_ready_timestamp;

    // dispatch Response or ContractError
    match withdrawal_claimable {
        true => {
            // set up a bank send to the withdraw address
            // from this contract
            // for everything held by the contract
            let msgs: Vec<CosmosMsg> = vec![BankMsg::Send {
                to_address: withdraw_address.to_string(),
                amount,
            }
            .into()];

            Ok(Response::new()
                .add_attribute("action", "execute_withdraw")
                .add_attribute("withdraw_address", withdraw_address)
                .add_messages(msgs))
        }
        false => Err(ContractError::WithdrawalNotReady {}),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn sudo(deps: DepsMut, env: Env, msg: SudoMsg) -> Result<Response, ContractError> {
    match msg {
        SudoMsg::ExecuteBurn {} => execute_burn(deps, env),
        SudoMsg::ExecuteSend { recipient, amount } => execute_send(deps, env, recipient, amount),
    }
}

// this is the verbose way of doing this
// but obvious reasons for making as easy-to-read as possible
pub fn execute_burn(deps: DepsMut, env: Env) -> Result<Response, ContractError> {
    // this returns Vec<Coin>
    // in this case for the contract's holdings
    let amount: Vec<Coin> = deps.querier.query_all_balances(&env.contract.address)?;

    // create a burn msg struct
    let burn_msg = BankMsg::Burn { amount };

    // then msg we can add to Response
    let msgs: Vec<CosmosMsg> = vec![burn_msg.into()];

    let res = Response::new()
        .add_attribute("message_type", "sudo")
        .add_attribute("action", "burn")
        .add_messages(msgs);
    Ok(res)
}

// looks familiar?
// yep, basically the same logic, isn't it
// allows the community to send funds somewhere
pub fn execute_send(
    deps: DepsMut,
    env: Env,
    recipient: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    // get config
    let config = CONFIG.load(deps.storage)?;

    // get native denom
    let native_denom = config.native_denom;

    // get contract balance
    let contract_balances: Vec<Coin> = deps.querier.query_all_balances(&env.contract.address)?;

    // we are going to check the contract has enough to execute the send
    // now it should error if not,
    // but we handle it here just in case
    let native_balance = contract_balances
        .iter()
        .find(|&coin| coin.denom == native_denom);

    // should never be no native balance, but handle it
    // as well as insufficient funds case
    if let Some(nb) = native_balance {
        if nb.amount < amount {
            return Err(ContractError::InsufficientContractBalance {});
        }
    } else {
        return Err(ContractError::NoNativeBalance {});
    }

    // validate supplied address
    let validated_address = deps.api.addr_validate(&recipient)?;

    // create coins vec
    let amount: Vec<Coin> = coins(amount.u128(), native_denom);

    // create a send
    let send_msg = BankMsg::Send {
        to_address: validated_address.to_string(),
        amount,
    };

    // then msg we can add to Response
    let msgs: Vec<CosmosMsg> = vec![send_msg.into()];

    let res = Response::new()
        .add_attribute("message_type", "sudo")
        .add_attribute("action", "send")
        .add_messages(msgs);
    Ok(res)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetConfig {} => to_binary(&query_config(deps)?),
        QueryMsg::GetWithdrawalReadyTime {} => to_binary(&get_withdraw_ready(deps)?),
        QueryMsg::IsWithdrawalReady {} => to_binary(&query_withdraw_ready(deps, env)?),
    }
}

fn query_config(deps: Deps) -> StdResult<Config> {
    let config = CONFIG.load(deps.storage)?;
    Ok(config)
}

fn get_withdraw_ready(deps: Deps) -> StdResult<WithdrawalTimestampResponse> {
    let withdrawal_ready_timestamp = WITHDRAWAL_READY.load(deps.storage)?;
    Ok(WithdrawalTimestampResponse {
        withdrawal_ready_timestamp,
    })
}

fn query_withdraw_ready(deps: Deps, env: Env) -> StdResult<WithdrawalReadyResponse> {
    let withdrawal_ready_timestamp = WITHDRAWAL_READY.load(deps.storage)?;

    // check if we are have passed the point where withdrawal is possible
    let is_withdrawal_ready = env.block.time > withdrawal_ready_timestamp;

    Ok(WithdrawalReadyResponse {
        is_withdrawal_ready,
    })
}
