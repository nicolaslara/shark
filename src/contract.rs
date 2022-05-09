#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{BankMsg, Binary, Coin, Deps, DepsMut, Env, MessageInfo, QueryRequest, Response, StdResult, SubMsg, Uint128};
use cw2::set_contract_version;
use osmo_bindings::{OsmosisMsg, OsmosisQuery, SpotPriceResponse};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::state::{Config, Funds, LendPool, CONFIG, LENDERS, POOL, BORROWERS, Debt};

const CONTRACT_NAME: &str = "crates.io:shark";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let admin = msg.admin.unwrap_or_else(|| info.sender.to_string());
    let validated_admin = deps.api.addr_validate(&admin)?;
    let config = Config {
        admin: validated_admin.clone(),
        funds_denom: "uosmo".to_owned(),
        collateral_denom: "gamm/pool/1".to_owned()
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
        ExecuteMsg::SupplyCollateral { } => execute_supply_collateral(deps, info),
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
            expected: match_denom.to_string()
        }),
        _ => Err(ContractError::FundsRequired {}),
    }
}

fn execute_supply(deps: DepsMut<OsmosisQuery>, info: MessageInfo) -> Result<Response<OsmosisMsg>, ContractError> {
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

fn execute_supply_collateral(deps: DepsMut<OsmosisQuery>, info: MessageInfo) -> Result<Response<OsmosisMsg>, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let funds = get_funds_from(&info, &config.collateral_denom)?;

    let debt = BORROWERS.update(deps.storage, &info.sender, |borrower: Option<Debt>| -> Result<Debt, ContractError>{
        match borrower {
            Some(mut debt) => {
                debt.collateral += funds.value;
                Ok(debt)
            },
            None => Ok(Debt { debt: Uint128::new(0), collateral: funds.value })
        }
    })?;

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

fn execute_borrow(deps: DepsMut<OsmosisQuery>, info: MessageInfo, amount: u128) -> Result<Response<OsmosisMsg>, ContractError>{
    let config = CONFIG.load(deps.storage)?;
    let debt = BORROWERS.load(deps.storage, &info.sender)?;

    let spot_price = OsmosisQuery::spot_price(1, "uosmo", "uion");
    let query = QueryRequest::from(spot_price);
    let response: SpotPriceResponse = deps.querier.query(&query)?;

    if debt.capacity(response.price) < Uint128::new(amount) {
        return Err(ContractError::SimpleError{ msg: format!("Price: {:?}", response)});
    }

    let to_borrow = Coin{ denom: config.funds_denom.clone(), amount: Uint128::new(amount) };
    let send = BankMsg::Send { to_address: info.sender.to_string(), amount: vec![to_borrow] };

    Ok(Response::new()
        .add_attribute("action", "execute_borrow")
        .add_attribute("borrowed_denom", config.funds_denom.to_string())
        .add_attribute("borrowed_amount", amount.to_string())
        .add_message(send)
    )
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
pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    unimplemented!()
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    unimplemented!()
}

#[cfg(test)]
mod tests {}
