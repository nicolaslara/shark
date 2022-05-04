#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Coin, Deps, DepsMut, Env, MessageInfo, Response, StdResult, SubMsg, Uint128};
use cw2::set_contract_version;
use osmo_bindings::{OsmosisMsg, SwapAmountWithLimit};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::state::{Config, Funds, LendPool, CONFIG, LENDERS, POOL};

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
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response<OsmosisMsg>, ContractError> {
    match msg {
        ExecuteMsg::SupplyFunds {} => execute_supply(deps, info),
        ExecuteMsg::Swap {input, min_output} => execute_swap(deps, info, input, min_output),
        ExecuteMsg::SupplyCollateral { collateral: _ } => unimplemented!(),
        ExecuteMsg::Borrow { amount: _ } => unimplemented!(),
    }
}

fn get_funds_from(info: &MessageInfo, match_denom: &str) -> Result<Funds, ContractError> {
    match &info.funds[..] {
        [Coin { denom, amount }] if denom == match_denom => Ok(Funds { value: *amount }),
        _ => Err(ContractError::InvalidFunds {}),
    }
}

fn execute_supply(deps: DepsMut, info: MessageInfo) -> Result<Response<OsmosisMsg>, ContractError> {
    let funds = get_funds_from(&info, "uosmo")?;

    LENDERS.save(deps.storage, &info.sender, &funds)?;
    let pool = POOL.update(deps.storage, |mut pool| -> Result<_, ContractError> {
        pool.available += funds.value;
        Ok(pool)
    })?;
    Ok(Response::new()
        .add_attribute("action", "supply_funds")
        .add_attribute("available_funds", pool.available))
}

fn execute_swap(_deps: DepsMut, _info: MessageInfo, input: i32, min_output: i32) -> Result<Response<OsmosisMsg>, ContractError> {
    let swap = OsmosisMsg::simple_swap(
        1,
        "uosmo",
        "uion",
        SwapAmountWithLimit::ExactIn {
            input: Uint128::from(input as u128),
            min_output: Uint128::from(min_output as u128)
        }
    );
    let msgs = vec![SubMsg::new(swap)];

    Ok(Response::new()
        .add_attribute("action", "execute_swap")
        .add_submessages(msgs)
    )
}

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
