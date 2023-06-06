#[cfg(test)]
use andromeda_std::testing::mock_querier::{mock_dependencies_custom, MOCK_KERNEL_CONTRACT};
use cosmwasm_std::{Addr, Uint128};
use cw_asset::AssetInfo;

use crate::contract::{execute, instantiate};
use crate::state::{ACTION_FEES, CODE_ID, PUBLISHER, VERSION_CODE_ID};

use andromeda_std::ado_contract::ADOContract;
use andromeda_std::error::ContractError;
use andromeda_std::os::adodb::{ActionFee, ExecuteMsg, InstantiateMsg};

use cosmwasm_std::{
    attr,
    testing::{mock_dependencies, mock_env, mock_info},
    Response,
};

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies();
    let info = mock_info("creator", &[]);
    let msg = InstantiateMsg {
        kernel_address: MOCK_KERNEL_CONTRACT.to_string(),
        owner: None,
    };
    let env = mock_env();

    let res = instantiate(deps.as_mut(), env, info, msg).unwrap();
    assert_eq!(0, res.messages.len());
}

#[test]
fn test_update_code_id() {
    let owner = String::from("owner");
    let mut deps = mock_dependencies_custom(&[]);
    let env = mock_env();
    let info = mock_info(owner.as_str(), &[]);

    instantiate(
        deps.as_mut(),
        mock_env(),
        mock_info(&owner, &[]),
        InstantiateMsg {
            kernel_address: MOCK_KERNEL_CONTRACT.to_string(),
            owner: None,
        },
    )
    .unwrap();

    let msg = ExecuteMsg::UpdateCodeId {
        code_id_key: "address_list".to_string(),
        code_id: 1u64,
    };

    let resp = execute(deps.as_mut(), env, info, msg).unwrap();

    let expected = Response::new().add_attributes(vec![
        attr("action", "add_update_code_id"),
        attr("code_id_key", "address_list"),
        attr("code_id", "1"),
    ]);

    assert_eq!(resp, expected);
}

#[test]
fn test_update_code_id_operator() {
    let owner = String::from("owner");
    let mut deps = mock_dependencies_custom(&[]);
    let env = mock_env();
    let info = mock_info(owner.as_str(), &[]);

    instantiate(
        deps.as_mut(),
        mock_env(),
        mock_info(&owner, &[]),
        InstantiateMsg {
            kernel_address: MOCK_KERNEL_CONTRACT.to_string(),
            owner: None,
        },
    )
    .unwrap();

    let operator = String::from("operator");
    ADOContract::default()
        .execute_update_operators(deps.as_mut(), info, vec![operator.clone()])
        .unwrap();

    let msg = ExecuteMsg::UpdateCodeId {
        code_id_key: "address_list".to_string(),
        code_id: 1u64,
    };

    let info = mock_info(&operator, &[]);
    let resp = execute(deps.as_mut(), env, info, msg).unwrap();

    let expected = Response::new().add_attributes(vec![
        attr("action", "add_update_code_id"),
        attr("code_id_key", "address_list"),
        attr("code_id", "1"),
    ]);

    assert_eq!(resp, expected);
}

#[test]
fn test_update_code_id_unauthorized() {
    let owner = String::from("owner");
    let mut deps = mock_dependencies_custom(&[]);
    let env = mock_env();

    instantiate(
        deps.as_mut(),
        mock_env(),
        mock_info(&owner, &[]),
        InstantiateMsg {
            kernel_address: MOCK_KERNEL_CONTRACT.to_string(),
            owner: None,
        },
    )
    .unwrap();

    let msg = ExecuteMsg::UpdateCodeId {
        code_id_key: "address_list".to_string(),
        code_id: 1u64,
    };

    let info = mock_info("not_owner", &[]);
    let resp = execute(deps.as_mut(), env, info, msg);

    assert_eq!(ContractError::Unauthorized {}, resp.unwrap_err());
}

