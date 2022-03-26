#[cfg(test)]
mod tests {
    use crate::helpers::CwTemplateContract;
    use crate::msg::{InstantiateMsg, QueryMsg, WithdrawalReadyResponse};
    use cosmwasm_std::{Addr, Coin, Empty, Uint128};
    use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};

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
    const ADMIN: &str = "ADMIN";
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
                Addr::unchecked(ADMIN), // in reality we would set --no-admin
                &msg,
                &[],
                "test",
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

    mod start_withdraw {
        use super::*;
        use crate::msg::ExecuteMsg;

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
}
