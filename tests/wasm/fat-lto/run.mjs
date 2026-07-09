// Instantiates the fat-LTO `app.wasm`, runs its constructors, and prints the
// number of registered items and their value sum. Used by the `wasm_fat_lto`
// regression test.
import { readFile } from "node:fs/promises";

const path =
  process.argv[2] ?? "target/wasm32-unknown-unknown/release/app.wasm";

const buf = await readFile(path);
const { instance } = await WebAssembly.instantiate(buf, {});

instance.exports.init();
const len = instance.exports.items_len();
const sum = instance.exports.items_sum();

console.log(`items_len=${len} items_sum=${sum}`);
