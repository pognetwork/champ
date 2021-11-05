# pog-wasm

pog-wasm exposes champ's wallet generation and decryption/encryption to JavaScript and TypeScript applications.

## Usage

```ts
import { Wallet } from "@pognetwork/champ-wasm";

const wallet = Wallet.generate("hunter2");
const unlockedWallet = Wallet.unlock(wallet, "hunter2");

console.log(unlockedWallet); // Don't do this :)
```

## Building

`$ wasm-pack build --target web`
