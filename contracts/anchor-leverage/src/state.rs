use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{CanonicalAddr, HumanAddr, StdResult, Storage};
use cosmwasm_storage::{singleton, ReadonlySingleton, Singleton};

pub static CONFIG_KEY: &[u8] = b"config";

/// Config struct
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub bluna_hub_contract: CanonicalAddr,
    pub bluna_token_contract: CanonicalAddr,
    pub bluna_collateral_contract: CanonicalAddr,
    pub anchor_overseer_contract: CanonicalAddr,
    pub anchor_market_contract: CanonicalAddr,
    pub terraswap_luna_ust: CanonicalAddr,
    pub preferred_validator: HumanAddr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigMsg {
    pub bluna_hub_contract: HumanAddr, //terra1fflas6wv4snv8lsda9knvq2w0cyt493r8puh2e
    pub bluna_token_contract: HumanAddr, //terra1ltnkx0mv7lf2rca9f8w740ashu93ujughy4s7p
    pub bluna_collateral_contract: HumanAddr, //terra1u0t35drzyy0mujj8rkdyzhe264uls4ug3wdp3x
    pub anchor_overseer_contract: HumanAddr, //terra1qljxd0y3j3gk97025qvl3lgq8ygup4gsksvaxv
    pub anchor_market_contract: HumanAddr, //terra15dwd5mj8v59wpj0wvt233mf5efdff808c5tkal
    pub terraswap_luna_ust: HumanAddr, //terra156v8s539wtz0sjpn8y8a8lfg8fhmwa7fy22aff
    pub preferred_validator: HumanAddr, //terravaloper1krj7amhhagjnyg2tkkuh6l0550y733jnjnnlzy
}

/// Config singleton initialization
pub fn config<S: Storage>(storage: &mut S) -> Singleton<S, Config> {
    singleton(storage, CONFIG_KEY)
}

/// Get config
pub fn config_get<S: Storage>(storage: &S) -> StdResult<Config> {
    ReadonlySingleton::new(storage, CONFIG_KEY).load()
}

/// Set config
pub fn config_set<S: Storage>(storage: &mut S, config: &Config) -> StdResult<()> {
    Singleton::new(storage, CONFIG_KEY).save(config)
}
