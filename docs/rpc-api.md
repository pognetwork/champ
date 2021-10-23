# gRPC API

The following document contains general information and caveats about all endpoints available in champ's gRPC API.

If you are interested in the specific message types and parameters, check out [pog-proto](https://github.com/pognetwork/proto) which contains all `proto` service definitions and prebuild libraries for rust, typescript and javascript.

The gRPC API is exposed (by default) on `[::1]:50051`. For interactions via websites, `grpc-web` support is available.

## Authentication

Work in Progress

## Account Service

<!-- prettier-ignore -->
??? info "getBalance"
    is the balance pending? at what block?

<!-- prettier-ignore -->
??? info "getNextBlockHeight"

<!-- prettier-ignore -->
??? info "getAccountInfo"

<!-- prettier-ignore -->
??? info "getAccountDelegate"

<!-- prettier-ignore -->
??? info "getAccountPower"

<!-- prettier-ignore -->
??? info "getAccountBlockCount"

<!-- prettier-ignore -->
??? info "getGlobalBlockCount"

<!-- prettier-ignore -->
??? info "getGlobalTransactionCount"

<!-- prettier-ignore -->
??? info "getPendingBlocks"

<!-- prettier-ignore -->
??? info "getUnacknowledgedTransactions"
    transactions without a counterpart receive

<!-- prettier-ignore -->
??? info "getBlockByID"

<!-- prettier-ignore -->
??? info "getBlockByID"

<!-- prettier-ignore -->
??? info "getBlockByNumber"

<!-- prettier-ignore -->
??? info "getTransactionByID"

<!-- prettier-ignore -->
??? info "getTransactions"

<!-- prettier-ignore -->
??? info "sendBlock"

## Private Service

<!-- prettier-ignore -->
??? info "getAccounts"

<!-- prettier-ignore -->
??? info "getAccount"

<!-- prettier-ignore -->
??? info "getDefaultAccount"

<!-- prettier-ignore -->
??? info "setDefaultAccount"

<!-- prettier-ignore -->
??? info "addAccount"

<!-- prettier-ignore -->
??? info "removeAccount"

<!-- prettier-ignore -->
??? info "signMessage"

<!-- prettier-ignore -->
??? info "signBlock"

<!-- prettier-ignore -->
??? info "verifySignature"

<!-- prettier-ignore -->
??? info "encryptMessage"

<!-- prettier-ignore -->
??? info "decryptMessage"

## Admin Service

<!-- prettier-ignore -->
??? info "getPeers"

<!-- prettier-ignore -->
??? info "getVersion"

<!-- prettier-ignore -->
??? info "upgradeNode"

<!-- prettier-ignore -->
??? info "getPendingBlocks"

<!-- prettier-ignore -->
??? info "setPendingBlockLimit"

<!-- prettier-ignore -->
??? info "getNodeStatus"

<!-- prettier-ignore -->
??? info "getMode"

<!-- prettier-ignore -->
??? info "setMode"

<!-- prettier-ignore -->
??? info "getNodeName"

<!-- prettier-ignore -->
??? info "setNodeName"

<!-- prettier-ignore -->
??? info "getChain"

<!-- prettier-ignore -->
??? info "getLogs"
