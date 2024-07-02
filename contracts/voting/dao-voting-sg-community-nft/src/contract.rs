#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Addr, Binary, Deps, DepsMut, Empty, Env, MessageInfo, Response, StdError,
    StdResult, Uint128,
};
use cw2::{get_contract_version, set_contract_version, ContractVersion};
use cw_storage_plus::Bound;
use dao_hooks::nft_stake::{stake_nft_hook_msgs, unstake_nft_hook_msgs};

use crate::msg::{
    ExecuteMsg, InstantiateMsg, ListVotersResponse, MigrateMsg, QueryMsg, RegisteredNftResponse,
};
use crate::state::{
    TokenMetadata, DAO, HOOKS, NFT_CONTRACT, TOKENS, TOTAL_VOTING_POWER, VOTER_TOKENS,
    VOTER_VOTING_POWER,
};
use crate::ContractError;

pub(crate) const CONTRACT_NAME: &str = "crates.io:dao-voting-sg-community-nft";
pub(crate) const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response<Empty>, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // Default the owner to the DAO if not specified.
    cw_ownable::initialize_owner(
        deps.storage,
        deps.api,
        Some(&msg.owner.unwrap_or_else(|| info.sender.to_string())),
    )?;

    DAO.save(deps.storage, &info.sender)?;

    let nft_contract = deps.api.addr_validate(&msg.nft_contract)?;
    NFT_CONTRACT.save(deps.storage, &nft_contract)?;

    TOTAL_VOTING_POWER.save(deps.storage, &Uint128::zero(), env.block.height)?;

    Ok(Response::default()
        .add_attribute("method", "instantiate")
        .add_attribute("nft", nft_contract))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response<Empty>, ContractError> {
    match msg {
        ExecuteMsg::Register {} => execute_register(deps, env, info),
        ExecuteMsg::Unregister {} => execute_unregister(deps, env, info),
        ExecuteMsg::SetVotingPower { token_id, power } => {
            execute_set_voting_power(deps, env, info, token_id, power)
        }
        ExecuteMsg::AddHook { addr } => execute_add_hook(deps, info, addr),
        ExecuteMsg::RemoveHook { addr } => execute_remove_hook(deps, info, addr),
        ExecuteMsg::UpdateOwnership(action) => execute_update_owner(deps, info, env, action),
    }
}

pub fn execute_register(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    if VOTER_TOKENS.has(deps.storage, &info.sender) {
        return Err(ContractError::AlreadyRegistered {});
    }

    let nft = NFT_CONTRACT.load(deps.storage)?;

    // Make sure voter owns exactly one NFT.
    let owned_tokens: cw721::TokensResponse = deps.querier.query_wasm_smart(
        nft,
        &cw721::Cw721QueryMsg::Tokens {
            owner: info.sender.to_string(),
            start_after: None,
            limit: None,
        },
    )?;
    if owned_tokens.tokens.is_empty() {
        return Err(ContractError::CannotRegister {});
    }
    if owned_tokens.tokens.len() > 1 {
        return Err(ContractError::TooManyNfts {});
    }

    let token_id = &owned_tokens.tokens[0];

    let token = if let Some(mut existing) = TOKENS.may_load(deps.storage, token_id)? {
        // Ensure this token is not already registered to another voter. This
        // should be impossible, but just in case...
        if existing.voter.is_some() {
            return Err(ContractError::NftAlreadyRegistered {});
        }

        // Update existing token's voter.
        existing.voter = Some(info.sender.clone());
        existing
    } else {
        // Create new token with zero voting power if none exists.
        TokenMetadata {
            voter: Some(info.sender.clone()),
            vp: Uint128::zero(),
        }
    };

    // Update existing token with new voter or save new token.
    TOKENS.save(deps.storage, token_id, &token)?;

    // Set voter's VP to token's VP and update total VP.
    let adder = |prev: Option<Uint128>| {
        prev.unwrap_or_default()
            .checked_add(token.vp)
            .map_err(StdError::overflow)
    };
    VOTER_VOTING_POWER.update(deps.storage, &info.sender, env.block.height, adder)?;
    TOTAL_VOTING_POWER.update(deps.storage, env.block.height, adder)?;

    let hook_msgs = stake_nft_hook_msgs(
        HOOKS,
        deps.storage,
        info.sender.clone(),
        token_id.to_string(),
    )?;

    Ok(Response::default()
        .add_submessages(hook_msgs)
        .add_attribute("action", "register")
        .add_attribute("voter", info.sender)
        .add_attribute("token_id", token_id))
}

