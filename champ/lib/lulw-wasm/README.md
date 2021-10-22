# LULW-wasm

LULW-wasm exposes champ's wallet generation and decryption/encryption to javascript applications.

## Usage

```ts
import { generateWallet, unlockWallet } from "lulw-wasm";

const wallet = generateWallet("hunter2");
const unlockedWallet = unlockWallet(wallet, "hunter2");

console.log(unlockedWallet); // Don't do this :)
```

## Building

`$ wasm-pack build --target web`
