use super::mock_querier::mock_dependencies_custom;
use crate::contract::{execute, get_tax_deducted_funds, instantiate, query};
use andromeda_protocol::{
    error::ContractError,
    mirror_wrapped_cdp::{
        ConfigResponse, Cw20HookMsg, ExecuteMsg, InstantiateMsg, MirrorGovCw20HookMsg,
        MirrorGovExecuteMsg, MirrorLockExecuteMsg, MirrorMintCw20HookMsg, MirrorMintExecuteMsg,
        MirrorStakingCw20HookMsg, MirrorStakingExecuteMsg, QueryMsg,
    },
    operators::IsOperatorResponse,
};
use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{
    coin, coins, from_binary, to_binary, Binary, CosmosMsg, Decimal, Deps, DepsMut, MessageInfo,
    Response, Uint128, WasmMsg,
};
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};
use mirror_protocol::gov::VoteOption;
use serde::de::DeserializeOwned;
use std::fmt::Debug;
use terraswap::asset::{Asset, AssetInfo};

const TEST_TOKEN: &str = "TEST_TOKEN";
const TEST_AMOUNT: u128 = 100u128;
const MOCK_MIRROR_MINT_ADDR: &str = "mirror_mint";
const MOCK_MIRROR_STAKING_ADDR: &str = "mirror_staking";
const MOCK_MIRROR_GOV_ADDR: &str = "mirror_gov";
const MOCK_MIRROR_LOCK_ADDR: &str = "mirror_lock";

fn assert_mint_execute_msg(deps: DepsMut, info: MessageInfo, mirror_msg: MirrorMintExecuteMsg) {
    let msg = ExecuteMsg::MirrorMintExecuteMsg(mirror_msg.clone());
    assert_execute_msg(
        deps,
        info,
        msg,
        to_binary(&mirror_msg).unwrap(),
        MOCK_MIRROR_MINT_ADDR.to_string(),
    );
}

fn assert_mint_execute_cw20_msg(
    deps: DepsMut,
    info: MessageInfo,
    mirror_msg: MirrorMintCw20HookMsg,
) {
    let msg = Cw20HookMsg::MirrorMintCw20HookMsg(mirror_msg.clone());
    assert_execute_cw20_msg(
        deps,
        info,
        msg,
        to_binary(&mirror_msg).unwrap(),
        MOCK_MIRROR_MINT_ADDR.to_string(),
    );
}

fn assert_staking_execute_msg(
    deps: DepsMut,
    info: MessageInfo,
    mirror_msg: MirrorStakingExecuteMsg,
) {
    let msg = ExecuteMsg::MirrorStakingExecuteMsg(mirror_msg.clone());
    assert_execute_msg(
        deps,
        info,
        msg,
        to_binary(&mirror_msg).unwrap(),
        MOCK_MIRROR_STAKING_ADDR.to_string(),
    );
}

fn assert_staking_execute_cw20_msg(
    deps: DepsMut,
    info: MessageInfo,
    mirror_msg: MirrorStakingCw20HookMsg,
) {
    let msg = Cw20HookMsg::MirrorStakingCw20HookMsg(mirror_msg.clone());
    assert_execute_cw20_msg(
        deps,
        info,
        msg,
        to_binary(&mirror_msg).unwrap(),
        MOCK_MIRROR_STAKING_ADDR.to_string(),
    );
}

fn assert_gov_execute_msg(deps: DepsMut, info: MessageInfo, mirror_msg: MirrorGovExecuteMsg) {
    let msg = ExecuteMsg::MirrorGovExecuteMsg(mirror_msg.clone());
    assert_execute_msg(
        deps,
        info,
        msg,
        to_binary(&mirror_msg).unwrap(),
        MOCK_MIRROR_GOV_ADDR.to_string(),
    );
}

fn assert_gov_execute_cw20_msg(deps: DepsMut, info: MessageInfo, mirror_msg: MirrorGovCw20HookMsg) {
    let msg = Cw20HookMsg::MirrorGovCw20HookMsg(mirror_msg.clone());
    assert_execute_cw20_msg(
        deps,
        info,
        msg,
        to_binary(&mirror_msg).unwrap(),
        MOCK_MIRROR_GOV_ADDR.to_string(),
    );
}

