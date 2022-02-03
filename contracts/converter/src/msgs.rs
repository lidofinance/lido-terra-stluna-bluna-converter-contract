use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;

/// ## Description
/// This structure describes the basic settings for creating a contract.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    /// the Lido Terra token addresses
    pub stluna_address: Addr,
    pub bluna_address: Addr,

    /// the Lido Terra Hub address
    pub hub_address: Addr,
}
