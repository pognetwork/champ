# gRPC API

The following document contains general information and caveats about champ's gRPC API.

If you are interested in the specific message types and parameters, check out [pog-proto](https://pog.network/proto/) which contains all `proto` service definitions and prebuilt libraries for rust, typescript and javascript. An auto-generated API documentation is also available ([BlockService](https://pog.network/proto/proto/block/), [NodeAdminService](https://pog.network/proto/proto/node_admin/), [NodeUserService](https://pog.network/proto/proto/node_user/), [NodeWalletManager](https://pog.network/proto/proto/node_wallet_manager/)).

The gRPC API is exposed (by default) on `[::1]:50051`. For interactions via websites, `grpc-web` support is available.

In short, the 3 different services provided by Pog.Network are the _Block Service_, _Node Wallet Manager Service_ and the _Node Admin Service_.

## Admin interface

While the admin interface is not yet ready for primetime, the general login/authentication flow is fully implemented. Instructions for running it locally are available [here](https://github.com/pognetwork/catjam). Once the webinterface reaches a more stable state, it will be automatically bundled with champ for release builds.

## Authentication and Authorization

Authentication is done with JSON-Web-Tokens. New users can only created through the champ cli using `champ user create`.

This is the case since all of our authenticated APIs are essentially for `intranet` type usage and should not be exposed to the public internet. More on that in our _securing your champ node_ section.

After a new account has been created, `NodeUser.Login` can be called with your username and password to obtain a jwt.
From this endpoint you will recieve your token which you can include in the the `authorization` metadata field to authenticate your request. For endpoints requiring authentication, the node will then validate your token and check if you have the neccecary permisions. Currently, the following permissions are available and can be configured in your `champ.toml` config file.

```ini
"admin.read"         -> Read-access to the NodeAdmin Service
"admin.write"        -> Write-access to the NodeAdmin Service
"wallet.create"      -> Create a wallet on a node
"wallet.{id}.sign"   -> Write.access to a wallet with {id}
"wallet.{id}.manage" -> Edit.access to a wallet with {id}
"wallet.{id}.unlock" -> Write.access to a wallet with {id}
"superadmin"         -> Access to all
```

## Block Service

The Block Service acts as the public interface of a node which external applications that, for example, a user's wallet talks to.

<!-- prettier-ignore -->
??? info "getBalance"
    Gets the current balance of an account.

<!-- prettier-ignore -->
??? info "getBlockHeight"
    Gets the number of blocks in the chain. It can be passed a flag to return the next block index.

<!-- prettier-ignore -->
??? info "getDelegate"
    Gets the address of the delegate of an account.

<!-- prettier-ignore -->
??? info "getVotingPower"
    Gets either the actual or the active voting power.
    - Active voting power includes delegate voting power.
    - Actual voting power excludes delegate voting power.

<!-- prettier-ignore -->
??? warning "[not yet implemented] getAccountBlockCount"
    Gets the count of all blocks sent by an account.

<!-- prettier-ignore -->
??? warning "[not yet implemented] getGlobalBlockCount"
    Gets the count of all blocks by all accounts.

<!-- prettier-ignore -->
??? warning "[not yet implemented] getGlobalTransactionCount"
    Gets the count of all transactions in the network.

<!-- prettier-ignore -->
??? warning "[not yet implemented] getPendingBlocks"
    Gets all the blocks that are not validated yet.

<!-- prettier-ignore -->
??? warning " [not yet implemented] getUnacknowledgedTransactions"
    Gets transactions without a counterpart receive.

<!-- prettier-ignore -->
??? info "getBlockByID"
    Gets the block based on its ID.

<!-- prettier-ignore -->
??? info "getBlockByHeight"
    Gets the block using its index in the chain.

<!-- prettier-ignore -->
??? info "getTransactionByID"
    Gets a transaction using its  ID.

<!-- prettier-ignore -->
??? warning "[not yet implemented] getTransactions"
    Gets all the transactions before a certain transactions with a limit.

<!-- prettier-ignore -->
??? warning "[not yet implemented] sendBlock"
    Sends a block into the network.

## Node Wallet Manager Service

The Node Wallet Manager Service enables authorized users to interact with wallets stored on a node. This is especially usefull for integrating services like centralized exchanges and online shops.

<!-- prettier-ignore -->
??? warning "[not yet implemented] getWallets"
    Gets all accounts on a node.

<!-- prettier-ignore -->
??? warning "[not yet implemented] getWallet"
    Gets an account using a address on a node.

<!-- prettier-ignore -->
??? warning "[not yet implemented] addWallet"
    Creates a wallet for a user using a password.

<!-- prettier-ignore -->
??? warning "[not yet implemented] removeWallet"
    Deletes a wallet from the node.

<!-- prettier-ignore -->
??? warning "[not yet implemented] signMessage"
    Signs the message given using the wallets private key stored on the node.

<!-- prettier-ignore -->
??? warning "[not yet implemented] signBlock"
    Signs the block given using the wallets private key stored on the node.

<!-- prettier-ignore -->
??? warning "[not yet implemented] verifySignature"
    Verifies the signature of a message.

<!-- prettier-ignore -->
??? warning "[not yet implemented] encryptMessage"
    Encrypts a message using the recipients public key.

<!-- prettier-ignore -->
??? warning "[not yet implemented] decryptMessage"
    Decrypts a message using the wallets private key stored on the node.

## Node Admin Service

The Node Admin Service provides endpoints for managing and administering a node. It is primarily used by the node admin webinterface.

<!-- prettier-ignore -->
??? warning "[not yet implemented] getPeers"
    Gets a list of all the nodes that this node is directly connected to.

<!-- prettier-ignore -->
??? info "getVersion"
    Gets the nodes version.

<!-- prettier-ignore -->
??? warning "[not yet implemented] upgradeNode"
    Updates the nodes software.

<!-- prettier-ignore -->
??? warning "[not yet implemented] getPendingBlocks"
    Gets all the blocks that are not validated yet.

<!-- prettier-ignore -->
??? info "getPendingBlockCount"
    Gets the count of unvalidated blocks.

<!-- prettier-ignore -->
??? warning "[not yet implemented] setPendingBlockLimit"
    Sets a limit to the amount of blocks that are unvalidated.

<!-- prettier-ignore -->
??? info warning "[not yet implemented] getNodeStatus"
    Gets the status of the node.

<!-- prettier-ignore -->
??? info warning "[not yet implemented] getMode"
    Gets the mode of the node.

<!-- prettier-ignore -->
??? info warning "[not yet implemented] setMode"
    Sets the mode of the node.

<!-- prettier-ignore -->
??? info "getNodeName"
    Gets the name of the node.

<!-- prettier-ignore -->
??? info "setNodeName"
    Sets the name of the node.

<!-- prettier-ignore -->
??? warning "[not yet implemented] getChain"
    Gets chain name. For example _MainNet_, _testNet_ etc.

<!-- prettier-ignore -->
??? warning "[not yet implemented] getLogs"
    Gets the node logs.
