# Voting

## Modes of Conflict:
- BlockHeightError and PreviousBlockError (vote which one goes first or if one needs to be removed here)
- double spending (i think checked in validation),
- 2 same blocks (needs to be checked in validation),
- BlockHeightError (could mean a node missed 1 block, needs to be checked here),
- TrxDataNotFoundError (maybe remove trx and try again here?),
- TooManyTrx (maybe split block and vote on order here?)

## Conclusion:
I think after each ValidationError in `node/validation/block.rs` we should need to call a vote.
This allows syncing of nodes if anything goes wrong.
Even if the order doesnt matter, the nodes should be in sync and all have the same version of the chain

## When a vote is called
- Go through all Prime Delegates and establish their voting power
- each Prime Delegate propose the block they received
- Collect each blocks with the Prime Delegates voting power
- Duplicate blocks add together voting power
- Block with heighest voting power is selected as new block in the chain
- !!! Note "Important, if no block is put forward, use that as a block to avoid false blocks"
- !!! Warning "What to do if there is a tie?"

## BlockHeightError or PreviousBlockError
These could come from a node missing a block and the new block being after the missed block in the chain.

*For Example:* <br>
Node A, Wallet-chain 1: <br>
`---[B0|+10]---[B1|-10]---[B2|+50] > new [B3|-50]`

Node X, Wallet-chain 1: <br>
`---[B0|+10]---[B1|-10] > new [B3|-50]`

Node X will raise an error as the block height does not match the previous block. Therefore, the network will need to vote on the legitimacy of the node.
If node X is the only node that cannot validate this block (ie. all other Prime Delegate nodes have this block in the chain already), the block is added to the chain in node X.
**However**, node X will need to sync their wallet-chain 1 with the other Prime Delegates to ensure that no blocks are missing. <br>
If the Prime Delegates do not have the new block in their chain, the new block is discarded.

Or, this error could come from two blocks being sent and the second block reaching the node first. Here, we may want to implement a retry system to decrease faulty blocks being discarded.