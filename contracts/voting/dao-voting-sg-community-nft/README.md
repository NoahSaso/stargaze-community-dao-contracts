# `dao-voting-sg-community-nft`

This is a DAO DAO voting module to power the Stargaze Community DAO that votes
on which projects to list on the Stargaze NFT marketplace.

This contract expects that the NFT contract obeys the following rules:

- one address can only ever hold 0 or 1 NFT.
- NFTs are soul-bound—they cannot be transferred. only minted/burned.

## Instantiate

To instantiate this contract, you just need an NFT contract that follows the
adheres to the rules listed above, and optionally an owner that is responsible
for setting voting powers.

The message looks like:

```json
{
  "nft_contract": "NFT_CONTRACT",
  "owner": "OWNER"
}
```

## Execute

### `Register`

Register to vote. This checks that the caller owns exactly one NFT and registers
them to vote if so. Their voting power is derived from the token they hold. The
token's voting power is set by the `SetVotingPower` call, which can update the
voting power at any time.

### `Unregister`

Unregister to vote.

### `SetVotingPower`

```json
{
  "token_id": "TOKEN_ID",
  "power": "1"
}
```

Only callable by the instantiator or the owner, if specified.

This is required to provide voting power for a token. This can be called before
or after a token is registered. The voting power will only be counted toward the
total voting power when the token is registered.

### `Sync`

```json
{
  "token_id": "TOKEN_ID"
}
```

This can be called by anyone.

Ensure the ownership of an NFT is up to date. If a voter registered with the
specified token, this ensures they still own the NFT. If they no longer own the
NFT, this forcibly unregisters them. Otherwise, it does nothing.
