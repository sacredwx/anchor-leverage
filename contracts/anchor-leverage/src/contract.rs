use cosmwasm_std::{
    coin, log, to_binary, Api, Binary, CosmosMsg, Empty, Env, Extern, HandleResponse,
    InitResponse, Querier, StdError, StdResult, Storage, WasmMsg,
};

use crate::msg::{HandleAnswer, HandleMsg, InitMsg, QueryMsg};
use crate::{querier, utils};
use crate::state::{Config, config, config_get};

pub const DECIMAL_FRACTIONAL: u128 = 1_000_000_000_000_000_000;
pub const ACCEPTED_DENOM: &str = "uluna";
pub const TERRASWAP_PAIR: &str = "uusd";

/// Contract instantiation tx
/// tx inputs are specified in InitMsg in msg.rs file
pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    config(&mut deps.storage).save(&Config {
        bluna_hub_contract: deps.api.canonical_address(&msg.config.bluna_hub_contract)?,
        bluna_token_contract: deps.api.canonical_address(&msg.config.bluna_token_contract)?,
        bluna_collateral_contract: deps.api.canonical_address(&msg.config.bluna_collateral_contract)?,
        anchor_overseer_contract: deps.api.canonical_address(&msg.config.anchor_overseer_contract)?,
        anchor_market_contract: deps.api.canonical_address(&msg.config.anchor_market_contract)?,
        terraswap_luna_ust: deps.api.canonical_address(&msg.config.terraswap_luna_ust)?,
        preferred_validator: msg.config.preferred_validator,
    })?;

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
        HandleMsg::DepositMsg {} => deposit(deps, env),
        HandleMsg::Borrow {} => borrow(deps, env),
    }
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    _deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {}
}

fn deposit<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
) -> StdResult<HandleResponse> {
    let config = config_get(&deps.storage)?;
    let received = env.message.sent_funds.get(0);
    if env.message.sent_funds.len() != 1 || received.unwrap().denom.ne(ACCEPTED_DENOM) {
        Err(StdError::generic_err(format!(
            "Only '{}' is accepted",
            ACCEPTED_DENOM
        )))
    } else {
        let received = received.unwrap();
        let opposite_exchange_rate = utils::get_opposite_ratio(querier::query_bonded_exchange_rate(
            deps,
            &deps.api.human_address(&config.bluna_hub_contract)?,
        )?);
        let bonded = received.amount * opposite_exchange_rate;
        Ok(HandleResponse {
            messages: vec![
                CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: deps.api.human_address(&config.bluna_hub_contract)?,
                    send: vec![coin(received.amount.u128(), &received.denom)],
                    msg: querier::bond_luna(&config.preferred_validator)?,
                }),
                CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: deps.api.human_address(&config.bluna_collateral_contract)?,
                    send: vec![],
                    msg: querier::deposit_bluna_collateral(
                        &deps.api.human_address(&config.bluna_token_contract)?,
                        bonded.into(),
                    )?,
                }),
                CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: deps.api.human_address(&config.anchor_overseer_contract)?,
                    send: vec![],
                    msg: querier::overseer_lock_collateral(
                        &deps.api.human_address(&config.bluna_collateral_contract)?,
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
                log("action", "deposit"),
                log(
                    "deposited",
                    format!("{} {}", received.amount, received.denom),
                ),
                log("bonded asset", format!("{}", bonded)),
            ],
            data: Some(to_binary(&HandleAnswer::Deposit)?),
        })
    }
}

fn borrow<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
) -> StdResult<HandleResponse> {
    if env.message.sender != env.contract.address {
        return Err(StdError::unauthorized());
    }

    let config = config_get(&deps.storage)?;
    // TODO: not safe enough for use by several people, possible fix: to factor a contract per client
    let borrow_limit =
        querier::query_borrow_limit(deps, &config, &env.contract.address, Some(env.block.time))?;
    let already_borrowed =
        querier::query_loan_amount(deps, &config, &env.contract.address, Some(env.block.height))?;
    // 70% of 50% of TVL, borrow limit is at 50% of LTV, we use recommended 35%
    let borrow_amount = utils::calculate_borrow(borrow_limit, already_borrowed);

    Ok(HandleResponse {
        messages: vec![
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: deps.api.human_address(&config.anchor_market_contract)?,
                send: vec![],
                msg: querier::anchor_borrow(borrow_amount)?,
            }),
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: deps.api.human_address(&config.terraswap_luna_ust)?,
                send: vec![coin(borrow_amount.into(), TERRASWAP_PAIR)],
                msg: querier::swap_to_collateral(borrow_amount.into())?,
            }),
        ],
        log: vec![
            log("action", "borrow"),
            log("borrow_amount", borrow_amount.to_string()),
        ],
        data: Some(to_binary(&HandleAnswer::Borrow)?),
    })
}

#[cfg(test)]
mod tests {
    // TODO: Add test cases
}