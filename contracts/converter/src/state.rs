// Copyright 2022 Astroport. Modified by Lido
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// ## Description
/// This structure describes the main control config of pair.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    /// The last time block
    pub block_time_last: u64,
    /// The last cumulative price 0 asset in pool
    pub price0_cumulative_last: Uint128,
    /// The last cumulative price 1 asset in pool
    pub price1_cumulative_last: Uint128,

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
    pub block_time_last: u64,
}