pub fn execute_unregister(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let Some(token_id) = VOTER_TOKENS.may_load(deps.storage, &info.sender)? else {
        return Err(ContractError::NotRegistered {});
    };

    // Remove voter from token.
    VOTER_TOKENS.remove(deps.storage, &info.sender);
    TOKENS.update(deps.storage, &token_id, |token| -> StdResult<_> {
        let mut token = token.expect("token should exist but could not be found");
        token.voter = None;
        Ok(token)
    })?;

    let curr_voting_power = VOTER_VOTING_POWER.load(deps.storage, &info.sender)?;

    // Remove voter's VP and update total VP.
    VOTER_VOTING_POWER.remove(deps.storage, &info.sender, env.block.height)?;
    TOTAL_VOTING_POWER.update(deps.storage, env.block.height, |total| -> StdResult<_> {
        total
            .expect("total voting power should be set but could not be found")
            .checked_sub(curr_voting_power)
            .map_err(StdError::overflow)
    })?;

    let hook_msgs = unstake_nft_hook_msgs(
        HOOKS,
        deps.storage,
        info.sender.clone(),
        vec![token_id.clone()],
    )?;

    Ok(Response::default()
        .add_submessages(hook_msgs)
        .add_attribute("action", "unregister")
        .add_attribute("voter", info.sender)
        .add_attribute("token_id", token_id))
}

pub fn execute_set_voting_power(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: String,
    power: Uint128,
) -> Result<Response, ContractError> {
    let dao = DAO.load(deps.storage)?;

    // Only the DAO or the owner can set the voting power
    if info.sender != dao {
        cw_ownable::assert_owner(deps.storage, &info.sender)?;
    }

    let token = TOKENS.may_load(deps.storage, &token_id)?;

    // If the token exists, update voting power accordingly.
    if let Some(mut token) = token {
        // If a voter has registered with this token already, apply the
        // difference in voting power to the voter's VP and total VP.
        if let Some(voter) = &token.voter {
            let diff = power.abs_diff(token.vp);
            let should_add = power > token.vp;

            let updater = |prev: Option<Uint128>| -> StdResult<Uint128> {
                let prev = prev
                    .expect("voter should be registered but their voting power could not be found");
                if should_add {
                    prev.checked_add(diff).map_err(StdError::overflow)
                } else {
                    prev.checked_sub(diff).map_err(StdError::overflow)
                }
            };

            VOTER_VOTING_POWER.update(deps.storage, voter, env.block.height, updater)?;
            TOTAL_VOTING_POWER.update(deps.storage, env.block.height, updater)?;
        }

        // Update the token's voting power.
        token.vp = power;
        TOKENS.save(deps.storage, &token_id, &token)?;

        Ok(Response::default()
            .add_attribute("action", "set_voting_power")
            .add_attribute("token_id", token_id)
            .add_attribute("voting_power", power.to_string())
            .add_attribute(
                "voter",
                token
                    .voter
                    .map_or_else(|| "none".to_string(), |v| v.to_string()),
            ))
    } else {
        // If no token exists, the token has never been registered before, so
        // create a new one.
        TOKENS.save(
            deps.storage,
            &token_id,
            &TokenMetadata {
                voter: None,
                vp: power,
            },
        )?;

        Ok(Response::default()
            .add_attribute("action", "set_voting_power")
            .add_attribute("token_id", token_id)
            .add_attribute("voting_power", power.to_string()))
    }
}

pub fn execute_add_hook(
    deps: DepsMut,
    info: MessageInfo,
    addr: String,
) -> Result<Response, ContractError> {
    let dao = DAO.load(deps.storage)?;

    // Only the DAO or the owner can add a hook
    if info.sender != dao {
        cw_ownable::assert_owner(deps.storage, &info.sender)?;
    }

    let hook = deps.api.addr_validate(&addr)?;
    HOOKS.add_hook(deps.storage, hook)?;

    Ok(Response::default()
        .add_attribute("action", "add_hook")
        .add_attribute("hook", addr))
}

pub fn execute_remove_hook(
    deps: DepsMut,
    info: MessageInfo,
    addr: String,
) -> Result<Response, ContractError> {
    let dao = DAO.load(deps.storage)?;

    // Only the DAO or the owner can remove a hook
    if info.sender != dao {
        cw_ownable::assert_owner(deps.storage, &info.sender)?;
    }

    let hook = deps.api.addr_validate(&addr)?;
    HOOKS.remove_hook(deps.storage, hook)?;

    Ok(Response::default()
        .add_attribute("action", "remove_hook")
        .add_attribute("hook", addr))
}

