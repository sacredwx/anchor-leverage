use cosmwasm_bignumber::Uint256;
use cosmwasm_std::{
    to_binary, Api, Binary, Decimal, Extern, HumanAddr, Querier, QueryRequest, StdResult, Storage,
    Uint128, WasmQuery,
};

use cw20::Cw20HandleMsg;
use hub_querier::StateResponse;

use crate::state::Config;

pub fn query_bonded_exchange_rate<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    bluna_hub_contract: &HumanAddr,
) -> StdResult<Decimal> {
    Ok(deps
        .querier
        .query::<StateResponse>(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: bluna_hub_contract.clone(),
            msg: to_binary(&hub_querier::QueryMsg::State {})?,
        }))?
        .exchange_rate)
}

pub fn query_borrow_limit<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    config: &Config,
    borrower: &HumanAddr,
    block_time: Option<u64>,
) -> StdResult<Uint256> {
    Ok(deps
        .querier
        .query::<moneymarket::overseer::BorrowLimitResponse>(&QueryRequest::Wasm(
            WasmQuery::Smart {
                contract_addr: deps.api.human_address(&config.anchor_overseer_contract)?,
                msg: to_binary(&moneymarket::overseer::QueryMsg::BorrowLimit {
                    borrower: borrower.clone(),
                    block_time,
                })?,
            },
        ))?
        .borrow_limit)
}

pub fn query_loan_amount<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    config: &Config,
    borrower: &HumanAddr,
    block_height: Option<u64>,
) -> StdResult<Uint256> {
    Ok(deps
        .querier
        .query::<moneymarket::market::BorrowerInfoResponse>(&QueryRequest::Wasm(
            WasmQuery::Smart {
                contract_addr: deps.api.human_address(&config.anchor_market_contract)?,
                msg: to_binary(&moneymarket::market::QueryMsg::BorrowerInfo {
                    borrower: borrower.clone(),
                    block_height,
                })?,
            },
        ))?
        .loan_amount)
}

pub fn bond_luna(preferred_validator: &HumanAddr) -> StdResult<Binary> {
    to_binary(&hub_querier::HandleMsg::Bond {
        validator: preferred_validator.clone(),
    })
}

pub fn deposit_bluna_collateral(
    bluna_token_contract: &HumanAddr,
    amount: Uint128,
) -> StdResult<Binary> {
    to_binary(&Cw20HandleMsg::Send {
        contract: bluna_token_contract.clone(),
        amount,
        msg: Some(to_binary(
            &moneymarket::custody::Cw20HookMsg::DepositCollateral {},
        )?),
    })
}

pub fn overseer_lock_collateral(
    bluna_collateral_contract: &HumanAddr,
    amount: Uint256,
) -> StdResult<Binary> {
    to_binary(&moneymarket::overseer::HandleMsg::LockCollateral {
        collaterals: vec![(bluna_collateral_contract.clone(), amount)],
    })
}

pub fn anchor_borrow(borrow_amount: Uint256) -> StdResult<Binary> {
    to_binary(&moneymarket::market::HandleMsg::BorrowStable {
        borrow_amount,
        to: None, // TODO: and other msgs too
    })
}

pub fn swap_to_collateral(amount: Uint128) -> StdResult<Binary> {
    to_binary(&terraswap::pair::HandleMsg::Swap {
        offer_asset: terraswap::asset::Asset {
            amount,
            info: terraswap::asset::AssetInfo::NativeToken {
                denom: crate::contract::TERRASWAP_PAIR.to_string(),
            },
        },
        belief_price: None,
        max_spread: None,
        to: None,
    })
}
