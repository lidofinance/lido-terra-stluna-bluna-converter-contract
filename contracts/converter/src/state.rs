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
}

/// ## Description
/// Stores config at the given key
pub const CONFIG: Item<Config> = Item::new("config");

/// ## Description
/// Describes user's swap request for processing in reply handler
/// (<USER_ADDR>, <ASK_TOKEN_ADDR>)
pub type SwapRequest = (Addr, Addr);

/// ## Description
/// Stores addr of recipient who should get converted tokens
pub const SWAP_RECIPIENT: Item<SwapRequest> = Item::new("swap_recipient");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub hub_addr: Addr,
    pub stluna_addr: Addr,
    pub bluna_addr: Addr,
}
