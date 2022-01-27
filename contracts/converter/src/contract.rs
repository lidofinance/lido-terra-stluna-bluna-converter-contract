use crate::error::ContractError;
use crate::state::{Config, ConfigResponse, CONFIG};

use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{
    attr, entry_point, from_binary, to_binary, Addr, Binary, Coin, CosmosMsg, Decimal, Deps,
    DepsMut, Env, MessageInfo, Reply, ReplyOn, Response, StdError, StdResult, SubMsg, Uint128,
    WasmMsg,
};

use crate::error::ContractError::NonSupported;
use crate::msgs::InstantiateMsg;
use astroport::asset::{addr_validate_to_lower, format_lp_token_name, Asset, AssetInfo, PairInfo};
use astroport::factory::PairType;
use astroport::generator::Cw20HookMsg as GeneratorHookMsg;
use astroport::pair::{
    CumulativePricesResponse, Cw20HookMsg, ExecuteMsg, MigrateMsg, PoolResponse, QueryMsg,
    ReverseSimulationResponse, SimulationResponse, TWAP_PRECISION,
};
use astroport::pair::{DEFAULT_SLIPPAGE, MAX_ALLOWED_SLIPPAGE};
use basset::hub::Cw20HookMsg as HubCw20HookMsg;
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg, MinterResponse};
use std::str::FromStr;
use std::vec;

/// ## Description
/// Creates a new contract with the specified parameters in the [`InstantiateMsg`].
/// Returns the [`Response`] with the specified attributes if the operation was successful, or a [`ContractError`] if the contract was not created
/// ## Params
/// * **deps** is the object of type [`DepsMut`].
///
/// * **env** is the object of type [`Env`].
///
/// * **_info** is the object of type [`MessageInfo`].
/// * **msg** is a message of type [`InstantiateMsg`] which contains the basic settings for creating a contract
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    msg.asset_infos[0].check(deps.api)?;
    msg.asset_infos[1].check(deps.api)?;

    if msg.asset_infos[0] == msg.asset_infos[1] {
        return Err(ContractError::DoublingAssets {});
    }

    let config = Config {
        pair_info: PairInfo {
            contract_addr: env.contract.address.clone(),
            liquidity_token: Addr::unchecked(""),
            asset_infos: msg.asset_infos.clone(),
            pair_type: PairType::Xyk {},
        },
        hub_addr: addr_validate_to_lower(deps.api, msg.hub_address.as_str())?,
    };

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new())
}

/// ## Description
/// Available the execute messages of the contract.
/// ## Params
/// * **deps** is the object of type [`Deps`].
///
/// * **env** is the object of type [`Env`].
///
/// * **info** is the object of type [`MessageInfo`].
///
/// * **msg** is the object of type [`ExecuteMsg`].
///
/// ## Queries
/// * **ExecuteMsg::UpdateConfig { params: Binary }** Not supported.
///
/// * **ExecuteMsg::Receive(msg)** Receives a message of type [`Cw20ReceiveMsg`] and processes
/// it depending on the received template.
///
/// * **ExecuteMsg::ProvideLiquidity {
///             assets,
///             slippage_tolerance,
///             auto_stake,
///             receiver,
///         }** Provides liquidity with the specified input parameters.
///
/// * **ExecuteMsg::Swap {
///             offer_asset,
///             belief_price,
///             max_spread,
///             to,
///         }** Performs an swap operation with the specified parameters.
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdateConfig { .. } => Err(ContractError::NonSupported {}),
        ExecuteMsg::Receive(msg) => receive_cw20(deps, env, info, msg),
        ExecuteMsg::ProvideLiquidity {
            assets,
            slippage_tolerance,
            auto_stake,
            receiver,
        } => Err(ContractError::NonSupported {}),
        ExecuteMsg::Swap {
            offer_asset,
            belief_price,
            max_spread,
            to,
        } => {
            return Err(ContractError::Unauthorized {});
        }
    }
}

