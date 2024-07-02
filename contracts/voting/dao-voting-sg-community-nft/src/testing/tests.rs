use cosmwasm_std::{
    testing::{mock_dependencies, mock_env},
    Addr, Uint128,
};
use cw_multi_test::next_block;

use crate::{
    contract::{migrate, CONTRACT_NAME, CONTRACT_VERSION},
    msg::MigrateMsg,
    testing::{
        execute::{
            add_hook, burn_nft, mint_and_register_nft, mint_nft, register, remove_hook,
            set_voting_power, sync, unregister, update_owner,
        },
        is_error,
        queries::{
            query_hooks, query_info, query_list_voters, query_ownership, query_registered_nft,
            query_total_and_voting_power, query_total_power, query_voting_power,
        },
        setup_test, CommonTest, CREATOR_ADDR, OWNER,
    },
    ContractError,
};

// I can register, voting power and total power is updated one block later.
#[test]
fn test_register() -> anyhow::Result<()> {
    let CommonTest {
        mut app,
        module,
        nft,
    } = setup_test();

    let total_power = query_total_power(&app, &module, None)?;
    let voting_power = query_voting_power(&app, &module, CREATOR_ADDR, None)?;

    assert_eq!(total_power.power, Uint128::zero());
    assert_eq!(total_power.height, app.block_info().height);

    assert_eq!(voting_power.power, Uint128::zero());
    assert_eq!(voting_power.height, app.block_info().height);

    mint_and_register_nft(&mut app, &nft, &module, CREATOR_ADDR, "1")?;
    mint_and_register_nft(&mut app, &nft, &module, "other", "2")?;

    // Voting powers are not updated until a block has passed.
    let (total, personal) = query_total_and_voting_power(&app, &module, CREATOR_ADDR, None)?;
    assert!(total.is_zero());
    assert!(personal.is_zero());

    app.update_block(next_block);

    let (total, personal) = query_total_and_voting_power(&app, &module, CREATOR_ADDR, None)?;
    assert_eq!(total, Uint128::new(2));
    assert_eq!(personal, Uint128::new(1));

    Ok(())
}

// I can unregister. Voting power and total power is updated when I unregister.
#[test]
fn test_unregister() -> anyhow::Result<()> {
    let CommonTest {
        mut app,
        module,
        nft,
    } = setup_test();

    mint_and_register_nft(&mut app, &nft, &module, CREATOR_ADDR, "1")?;
    mint_and_register_nft(&mut app, &nft, &module, "other", "2")?;

    app.update_block(next_block);

    let (total, personal) = query_total_and_voting_power(&app, &module, CREATOR_ADDR, None)?;
    assert_eq!(total, Uint128::new(2));
    assert_eq!(personal, Uint128::new(1));

    unregister(&mut app, &module, CREATOR_ADDR)?;

    // Voting power is updated when I unregister. Waits a block as it's a
    // snapshot map.
    let (total, personal) = query_total_and_voting_power(&app, &module, CREATOR_ADDR, None)?;
    assert_eq!(total, Uint128::new(2));
    assert_eq!(personal, Uint128::new(1));

    app.update_block(next_block);

    let (total, personal) = query_total_and_voting_power(&app, &module, CREATOR_ADDR, None)?;
    assert_eq!(total, Uint128::new(1));
    assert_eq!(personal, Uint128::zero());

    // I cannot unregister if already unregistered.
    let res = unregister(&mut app, &module, CREATOR_ADDR);
    is_error!(res => "You have not yet registered to vote");

    Ok(())
}

