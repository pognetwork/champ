# champ-wasm

champ-wasm exposes champ's wallet generation and decryption/encryption to JavaScript and TypeScript applications.

## Usage

```ts
import init, { Wallet } from "champ-wasm";

await init();

const wallet = Wallet.generate("hunter2");
const unlockedWallet = Wallet.unlock(wallet, "hunter2");

console.log(unlockedWallet); // Don't do this :)
```

## Building

`$ wasm-pack build --target web --release`