/// ## Description
/// Receives a message of type [`Cw20ReceiveMsg`] and processes it depending on the received template.
/// If the template is not found in the received message, then an [`ContractError`] is returned,
/// otherwise returns the [`Response`] with the specified attributes if the operation was successful
/// ## Params
/// * **deps** is the object of type [`DepsMut`].
///
/// * **env** is the object of type [`Env`].
///
/// * **info** is the object of type [`MessageInfo`].
///
/// * **cw20_msg** is the object of type [`Cw20ReceiveMsg`].
pub fn receive_cw20(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    let contract_addr = info.sender.clone();
    match from_binary(&cw20_msg.msg) {
        Ok(Cw20HookMsg::Swap {
            belief_price,
            max_spread,
            to,
        }) => {
            // only asset contract can execute this message
            let mut authorized: bool = false;
            let config: Config = CONFIG.load(deps.storage)?;

            for pool in config.pair_info.asset_infos {
                if let AssetInfo::Token { contract_addr, .. } = &pool {
                    if contract_addr == &info.sender {
                        authorized = true;
                    }
                }
            }

            if !authorized {
                return Err(ContractError::Unauthorized {});
            }

            let to_addr = if let Some(to_addr) = to {
                Some(addr_validate_to_lower(deps.api, to_addr.as_str())?)
            } else {
                None
            };

            swap(
                deps,
                env,
                info,
                Addr::unchecked(cw20_msg.sender),
                Asset {
                    info: AssetInfo::Token { contract_addr },
                    amount: cw20_msg.amount,
                },
                belief_price,
                max_spread,
                to_addr,
            )
        }
        Ok(Cw20HookMsg::WithdrawLiquidity {}) => Err(ContractError::NonSupported {}),
        Err(err) => Err(ContractError::Std(err)),
    }
}

/// ## Description
/// Performs an swap operation with the specified parameters. CONTRACT - a user must do token approval.
/// Returns an [`ContractError`] on failure, otherwise returns the [`Response`] with the specified attributes if the operation was successful.
/// ## Params
/// * **deps** is the object of type [`DepsMut`].
///
/// * **env** is the object of type [`Env`].
///
/// * **info** is the object of type [`MessageInfo`].
///
/// * **sender** is the object of type [`Addr`]. Sets the default recipient of the swap operation.
///
/// * **offer_asset** is the object of type [`Asset`]. Proposed asset for swapping.
///
/// * **belief_price** is the object of type [`Option<Decimal>`]. Used to calculate the maximum spread.
///
/// * **max_spread** is the object of type [`Option<Decimal>`]. Sets the maximum spread of the swap operation.
///
/// * **to** is the object of type [`Option<Addr>`]. Sets the recipient of the swap operation.
#[allow(clippy::too_many_arguments)]
pub fn swap(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _sender: Addr,
    offer_asset: Asset,
    _belief_price: Option<Decimal>,
    _max_spread: Option<Decimal>,
    _to: Option<Addr>,
) -> Result<Response, ContractError> {
    let config: Config = CONFIG.load(deps.storage)?;

    let token_addr = if let AssetInfo::Token { contract_addr } = offer_asset.info {
        contract_addr
    } else {
        return Err(ContractError::NonSupported {});
    };

    let convert_message = HubCw20HookMsg::Convert {};

    let msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: token_addr.to_string(),
        msg: to_binary(&cw20::Cw20ExecuteMsg::Send {
            contract: config.hub_addr.to_string(),
            amount: offer_asset.amount,
            msg: to_binary(&convert_message)?,
        })?,
        funds: vec![],
    });

    Ok(Response::new().add_message(msg))
}

/// ## Description
/// Available the query messages of the contract.
/// ## Params
/// * **deps** is the object of type [`Deps`].
///
/// * **_env** is the object of type [`Env`].
///
/// * **msg** is the object of type [`QueryMsg`].
///
/// ## Queries
/// * **QueryMsg::Pair {}** Returns information about a pair in an object of type [`PairInfo`].
///
/// * **QueryMsg::Pool {}** Returns information about a pool in an object of type [`PoolResponse`].
///
/// * **QueryMsg::Share { amount }** Returns information about the share of the pool in a vector
/// that contains objects of type [`Asset`].
///
/// * **QueryMsg::Simulation { offer_asset }** Returns information about the simulation of the
/// swap in a [`SimulationResponse`] object.
///
/// * **QueryMsg::ReverseSimulation { ask_asset }** Returns information about the reverse simulation
/// in a [`ReverseSimulationResponse`] object.
///
/// * **QueryMsg::CumulativePrices {}** Returns information about the cumulative prices in a
/// [`CumulativePricesResponse`] object.
///
/// * **QueryMsg::Config {}** Returns information about the controls settings in a
/// [`ConfigResponse`] object.
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Pair {} => to_binary(&query_pair_info(deps)?),
        QueryMsg::Pool {} => to_binary(&query_pool(deps)?),
        QueryMsg::Share { amount } => to_binary(&query_share(deps, amount)?),
        QueryMsg::Simulation { offer_asset } => to_binary(&query_simulation(deps, offer_asset)?),
        QueryMsg::ReverseSimulation { ask_asset } => {
            to_binary(&query_reverse_simulation(deps, ask_asset)?)
        }
        QueryMsg::CumulativePrices {} => to_binary(&query_cumulative_prices(deps, env)?),
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
    }
}

