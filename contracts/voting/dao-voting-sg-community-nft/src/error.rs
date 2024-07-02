use cosmwasm_std::{OverflowError, StdError};
use cw_ownable::OwnershipError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error(transparent)]
    Std(#[from] StdError),

    #[error(transparent)]
    HookError(#[from] cw_hooks::HookError),

    #[error(transparent)]
    Ownable(#[from] OwnershipError),

    #[error(transparent)]
    Overflow(#[from] OverflowError),

    #[error("You are already registered to vote")]
    AlreadyRegistered {},

    #[error("You have not yet registered to vote")]
    NotRegistered {},

    #[error("You must own an NFT before registering to vote")]
    CannotRegister {},

    #[error("You should not be able to own more than one NFT at a time")]
    TooManyNfts {},

    #[error("Your NFT was somehow registered by another voter")]
    NftAlreadyRegistered {},
}
