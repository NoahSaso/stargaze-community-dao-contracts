use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use cw_hooks::Hooks;
use cw_storage_plus::{Item, Map, SnapshotItem, SnapshotMap, Strategy};

/// DAO.
pub const DAO: Item<Addr> = Item::new("dao");

/// Address of the NFT contract.
pub const NFT_CONTRACT: Item<Addr> = Item::new("nft");

/// Token metadata.
pub const TOKENS: Map<&String, TokenMetadata> = Map::new("tokens");

/// The token ID registered by a voter.
pub const VOTER_TOKENS: Map<&Addr, String> = Map::new("vts");

/// The voting power registered for an address as a function of block
/// height.
pub const VOTER_VOTING_POWER: SnapshotMap<&Addr, Uint128> = SnapshotMap::new(
    "vp",
    "vp__checkpoints",
    "vp__changelog",
    Strategy::EveryBlock,
);

/// The voting power registered with this contract as a function of
/// block height.
pub const TOTAL_VOTING_POWER: SnapshotItem<Uint128> = SnapshotItem::new(
    "tvp",
    "tvp__checkpoints",
    "tvp__changelog",
    Strategy::EveryBlock,
);

// Hooks to contracts that will receive registration and unregistration
// messages.
pub const HOOKS: Hooks = Hooks::new("hooks");

/// Metadata about a token that exists. It may or may not have been registered
/// to vote.
#[cw_serde]
pub struct TokenMetadata {
    /// If registered, the voter that owns this token.
    pub voter: Option<Addr>,
    /// The voting power of the token.
    pub vp: Uint128,
}