fn assert_lock_execute_msg(deps: DepsMut, info: MessageInfo, mirror_msg: MirrorLockExecuteMsg) {
    let msg = ExecuteMsg::MirrorLockExecuteMsg(mirror_msg.clone());
    assert_execute_msg(
        deps,
        info,
        msg,
        to_binary(&mirror_msg).unwrap(),
        MOCK_MIRROR_LOCK_ADDR.to_string(),
    );
}

fn assert_execute_msg(
    deps: DepsMut,
    info: MessageInfo,
    msg: ExecuteMsg,
    mirror_msg_binary: Binary,
    contract_addr: String,
) {
    let tax_deducted_funds = get_tax_deducted_funds(&deps, info.funds.clone()).unwrap();
    let res = execute(deps, mock_env(), info, msg).unwrap();
    let execute_msg = WasmMsg::Execute {
        contract_addr,
        funds: tax_deducted_funds,
        msg: mirror_msg_binary,
    };
    assert_eq!(
        Response::new().add_messages(vec![CosmosMsg::Wasm(execute_msg)]),
        res
    );
}

fn assert_execute_cw20_msg(
    deps: DepsMut,
    info: MessageInfo,
    cw20_hook_msg: Cw20HookMsg,
    mirror_msg_binary: Binary,
    contract_addr: String,
) {
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: info.sender.to_string(),
        amount: Uint128::from(TEST_AMOUNT),
        msg: to_binary(&cw20_hook_msg).unwrap(),
    });
    let res = execute(deps, mock_env(), mock_info(TEST_TOKEN, &[]), msg).unwrap();
    let send_msg = Cw20ExecuteMsg::Send {
        contract: contract_addr,
        amount: Uint128::from(TEST_AMOUNT),
        msg: mirror_msg_binary,
    };
    let execute_msg = WasmMsg::Execute {
        contract_addr: TEST_TOKEN.to_string(),
        funds: vec![],
        msg: to_binary(&send_msg).unwrap(),
    };
    assert_eq!(
        Response::new().add_messages(vec![CosmosMsg::Wasm(execute_msg)]),
        res
    );
}

fn assert_query_msg<T: DeserializeOwned + Debug + PartialEq>(
    deps: Deps,
    msg: QueryMsg,
    expected_res: T,
) {
    let actual_res: T = from_binary(&query(deps, mock_env(), msg).unwrap()).unwrap();
    assert_eq!(expected_res, actual_res);
}

fn assert_intantiate(deps: DepsMut, info: MessageInfo) {
    let msg = InstantiateMsg {
        mirror_mint_contract: MOCK_MIRROR_MINT_ADDR.to_string(),
        mirror_staking_contract: MOCK_MIRROR_STAKING_ADDR.to_string(),
        mirror_gov_contract: MOCK_MIRROR_GOV_ADDR.to_string(),
        mirror_lock_contract: MOCK_MIRROR_LOCK_ADDR.to_string(),
        operators: None,
    };
    let res = instantiate(deps, mock_env(), info.clone(), msg).unwrap();
    assert_eq!(
        Response::new()
            .add_attribute("method", "instantiate")
            .add_attribute("owner", info.sender),
        res
    );
}

#[test]
fn test_instantiate() {
    let mut deps = mock_dependencies_custom(&[]);
    let info = mock_info("creator", &[]);
    assert_intantiate(deps.as_mut(), info);

    // Verify that we can query our contract's config.
    let msg = QueryMsg::Config {};
    assert_query_msg(
        deps.as_ref(),
        msg,
        ConfigResponse {
            mirror_mint_contract: MOCK_MIRROR_MINT_ADDR.to_string(),
            mirror_staking_contract: MOCK_MIRROR_STAKING_ADDR.to_string(),
            mirror_gov_contract: MOCK_MIRROR_GOV_ADDR.to_string(),
            mirror_lock_contract: MOCK_MIRROR_LOCK_ADDR.to_string(),
        },
    );
}