// I can register a token with no voting power set yet. My voting power is
// updated when the token's voting power is updated. Voting power can be updated
// for a token when it has not yet been registered.
#[test]
fn test_set_voting_power() -> anyhow::Result<()> {
    let CommonTest {
        mut app,
        module,
        nft,
    } = setup_test();

    let (total, personal) = query_total_and_voting_power(&app, &module, CREATOR_ADDR, None)?;
    assert!(total.is_zero());
    assert!(personal.is_zero());

    mint_nft(&mut app, &nft, CREATOR_ADDR, "1")?;
    register(&mut app, &module, CREATOR_ADDR)?;
    app.update_block(next_block);

    // Voting power is still zero since NFT voting power hasn't been set.
    let (total, personal) = query_total_and_voting_power(&app, &module, CREATOR_ADDR, None)?;
    assert!(total.is_zero());
    assert!(personal.is_zero());

    // only owner or creator can set voting power
    let err: ContractError = set_voting_power(&mut app, &module, "nobody", "1", Uint128::new(5))
        .unwrap_err()
        .downcast()
        .unwrap();
    assert_eq!(
        err,
        ContractError::Ownable(cw_ownable::OwnershipError::NotOwner)
    );

    // owner and creator can both set voting power
    set_voting_power(&mut app, &module, OWNER, "1", Uint128::new(5))?;
    set_voting_power(&mut app, &module, CREATOR_ADDR, "1", Uint128::new(5))?;
    app.update_block(next_block);

    let (total, personal) = query_total_and_voting_power(&app, &module, CREATOR_ADDR, None)?;
    assert_eq!(total, Uint128::new(5));
    assert_eq!(personal, Uint128::new(5));

    // Unregister token.
    unregister(&mut app, &module, CREATOR_ADDR)?;
    app.update_block(next_block);

    // Voting power should be zero.
    let (total, personal) = query_total_and_voting_power(&app, &module, CREATOR_ADDR, None)?;
    assert!(total.is_zero());
    assert!(personal.is_zero());

    // Set voting power.
    set_voting_power(&mut app, &module, OWNER, "1", Uint128::new(10))?;
    app.update_block(next_block);

    // Voting power should still be zero.
    let (total, personal) = query_total_and_voting_power(&app, &module, CREATOR_ADDR, None)?;
    assert!(total.is_zero());
    assert!(personal.is_zero());

    // Register token.
    register(&mut app, &module, CREATOR_ADDR)?;
    app.update_block(next_block);

    // Voting power should now reflect the latest update.
    let (total, personal) = query_total_and_voting_power(&app, &module, CREATOR_ADDR, None)?;
    assert_eq!(total, Uint128::new(10));
    assert_eq!(personal, Uint128::new(10));

    // Token voting power can be set to 0.
    set_voting_power(&mut app, &module, OWNER, "1", Uint128::zero())?;
    app.update_block(next_block);

    // Voting power should now be zero.
    let (total, personal) = query_total_and_voting_power(&app, &module, CREATOR_ADDR, None)?;
    assert!(total.is_zero());
    assert!(personal.is_zero());

    Ok(())
}

// I can register. Sync does nothing if registered properly. Once I burn,
// nothing happens without unregistering. Voting power is updated only after
// sync is called if unregister is not called.
#[test]
fn test_sync() -> anyhow::Result<()> {
    let CommonTest {
        mut app,
        module,
        nft,
    } = setup_test();

    let (total, personal) = query_total_and_voting_power(&app, &module, CREATOR_ADDR, None)?;
    assert!(total.is_zero());
    assert!(personal.is_zero());

    mint_and_register_nft(&mut app, &nft, &module, CREATOR_ADDR, "1")?;
    mint_and_register_nft(&mut app, &nft, &module, "other", "2")?;
    // Voting powers are not updated until a block has passed.
    app.update_block(next_block);

    let (total, personal) = query_total_and_voting_power(&app, &module, CREATOR_ADDR, None)?;
    assert_eq!(total, Uint128::new(2));
    assert_eq!(personal, Uint128::new(1));

    // Sync does nothing if I have not registered.
    sync(&mut app, &module, CREATOR_ADDR, "1")?;
    app.update_block(next_block);

    let (total, personal) = query_total_and_voting_power(&app, &module, CREATOR_ADDR, None)?;
    assert_eq!(total, Uint128::new(2));
    assert_eq!(personal, Uint128::new(1));

    let registered = query_registered_nft(&app, &module, CREATOR_ADDR)?;
    assert_eq!(registered.token_id, Some("1".to_string()));

    // Burn NFT.
    burn_nft(&mut app, &nft, CREATOR_ADDR, "1")?;
    app.update_block(next_block);

    // Nothing changes.
    let (total, personal) = query_total_and_voting_power(&app, &module, CREATOR_ADDR, None)?;
    assert_eq!(total, Uint128::new(2));
    assert_eq!(personal, Uint128::new(1));

    let registered = query_registered_nft(&app, &module, CREATOR_ADDR)?;
    assert_eq!(registered.token_id, Some("1".to_string()));

    // Sync unregisters.
    sync(&mut app, &module, CREATOR_ADDR, "1")?;
    app.update_block(next_block);

    let (total, personal) = query_total_and_voting_power(&app, &module, CREATOR_ADDR, None)?;
    assert_eq!(total, Uint128::new(1));
    assert_eq!(personal, Uint128::new(0));

    let registered = query_registered_nft(&app, &module, CREATOR_ADDR)?;
    assert_eq!(registered.token_id, None);

    Ok(())
}