pub fn execute_update_owner(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    action: cw_ownable::Action,
) -> Result<Response, ContractError> {
    // If renouncing ownership, set the DAO as the owner instead.
    if action == cw_ownable::Action::RenounceOwnership {
        cw_ownable::assert_owner(deps.storage, &info.sender)?;

        let dao = DAO.load(deps.storage)?;
        cw_ownable::initialize_owner(deps.storage, deps.api, Some(dao.as_str()))?;

        return Ok(Response::default()
            .add_attribute("action", "update_owner")
            .add_attribute("new_owner", dao));
    }

    // Allow the DAO to transfer ownership on behalf of the current owner by
    // imitating the current owner, since cw_ownable handles the validation
    // logic internally.
    let mut sender = info.sender;
    if let cw_ownable::Action::TransferOwnership { .. } = action {
        let dao = DAO.load(deps.storage)?;
        if sender == dao {
            // should always unwrap since renouncing ownership above just sets
            // the owner to the DAO
            let owner = cw_ownable::get_ownership(deps.storage)?.owner.unwrap();
            sender = owner;
        }
    }

    let ownership = cw_ownable::update_ownership(deps, &env.block, &sender, action)?;
    Ok(Response::default().add_attributes(ownership.into_attributes()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Dao {} => query_dao(deps),
        QueryMsg::NftContract {} => query_nft_contract(deps),
        QueryMsg::Info {} => query_info(deps),
        QueryMsg::Hooks {} => query_hooks(deps),
        QueryMsg::RegisteredNft { address } => query_registered_nft(deps, address),
        QueryMsg::ListVoters { start_after, limit } => query_list_voters(deps, start_after, limit),
        QueryMsg::TotalPowerAtHeight { height } => query_total_power_at_height(deps, env, height),
        QueryMsg::VotingPowerAtHeight { address, height } => {
            query_voting_power_at_height(deps, env, address, height)
        }
        QueryMsg::Ownership {} => to_json_binary(&cw_ownable::get_ownership(deps.storage)?),
    }
}

pub fn query_voting_power_at_height(
    deps: Deps,
    env: Env,
    address: String,
    height: Option<u64>,
) -> StdResult<Binary> {
    let address = deps.api.addr_validate(&address)?;
    let height = height.unwrap_or(env.block.height);
    let power = VOTER_VOTING_POWER
        .may_load_at_height(deps.storage, &address, height)?
        .unwrap_or_default();
    to_json_binary(&dao_interface::voting::VotingPowerAtHeightResponse { power, height })
}

pub fn query_total_power_at_height(deps: Deps, env: Env, height: Option<u64>) -> StdResult<Binary> {
    let height = height.unwrap_or(env.block.height);
    let power = TOTAL_VOTING_POWER
        .may_load_at_height(deps.storage, height)?
        .unwrap_or_default();
    to_json_binary(&dao_interface::voting::TotalPowerAtHeightResponse { power, height })
}

pub fn query_dao(deps: Deps) -> StdResult<Binary> {
    let dao = DAO.load(deps.storage)?;
    to_json_binary(&dao)
}

pub fn query_nft_contract(deps: Deps) -> StdResult<Binary> {
    let nft_contract = NFT_CONTRACT.load(deps.storage)?;
    to_json_binary(&nft_contract)
}

pub fn query_hooks(deps: Deps) -> StdResult<Binary> {
    to_json_binary(&HOOKS.query_hooks(deps)?)
}

pub fn query_info(deps: Deps) -> StdResult<Binary> {
    let info = cw2::get_contract_version(deps.storage)?;
    to_json_binary(&dao_interface::voting::InfoResponse { info })
}

pub fn query_registered_nft(deps: Deps, address: String) -> StdResult<Binary> {
    let address = deps.api.addr_validate(&address)?;
    let token_id = VOTER_TOKENS.may_load(deps.storage, &address)?;
    to_json_binary(&RegisteredNftResponse { token_id })
}

pub fn query_list_voters(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<Binary> {
    let start_after = start_after
        .map(|addr| deps.api.addr_validate(&addr))
        .transpose()?;
    let min = start_after.as_ref().map(Bound::<&Addr>::exclusive);

    let limit = limit.unwrap_or(30);
    let voters = VOTER_TOKENS
        .keys(deps.storage, min, None, cosmwasm_std::Order::Ascending)
        .take(limit as usize)
        .collect::<Result<Vec<Addr>, _>>()?
        .into_iter()
        .map(|addr| addr.to_string())
        .collect();

    to_json_binary(&ListVotersResponse { voters })
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    let storage_version: ContractVersion = get_contract_version(deps.storage)?;

    // Only migrate if newer
    if storage_version.version.as_str() < CONTRACT_VERSION {
        // Set contract to version to latest
        set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    }

    Ok(Response::new().add_attribute("action", "migrate"))
}
