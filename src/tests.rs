#![cfg(test)]
use std::marker::PhantomData;

use cosmwasm_std::testing::{mock_env, mock_info, MockApi, MockQuerier, MockStorage};
use cosmwasm_std::{
    attr, coin, to_binary, Addr, Coin, CosmosMsg, CustomMsg, Empty, OwnedDeps, SystemError,
    SystemResult, WasmMsg,
};
use cw_multi_test::{Contract, Executor};
use osmo_bindings::{OsmosisMsg, OsmosisQuery};
use osmo_bindings_test::{OsmosisApp, Pool};
use serde::Serialize;

use crate::contract::contract;
use crate::msg::{ExecuteMsg, InstantiateMsg};

pub fn mock_dependencies(
) -> OwnedDeps<MockStorage, MockApi, MockQuerier<OsmosisQuery>, OsmosisQuery> {
    let custom_querier: MockQuerier<OsmosisQuery> =
        MockQuerier::new(&[]).with_custom_handler(|query| {
            println!("{:?}", query);
            SystemResult::Err(SystemError::InvalidRequest {
                error: "not implemented".to_string(),
                request: Default::default(),
            })
        });
    OwnedDeps {
        storage: MockStorage::default(),
        api: MockApi::default(),
        querier: custom_querier,
        custom_query_type: PhantomData,
    }
}

pub const OWNER_ADDR: &str = "osmo1t3gjpqadhhqcd29v64xa06z66mmz7kazsvkp69";

pub const LENDER_ADDR: &str = "osmo1t3gjpqadhhqcd29v64xa06z66mmz7kazsvkp69";
pub const BORROWER_ADDR: &str = "osmo1y244hh4g6ku4kznyy5c53adgu9m8jucf0kmz82";

// fn build_wasm_msg<T>(contract_addr: Addr, inner_msg: T) -> CosmosMsg<OsmosisMsg>
// where
//     T: Serialize + ?Sized,
// {
//     CosmosMsg::Wasm(WasmMsg::Execute {
//         contract_addr: contract_addr.into(),
//         msg: to_binary(&inner_msg).unwrap(),
//         funds: vec![],
//     })
// }

#[test]
fn test_execute_borrow() {
    // let mut router = load();
    let mut app = OsmosisApp::new();
    let coin_a = coin(6_000_000u128, "osmo");
    let coin_b = coin(1_500_000u128, "atom");
    let pool = Pool::new(coin_a.clone(), coin_b.clone());
    app.init_modules(|router, _, storage| {
        router.custom.set_pool(storage, 1, &pool).unwrap();
    });
    let contract: Box<dyn Contract<OsmosisMsg, OsmosisQuery>> = contract();
    let code_id = app.store_code(contract);

    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info(LENDER_ADDR, &vec![Coin::new(10, "uosmo")]);

    // Instantiate the contract
    let msg = InstantiateMsg { admin: None };
    let contract_addr = app
        .instantiate_contract(
            code_id,
            Addr::unchecked(OWNER_ADDR),
            &msg,
            &[],
            "shark",
            None,
        )
        .unwrap();

    // Add funds to be lent
    let msg = ExecuteMsg::SupplyFunds {};
    let wasm_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: contract_addr.into(),
        msg: to_binary(&msg).unwrap(),
        funds: vec![],
    });
    // let wasm_msg = build_wasm_msg(contract_addr, msg);
    let res = app.execute(Addr::unchecked(LENDER_ADDR), wasm_msg).unwrap();

    // let msg = ExecuteMsg::SupplyCollateral {};
    // let info = mock_info(BORROWER_ADDR, &vec![Coin::new(10, "gamm/pool/1")]);
    // let res = execute_supply_collateral(deps.as_mut(), info.clone()).unwrap();
    //
    // let msg = ExecuteMsg::Borrow { amount: 1 };
    // let info = mock_info(BORROWER_ADDR, &vec![]);
    // let res = execute(deps.as_mut(), env, info.clone(), msg).unwrap();

    // Unwrap to assert success
    // let res = execute(deps.as_mut(), env, info, msg).unwrap();
}