// I can list all of the currently registered voters and get their NFTs.
#[test]
fn test_list_voters_and_get_registered_nfts() -> anyhow::Result<()> {
    let CommonTest {
        mut app,
        module,
        nft,
    } = setup_test();

    mint_and_register_nft(&mut app, &nft, &module, CREATOR_ADDR, "1")?;

    let deardrie = "deardrie";
    mint_nft(&mut app, &nft, deardrie, "2")?;

    let voters = query_list_voters(&app, &module)?.voters;
    assert_eq!(voters.len(), 1);
    assert_eq!(voters[0], CREATOR_ADDR.to_string());

    let creator_registered = query_registered_nft(&app, &module, CREATOR_ADDR)?;
    assert_eq!(creator_registered.token_id, Some("1".to_string()));

    let deadrie_registered = query_registered_nft(&app, &module, deardrie)?;
    assert_eq!(deadrie_registered.token_id, None);

    register(&mut app, &module, deardrie)?;

    let deadrie_registered: crate::msg::RegisteredNftResponse =
        query_registered_nft(&app, &module, deardrie)?;
    assert_eq!(deadrie_registered.token_id, Some("2".to_string()));

    let voters = query_list_voters(&app, &module)?.voters;
    assert_eq!(voters.len(), 2);
    assert_eq!(voters[0], CREATOR_ADDR.to_string());
    assert_eq!(voters[1], deardrie.to_string());

    unregister(&mut app, &module, CREATOR_ADDR)?;
    unregister(&mut app, &module, deardrie)?;

    let creator_registered = query_registered_nft(&app, &module, CREATOR_ADDR)?;
    assert_eq!(creator_registered.token_id, None);

    let deadrie_registered = query_registered_nft(&app, &module, deardrie)?;
    assert_eq!(deadrie_registered.token_id, None);

    let voters = query_list_voters(&app, &module)?.voters;
    assert_eq!(voters.len(), 0);

    Ok(())
}

#[test]
fn test_info_query_works() -> anyhow::Result<()> {
    let CommonTest { app, module, .. } = setup_test();
    let info = query_info(&app, &module)?;
    assert_eq!(info.info.version, env!("CARGO_PKG_VERSION").to_string());
    Ok(())
}

// The owner may add and remove hooks.
#[test]
fn test_add_remove_hooks() -> anyhow::Result<()> {
    let CommonTest {
        mut app,
        module,
        nft,
    } = setup_test();

    add_hook(&mut app, &module, CREATOR_ADDR, "meow")?;
    remove_hook(&mut app, &module, CREATOR_ADDR, "meow")?;

    // Minting NFT works if no hooks
    mint_and_register_nft(&mut app, &nft, &module, CREATOR_ADDR, "1").unwrap();

    // Add a hook to a fake contract called "meow"
    add_hook(&mut app, &module, CREATOR_ADDR, "meow")?;

    let hooks = query_hooks(&app, &module)?;
    assert_eq!(hooks.hooks, vec!["meow".to_string()]);

    // Minting / registering now doesn't work because meow isn't a contract This
    // failure means the hook is working
    mint_and_register_nft(&mut app, &nft, &module, CREATOR_ADDR, "1").unwrap_err();

    let res = add_hook(&mut app, &module, CREATOR_ADDR, "meow");
    is_error!(res => "Given address already registered as a hook");

    let res = remove_hook(&mut app, &module, CREATOR_ADDR, "blue");
    is_error!(res => "Given address not registered as a hook");

    let res = add_hook(&mut app, &module, "ekez", "evil");
    is_error!(res => "Caller is not the contract's current owner");

    Ok(())
}

