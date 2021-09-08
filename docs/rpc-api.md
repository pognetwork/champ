# RPC API

This document is work in progress and just contains some ideas for required enpoints.

## Account Service

- getBalance (is the balance pending? at what block (latest?)?)
- getNextNonce
- getInfo

  - getDelegate
  - getPower (voting power)
  - getBlockCount
  - getPendingBlocks
  - getUnacknowledgedTransactions (transactions without a counterpart recive)

- getBlockByID
- getBlockByNumber
- getTransactionByID
- getTransactionByNumber (per account)
- sendBlock

## Private Service

- getAccounts
- signMessage
- signBlock
- newAccount
- removeAccount
- encryptMessage
- decryptMessage

## Admin Service

- getPeerCount
- getVersion
- getPendingBlocks
- getTotalBlockCount
