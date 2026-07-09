// WASI (preview1) loader for `wasm32-wasip1` builds.
//
// Usage: node --no-warnings loader-wasi.mjs <path-to-wasm>
// (`--no-warnings` silences node's experimental-WASI notice so the wasm
// program's stdout is the only output.)
import { readFile } from "node:fs/promises";
import { argv, env, exit } from "node:process";
import { WASI } from "node:wasi";

const wasi = new WASI({
  version: "preview1",
  args: argv,
  env: Object.fromEntries(Object.entries(env).filter(([, v]) => v !== undefined)),
});

const module = await WebAssembly.compile(await readFile(argv[2]));
const instance = await WebAssembly.instantiate(module, {
  wasi_snapshot_preview1: wasi.wasiImport,
});
exit(wasi.start(instance));
