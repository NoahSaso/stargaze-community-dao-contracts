# `dao-voting-sg-community-nft`

This is a DAO DAO voting module to power the Stargaze Community DAO that votes
on which projects to list on the Stargaze NFT marketplace.

This contract expects that the NFT contract obeys the following rules:

- one address can only ever hold 0 or 1 NFT.
- NFTs are soul-bound—they cannot be transferred. only minted/burned.
