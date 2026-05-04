/**
 * Run (from repo root): `npx tsx tests/wasm/js/test-wasm32-wasi.ts`
 */
import { readFile } from "node:fs/promises";
import { dirname, join } from "node:path";
import { argv } from "node:process";
import { fileURLToPath } from "node:url";
import { WASI } from "node:wasi";

import { readCustomSection } from "./link-section.mts";

const here = dirname(fileURLToPath(import.meta.url));
const wasmPath = join(here, "../rust/target/wasm32-wasip1/debug/wasm_rust.wasm");

async function main(): Promise<void> {
  const wasi = new WASI({
    version: "preview1",
    args: argv,
    env: Object.fromEntries(
      Object.entries(process.env).filter((e): e is [string, string] => e[1] !== undefined),
    ),
  });

  const wasmBuf = await readFile(wasmPath);
  const wasmModule = await WebAssembly.compile(wasmBuf);
  const instance = await WebAssembly.instantiate(wasmModule, {
    wasi_snapshot_preview1: wasi.wasiImport,
    env: {
        read_custom_section: (...args: [number, number, number, number]) => readCustomSection.apply(null, [wasmModule, instance, ...args]),
    },
  });

  const exitCode = wasi.start(instance);
  process.exit(exitCode);
}

main().catch((err: unknown) => {
  console.error(err);
  process.exit(1);
});
