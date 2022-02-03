use super::mock_querier::mock_dependencies as dependencies;
use crate::contract::{execute, instantiate, query_reverse_simulation, query_simulation};
use crate::msgs::InstantiateMsg;
use crate::testing::mock_querier::{
    MOCK_BLUNA_TOKEN_CONTRACT_ADDR, MOCK_HUB_CONTRACT_ADDR, MOCK_STLUNA_TOKEN_CONTRACT_ADDR,
};
use astroport::asset::{Asset, AssetInfo};
use astroport::pair::ExecuteMsg::Receive;
use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{
    to_binary, Addr, Api, CosmosMsg, OwnedDeps, Querier, Storage, Uint128, WasmMsg,
};
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};
use std::borrow::BorrowMut;

pub fn initialize<S: Storage, A: Api, Q: Querier>(deps: &mut OwnedDeps<S, A, Q>) {
    let msg = InstantiateMsg {
        stluna_address: Addr::unchecked(MOCK_STLUNA_TOKEN_CONTRACT_ADDR),
        bluna_address: Addr::unchecked(MOCK_BLUNA_TOKEN_CONTRACT_ADDR),
        hub_address: Addr::unchecked(MOCK_HUB_CONTRACT_ADDR),
    };

    let owner_info = mock_info("owner", &[]);
    instantiate(deps.as_mut(), mock_env(), owner_info, msg).unwrap();
}

#[test]
fn proper_swap_stluna_bluna() {
    let mut deps = dependencies(&[]);

    initialize(deps.borrow_mut());

    let sender = "addr";
    let amount = Uint128::from(100u128);

    let stluna_info = mock_info(MOCK_STLUNA_TOKEN_CONTRACT_ADDR, &[]);
    let swap = astroport::pair::Cw20HookMsg::Swap {
        belief_price: None,
        max_spread: None,
        to: None,
    };
    let receive = Receive(Cw20ReceiveMsg {
        sender: sender.to_string(),
        amount,
        msg: to_binary(&swap).unwrap(),
    });
    let res = execute(deps.as_mut(), mock_env(), stluna_info, receive).unwrap();
    assert_eq!(1, res.messages.len());

    let msg = &res.messages[0];
    match msg.msg.clone() {
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr,
            msg,
            funds: _,
        }) => {
            assert_eq!(contract_addr, MOCK_STLUNA_TOKEN_CONTRACT_ADDR);
            assert_eq!(
                msg,
                to_binary(&Cw20ExecuteMsg::Send {
                    contract: MOCK_HUB_CONTRACT_ADDR.to_string(),
                    amount,
                    msg: to_binary(&basset::hub::Cw20HookMsg::Convert {}).unwrap()
                })
                .unwrap()
            );
        }
        _ => panic!("Unexpected message: {:?}", msg),
    }
}

#[test]
fn proper_swap_bluna_stluna() {
    let mut deps = dependencies(&[]);

    initialize(deps.borrow_mut());

    let sender = "addr";
    let amount = Uint128::from(100u128);

    let stluna_info = mock_info(MOCK_BLUNA_TOKEN_CONTRACT_ADDR, &[]);
    let swap = astroport::pair::Cw20HookMsg::Swap {
        belief_price: None,
        max_spread: None,
        to: None,
    };
    let receive = Receive(Cw20ReceiveMsg {
        sender: sender.to_string(),
        amount,
        msg: to_binary(&swap).unwrap(),
    });
    let res = execute(deps.as_mut(), mock_env(), stluna_info, receive).unwrap();
    assert_eq!(1, res.messages.len());

    let msg = &res.messages[0];
    match msg.msg.clone() {
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr,
            msg,
            funds: _,
        }) => {
            assert_eq!(contract_addr, MOCK_BLUNA_TOKEN_CONTRACT_ADDR);
            assert_eq!(
                msg,
                to_binary(&Cw20ExecuteMsg::Send {
                    contract: MOCK_HUB_CONTRACT_ADDR.to_string(),
                    amount,
                    msg: to_binary(&basset::hub::Cw20HookMsg::Convert {}).unwrap()
                })
                .unwrap()
            );
        }
        _ => panic!("Unexpected message: {:?}", msg),
    }
}

#[test]
fn proper_simulation_query() {
    let mut deps = dependencies(&[]);

    initialize(deps.borrow_mut());

    let bluna_amount = Uint128::from(150u128);
    let expected_return_stluna_amount = Uint128::from(90u128);
    let simulation_response = query_simulation(
        deps.as_ref(),
        Asset {
            info: AssetInfo::Token {
                contract_addr: Addr::unchecked(MOCK_BLUNA_TOKEN_CONTRACT_ADDR),
            },
            amount: bluna_amount,
        },
    )
    .unwrap();
    assert_eq!(
        expected_return_stluna_amount,
        simulation_response.return_amount
    );

    let stluna_amount = Uint128::from(100u128);
    let expected_return_bluna_amount = Uint128::from(150u128);
    let simulation_response = query_simulation(
        deps.as_ref(),
        Asset {
            info: AssetInfo::Token {
                contract_addr: Addr::unchecked(MOCK_STLUNA_TOKEN_CONTRACT_ADDR),
            },
            amount: stluna_amount,
        },
    )
    .unwrap();
    assert_eq!(
        expected_return_bluna_amount,
        simulation_response.return_amount
    )
}

#[test]
fn proper_reverse_simulation_query() {
    let mut deps = dependencies(&[]);

    initialize(deps.borrow_mut());

    let bluna_amount = Uint128::from(150u128);
    let expected_offer_stluna_amount = Uint128::from(99u128); // ~100
    let simulation_response = query_reverse_simulation(
        deps.as_ref(),
        Asset {
            info: AssetInfo::Token {
                contract_addr: Addr::unchecked(MOCK_BLUNA_TOKEN_CONTRACT_ADDR),
            },
            amount: bluna_amount,
        },
    )
    .unwrap();
    assert_eq!(
        expected_offer_stluna_amount,
        simulation_response.offer_amount
    );

    let stluna_amount = Uint128::from(90u128);
    let expected_offer_bluna_amount = Uint128::from(149u128); // ~150
    let simulation_response = query_reverse_simulation(
        deps.as_ref(),
        Asset {
            info: AssetInfo::Token {
                contract_addr: Addr::unchecked(MOCK_STLUNA_TOKEN_CONTRACT_ADDR),
            },
            amount: stluna_amount,
        },
    )
    .unwrap();
    assert_eq!(
        expected_offer_bluna_amount,
        simulation_response.offer_amount
    )
}
