use cosmwasm_std::{Addr, StdResult, Uint128};
use cw_controllers::HooksResponse;
use cw_multi_test::App;
use cw_ownable::Ownership;
use dao_interface::voting::{
    InfoResponse, TotalPowerAtHeightResponse, VotingPowerAtHeightResponse,
};

use crate::msg::{ListVotersResponse, QueryMsg, RegisteredNftResponse};

pub fn query_hooks(app: &App, module: &Addr) -> StdResult<HooksResponse> {
    let hooks = app.wrap().query_wasm_smart(module, &QueryMsg::Hooks {})?;
    Ok(hooks)
}

pub fn query_voting_power(
    app: &App,
    module: &Addr,
    addr: &str,
    height: Option<u64>,
) -> StdResult<VotingPowerAtHeightResponse> {
    let power = app.wrap().query_wasm_smart(
        module,
        &QueryMsg::VotingPowerAtHeight {
            address: addr.to_string(),
            height,
        },
    )?;
    Ok(power)
}

pub fn query_total_power(
    app: &App,
    module: &Addr,
    height: Option<u64>,
) -> StdResult<TotalPowerAtHeightResponse> {
    let power = app
        .wrap()
        .query_wasm_smart(module, &QueryMsg::TotalPowerAtHeight { height })?;
    Ok(power)
}

pub fn query_list_voters(app: &App, module: &Addr) -> StdResult<ListVotersResponse> {
    let power = app.wrap().query_wasm_smart(
        module,
        &QueryMsg::ListVoters {
            start_after: None,
            limit: None,
        },
    )?;
    Ok(power)
}

pub fn query_registered_nft(
    app: &App,
    module: &Addr,
    address: impl Into<String>,
) -> StdResult<RegisteredNftResponse> {
    let power = app.wrap().query_wasm_smart(
        module,
        &QueryMsg::RegisteredNft {
            address: address.into(),
        },
    )?;
    Ok(power)
}

pub fn query_info(app: &App, module: &Addr) -> StdResult<InfoResponse> {
    let info = app.wrap().query_wasm_smart(module, &QueryMsg::Info {})?;
    Ok(info)
}

pub fn query_dao(app: &App, module: &Addr) -> StdResult<Addr> {
    let dao = app.wrap().query_wasm_smart(module, &QueryMsg::Dao {})?;
    Ok(dao)
}

pub fn query_nft_contract(app: &App, module: &Addr) -> StdResult<Addr> {
    let nft = app
        .wrap()
        .query_wasm_smart(module, &QueryMsg::NftContract {})?;
    Ok(nft)
}

pub fn query_ownership(app: &App, module: &Addr) -> StdResult<Ownership<Addr>> {
    let ownership = app
        .wrap()
        .query_wasm_smart(module, &QueryMsg::Ownership {})?;
    Ok(ownership)
}

pub fn query_total_and_voting_power(
    app: &App,
    module: &Addr,
    addr: &str,
    height: Option<u64>,
) -> StdResult<(Uint128, Uint128)> {
    let total_power = query_total_power(app, module, height)?;
    let voting_power = query_voting_power(app, module, addr, height)?;

    Ok((total_power.power, voting_power.power))
}
