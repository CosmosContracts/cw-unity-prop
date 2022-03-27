#[cfg(test)]
mod tests {
    use crate::helpers::CwTemplateContract;
    use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, SudoMsg, WithdrawalReadyResponse};

    use cosmwasm_std::{coins, Addr, Coin, Empty, Uint128};
    use cw_multi_test::{App, AppBuilder, AppResponse, Contract, ContractWrapper, Executor};

    pub fn contract_template() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        );
        let contract_with_sudo = contract.with_sudo(crate::contract::sudo);
        Box::new(contract_with_sudo)
    }

    const USER: &str = "USER";
    //const ADMIN: &str = "ADMIN";
    const NATIVE_DENOM: &str = "ujuno";

    fn mock_app() -> App {
        AppBuilder::new().build(|router, _, storage| {
            router
                .bank
                .init_balance(
                    storage,
                    &Addr::unchecked(USER),
                    vec![Coin {
                        denom: NATIVE_DENOM.to_string(),
                        amount: Uint128::new(3_000_000),
                    }],
                )
                .unwrap();
        })
    }

    fn mock_instantiate() -> (App, CwTemplateContract, Addr) {
        let mut app = mock_app();
        let cw_template_id = app.store_code(contract_template());

        let withdraw_address = String::from("gordon-gekko-address"); // in reality this would be e.g. juno16g2rahf5846rxzp3fwlswy08fz8ccuwk03k57y
        let _validated_addr = Addr::unchecked(&withdraw_address);
        let withdraw_delay_in_days = 28; // this is what we are expecting to set it to

        let msg = InstantiateMsg {
            withdraw_address,
            withdraw_delay_in_days,
            native_denom: NATIVE_DENOM.to_string(),
        };

        let cw_template_contract_addr = app
            .instantiate_contract(
                cw_template_id,
                Addr::unchecked(USER), // in reality we would set --no-admin
                &msg,
                &[Coin {
                    denom: NATIVE_DENOM.to_string(),
                    amount: Uint128::new(3_000_000),
                }], // set a contract balance
                "cw-unity-prop",
                None,
            )
            .unwrap();

        let cw_template_contract = CwTemplateContract(cw_template_contract_addr.clone());

        (app, cw_template_contract, cw_template_contract_addr)
    }

    fn mock_instantiate_no_balance() -> (App, CwTemplateContract, Addr) {
        let mut app = mock_app();
        let cw_template_id = app.store_code(contract_template());

        let withdraw_address = String::from("gordon-gekko-address"); // in reality this would be e.g. juno16g2rahf5846rxzp3fwlswy08fz8ccuwk03k57y
        let _validated_addr = Addr::unchecked(&withdraw_address);
        let withdraw_delay_in_days = 28; // this is what we are expecting to set it to

        let msg = InstantiateMsg {
            withdraw_address,
            withdraw_delay_in_days,
            native_denom: NATIVE_DENOM.to_string(),
        };

        let cw_template_contract_addr = app
            .instantiate_contract(
                cw_template_id,
                Addr::unchecked(USER), // in reality we would set --no-admin
                &msg,
                &[],
                "cw-unity-prop",
                None,
            )
            .unwrap();

        let cw_template_contract = CwTemplateContract(cw_template_contract_addr.clone());

        (app, cw_template_contract, cw_template_contract_addr)
    }

    fn is_withdrawal_ready(app: &mut App, contract_address: Addr) -> WithdrawalReadyResponse {
        let msg = QueryMsg::IsWithdrawalReady {};
        let result: WithdrawalReadyResponse =
            app.wrap().query_wasm_smart(contract_address, &msg).unwrap();
        result
    }

    mod withdraw {
        use super::*;

        #[test]
        fn start_withdraw() {
            let (mut app, cw_template_contract, contract_addr) = mock_instantiate();

            let withdraw_address = String::from("gordon-gekko-address");
            let validated_addr = Addr::unchecked(&withdraw_address);

            let msg = ExecuteMsg::StartWithdraw {};
            let cosmos_msg = cw_template_contract.call(msg).unwrap();
            app.execute(validated_addr, cosmos_msg).unwrap();

            let withdrawal_ready = is_withdrawal_ready(&mut app, contract_addr);

            assert_eq!(
                withdrawal_ready,
                WithdrawalReadyResponse {
                    is_withdrawal_ready: false,
                }
            );
        }

        #[test]
        fn start_withdraw_fails_with_wrong_address() {
            let (mut app, cw_template_contract, _contract_addr) = mock_instantiate();

            let msg = ExecuteMsg::StartWithdraw {};
            let cosmos_msg = cw_template_contract.call(msg).unwrap();

            // we expect this to fail
            // hence unwrap_err
            app.execute(Addr::unchecked("some-random-address"), cosmos_msg)
                .unwrap_err();
        }
    }

    fn exec_sudo_burn(app: &mut App, contract_address: Addr) -> anyhow::Result<AppResponse> {
        let msg = SudoMsg::ExecuteBurn {};
        app.wasm_sudo(contract_address, &msg)
    }

    fn exec_sudo_send(
        app: &mut App,
        contract_address: Addr,
        recipient: String,
        amount: Uint128,
    ) -> anyhow::Result<AppResponse> {
        let msg = SudoMsg::ExecuteSend { recipient, amount };
        app.wasm_sudo(contract_address, &msg)
    }

    fn get_balance(app: &mut App, address: &Addr) -> Vec<Coin> {
        app.wrap().query_all_balances(address).unwrap()
    }

    mod sudo {
        use super::*;

        #[test]
        fn sudo_burn() {
            let (mut app, _cw_template_contract, contract_addr) = mock_instantiate();

            // this tests for success
            exec_sudo_burn(&mut app, contract_addr.clone()).unwrap();

            let contract_balance = get_balance(&mut app, &contract_addr);

            assert_eq!(contract_balance, &[]);
        }

        #[test]
        fn sudo_send() {
            let (mut app, _cw_template_contract, contract_addr) = mock_instantiate();

            let nominated_address = String::from("carl-fox-address");
            let validated_addr = Addr::unchecked(&nominated_address);

            // this tests for success
            exec_sudo_send(
                &mut app,
                contract_addr.clone(),
                validated_addr.to_string(),
                Uint128::new(2_000_000),
            )
            .unwrap();

            let community_nominated_address_balance = get_balance(&mut app, &validated_addr);

            assert_eq!(
                community_nominated_address_balance,
                coins(2_000_000, NATIVE_DENOM),
            );

            let contract_balance = get_balance(&mut app, &contract_addr);

            assert_eq!(contract_balance, coins(1_000_000, NATIVE_DENOM),);
        }

        #[test]
        fn sudo_send_fails() {
            let (mut app, _cw_template_contract, contract_addr) = mock_instantiate_no_balance();

            let nominated_address = String::from("carl-fox-address");
            let validated_addr = Addr::unchecked(&nominated_address);

            // this tests for error
            let _err = exec_sudo_send(
                &mut app,
                contract_addr,
                validated_addr.to_string(),
                Uint128::new(2_000_000),
            )
            .unwrap_err();
        }

        #[test]
        fn sudo_send_fails_balance_too_small() {
            let (mut app, _cw_template_contract, contract_addr) = mock_instantiate();

            let nominated_address = String::from("carl-fox-address");
            let validated_addr = Addr::unchecked(&nominated_address);

            // this tests for error
            let _err = exec_sudo_send(
                &mut app,
                contract_addr,
                validated_addr.to_string(),
                Uint128::new(4_000_000),
            )
            .unwrap_err();
        }
    }
}
