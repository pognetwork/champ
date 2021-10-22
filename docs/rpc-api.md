# RPC API

This document is work in progress and just contains some ideas for required endpoints.

## Account Service

- getBalance (is the balance pending? at what block? latest?)
- getNextBlockHeight
- getInfo

  - getDelegate
  - getPower (voting power)
  - getBlockCount
  - getPendingBlocks
  - getUnacknowledgedTransactions (transactions without a counterpart receive)

- getBlockByID
- getBlockByNumber
- getTransactionByID
- getTransactions (multiple query options)
- sendBlock
- getTotalTransactionCount

## Private Service

- getAccounts
- getAccount
- getDefaultAccount
- setDefaultAccount
- addAccount
- removeAccount
- signMessage
- signBlock
- verifySignature
- encryptMessage
- decryptMessage

## Admin Service

- getPeers
- getVersion
- upgradeNode
- getPendingBlocks
- setPendingBlockLimit
- getNodeStatus
- getMode
- setMode
- getNodeName
- setNodeName
- getChain
- getLogs