#[test]
fn test_instantiate_with_operator() {
    let mut deps = mock_dependencies_custom(&[]);
    let info = mock_info("creator", &[]);
    let operator = mock_info("operator", &[]);
    let msg = InstantiateMsg {
        mirror_mint_contract: MOCK_MIRROR_MINT_ADDR.to_string(),
        mirror_staking_contract: MOCK_MIRROR_STAKING_ADDR.to_string(),
        mirror_gov_contract: MOCK_MIRROR_GOV_ADDR.to_string(),
        mirror_lock_contract: MOCK_MIRROR_LOCK_ADDR.to_string(),
        operators: Some(vec![operator.sender.to_string()]),
    };
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    assert_query_msg(
        deps.as_ref(),
        QueryMsg::IsOperator {
            address: operator.sender.to_string(),
        },
        IsOperatorResponse { is_operator: true },
    );
}

#[test]
fn test_mirror_mint_open_position() {
    let mut deps = mock_dependencies_custom(&[]);
    let info = mock_info("creator", &[]);
    assert_intantiate(deps.as_mut(), info.clone());

    let mirror_msg = MirrorMintExecuteMsg::OpenPosition {
        collateral: Asset {
            info: AssetInfo::NativeToken {
                denom: "uusd".to_string(),
            },
            amount: Uint128::from(10_u128),
        },
        asset_info: AssetInfo::Token {
            contract_addr: "token_address".to_string(),
        },
        collateral_ratio: Decimal::one(),
        short_params: None,
    };
    assert_mint_execute_msg(deps.as_mut(), info, mirror_msg);
}

#[test]
fn test_mirror_mint_deposit() {
    let mut deps = mock_dependencies_custom(&[]);
    deps.querier.with_tax(
        Decimal::percent(10),
        &[(&"uusd".to_string(), &Uint128::from(1500000u128))],
    );
    let info = mock_info("creator", &coins(10u128, "uusd"));
    assert_intantiate(deps.as_mut(), info.clone());

    let mirror_msg = MirrorMintExecuteMsg::Deposit {
        collateral: Asset {
            info: AssetInfo::NativeToken {
                denom: "uusd".to_string(),
            },
            amount: Uint128::from(10u128),
        },
        position_idx: Uint128::from(1u128),
    };

    assert_mint_execute_msg(deps.as_mut(), info, mirror_msg);
}

#[test]
fn test_mirror_mint_withdraw() {
    let mut deps = mock_dependencies_custom(&[]);
    let info = mock_info("creator", &[]);
    assert_intantiate(deps.as_mut(), info.clone());

    let operator = mock_info("operator", &[]);
    let _res = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::UpdateOperators {
            operators: vec![operator.sender.to_string()],
        },
    )
    .unwrap();

    let mirror_msg = MirrorMintExecuteMsg::Withdraw {
        position_idx: Uint128::from(1_u128),
        collateral: None,
    };

    assert_mint_execute_msg(deps.as_mut(), operator, mirror_msg);
}

#[test]
fn test_mirror_mint_mint() {
    let mut deps = mock_dependencies_custom(&[]);
    let info = mock_info("creator", &[]);
    assert_intantiate(deps.as_mut(), info.clone());

    let mirror_msg = MirrorMintExecuteMsg::Mint {
        asset: Asset {
            info: AssetInfo::NativeToken {
                denom: "uusd".to_string(),
            },
            amount: Uint128::from(10_u128),
        },
        position_idx: Uint128::from(1_u128),
        short_params: None,
    };

    assert_mint_execute_msg(deps.as_mut(), info, mirror_msg);
}

#[test]
fn test_mirror_mint_open_position_cw20() {
    let mut deps = mock_dependencies_custom(&[]);
    let info = mock_info("creator", &[]);
    assert_intantiate(deps.as_mut(), info.clone());

    let mirror_msg = MirrorMintCw20HookMsg::OpenPosition {
        asset_info: AssetInfo::Token {
            contract_addr: TEST_TOKEN.to_string(),
        },
        collateral_ratio: Decimal::one(),
        short_params: None,
    };

    assert_mint_execute_cw20_msg(deps.as_mut(), info, mirror_msg);
}

