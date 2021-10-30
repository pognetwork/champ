# Voting

## Modes of Conflict:
- BlockHeightError and PreviousBlockError (vote which one goes first or if one needs to be removed here)
- double spending (i think checked in validation),
- 2 same blocks (needs to be checked in validation),
- BlockHeightError (should be double checked here because thread issues maybe?),
- TrxDataNotFoundError (maybe remove trx and try again here?),
- TooManyTrx (maybe split block and vote on order here?)

## Conclusion:
I think after each ValidationError in `node/validation/block.rs` we should need to call a vote. (exclude some errs like tooManyTrx)
This allows syncing of nodes if anything goes wrong.
Even if the order doesnt matter, the nodes should be in sync and all have the same version of the chain

## When a vote is called
- Go through all Prime Delegates and establish their voting power
- each Prime Delegate propose the block they received
- Collect each blocks with the Prime Delegates voting power
- Duplicate blocks add together voting power
- Block with heighest voting power is selected as new block in the chain
- !!! Note "Important, if no block is put forward, use that as a block to avoid false blocks"