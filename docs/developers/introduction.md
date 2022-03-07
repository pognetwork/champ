# Introduction to pog.network

## The Codebase

```
├── Cargo.lock    # Rust's LockFile, specifing the exact versions for all dependencies
├── Cargo.toml    # Workspace root (we're set up as a monorepo)
├── champ             # All of our rust code
│   ├── dev              # Workspace-wide dev-Dependencies
│   ├── lib                 # Re-usable Code
│   │   ├── champ-wasm          # Wrapper so we can use our code in webbrowsers
│   │   ├── crypto              # Cryptography related code
│   │   ├── encoding            # Encoding related code
│   │   ├── jwt                 # JSON-Web-Tokens
│   │   ├── lulw                # Our JSON wallet format
│   │   ├── pwned               # Pawned Passwords API
│   │   ├── roughtime           # Decentralised Time Syncing
│   │   └── wasm                # Smart Contract Runtime
│   └── node              # Node Specific Code
│       ├── auth                # HTTP Authentication
│       ├── blockpool           # Prioritization of incoming blocks
│       ├── cli                 # Commandline Interface
│       ├── config.rs           # Config File loading
│       ├── consensus           # Consensus Module
│       ├── http.rs             # Integrated HTTP Server for Admin Panel
│       ├── lib.rs              # Entrypoint for embeding champ
│       ├── main.rs             # Entrypoint
│       ├── metrics.rs          # Metrics (Node Health and Stats Endpoint)
│       ├── p2p                 # Peer to Peer
│       ├── rpc                 # gRPC (Our API)
│       ├── state.rs            # Shared State across the node code
│       ├── storage             # Database Code
│       ├── tests               # Shared test code/Integration tests
│       ├── validation          # Block/Transaction validation
│       └── wallets             # Integrated 2Wallet Manager
├── cliff.toml          # Changelog Generation
├── docs                # Documentation
├── justfile            # Makefile
├── mkdocs.yml          # Documentation Index
├── scripts
│   └── Dockerfile      # Dockerfile for nightly builds
└── target              # Rust build results
```

## A Transaction's journey (Work in progress)

- User goes to https://wallet.pog.network
  - React Website is loaded (hosted on cloudflare pages) (build from [pognetwork/catjam](https://github.com/pognetwork/catjam))
- User creates a new Wallet
  - Rust Code is called through `lib/champ-wasm`
    - `lib/crypto` is used to generate a new private key and derive a public key (`lib/crypto/signatures/es25519.rs`)
    - `lib/crypto` is used to encrypt this private key (`lib/crypto/aead/chacha.rs`)
    - `lib/lulw` is used to save this as a JSON file in the LULW format (Specified as [PRC-3](https://pog.network/specification/PIPs/03-LULW/))
  - Wallet is downloaded to the users PC
  - A Wallet Address (which they can use to recieve funds) is generated from their public key (`lib/encoding/account.rs`)

> Let's now assume that the user has recieved 10 POG from their friend and wants to use this to buy some robux.

- The user goes back to the wallet website and selects their previously downloaded wallet file
- The user enters their password and their private key is decrypted (`lib/crypto/aead/chacha.rs`)
- The wallet shows 10 unclaimed POG
- The user presses send and enters the recievers details
- The wallet creates a new block
  - The block format is defined in [pognetwork/proto](https://github.com/pognetwork/proto). We're mainly using the protocol buffers format for data serialization.
  - The wallet includes a claim tranaction pointing to the transaction the user's friend send earlier
  - The wallet adds another send transaction to roblox's wallet address to buy the robux
- This block is send to a pog-node using its gRPC-API
  - The node recieves this add block request (`node/rpc/block.rs`) (currently not implemented)