#[test]
fn test_mirror_mint_deposit_cw20() {
    let mut deps = mock_dependencies_custom(&[]);
    let info = mock_info("creator", &[]);
    assert_intantiate(deps.as_mut(), info.clone());

    let operator = mock_info("operator", &[]);
    let _res = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::UpdateOperators {
            operators: vec![operator.sender.to_string()],
        },
    )
    .unwrap();

    let mirror_msg = MirrorMintCw20HookMsg::Deposit {
        position_idx: Uint128::from(1u128),
    };

    assert_mint_execute_cw20_msg(deps.as_mut(), operator, mirror_msg);
}

#[test]
fn test_mirror_mint_burn_cw20() {
    let mut deps = mock_dependencies_custom(&[]);
    let info = mock_info("creator", &[]);
    assert_intantiate(deps.as_mut(), info.clone());

    let mirror_msg = MirrorMintCw20HookMsg::Burn {
        position_idx: Uint128::from(1u128),
    };

    assert_mint_execute_cw20_msg(deps.as_mut(), info, mirror_msg);
}

#[test]
fn test_mirror_mint_auction_cw20() {
    let mut deps = mock_dependencies_custom(&[]);
    let info = mock_info("creator", &[]);
    assert_intantiate(deps.as_mut(), info.clone());

    let mirror_msg = MirrorMintCw20HookMsg::Auction {
        position_idx: Uint128::from(1u128),
    };

    assert_mint_execute_cw20_msg(deps.as_mut(), info, mirror_msg);
}

#[test]
fn test_mirror_staking_unbond() {
    let mut deps = mock_dependencies_custom(&[]);
    let info = mock_info("creator", &[]);
    assert_intantiate(deps.as_mut(), info.clone());

    let mirror_msg = MirrorStakingExecuteMsg::Unbond {
        asset_token: "asset_token".to_string(),
        amount: Uint128::from(1_u128),
    };

    assert_staking_execute_msg(deps.as_mut(), info, mirror_msg);
}

#[test]
fn test_mirror_staking_withdraw() {
    let mut deps = mock_dependencies_custom(&[]);
    let info = mock_info("creator", &[]);
    assert_intantiate(deps.as_mut(), info.clone());

    let mirror_msg = MirrorStakingExecuteMsg::Withdraw { asset_token: None };

    assert_staking_execute_msg(deps.as_mut(), info, mirror_msg);
}

#[test]
fn test_mirror_staking_autostake() {
    let mut deps = mock_dependencies_custom(&[]);
    let info = mock_info("creator", &[]);
    assert_intantiate(deps.as_mut(), info.clone());

    let mirror_msg = MirrorStakingExecuteMsg::AutoStake {
        assets: [
            Asset {
                info: AssetInfo::NativeToken {
                    denom: "uusd".to_string(),
                },
                amount: Uint128::from(10_u128),
            },
            Asset {
                info: AssetInfo::NativeToken {
                    denom: "uusd".to_string(),
                },
                amount: Uint128::from(10_u128),
            },
        ],
        slippage_tolerance: None,
    };

    assert_staking_execute_msg(deps.as_mut(), info, mirror_msg);
}

#[test]
fn test_mirror_staking_bond_cw20() {
    let mut deps = mock_dependencies_custom(&[]);
    let info = mock_info("creator", &[]);
    assert_intantiate(deps.as_mut(), info.clone());

    let mirror_msg = MirrorStakingCw20HookMsg::Bond {
        asset_token: TEST_TOKEN.to_string(),
    };

    assert_staking_execute_cw20_msg(deps.as_mut(), info, mirror_msg);
}

