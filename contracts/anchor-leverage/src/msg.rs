use cosmwasm_bignumber::Uint256;
use cosmwasm_std::{HumanAddr, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::state::ConfigMsg;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub config: ConfigMsg,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    Deposit {},
    DepositCollateral {},
    Borrow {},
    Swap { amount: Uint128 },
    Redeposit {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    PossibleBorrow {
        contract_addr: HumanAddr,
        block_time: Option<u64>,
        block_height: Option<u64>,
    },
    Collateral {
        contract_addr: HumanAddr,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleAnswer {
    Deposit,
    Borrow,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct PossibleBorrowResponse {
    pub borrow_limit: Uint256,
    pub already_borrowed: Uint256,
    pub borrow_amount: Uint256,
}
