use std::ops::Mul;

use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{
    coin, log, to_binary, Api, Binary, Coin, CosmosMsg, Empty, Env, Extern, HandleResponse,
    HumanAddr, InitResponse, Querier, StdError, StdResult, Storage, Uint128, WasmMsg,
};

use crate::msg::{HandleAnswer, HandleMsg, InitMsg, PossibleBorrowResponse, QueryMsg};
use crate::querier;
use crate::state::{get_config, set_config, Config};

pub const DECIMAL_FRACTIONAL: u128 = 1_000_000_000_000_000_000;
pub const BORROW_LTV_PERCENTAGE: u64 = 70; // 70% of 50% of TVL, borrow limit is at 50% of LTV, we use recommended 35%
pub const STOP_SWAPPING_ON: u128 = 10_000_000;
pub const ACCEPTED_DENOM: &str = "uluna";
pub const TERRASWAP_PAIR: &str = "uusd";

/// Contract instantiation tx
/// tx inputs are specified in InitMsg in msg.rs file
pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    set_config(
        &mut deps.storage,
        &Config {
            basset_hub_contract: deps.api.canonical_address(&msg.config.basset_hub_contract)?,
            basset_token_contract: deps
                .api
                .canonical_address(&msg.config.basset_token_contract)?,
            basset_collateral_contract: deps
                .api
                .canonical_address(&msg.config.basset_collateral_contract)?,
            anchor_overseer_contract: deps
                .api
                .canonical_address(&msg.config.anchor_overseer_contract)?,
            anchor_market_contract: deps
                .api
                .canonical_address(&msg.config.anchor_market_contract)?,
            terraswap_luna_ust: deps.api.canonical_address(&msg.config.terraswap_luna_ust)?,
            preferred_validator: msg.config.preferred_validator,
        },
    )?;

    Ok(InitResponse::default())
}

/// General handler for contract tx input
/// tx inputs are defined HandleMsg enum in msg.rs file
pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse<Empty>> {
    match msg {
        HandleMsg::Deposit {} => deposit(deps, env),
        HandleMsg::DepositCollateral {} => deposit_collateral(deps, env),
        HandleMsg::Borrow {} => borrow(deps, env),
        HandleMsg::Swap { amount } => swap(deps, env, amount),
        HandleMsg::Redeposit {} => redeposit(deps, env),
    }
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::PossibleBorrow {
            contract_addr,
            block_time,
            block_height,
        } => to_binary(&query_possible_borrow(
            &deps,
            &contract_addr,
            block_time,
            block_height,
        )?),
        QueryMsg::Collateral {
            contract_addr,
        } => to_binary(&query_collateral(
            &deps,
            &contract_addr,
        )?),
    }
}

fn deposit<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
) -> StdResult<HandleResponse> {
    let received = env.message.sent_funds.get(0);
    if env.message.sent_funds.len() != 1 || received.unwrap().denom.ne(ACCEPTED_DENOM) {
        Err(StdError::generic_err(format!(
            "Only '{}' is accepted",
            ACCEPTED_DENOM
        )))
    } else {
        deposit_msgs(deps, &env, received.unwrap())
    }
}

fn deposit_collateral<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
) -> StdResult<HandleResponse> {
    if env.message.sender != env.contract.address {
        return Err(StdError::unauthorized());
    }

    let config = get_config(&deps.storage)?;
    let bonded = querier::query_bonded_asset(
        &deps,
        &deps.api.human_address(&config.basset_collateral_contract)?,
        &env.contract.address,
    )?;

    Ok(HandleResponse {
        messages: vec![
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: deps.api.human_address(&config.basset_collateral_contract)?,
                send: vec![],
                msg: querier::deposit_basset_collateral(
                    &deps.api.human_address(&config.basset_token_contract)?,
                    bonded.into(),
                )?,
            }),
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: deps.api.human_address(&config.anchor_overseer_contract)?,
                send: vec![],
                msg: querier::overseer_lock_collateral(
                    &deps.api.human_address(&config.basset_collateral_contract)?,
                    bonded.into(),
                )?,
            }),
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: env.contract.address,
                send: vec![],
                msg: to_binary(&HandleMsg::Borrow {})?,
            }),
        ],
        log: vec![
            log("action", "deposit_collateral"),
            log("bonded", bonded.to_string()),
        ],
        data: Some(to_binary(&HandleAnswer::Deposit)?),
    })
}

