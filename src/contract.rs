#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::CosmosMsg::Bank;
use cosmwasm_std::{
    BankMsg, Binary, Coin, CustomQuery, Deps, DepsMut, Env, MessageInfo, QueryRequest, Reply,
    Response, StdError, StdResult, SubMsg, Uint128,
};
use cw2::set_contract_version;
use osmo_bindings::{OsmosisMsg, OsmosisQuery, SpotPriceResponse};
use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use std::fmt::Debug;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::state::{Config, Debt, Funds, LendPool, BORROWERS, CONFIG, LENDERS, POOL};

const CONTRACT_NAME: &str = "crates.io:shark";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut<OsmosisQuery>,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response<OsmosisMsg>, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let admin = msg.admin.unwrap_or_else(|| info.sender.to_string());
    let validated_admin = deps.api.addr_validate(&admin)?;
    let config = Config {
        admin: validated_admin.clone(),
        funds_denom: "uosmo".to_owned(),
        collateral_denom: "gamm/pool/1".to_owned(),
    };
    CONFIG.save(deps.storage, &config)?;
    let zero = Uint128::new(0);
    POOL.save(
        deps.storage,
        &LendPool {
            available: zero,
            used: zero,
        },
    )?;
    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("admin", validated_admin.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut<OsmosisQuery>,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response<OsmosisMsg>, ContractError> {
    match msg {
        ExecuteMsg::SupplyFunds {} => execute_supply(deps, info),
        // ExecuteMsg::Swap { input, min_output } => execute_swap(deps, info, input, min_output),
        ExecuteMsg::SupplyCollateral {} => execute_supply_collateral(deps, info),
        ExecuteMsg::Borrow { amount } => execute_borrow(deps, info, amount),
        ExecuteMsg::Repay { .. } => unimplemented!(),
        ExecuteMsg::DistributeRewards { .. } => unimplemented!(),
    }
}

fn get_funds_from(info: &MessageInfo, match_denom: &str) -> Result<Funds, ContractError> {
    match &info.funds[..] {
        [Coin { denom, amount }] if denom == match_denom => Ok(Funds { value: *amount }),
        [coin] => Err(ContractError::InvalidFunds {
            funds: Some(coin.clone() as Coin),
            expected: match_denom.to_string(),
        }),
        _ => Err(ContractError::FundsRequired {}),
    }
}

fn execute_supply(
    deps: DepsMut<OsmosisQuery>,
    info: MessageInfo,
) -> Result<Response<OsmosisMsg>, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let funds = get_funds_from(&info, &config.funds_denom)?;

    LENDERS.save(deps.storage, &info.sender, &funds)?;
    let pool = POOL.update(deps.storage, |mut pool| -> Result<_, ContractError> {
        pool.available += funds.value;
        Ok(pool)
    })?;
    Ok(Response::new()
        .add_attribute("action", "supply_funds")
        .add_attribute("available_funds", pool.available))
}

fn execute_supply_collateral(
    deps: DepsMut<OsmosisQuery>,
    info: MessageInfo,
) -> Result<Response<OsmosisMsg>, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let funds = get_funds_from(&info, &config.collateral_denom)?;

    let debt = BORROWERS.update(
        deps.storage,
        &info.sender,
        |borrower: Option<Debt>| -> Result<Debt, ContractError> {
            match borrower {
                Some(mut debt) => {
                    debt.collateral += funds.value;
                    Ok(debt)
                }
                None => Ok(Debt {
                    debt: Uint128::new(0),
                    collateral: funds.value,
                }),
            }
        },
    )?;

    let lock = OsmosisMsg::LockTokens {
        denom: "gamm/pool/1".to_owned(),
        amount: funds.value,
        duration: "336h".to_owned(),
    };
    let msgs = vec![SubMsg::new(lock)];

    Ok(Response::new()
        .add_attribute("action", "execute_supply_collateral")
        .add_attribute("collateral", debt.collateral)
        .add_attribute("debt", debt.debt)
        .add_submessages(msgs))
}

fn execute_borrow(
    deps: DepsMut<OsmosisQuery>,
    info: MessageInfo,
    amount: u128,
) -> Result<Response<OsmosisMsg>, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let debt = BORROWERS
        .load(deps.storage, &info.sender)
        .or(Err(ContractError::InsuficientCollateral {}))?;

    let spot_price = OsmosisQuery::spot_price(1, "uosmo", "uion");
    let query = QueryRequest::from(spot_price);
    let response: SpotPriceResponse = deps.querier.query(&query)?;

    return Err(ContractError::SimpleError {
        msg: format!("Price: {:?}", response),
    });

    let to_borrow = Coin {
        denom: config.funds_denom.clone(),
        amount: Uint128::new(amount),
    };
    let send = BankMsg::Send {
        to_address: info.sender.to_string(),
        amount: vec![to_borrow],
    };

    Ok(Response::new()
        .add_attribute("action", "execute_borrow")
        .add_attribute("borrowed_denom", config.funds_denom.to_string())
        .add_attribute("borrowed_amount", amount.to_string())
        .add_message(send))
}

// fn execute_swap(
//     _deps: DepsMut,
//     _info: MessageInfo,
//     input: u128,
//     min_output: u128,
// ) -> Result<Response<OsmosisMsg>, ContractError> {
//     let swap = OsmosisMsg::simple_swap(
//         1,
//         "uosmo",
//         "uion",
//         SwapAmountWithLimit::ExactIn {
//             input: Uint128::from(input),
//             min_output: Uint128::from(min_output),
//         },
//     );
//     let msgs = vec![SubMsg::new(swap)];
//
//     Ok(Response::new()
//         .add_attribute("action", "execute_swap")
//         .add_submessages(msgs))
// }

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps<OsmosisQuery>, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    unimplemented!()
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    unimplemented!()
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(
    _deps: DepsMut<OsmosisQuery>,
    _env: Env,
    _msg: Reply,
) -> Result<Response<OsmosisMsg>, ContractError> {
    unimplemented!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::marker::PhantomData;

    use cosmwasm_std::testing::{mock_env, mock_info, MockApi, MockQuerier, MockStorage};
    use cosmwasm_std::{
        attr, coin, to_binary, Addr, Coin, CosmosMsg, CustomMsg, Empty, OwnedDeps, SystemError,
        SystemResult, WasmMsg,
    };
    use cw_multi_test::{Contract, ContractWrapper, Executor};
    use osmo_bindings::{OsmosisMsg, OsmosisQuery};
    use osmo_bindings_test::{OsmosisApp, Pool};
    use serde::Serialize;

    use crate::msg::{ExecuteMsg, InstantiateMsg};

    pub fn contract<C, Q>() -> Box<dyn Contract<C, Q>>
    where
        C: Clone + Debug + PartialEq + JsonSchema + DeserializeOwned + 'static,
        Q: CustomQuery + DeserializeOwned + 'static,
        ContractWrapper<
            ExecuteMsg,
            InstantiateMsg,
            QueryMsg,
            ContractError,
            ContractError,
            cosmwasm_std::StdError,
            OsmosisMsg,
            OsmosisQuery,
        >: Contract<C, Q>,
    {
        let contract = ContractWrapper::new(execute, instantiate, query); //.with_reply(reply);
        Box::new(contract)
    }

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
}
