use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;

/// ## Description
/// This structure describes the basic settings for creating a contract.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    /// the Lido Terra token addresses
    pub stluna_addr: Addr,
    pub bluna_addr: Addr,

    /// the Lido Terra Hub address
    pub hub_address: Addr,
}
