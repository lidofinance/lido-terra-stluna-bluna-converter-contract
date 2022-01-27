use std::ops::Add;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use astroport::asset::{Asset, AssetInfo};

use cosmwasm_std::{Addr, Binary, Decimal, Uint128};
use cw20::Cw20ReceiveMsg;

/// ## Description
/// This structure describes the basic settings for creating a contract.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    /// the type of asset infos available in [`AssetInfo`]
    pub asset_infos: [AssetInfo; 2],

    /// the Lido Terra Hub address
    pub hub_address: Addr
}