#[test]
fn test_publish() {
    let owner = String::from("owner");
    let mut deps = mock_dependencies_custom(&[]);
    let env = mock_env();
    let info = mock_info(owner.as_str(), &[]);

    instantiate(
        deps.as_mut(),
        mock_env(),
        mock_info(&owner, &[]),
        InstantiateMsg {
            kernel_address: MOCK_KERNEL_CONTRACT.to_string(),
            owner: None,
        },
    )
    .unwrap();

    let action_fees = vec![
        ActionFee {
            action: "action".to_string(),
            fee_amount: Uint128::from(1u128),
            fee_asset: AssetInfo::Cw20(Addr::unchecked("somecw20token")),
        },
        ActionFee {
            action: "action2".to_string(),
            fee_amount: Uint128::from(2u128),
            fee_asset: AssetInfo::Native("uusd".to_string()),
        },
    ];

    let msg = ExecuteMsg::Publish {
        ado_type: "ado_type".to_string(),
        version: "0.1.0".to_string(),
        code_id: 1,
        action_fees: Some(action_fees.clone()),
        publisher: Some(owner.clone()),
    };

    let resp = execute(deps.as_mut(), env.clone(), info, msg.clone());

    assert!(resp.is_ok());

    let publisher = PUBLISHER
        .load(deps.as_ref().storage, "ado_type".to_string())
        .unwrap();
    assert_eq!(publisher, owner.to_string());

    let code_id = CODE_ID.load(deps.as_ref().storage, "ado_type").unwrap();
    assert_eq!(code_id, 1u64);

    let vers_code_id = VERSION_CODE_ID
        .load(
            deps.as_ref().storage,
            ("ado_type".to_string(), "0.1.0".to_string()),
        )
        .unwrap();
    assert_eq!(vers_code_id, 1u64);

    let fee_one = ACTION_FEES
        .load(
            deps.as_ref().storage,
            ("ado_type".to_string(), "action".to_string()),
        )
        .unwrap();
    assert_eq!(fee_one, action_fees[0]);

    let fee_two = ACTION_FEES
        .load(
            deps.as_ref().storage,
            ("ado_type".to_string(), "action2".to_string()),
        )
        .unwrap();
    assert_eq!(fee_two, action_fees[1]);

    // Test unauthorised
    let unauth_info = mock_info("not_owner", &[]);
    let resp = execute(deps.as_mut(), env, unauth_info, msg);
    assert!(resp.is_err());
}

#[test]
fn test_update_action_fees() {
    let owner = String::from("owner");
    let mut deps = mock_dependencies_custom(&[]);
    let env = mock_env();
    let info = mock_info(owner.as_str(), &[]);

    instantiate(
        deps.as_mut(),
        mock_env(),
        mock_info(&owner, &[]),
        InstantiateMsg {
            kernel_address: MOCK_KERNEL_CONTRACT.to_string(),
            owner: None,
        },
    )
    .unwrap();

    let action_fees = vec![
        ActionFee {
            action: "action".to_string(),
            fee_amount: Uint128::from(1u128),
            fee_asset: AssetInfo::Cw20(Addr::unchecked("somecw20token")),
        },
        ActionFee {
            action: "action2".to_string(),
            fee_amount: Uint128::from(2u128),
            fee_asset: AssetInfo::Native("uusd".to_string()),
        },
    ];

    let msg = ExecuteMsg::UpdateActionFees {
        action_fees: action_fees.clone(),
        ado_type: "ado_type".to_string(),
    };

    let res = execute(deps.as_mut(), env.clone(), info, msg.clone());
    assert!(res.is_ok());

    let fee_one = ACTION_FEES
        .load(
            deps.as_ref().storage,
            ("ado_type".to_string(), "action".to_string()),
        )
        .unwrap();
    assert_eq!(fee_one, action_fees[0]);

    let fee_two = ACTION_FEES
        .load(
            deps.as_ref().storage,
            ("ado_type".to_string(), "action2".to_string()),
        )
        .unwrap();
    assert_eq!(fee_two, action_fees[1]);

    // Test unauthorised
    let unauth_info = mock_info("not_owner", &[]);
    let resp = execute(deps.as_mut(), env, unauth_info, msg);
    assert!(resp.is_err());
}
