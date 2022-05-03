use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Coin, Uint128};
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub admin: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct LendPool {
    pub used: Uint128,
    pub available: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Funds {
    pub value: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Debt {
    pub debt: Option<Coin>,
    pub collateral: Coin,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const LENDERS: Map<&Addr, Funds> = Map::new("lenders");
pub const BORROWERS: Map<&Addr, Debt> = Map::new("borrowers");
pub const POOL: Item<LendPool> = Item::new("pool");