#[test]
fn test_mirror_gov_castvote() {
    let mut deps = mock_dependencies_custom(&[]);
    let info = mock_info("creator", &[]);
    assert_intantiate(deps.as_mut(), info.clone());

    let mirror_msg = MirrorGovExecuteMsg::CastVote {
        poll_id: 1_u64,
        amount: Uint128::from(1_u128),
        vote: VoteOption::Yes,
    };

    assert_gov_execute_msg(deps.as_mut(), info, mirror_msg);
}

#[test]
fn test_mirror_gov_withdraw_voting_tokens() {
    let mut deps = mock_dependencies_custom(&[]);
    let info = mock_info("creator", &[]);
    assert_intantiate(deps.as_mut(), info.clone());

    let mirror_msg = MirrorGovExecuteMsg::WithdrawVotingTokens { amount: None };

    assert_gov_execute_msg(deps.as_mut(), info, mirror_msg);
}

#[test]
fn test_mirror_gov_withdraw_voting_rewards() {
    let mut deps = mock_dependencies_custom(&[]);
    let info = mock_info("creator", &[]);
    assert_intantiate(deps.as_mut(), info.clone());

    let mirror_msg = MirrorGovExecuteMsg::WithdrawVotingRewards { poll_id: None };

    assert_gov_execute_msg(deps.as_mut(), info, mirror_msg);
}

#[test]
fn test_mirror_gov_stake_voting_rewards() {
    let mut deps = mock_dependencies_custom(&[]);
    let info = mock_info("creator", &[]);
    assert_intantiate(deps.as_mut(), info.clone());

    let mirror_msg = MirrorGovExecuteMsg::StakeVotingRewards { poll_id: None };

    assert_gov_execute_msg(deps.as_mut(), info, mirror_msg);
}

#[test]
fn test_mirror_gov_end_poll() {
    let mut deps = mock_dependencies_custom(&[]);
    let info = mock_info("creator", &[]);
    assert_intantiate(deps.as_mut(), info.clone());

    let mirror_msg = MirrorGovExecuteMsg::EndPoll { poll_id: 1_u64 };

    assert_gov_execute_msg(deps.as_mut(), info, mirror_msg);
}

#[test]
fn test_mirror_gov_execute_poll() {
    let mut deps = mock_dependencies_custom(&[]);
    let info = mock_info("creator", &[]);
    assert_intantiate(deps.as_mut(), info.clone());

    let mirror_msg = MirrorGovExecuteMsg::ExecutePoll { poll_id: 1_u64 };

    assert_gov_execute_msg(deps.as_mut(), info, mirror_msg);
}

#[test]
fn test_mirror_gov_snapshot_poll() {
    let mut deps = mock_dependencies_custom(&[]);
    let info = mock_info("creator", &[]);
    assert_intantiate(deps.as_mut(), info.clone());

    let mirror_msg = MirrorGovExecuteMsg::SnapshotPoll { poll_id: 1_u64 };

    assert_gov_execute_msg(deps.as_mut(), info, mirror_msg);
}

#[test]
fn test_mirror_gov_stake_voting_tokens_cw20() {
    let mut deps = mock_dependencies_custom(&[]);
    let info = mock_info("creator", &[]);
    assert_intantiate(deps.as_mut(), info.clone());

    let mirror_msg = MirrorGovCw20HookMsg::StakeVotingTokens {};

    assert_gov_execute_cw20_msg(deps.as_mut(), info, mirror_msg);
}

#[test]
fn test_mirror_gov_create_poll_cw20() {
    let mut deps = mock_dependencies_custom(&[]);
    let info = mock_info("creator", &[]);
    assert_intantiate(deps.as_mut(), info.clone());

    let mirror_msg = MirrorGovCw20HookMsg::CreatePoll {
        title: "title".to_string(),
        description: "description".to_string(),
        link: None,
        execute_msg: None,
    };

    assert_gov_execute_cw20_msg(deps.as_mut(), info, mirror_msg);
}

#[test]
fn test_lock_unlock_position_funds() {
    let mut deps = mock_dependencies_custom(&[]);
    let info = mock_info("creator", &[]);
    assert_intantiate(deps.as_mut(), info.clone());

    let mirror_msg = MirrorLockExecuteMsg::UnlockPositionFunds {
        positions_idx: vec![Uint128::from(1u128)],
    };
    assert_lock_execute_msg(deps.as_mut(), info, mirror_msg);
}

