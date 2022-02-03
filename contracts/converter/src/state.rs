use cosmwasm_std::Addr;
use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// ## Description
/// This structure describes the main control config of pair.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    /// the Lido contract addresses
    pub hub_addr: Addr,
    pub stluna_addr: Addr,
    pub bluna_addr: Addr,

    pub owner: Addr,
}

/// ## Description
/// Stores config at the given key
pub const CONFIG: Item<Config> = Item::new("config");

/// ## Description
/// Describes user's swap request for processing in reply handler
/// (<USER_ADDR>, <ASK_TOKEN_ADDR>)
pub type SwapRequest = (Addr, Addr);

/// ## Description
/// Stores addr of recipient who should get converted tokens and address of converted tokens contract
pub const SWAP_REQUEST: Item<SwapRequest> = Item::new("swap_recipient");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub hub_address: Addr,
    pub stluna_address: Addr,
    pub bluna_address: Addr,
    pub owner: Addr,
}
