# champ-wasm

champ-wasm exposes champ's wallet generation and decryption/encryption to JavaScript and TypeScript applications.

## Usage

```ts
import init, { Wallet } from "champ-wasm";

await init();

const wallet = Wallet.generate("hunter2");
const { json, address } = Wallet.unlock(wallet, "hunter2");

console.log(json, address); // Don't do this :)
```

## Building

`$ wasm-pack build --target web --release`
