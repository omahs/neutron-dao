use cosmwasm_std::{
    from_binary,
    testing::{mock_env, mock_info},
    to_binary, Addr, CosmosMsg, DepsMut, Empty, SubMsg, WasmMsg,
};
use std::collections::HashMap;

use crate::{
    contract::{execute, instantiate, query},
    testing::mock_querier::{mock_dependencies, MOCK_DAO_CORE, MOCK_TIMELOCK_CONTRACT},
};
use neutron_dao_pre_propose_overrule::msg::{
    ExecuteMsg, InstantiateMsg, ProposeMessage, QueryExt, QueryMsg,
};

use crate::error::PreProposeOverruleError;
use crate::testing::mock_querier::{
    get_dao_with_impostor_subdao, get_dao_with_impostor_timelock, get_properly_initialized_dao,
    ContractQuerier, MOCK_DAO_PROPOSE_MODULE, MOCK_IMPOSTOR_TIMELOCK_CONTRACT, MOCK_SUBDAO_CORE,
    NON_TIMELOCKED_PROPOSAL_ID, PROPOSALS_COUNT, SUBDAO_NAME, TIMELOCKED_PROPOSAL_ID,
};
use cwd_pre_propose_base::state::Config;
use cwd_proposal_single::msg::ExecuteMsg as ProposeMessageInternal;
use neutron_subdao_timelock_single::msg as TimelockMsg;

pub fn init_base_contract(deps: DepsMut<Empty>) {
    let msg = InstantiateMsg {};
    let info = mock_info(MOCK_DAO_PROPOSE_MODULE, &[]);
    instantiate(deps, mock_env(), info, msg).unwrap();
}

#[test]
fn test_create_overrule_proposal() {
    let contracts: HashMap<String, Box<dyn ContractQuerier>> = get_properly_initialized_dao();
    let mut deps = mock_dependencies(contracts);
    init_base_contract(deps.as_mut());
    const PROPOSAL_ID: u64 = TIMELOCKED_PROPOSAL_ID;
    const PROPOSER_ADDR: &str = "whatever";
    let msg = ExecuteMsg::Propose {
        msg: ProposeMessage::ProposeOverrule {
            timelock_contract: MOCK_TIMELOCK_CONTRACT.to_string(),
            proposal_id: PROPOSAL_ID,
        },
    };
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info(PROPOSER_ADDR, &[]),
        msg,
    );
    assert!(res.is_ok());
    let prop_name: String = format!(
        "Reject the proposal #{} of the '{}' subdao",
        PROPOSAL_ID, SUBDAO_NAME
    );
    let prop_desc: String = format!(
        "If this proposal will be accepted, the DAO is going to \
overrule the proposal #{} of '{}' subdao (address {})",
        PROPOSAL_ID, SUBDAO_NAME, MOCK_SUBDAO_CORE
    );
    assert_eq!(
        res.unwrap().messages,
        vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: MOCK_DAO_PROPOSE_MODULE.to_string(),
            msg: to_binary(&ProposeMessageInternal::Propose {
                title: prop_name,
                description: prop_desc,
                msgs: vec![CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: MOCK_TIMELOCK_CONTRACT.to_string(),
                    msg: to_binary(&TimelockMsg::ExecuteMsg::OverruleProposal {
                        proposal_id: PROPOSAL_ID
                    })
                    .unwrap(),
                    funds: vec![],
                })],
                proposer: Some(PROPOSER_ADDR.to_string()),
            })
            .unwrap(),
            funds: vec![],
        }))]
    );
}

#[test]
fn test_base_queries() {
    let contracts: HashMap<String, Box<dyn ContractQuerier>> = get_properly_initialized_dao();
    let mut deps = mock_dependencies(contracts);
    init_base_contract(deps.as_mut());

    let res_config = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let queried_config = from_binary(&res_config).unwrap();
    let expected_config = Config {
        deposit_info: None,
        open_proposal_submission: true,
    };
    assert_eq!(expected_config, queried_config);

    let res_dao = query(deps.as_ref(), mock_env(), QueryMsg::Dao {}).unwrap();
    let queried_dao: Addr = from_binary(&res_dao).unwrap();
    let expected_dao = Addr::unchecked(MOCK_DAO_CORE);
    assert_eq!(expected_dao, queried_dao);

    let res_proposal_module =
        query(deps.as_ref(), mock_env(), QueryMsg::ProposalModule {}).unwrap();
    let queried_proposal_module: Addr = from_binary(&res_proposal_module).unwrap();
    let expected_proposal_module = Addr::unchecked(MOCK_DAO_PROPOSE_MODULE);
    assert_eq!(expected_proposal_module, queried_proposal_module);

    assert_eq!(expected_config, queried_config);
}