/// ## Description
/// Returns information about a pair in an object of type [`PairInfo`].
/// ## Params
/// * **deps** is the object of type [`Deps`].
pub fn query_pair_info(deps: Deps) -> StdResult<PairInfo> {
    let config: Config = CONFIG.load(deps.storage)?;
    Ok(config.pair_info)
}

/// ## Description
/// Returns information about a pool in an object of type [`PoolResponse`].
/// ## Params
/// * **deps** is the object of type [`Deps`].
pub fn query_pool(deps: Deps) -> StdResult<PoolResponse> {
    let config: Config = CONFIG.load(deps.storage)?;
    let (assets, total_share) = pool_info(deps, config)?;

    let resp = PoolResponse {
        assets,
        total_share,
    };

    Ok(resp)
}

/// ## Description
/// Returns information about the share of the pool in a vector that contains objects of type [`Asset`].
/// ## Params
/// * **deps** is the object of type [`Deps`].
///
/// * **amount** is the object of type [`Uint128`]. Sets the amount for which a share in the pool will be requested.
pub fn query_share(deps: Deps, amount: Uint128) -> StdResult<Vec<Asset>> {
    Err(StdError::generic_err("not supported"))
}

/// ## Description
/// Returns information about the simulation of the swap in a [`SimulationResponse`] object.
/// ## Params
/// * **deps** is the object of type [`Deps`].
///
/// * **offer_asset** is the object of type [`Asset`].
pub fn query_simulation(deps: Deps, offer_asset: Asset) -> StdResult<SimulationResponse> {
    Err(StdError::generic_err("not supported"))
}

/// ## Description
/// Returns information about the reverse simulation in a [`ReverseSimulationResponse`] object.
/// ## Params
/// * **deps** is the object of type [`Deps`].
///
/// * **ask_asset** is the object of type [`Asset`].
pub fn query_reverse_simulation(
    deps: Deps,
    ask_asset: Asset,
) -> StdResult<ReverseSimulationResponse> {
    Err(StdError::generic_err("not supported"))
}

/// ## Description
/// Returns information about the cumulative prices in a [`CumulativePricesResponse`] object.
/// ## Params
/// * **deps** is the object of type [`Deps`].
///
/// * **env** is the object of type [`Env`].
pub fn query_cumulative_prices(deps: Deps, env: Env) -> StdResult<CumulativePricesResponse> {
    Err(StdError::generic_err("not supported"))
}

/// ## Description
/// Returns information about the controls settings in a [`ConfigResponse`] object.
/// ## Params
/// * **deps** is the object of type [`Deps`].
pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config: Config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        hub_addr: config.hub_addr,
    })
}

/// ## Description
/// Returns an amount in the coin if the coin is found, otherwise returns [`zero`].
/// ## Params
/// * **coins** are an array of [`Coin`] type items. Sets the list of coins.
///
/// * **denom** is the object of type [`String`]. Sets the name of coin.
pub fn amount_of(coins: &[Coin], denom: String) -> Uint128 {
    match coins.iter().find(|x| x.denom == denom) {
        Some(coin) => coin.amount,
        None => Uint128::zero(),
    }
}

/// ## Description
/// Used for migration of contract. Returns the default object of type [`Response`].
/// ## Params
/// * **_deps** is the object of type [`DepsMut`].
///
/// * **_env** is the object of type [`Env`].
///
/// * **_msg** is the object of type [`MigrateMsg`].
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    Ok(Response::default())
}

/// ## Description
/// Returns information about the pool.
/// ## Params
/// * **deps** is the object of type [`Deps`].
///
/// * **config** is the object of type [`Config`].
pub fn pool_info(deps: Deps, config: Config) -> StdResult<([Asset; 2], Uint128)> {
    Err(StdError::generic_err("not supported"))
}
