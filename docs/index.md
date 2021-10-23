---
title: Getting Started
template: main.html
---

## Getting Started

### 1. Install Requirements

#### Development

- [`rust` >= `1.55.x`](https://rustup.rs/)
- `just` >= `0.10.x` &nbsp;(`$ cargo install just`)
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

### Run node

```
$ just node
```