#[test]
fn test_proposal_id_query() {
    let contracts: HashMap<String, Box<dyn ContractQuerier>> = get_properly_initialized_dao();
    let mut deps = mock_dependencies(contracts);
    init_base_contract(deps.as_mut());
    const PROPOSAL_ID: u64 = TIMELOCKED_PROPOSAL_ID;
    const PROPOSER_ADDR: &str = "whatever";
    let msg = ExecuteMsg::Propose {
        msg: ProposeMessage::ProposeOverrule {
            timelock_contract: MOCK_TIMELOCK_CONTRACT.to_string(),
            proposal_id: PROPOSAL_ID,
        },
    };
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info(PROPOSER_ADDR, &[]),
        msg,
    );
    assert!(res.is_ok());

    let res_id = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::QueryExtension {
            msg: QueryExt::OverruleProposalId {
                subdao_proposal_id: PROPOSAL_ID,
                timelock_address: MOCK_TIMELOCK_CONTRACT.to_string(),
            },
        },
    )
    .unwrap();
    let queried_id: u64 = from_binary(&res_id).unwrap();
    let expected_id = PROPOSALS_COUNT + 1;
    assert_eq!(expected_id, queried_id);
}

#[test]
fn test_base_prepropose_methods() {
    let contracts: HashMap<String, Box<dyn ContractQuerier>> = get_properly_initialized_dao();
    let mut deps = mock_dependencies(contracts);
    init_base_contract(deps.as_mut());
    const PROPOSER_ADDR: &str = "whatever";
    let msg = ExecuteMsg::UpdateConfig {
        deposit_info: None,
        open_proposal_submission: true,
    };
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info(PROPOSER_ADDR, &[]),
        msg,
    );
    assert!(res.is_err());
    assert_eq!(res, Err(PreProposeOverruleError::MessageUnsupported {}))
}

#[test]
fn test_impostor_subdao() {
    // test where timelock contract points to subdao that doesn't points to this timelock
    let contracts: HashMap<String, Box<dyn ContractQuerier>> = get_dao_with_impostor_subdao();
    let mut deps = mock_dependencies(contracts);
    init_base_contract(deps.as_mut());
    const PROPOSAL_ID: u64 = TIMELOCKED_PROPOSAL_ID;
    const PROPOSER_ADDR: &str = "whatever";
    let msg = ExecuteMsg::Propose {
        msg: ProposeMessage::ProposeOverrule {
            timelock_contract: MOCK_TIMELOCK_CONTRACT.to_string(),
            proposal_id: PROPOSAL_ID,
        },
    };
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info(PROPOSER_ADDR, &[]),
        msg,
    );
    assert!(res.is_err());
    assert_eq!(res, Err(PreProposeOverruleError::ForbiddenSubdao {}));
}

#[test]
fn test_impostor_timelock() {
    // test where timelock contract points to subdao that doesn't points to this timelock
    let contracts: HashMap<String, Box<dyn ContractQuerier>> = get_dao_with_impostor_timelock();
    let mut deps = mock_dependencies(contracts);
    init_base_contract(deps.as_mut());
    const PROPOSAL_ID: u64 = TIMELOCKED_PROPOSAL_ID;
    const PROPOSER_ADDR: &str = "whatever";
    let msg = ExecuteMsg::Propose {
        msg: ProposeMessage::ProposeOverrule {
            timelock_contract: MOCK_IMPOSTOR_TIMELOCK_CONTRACT.to_string(),
            proposal_id: PROPOSAL_ID,
        },
    };
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info(PROPOSER_ADDR, &[]),
        msg,
    );
    assert!(res.is_err());
    assert_eq!(res, Err(PreProposeOverruleError::SubdaoMisconfigured {}));
}

#[test]
fn test_proposal_is_not_timelocked() {
    // test where the proposal we're to create overrule for isn't timelocked already/yet
    let contracts: HashMap<String, Box<dyn ContractQuerier>> = get_properly_initialized_dao();
    let mut deps = mock_dependencies(contracts);
    init_base_contract(deps.as_mut());
    const PROPOSAL_ID: u64 = NON_TIMELOCKED_PROPOSAL_ID;
    const PROPOSER_ADDR: &str = "whatever";
    let msg = ExecuteMsg::Propose {
        msg: ProposeMessage::ProposeOverrule {
            timelock_contract: MOCK_TIMELOCK_CONTRACT.to_string(),
            proposal_id: PROPOSAL_ID,
        },
    };
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info(PROPOSER_ADDR, &[]),
        msg,
    );
    assert!(res.is_err());
    assert_eq!(res, Err(PreProposeOverruleError::ProposalWrongState {}));
}

#[test]
fn test_double_creation() {
    let contracts: HashMap<String, Box<dyn ContractQuerier>> = get_properly_initialized_dao();
    let mut deps = mock_dependencies(contracts);
    init_base_contract(deps.as_mut());
    const PROPOSAL_ID: u64 = TIMELOCKED_PROPOSAL_ID;
    const PROPOSER_ADDR: &str = "whatever";
    let msg = ExecuteMsg::Propose {
        msg: ProposeMessage::ProposeOverrule {
            timelock_contract: MOCK_TIMELOCK_CONTRACT.to_string(),
            proposal_id: PROPOSAL_ID,
        },
    };
    let res_ok = execute(
        deps.as_mut(),
        mock_env(),
        mock_info(PROPOSER_ADDR, &[]),
        msg.clone(),
    );
    assert!(res_ok.is_ok());
    let res_not_ok = execute(
        deps.as_mut(),
        mock_env(),
        mock_info(PROPOSER_ADDR, &[]),
        msg,
    );
    assert!(res_not_ok.is_err());
    assert_eq!(
        res_not_ok,
        Err(PreProposeOverruleError::AlreadyExists {
            id: PROPOSALS_COUNT + 1
        })
    );
}
