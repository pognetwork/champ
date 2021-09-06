---
title: Getting Started
template: main.html
---

## Getting Started

### 1. Install Requirements

- [`rust` >= `1.52.0`](https://rustup.rs/)
- `just` >= `0.9.4` &nbsp;(`$ cargo install just`)
- build tools ([windows](https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2019))

<br>

Required for using RocksDB (`--features backend-rocksdb`):

- `clang`/`llvm` ([windows](https://llvm.org/builds/))

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