#[test]
fn test_mirror_too_many_funds() {
    let mut deps = mock_dependencies_custom(&[]);
    let info = mock_info("creator", &[coin(1u128, "uusd"), coin(1u128, "uluna")]);
    assert_intantiate(deps.as_mut(), info.clone());
    let mirror_msg = MirrorMintExecuteMsg::OpenPosition {
        collateral: Asset {
            info: AssetInfo::NativeToken {
                denom: "uusd".to_string(),
            },
            amount: Uint128::from(10_u128),
        },
        asset_info: AssetInfo::Token {
            contract_addr: "token_address".to_string(),
        },
        collateral_ratio: Decimal::one(),
        short_params: None,
    };
    let res_err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::MirrorMintExecuteMsg(mirror_msg),
    )
    .unwrap_err();
    assert_eq!(
        ContractError::InvalidMirrorFunds {
            msg: "Mirror expects no funds or a single type of fund to be deposited.".to_string()
        },
        res_err
    );
}

#[test]
fn test_mirror_non_authorized_user() {
    let mut deps = mock_dependencies_custom(&[]);
    let info = mock_info("creator", &[]);
    assert_intantiate(deps.as_mut(), info);

    let unauth_user = mock_info("user", &[]);
    let mirror_msg = MirrorMintExecuteMsg::OpenPosition {
        collateral: Asset {
            info: AssetInfo::NativeToken {
                denom: "uusd".to_string(),
            },
            amount: Uint128::from(10_u128),
        },
        asset_info: AssetInfo::Token {
            contract_addr: "token_address".to_string(),
        },
        collateral_ratio: Decimal::one(),
        short_params: None,
    };
    let res_err = execute(
        deps.as_mut(),
        mock_env(),
        unauth_user,
        ExecuteMsg::MirrorMintExecuteMsg(mirror_msg),
    )
    .unwrap_err();
    assert_eq!(ContractError::Unauthorized {}, res_err);
}

#[test]
fn test_mirror_cw20_non_authorized_user() {
    let mut deps = mock_dependencies_custom(&[]);
    let info = mock_info("creator", &[]);
    assert_intantiate(deps.as_mut(), info);

    let unauth_user = mock_info("user", &[]);
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: unauth_user.sender.to_string(),
        amount: Uint128::from(TEST_AMOUNT),
        msg: to_binary(&Cw20HookMsg::MirrorMintCw20HookMsg(
            MirrorMintCw20HookMsg::Deposit {
                position_idx: Uint128::from(1u128),
            },
        ))
        .unwrap(),
    });
    let res_err = execute(deps.as_mut(), mock_env(), mock_info(TEST_TOKEN, &[]), msg).unwrap_err();
    assert_eq!(ContractError::Unauthorized {}, res_err);
}

#[test]
fn test_update_config() {
    let mut deps = mock_dependencies_custom(&[]);
    let info = mock_info("creator", &[]);
    assert_intantiate(deps.as_mut(), info.clone());

    let mirror_mint_contract = "new_mint".to_string();
    let mirror_staking_contract = "new_stake".to_string();
    let mirror_gov_contract = "new_gov".to_string();
    let mirror_lock_contract = "new_lock".to_string();

    let msg = ExecuteMsg::UpdateConfig {
        mirror_mint_contract: Some(mirror_mint_contract.clone()),
        mirror_staking_contract: Some(mirror_staking_contract.clone()),
        mirror_gov_contract: Some(mirror_gov_contract.clone()),
        mirror_lock_contract: Some(mirror_lock_contract.clone()),
    };
    let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    // Verify that config was updated.
    let msg = QueryMsg::Config {};
    assert_query_msg(
        deps.as_ref(),
        msg,
        ConfigResponse {
            mirror_mint_contract,
            mirror_staking_contract,
            mirror_gov_contract,
            mirror_lock_contract,
        },
    );
}