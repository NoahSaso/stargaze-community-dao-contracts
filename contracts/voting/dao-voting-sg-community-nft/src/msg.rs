use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128};
use cw_ownable::{cw_ownable_execute, cw_ownable_query};
use dao_dao_macros::voting_module_query;

#[cw_serde]
pub struct InstantiateMsg {
    /// Address of the soul-bound cw721 NFT contract.
    pub nft_contract: String,
    /// Address of the owner that has the authority to set voting power. If
    /// none, DAO will be the owner.
    pub owner: Option<String>,
}

#[cw_ownable_execute]
#[cw_serde]
pub enum ExecuteMsg {
    /// Register to vote.
    Register {},
    /// Unregister from voting.
    Unregister {},
    /// Set the voting power for a token. Only callable by the DAO that
    /// initialized this voting contract or the owner.
    SetVotingPower { token_id: String, power: Uint128 },
    /// Adds a hook which is called on registration / unregistration events.
    /// Only callable by the DAO that initialized this voting contract or the
    /// owner.
    AddHook { addr: String },
    /// Removes a hook which is called on registration / unregistration events.
    /// Only callable by the DAO that initialized this voting contract or the
    /// owner.
    RemoveHook { addr: String },
}

#[cw_ownable_query]
#[voting_module_query]
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Returns the address of the NFT contract.
    #[returns(Addr)]
    NftContract {},
    /// Returns the registered hooks.
    #[returns(::cw_controllers::HooksResponse)]
    Hooks {},
    // Returns the registered NFT (token ID) for a voter.
    #[returns(RegisteredNftResponse)]
    RegisteredNft { address: String },
    /// List the registered voters.
    #[returns(ListVotersResponse)]
    ListVoters {
        start_after: Option<String>,
        limit: Option<u32>,
    },
}

#[cw_serde]
pub struct RegisteredNftResponse {
    /// The registered token ID. None if voter is not registered.
    pub token_id: Option<String>,
}

#[cw_serde]
pub struct ListVotersResponse {
    /// The list of voters.
    pub voters: Vec<String>,
}

#[cw_serde]
pub struct MigrateMsg {}
