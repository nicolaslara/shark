use cosmwasm_std::{Coin, StdError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("InvalidFunds")]
    InvalidFunds { funds: Option<Coin>, expected: String },

    #[error("FundsRequired")]
    FundsRequired { },

    #[error("InsuficientCollateral")]
    InsuficientCollateral { },

    #[error("SimpleError")]
    SimpleError { msg: String },
}
