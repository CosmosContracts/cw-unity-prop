#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    ensure_eq, to_binary, BankMsg, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response,
    StdResult, Timestamp,
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
    }
}

// this is the verbose way of doing this
// but obvious reasons for making as easy-to-read as possible
pub fn execute_burn(deps: DepsMut, env: Env) -> Result<Response, ContractError> {
    // this returns Vec<Coin>
    // in this case for the contract's holdings
    let amount = deps.querier.query_all_balances(&env.contract.address)?;

    // create a burn msg struct
    let burn_msg = BankMsg::Burn { amount };

    // then msg we can add to Response
    let msgs: Vec<CosmosMsg> = vec![burn_msg.into()];

    let res = Response::new()
        .add_attribute("action", "burn")
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

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{from_binary, Addr};

    #[test]
    fn initialization() {
        let mut deps = mock_dependencies();

        let withdraw_address = String::from("gordon-gekko-address"); // in reality this would be e.g. juno16g2rahf5846rxzp3fwlswy08fz8ccuwk03k57y
        let validated_addr = Addr::unchecked(&withdraw_address);
        let withdraw_delay_in_days = 28; // this is what we are expecting to set it to

        let msg = InstantiateMsg {
            withdraw_address,
            withdraw_delay_in_days,
        };

        // the person instantiating
        let instantiate_info = mock_info("bud-fox-address", &[]);

        // call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), instantiate_info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // query state
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetConfig {}).unwrap();
        let contract_config: Config = from_binary(&res).unwrap();
        assert_eq!(
            Config {
                withdraw_address: validated_addr,
                withdraw_delay_in_days,
            },
            contract_config
        );
    }

    // #[test]
    // fn start_withdraw() {
    //     let mut deps = mock_dependencies();

    //     let withdraw_address = String::from("gordon-gekko-address"); // in reality this would be e.g. juno16g2rahf5846rxzp3fwlswy08fz8ccuwk03k57y
    //     let _validated_addr = Addr::unchecked(&withdraw_address);
    //     let withdraw_delay_in_days = 28; // this is what we are expecting to set it to

    //     let msg = InstantiateMsg {
    //         withdraw_address: withdraw_address.clone(),
    //         withdraw_delay_in_days,
    //     };

    //     // the person instantiating
    //     let instantiate_info = mock_info("bud-fox-address", &[]);

    //     // call .unwrap() to assert this was a success
    //     let res = instantiate(deps.as_mut(), mock_env(), instantiate_info, msg).unwrap();
    //     assert_eq!(0, res.messages.len());

    //     // only withdraw_address can call
    //     let info = mock_info(&withdraw_address, &[]);
    //     let msg = ExecuteMsg::StartWithdraw {};
    //     let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    //     // should increase counter by 1
    //     let res = query(
    //         deps.as_ref(),
    //         mock_env(),
    //         QueryMsg::GetWithdrawalReadyTime {},
    //     )
    //     .unwrap();
    //     let value: WithdrawalTimestampResponse = from_binary(&res).unwrap();

    //     // 28 days time from 'now', where 'now' is zero
    //     let delay_in_seconds = 28u64 * 86400u64;
    //     let twenty_eight_days_from_now_timestamp =
    //         Timestamp::from_seconds(0).plus_seconds(delay_in_seconds);

    //     assert_eq!(
    //         WithdrawalTimestampResponse {
    //             withdrawal_ready_timestamp: twenty_eight_days_from_now_timestamp,
    //         },
    //         value
    //     );
    // }
}
