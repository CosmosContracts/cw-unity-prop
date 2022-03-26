#[cfg(test)]
mod tests {
    use crate::contract::{execute, instantiate, query, sudo};
    use crate::msg::{
        ExecuteMsg, InstantiateMsg, QueryMsg, SudoMsg, WithdrawalReadyResponse,
        WithdrawalTimestampResponse,
    };
    use crate::state::Config;
    use crate::ContractError;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{
        coins, from_binary, Addr, BankMsg, CosmosMsg, Response, Timestamp, Uint128,
    };

    const NATIVE_DENOM: &str = "ujuno";

    #[test]
    fn initialization() {
        let mut deps = mock_dependencies();

        let withdraw_address = String::from("gordon-gekko-address"); // in reality this would be e.g. juno16g2rahf5846rxzp3fwlswy08fz8ccuwk03k57y
        let validated_addr = Addr::unchecked(&withdraw_address);
        let withdraw_delay_in_days = 28; // this is what we are expecting to set it to

        let msg = InstantiateMsg {
            withdraw_address,
            withdraw_delay_in_days,
            native_denom: NATIVE_DENOM.to_string(),
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
                native_denom: NATIVE_DENOM.to_string(),
            },
            contract_config
        );
    }

    #[test]
    fn sudo_burn() {
        let mut deps = mock_dependencies();
        let mut env = mock_env();

        env.block.time = Timestamp::from_seconds(0);

        let funds_sent_to_contract = coins(1_000_000, NATIVE_DENOM);

        let withdraw_address = String::from("gordon-gekko-address"); // in reality this would be e.g. juno16g2rahf5846rxzp3fwlswy08fz8ccuwk03k57y
        let _validated_addr = Addr::unchecked(&withdraw_address);
        let withdraw_delay_in_days = 28; // this is what we are expecting to set it to

        let msg = InstantiateMsg {
            withdraw_address,
            withdraw_delay_in_days,
            native_denom: NATIVE_DENOM.to_string(),
        };

        // the person instantiating
        let instantiate_info = mock_info("bud-fox-address", &funds_sent_to_contract);

        // call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), env.clone(), instantiate_info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // mock funds being added to contract
        let contract_addr = env.clone().contract.address;
        deps.querier
            .update_balance(&contract_addr, funds_sent_to_contract.clone());

        // mock sudo_msg
        let msg = SudoMsg::ExecuteBurn {};
        let res = sudo(deps.as_mut(), env, msg).unwrap();

        // cosmos msgs we expect
        let msgs: Vec<CosmosMsg> = vec![BankMsg::Burn {
            amount: funds_sent_to_contract,
        }
        .into()];

        assert_eq!(
            res,
            Response::new()
                .add_attribute("message_type", "sudo")
                .add_attribute("action", "burn")
                .add_messages(msgs)
        );
    }

    #[test]
    fn sudo_send() {
        let mut deps = mock_dependencies();
        let mut env = mock_env();

        env.block.time = Timestamp::from_seconds(0);

        let funds_sent_to_contract = coins(3_000_000, NATIVE_DENOM);

        let withdraw_address = String::from("gordon-gekko-address"); // in reality this would be e.g. juno16g2rahf5846rxzp3fwlswy08fz8ccuwk03k57y
        let _validated_addr = Addr::unchecked(&withdraw_address);
        let withdraw_delay_in_days = 28; // this is what we are expecting to set it to

        // look I get that this is an increasingly niche joke
        let community_nominated_address = String::from("carl-fox-address");

        let msg = InstantiateMsg {
            withdraw_address,
            withdraw_delay_in_days,
            native_denom: NATIVE_DENOM.to_string(),
        };

        // the person instantiating
        let instantiate_info = mock_info("bud-fox-address", &funds_sent_to_contract);

        // assert this was a success
        let res = instantiate(deps.as_mut(), env.clone(), instantiate_info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // mock funds being added to contract
        let contract_addr = env.clone().contract.address;
        deps.querier
            .update_balance(&contract_addr, funds_sent_to_contract.clone());

        // mock sudo_msg
        // as if we are just gonna send the funds to the maintenance union
        // note that amount (as documented) is NATIVE DENOM, not coins
        let msg = SudoMsg::ExecuteSend {
            recipient: community_nominated_address.clone(),
            amount: Uint128::new(3_000_000), // spell it out explicitly
        };
        let res = sudo(deps.as_mut(), env, msg).unwrap();

        // cosmos msgs we expect
        let msgs: Vec<CosmosMsg> = vec![BankMsg::Send {
            to_address: community_nominated_address,
            amount: funds_sent_to_contract,
        }
        .into()];

        assert_eq!(
            res,
            Response::new()
                .add_attribute("message_type", "sudo")
                .add_attribute("action", "send")
                .add_messages(msgs)
        );
    }

    #[test]
    fn start_withdraw() {
        let mut deps = mock_dependencies();
        let mut env = mock_env();

        env.block.time = Timestamp::from_seconds(0);

        let funds_sent_to_contract = coins(1_000_000, NATIVE_DENOM);

        let withdraw_address = String::from("gordon-gekko-address"); // in reality this would be e.g. juno16g2rahf5846rxzp3fwlswy08fz8ccuwk03k57y
        let _validated_addr = Addr::unchecked(&withdraw_address);
        let withdraw_delay_in_days = 28; // this is what we are expecting to set it to

        let msg = InstantiateMsg {
            withdraw_address: withdraw_address.clone(),
            withdraw_delay_in_days,
            native_denom: NATIVE_DENOM.to_string(),
        };

        // the person instantiating
        let instantiate_info = mock_info("bud-fox-address", &funds_sent_to_contract);

        // call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), env.clone(), instantiate_info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // mock funds being added to contract
        let contract_addr = env.clone().contract.address;
        deps.querier
            .update_balance(&contract_addr, funds_sent_to_contract);

        // only withdraw_address can call
        let info = mock_info(&withdraw_address, &[]);
        let msg = ExecuteMsg::StartWithdraw {};
        let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

        // is the withdrawal ready?
        let is_ready: WithdrawalReadyResponse = from_binary(
            &query(deps.as_ref(), env.clone(), QueryMsg::IsWithdrawalReady {}).unwrap(),
        )
        .unwrap();

        assert_eq!(
            WithdrawalReadyResponse {
                is_withdrawal_ready: false,
            },
            is_ready
        );

        // query timestamp
        let res = query(deps.as_ref(), env, QueryMsg::GetWithdrawalReadyTime {}).unwrap();
        let value: WithdrawalTimestampResponse = from_binary(&res).unwrap();

        // 28 days time from 'now', where 'now' is zero
        let delay_in_seconds = 28u64 * 86400u64;
        let twenty_eight_days_from_now_timestamp =
            Timestamp::from_seconds(0).plus_seconds(delay_in_seconds);

        assert_eq!(
            WithdrawalTimestampResponse {
                withdrawal_ready_timestamp: twenty_eight_days_from_now_timestamp,
            },
            value
        );
    }

    #[test]
    fn withdraw_is_claimable() {
        let mut deps = mock_dependencies();
        let mut env = mock_env();

        env.block.time = Timestamp::from_seconds(0);

        let funds_sent_to_contract = coins(1_000_000, NATIVE_DENOM);

        let withdraw_address = String::from("gordon-gekko-address"); // in reality this would be e.g. juno16g2rahf5846rxzp3fwlswy08fz8ccuwk03k57y
        let _validated_addr = Addr::unchecked(&withdraw_address);
        let withdraw_delay_in_days = 28; // this is what we are expecting to set it to

        let msg = InstantiateMsg {
            withdraw_address: withdraw_address.clone(),
            withdraw_delay_in_days,
            native_denom: NATIVE_DENOM.to_string(),
        };

        // the person instantiating
        let instantiate_info = mock_info("bud-fox-address", &funds_sent_to_contract);

        // call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), env.clone(), instantiate_info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // mock funds being added to contract
        let contract_addr = env.clone().contract.address;
        deps.querier
            .update_balance(&contract_addr, funds_sent_to_contract);

        // random address can't call
        let random = mock_info("some-random-guy", &[]);
        let msg = ExecuteMsg::StartWithdraw {};
        let err = execute(deps.as_mut(), env.clone(), random, msg).unwrap_err();
        assert_eq!(err, ContractError::Unauthorized {});

        // only withdraw_address can call
        let info = mock_info(&withdraw_address, &[]);
        let msg = ExecuteMsg::StartWithdraw {};
        let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

        // 28 days time from 'now', where 'now' is zero
        let delay_in_seconds = 28u64 * 86400u64;
        let twenty_eight_days_from_now_timestamp =
            Timestamp::from_seconds(0).plus_seconds(delay_in_seconds);

        // roll time forward in env to 1 hr after timestamp
        env.block.time = twenty_eight_days_from_now_timestamp.plus_seconds(3600);

        // is the withdrawal ready?
        let is_ready: WithdrawalReadyResponse =
            from_binary(&query(deps.as_ref(), env, QueryMsg::IsWithdrawalReady {}).unwrap())
                .unwrap();

        assert_eq!(
            WithdrawalReadyResponse {
                is_withdrawal_ready: true,
            },
            is_ready
        );
    }

    #[test]
    fn claim_withdrawal() {
        let mut deps = mock_dependencies();
        let mut env = mock_env();

        env.block.time = Timestamp::from_seconds(0);

        let funds_sent_to_contract = coins(1_000_000, NATIVE_DENOM);

        let withdraw_address = String::from("gordon-gekko-address"); // in reality this would be e.g. juno16g2rahf5846rxzp3fwlswy08fz8ccuwk03k57y
        let _validated_addr = Addr::unchecked(&withdraw_address);
        let withdraw_delay_in_days = 28; // this is what we are expecting to set it to

        let msg = InstantiateMsg {
            withdraw_address: withdraw_address.clone(),
            withdraw_delay_in_days,
            native_denom: NATIVE_DENOM.to_string(),
        };

        // the person instantiating
        let instantiate_info = mock_info("bud-fox-address", &funds_sent_to_contract);

        // call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), env.clone(), instantiate_info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // mock funds being added to contract
        let contract_addr = env.clone().contract.address;
        deps.querier
            .update_balance(&contract_addr, funds_sent_to_contract.clone());

        // only withdraw_address can call
        let info = mock_info(&withdraw_address, &[]);
        let msg = ExecuteMsg::StartWithdraw {};
        let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // 28 days time from 'now', where 'now' is zero
        let delay_in_seconds = 28u64 * 86400u64;
        let twenty_eight_days_from_now_timestamp =
            Timestamp::from_seconds(0).plus_seconds(delay_in_seconds);

        // roll time forward in env to 1 hr after timestamp
        env.block.time = twenty_eight_days_from_now_timestamp.plus_seconds(3600);

        // is the withdrawal ready?
        let is_ready: WithdrawalReadyResponse = from_binary(
            &query(deps.as_ref(), env.clone(), QueryMsg::IsWithdrawalReady {}).unwrap(),
        )
        .unwrap();

        assert_eq!(
            WithdrawalReadyResponse {
                is_withdrawal_ready: true,
            },
            is_ready
        );

        // random address can't call claim
        let random = mock_info("some-random-guy", &[]);
        let msg = ExecuteMsg::ExecuteWithdraw {};
        let err = execute(deps.as_mut(), env.clone(), random, msg).unwrap_err();
        assert_eq!(err, ContractError::Unauthorized {});

        // LFG
        let msg = ExecuteMsg::ExecuteWithdraw {};
        let res = execute(deps.as_mut(), env, info, msg).unwrap();

        // cosmos msgs we expect
        let msgs: Vec<CosmosMsg> = vec![BankMsg::Send {
            to_address: withdraw_address.clone(),
            amount: funds_sent_to_contract,
        }
        .into()];

        assert_eq!(
            res,
            Response::new()
                .add_attribute("action", "execute_withdraw")
                .add_attribute("withdraw_address", withdraw_address)
                .add_messages(msgs)
        );
    }

    #[test]
    fn claim_withdrawal_fails() {
        let mut deps = mock_dependencies();
        let mut env = mock_env();

        env.block.time = Timestamp::from_seconds(0);

        let funds_sent_to_contract = coins(1_000_000, NATIVE_DENOM);

        let withdraw_address = String::from("gordon-gekko-address"); // in reality this would be e.g. juno16g2rahf5846rxzp3fwlswy08fz8ccuwk03k57y
        let _validated_addr = Addr::unchecked(&withdraw_address);
        let withdraw_delay_in_days = 28; // this is what we are expecting to set it to

        let msg = InstantiateMsg {
            withdraw_address: withdraw_address.clone(),
            withdraw_delay_in_days,
            native_denom: NATIVE_DENOM.to_string(),
        };

        // the person instantiating
        let instantiate_info = mock_info("bud-fox-address", &funds_sent_to_contract);

        // call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), env.clone(), instantiate_info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // mock funds being added to contract
        let contract_addr = env.clone().contract.address;
        deps.querier
            .update_balance(&contract_addr, funds_sent_to_contract);

        // only withdraw_address can call
        let info = mock_info(&withdraw_address, &[]);
        let msg = ExecuteMsg::StartWithdraw {};
        let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

        // 28 days time from 'now', where 'now' is zero
        let delay_in_seconds = 28u64 * 86400u64;
        let twenty_eight_days_from_now_timestamp =
            Timestamp::from_seconds(0).plus_seconds(delay_in_seconds);

        // roll time forward in env to 1 hr BEFORE timestamp
        env.block.time = twenty_eight_days_from_now_timestamp.minus_seconds(3600);

        // is the withdrawal ready?
        let is_ready: WithdrawalReadyResponse =
            from_binary(&query(deps.as_ref(), env, QueryMsg::IsWithdrawalReady {}).unwrap())
                .unwrap();

        assert_eq!(
            WithdrawalReadyResponse {
                is_withdrawal_ready: false,
            },
            is_ready
        );
    }
}
