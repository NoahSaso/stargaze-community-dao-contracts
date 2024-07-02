use cosmwasm_std::Uint128;
use cw_multi_test::next_block;

use crate::testing::{
    execute::{mint_and_register_nft, unregister},
    queries::query_voting_power,
};

use super::{queries::query_total_and_voting_power, setup_test, CommonTest, CREATOR_ADDR};

/// Registered tokens has a one block delay before registered tokens are
/// reflected in voting power. Unregistering has a one block delay
/// before the unregister is reflected in voting power, yet you have
/// access to the NFT. If I immediately register an unregistered NFT, my
/// voting power should not change.
#[test]
fn test_circular_register() -> anyhow::Result<()> {
    let CommonTest {
        mut app,
        module,
        nft,
    } = setup_test();

    mint_and_register_nft(&mut app, &nft, &module, CREATOR_ADDR, "1")?;

    app.update_block(next_block);

    let (total, voting) = query_total_and_voting_power(&app, &module, CREATOR_ADDR, None)?;
    assert_eq!(total, Uint128::new(1));
    assert_eq!(voting, Uint128::new(1));

    unregister(&mut app, &module, CREATOR_ADDR)?;

    // Unchanged, one block delay.
    let (total, voting) = query_total_and_voting_power(&app, &module, CREATOR_ADDR, None)?;
    assert_eq!(total, Uint128::new(1));
    assert_eq!(voting, Uint128::new(1));

    app.update_block(next_block);

    // Changed.
    let (total, voting) = query_total_and_voting_power(&app, &module, CREATOR_ADDR, None)?;
    assert_eq!(total, Uint128::zero());
    assert_eq!(voting, Uint128::zero());

    Ok(())
}

/// I can immediately unregister after registering even though voting powers
/// aren't updated until one block later. Voting power does not change
/// if I do this.
#[test]
fn test_immediate_unregister() -> anyhow::Result<()> {
    let CommonTest {
        mut app,
        module,
        nft,
    } = setup_test();

    mint_and_register_nft(&mut app, &nft, &module, CREATOR_ADDR, "1")?;
    unregister(&mut app, &module, CREATOR_ADDR)?;

    app.update_block(next_block);

    let (total, voting) = query_total_and_voting_power(&app, &module, CREATOR_ADDR, None)?;
    assert_eq!(total, Uint128::zero());
    assert_eq!(voting, Uint128::zero());

    Ok(())
}

/// I can determine what my voting power _will_ be after registering by
/// asking for my voting power one block in the future.
#[test]
fn test_query_the_future() -> anyhow::Result<()> {
    let CommonTest {
        mut app,
        module,
        nft,
    } = setup_test();

    mint_and_register_nft(&mut app, &nft, &module, CREATOR_ADDR, "1")?;

    // Future voting power will be one under current conditions.
    let voting = query_voting_power(
        &app,
        &module,
        CREATOR_ADDR,
        Some(app.block_info().height + 100),
    )?;
    assert_eq!(voting.power, Uint128::new(1));

    // Current voting power is zero.
    let voting = query_voting_power(&app, &module, CREATOR_ADDR, None)?;
    assert_eq!(voting.power, Uint128::new(0));

    unregister(&mut app, &module, CREATOR_ADDR)?;

    // Future voting power is now zero.
    let voting = query_voting_power(
        &app,
        &module,
        CREATOR_ADDR,
        Some(app.block_info().height + 100),
    )?;
    assert_eq!(voting.power, Uint128::zero());

    Ok(())
}