fn borrow<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
) -> StdResult<HandleResponse> {
    if env.message.sender != env.contract.address {
        return Err(StdError::unauthorized());
    }

    let config = get_config(&deps.storage)?;
    let possible_borrow = get_possible_borrow(
        deps,
        &config,
        &env.contract.address,
        Some(env.block.time),
        Some(env.block.height),
    )?;
    let borrow_after_tax = moneymarket::querier::deduct_tax(
        &deps,
        coin(possible_borrow.borrow_amount.into(), TERRASWAP_PAIR),
    )?;

    Ok(HandleResponse {
        messages: vec![
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: deps.api.human_address(&config.anchor_market_contract)?,
                send: vec![],
                msg: querier::anchor_borrow(possible_borrow.borrow_amount)?,
            }),
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: env.contract.address,
                send: vec![],
                msg: to_binary(&HandleMsg::Swap {
                    amount: borrow_after_tax.amount,
                })?,
            }),
        ],
        log: vec![
            log("action", "borrow"),
            log("borrow_amount", possible_borrow.borrow_amount.to_string()),
            log(
                "borrow_amount_after_tax",
                borrow_after_tax.amount.to_string(),
            ),
        ],
        data: Some(to_binary(&HandleAnswer::Borrow)?),
    })
}

fn swap<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    amount: Uint128,
) -> StdResult<HandleResponse> {
    if env.message.sender != env.contract.address {
        return Err(StdError::unauthorized());
    }

    let config = get_config(&deps.storage)?;

    let amount: Uint128 = Uint256::from(amount.u128())
        .mul(Decimal256::from_ratio(998, 1000))
        .into();

    let mut messages=vec![
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: deps.api.human_address(&config.terraswap_luna_ust)?,
            send: vec![coin(amount.u128(), TERRASWAP_PAIR)],
            msg: querier::swap_to_collateral(amount)?,
        }),
    ];

    if amount.u128() > STOP_SWAPPING_ON {
        messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: env.contract.address,
            send: vec![],
            msg: to_binary(&HandleMsg::Redeposit {})?,
        }));
    }
    
    Ok(HandleResponse {
        messages,
        log: vec![log("action", "swap"), log("swap_amount", amount)],
        data: Some(to_binary(&HandleAnswer::Borrow)?),
    })
}

fn redeposit<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
) -> StdResult<HandleResponse> {
    let received = deps
        .querier
        .query_balance(env.contract.address.clone(), ACCEPTED_DENOM)?;
    deposit_msgs(deps, &env, &received)
}

fn deposit_msgs<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    received: &Coin,
) -> StdResult<HandleResponse> {
    let config = get_config(&deps.storage)?;

    Ok(HandleResponse {
        messages: vec![
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: deps.api.human_address(&config.basset_hub_contract)?,
                send: vec![coin(received.amount.u128(), &received.denom)],
                msg: querier::bond_luna(&config.preferred_validator)?,
            }),
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: env.contract.address.clone(),
                send: vec![],
                msg: to_binary(&HandleMsg::DepositCollateral {})?,
            }),
        ],
        log: vec![
            log("action", "deposit"),
            log(
                "deposited",
                format!("{} {}", received.amount, received.denom),
            ),
        ],
        data: Some(to_binary(&HandleAnswer::Deposit)?),
    })
}

pub fn query_possible_borrow<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    contract_addr: &HumanAddr,
    block_time: Option<u64>,
    block_height: Option<u64>,
) -> StdResult<PossibleBorrowResponse> {
    let config = get_config(&deps.storage)?;
    get_possible_borrow(deps, &config, contract_addr, block_time, block_height)
}

pub fn get_possible_borrow<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    config: &Config,
    contract_addr: &HumanAddr,
    block_time: Option<u64>,
    block_height: Option<u64>,
) -> StdResult<PossibleBorrowResponse> {
    // TODO: not safe enough for use by several people, possible fix: to factor a contract per client
    let borrow_limit = querier::query_borrow_limit(deps, &config, &contract_addr, block_time)?;
    let already_borrowed = querier::query_loan_amount(deps, &config, &contract_addr, block_height)?;
    let borrow_amount = borrow_limit.mul(Decimal256::percent(BORROW_LTV_PERCENTAGE)) - already_borrowed;
    Ok(PossibleBorrowResponse {
        borrow_limit,
        already_borrowed,
        borrow_amount,
    })
}

pub fn query_collateral<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    contract_addr: &HumanAddr,
) -> StdResult<moneymarket::custody::BorrowerResponse> {
    let config = get_config(&deps.storage)?;
    querier::query_collateral(deps, &config, &contract_addr)
}

#[cfg(test)]
mod tests {
    // TODO: Add test cases
}
