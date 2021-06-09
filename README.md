# PogChamp
> POG's reference implementation in rust

<big><pre>
**champ**
  ├── lib
  │   ├── consensus
  │   ├── crypto
  │   ├── db
  │   ├── log
  │   ├── p2p
  │   └── rpc
  ├── node
  └── wallet</pre></big>


## Getting started
### 1. Install Requirements

* [`rust` >= `1.52.0`](https://rustup.rs/)
* `just` >= `0.9.4` &nbsp;(`$ cargo install just`)

### 2. Clone Repo

```bash
$ git clone https://github.com/pognetwork/champ.git && cd champ
```

## Development

### List all Commands
```bash
$ just
```

### Run node/wallet
```
$ just node
$ just wallet
```