// The owner can be transferred by the current owner or the DAO. Renouncing
// ownership actually transfers ownership to the DAO.
#[test]
fn test_update_owner() -> anyhow::Result<()> {
    let CommonTest {
        mut app, module, ..
    } = setup_test();

    let ownership = query_ownership(&app, &module)?.owner;
    assert_eq!(ownership, Some(Addr::unchecked(OWNER)));

    // Transfer ownership works.
    let new_owner = "new";
    update_owner(
        &mut app,
        &module,
        OWNER,
        cw_ownable::Action::TransferOwnership {
            new_owner: new_owner.to_string(),
            expiry: None,
        },
    )?;
    update_owner(
        &mut app,
        &module,
        new_owner,
        cw_ownable::Action::AcceptOwnership {},
    )?;

    let ownership = query_ownership(&app, &module)?.owner;
    assert_eq!(ownership, Some(Addr::unchecked(new_owner)));

    // Only the current owner or DAO can transfer ownership.
    let err: ContractError = update_owner(
        &mut app,
        &module,
        OWNER,
        cw_ownable::Action::TransferOwnership {
            new_owner: OWNER.to_string(),
            expiry: None,
        },
    )
    .unwrap_err()
    .downcast()
    .unwrap();
    assert!(matches!(
        err,
        ContractError::Ownable(cw_ownable::OwnershipError::NotOwner)
    ));

    let ownership = query_ownership(&app, &module)?.owner;
    assert_eq!(ownership, Some(Addr::unchecked(new_owner)));

    // DAO (module creator) can forcibly transfer ownership.
    update_owner(
        &mut app,
        &module,
        CREATOR_ADDR,
        cw_ownable::Action::TransferOwnership {
            new_owner: OWNER.to_string(),
            expiry: None,
        },
    )?;
    update_owner(
        &mut app,
        &module,
        OWNER,
        cw_ownable::Action::AcceptOwnership {},
    )?;

    let ownership = query_ownership(&app, &module)?.owner;
    assert_eq!(ownership, Some(Addr::unchecked(OWNER)));

    // Only the owner or DAO (creator) can renounce.
    let err: ContractError = update_owner(
        &mut app,
        &module,
        "someone_else",
        cw_ownable::Action::RenounceOwnership {},
    )
    .unwrap_err()
    .downcast()
    .unwrap();
    assert!(matches!(
        err,
        ContractError::Ownable(cw_ownable::OwnershipError::NotOwner)
    ));

    // Renouncing ownership actually transfers ownership to the DAO.
    update_owner(
        &mut app,
        &module,
        OWNER,
        cw_ownable::Action::RenounceOwnership {},
    )?;

    let ownership = query_ownership(&app, &module)?.owner;
    assert_eq!(ownership, Some(Addr::unchecked(CREATOR_ADDR)));

    // Transfer back to OWNER.
    update_owner(
        &mut app,
        &module,
        CREATOR_ADDR,
        cw_ownable::Action::TransferOwnership {
            new_owner: OWNER.to_string(),
            expiry: None,
        },
    )?;
    update_owner(
        &mut app,
        &module,
        OWNER,
        cw_ownable::Action::AcceptOwnership {},
    )?;

    let ownership = query_ownership(&app, &module)?.owner;
    assert_eq!(ownership, Some(Addr::unchecked(OWNER)));

    // Forcibly renounce.
    update_owner(
        &mut app,
        &module,
        CREATOR_ADDR,
        cw_ownable::Action::RenounceOwnership {},
    )?;

    let ownership = query_ownership(&app, &module)?.owner;
    assert_eq!(ownership, Some(Addr::unchecked(CREATOR_ADDR)));

    Ok(())
}

#[test]
pub fn test_migrate_update_version() {
    let mut deps = mock_dependencies();
    cw2::set_contract_version(&mut deps.storage, "my-contract", "1.0.0").unwrap();
    migrate(deps.as_mut(), mock_env(), MigrateMsg {}).unwrap();
    let version = cw2::get_contract_version(&deps.storage).unwrap();
    assert_eq!(version.version, CONTRACT_VERSION);
    assert_eq!(version.contract, CONTRACT_NAME);
}
