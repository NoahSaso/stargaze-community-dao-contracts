use cosmwasm_std::{Addr, Empty, Uint128};
use cw_multi_test::{App, AppResponse, Executor};

use anyhow::Result as AnyResult;

use crate::msg::ExecuteMsg;

use super::CREATOR_ADDR;

// Shorthand for an unchecked address.
macro_rules! addr {
    ($x:expr ) => {
        Addr::unchecked($x)
    };
}

pub fn mint_nft(
    app: &mut App,
    cw721: &Addr,
    receiver: &str,
    token_id: &str,
) -> AnyResult<AppResponse> {
    app.execute_contract(
        addr!(CREATOR_ADDR),
        cw721.clone(),
        &cw721_base::ExecuteMsg::Mint::<Empty, Empty> {
            token_id: token_id.to_string(),
            owner: receiver.to_string(),
            token_uri: None,
            extension: Empty::default(),
        },
        &[],
    )
}

pub fn burn_nft(
    app: &mut App,
    cw721: &Addr,
    sender: &str,
    token_id: &str,
) -> AnyResult<AppResponse> {
    app.execute_contract(
        addr!(sender),
        cw721.clone(),
        &cw721_base::ExecuteMsg::Burn::<Empty, Empty> {
            token_id: token_id.to_string(),
        },
        &[],
    )
}

pub fn register(app: &mut App, module: &Addr, sender: &str) -> AnyResult<AppResponse> {
    app.execute_contract(addr!(sender), module.clone(), &ExecuteMsg::Register {}, &[])
}

pub fn set_voting_power(
    app: &mut App,
    module: &Addr,
    sender: &str,
    token_id: &str,
    power: Uint128,
) -> AnyResult<AppResponse> {
    app.execute_contract(
        addr!(sender),
        module.clone(),
        &ExecuteMsg::SetVotingPower {
            token_id: token_id.to_string(),
            power,
        },
        &[],
    )
}

pub fn mint_and_register_nft(
    app: &mut App,
    cw721: &Addr,
    module: &Addr,
    voter: &str,
    token_id: &str,
) -> AnyResult<()> {
    mint_nft(app, cw721, voter, token_id)?;
    set_voting_power(app, module, CREATOR_ADDR, token_id, Uint128::new(1))?;
    register(app, module, voter)?;
    Ok(())
}

pub fn unregister(app: &mut App, module: &Addr, sender: &str) -> AnyResult<AppResponse> {
    app.execute_contract(
        addr!(sender),
        module.clone(),
        &ExecuteMsg::Unregister {},
        &[],
    )
}

pub fn sync(
    app: &mut App,
    module: &Addr,
    sender: &str,
    token_id: impl Into<String>,
) -> AnyResult<AppResponse> {
    app.execute_contract(
        addr!(sender),
        module.clone(),
        &ExecuteMsg::Sync {
            token_id: token_id.into(),
        },
        &[],
    )
}

pub fn update_owner(
    app: &mut App,
    module: &Addr,
    sender: &str,
    action: cw_ownable::Action,
) -> AnyResult<AppResponse> {
    app.execute_contract(
        addr!(sender),
        module.clone(),
        &ExecuteMsg::UpdateOwnership(action),
        &[],
    )
}

pub fn add_hook(app: &mut App, module: &Addr, sender: &str, hook: &str) -> AnyResult<AppResponse> {
    app.execute_contract(
        addr!(sender),
        module.clone(),
        &ExecuteMsg::AddHook {
            addr: hook.to_string(),
        },
        &[],
    )
}

pub fn remove_hook(
    app: &mut App,
    module: &Addr,
    sender: &str,
    hook: &str,
) -> AnyResult<AppResponse> {
    app.execute_contract(
        addr!(sender),
        module.clone(),
        &ExecuteMsg::RemoveHook {
            addr: hook.to_string(),
        },
        &[],
    )
}
