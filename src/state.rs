use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub admin: Addr,
    pub funds_denom: String,
    pub collateral_denom: String,
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
    pub debt: Uint128,
    pub collateral: Uint128,
}

impl Debt {
    pub fn capacity(&self) -> Uint128 {
        Uint128::new(20)
    }
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const LENDERS: Map<&Addr, Funds> = Map::new("lenders");
pub const BORROWERS: Map<&Addr, Debt> = Map::new("borrowers");
pub const POOL: Item<LendPool> = Item::new("pool");
