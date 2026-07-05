/**
 * Loads `wasm_rust.wasm` built for `wasm32-unknown-unknown`.
 *
 * libc-print expects an `env::write` import on this target (see libc-print README); without it,
 * instantiation/`_initialize` would fail once the ctor tries to print.
 *
 * Run (from repo root): `npx tsx tests/wasm/js/test-wasm32-unknown-unknown.ts`
 */
import { readFile } from "node:fs/promises";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const wasmPath = join(here, "../rust/target/wasm32-unknown-unknown/debug/wasm_rust.wasm");

type InstanceExports = WebAssembly.Exports & {
  memory: WebAssembly.Memory;
  _initialize: () => void;
  _start: () => void;
  _call_atexit: (ptr: number) => void;
};

async function main(): Promise<void> {
  const ctx: { memory: WebAssembly.Memory | null } = { memory: null };

  function write(fd: number, ptr: number, len: number): number {
    const mem = ctx.memory;
    if (!mem) {
      throw new Error("write called before memory was bound");
    }
    const bytes = Buffer.from(new Uint8Array(mem.buffer, ptr, len));
    if (fd === 2) {
      process.stderr.write(bytes);
    } else {
      process.stdout.write(bytes);
    }
    return len;
  }

  const atexitHandlers: number[] = [];
  function atexit(ptr: number) {
    console.log("atexit: ", ptr);
    if (atexitHandlers) {
        atexitHandlers.push(ptr);
    }
  }

  const buf = await readFile(wasmPath);
  console.log("Instantiating...");
  const wasmModule = await WebAssembly.compile(buf);
  const instance = await WebAssembly.instantiate(wasmModule, {
    env: {
        write,
        atexit,
    },
  });
  console.log("Instantiated");

  ctx.memory = instance.exports.memory as WebAssembly.Memory;
  console.log(instance.exports);
  console.log("Starting...");
  (instance.exports as InstanceExports)._start();
  console.log("Started");

  while (atexitHandlers.length > 0) {
    const ptr = atexitHandlers.pop();
    console.log("Calling atexit: ", ptr);
    (instance.exports as InstanceExports)._call_atexit(ptr!);
  }
}

main().catch((err: unknown) => {
  console.error(err);
  process.exit(1);
});
