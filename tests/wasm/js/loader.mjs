// Loader for `wasm32-unknown-unknown` builds.
//
// libc-print expects an `env::write` import on this target (see libc-print
// README); without it, the first ctor `libc_println!` would trap. Destructors
// are registered through `env::atexit` during `_start` and drained afterwards
// via the module's `_call_atexit` export.
//
// Usage: node loader.mjs <path-to-wasm>
import { readFile } from "node:fs/promises";

const ctx = { memory: null };

function write(fd, ptr, len) {
  const bytes = Buffer.from(new Uint8Array(ctx.memory.buffer, ptr, len));
  (fd === 2 ? process.stderr : process.stdout).write(bytes);
  return len;
}

const atexitHandlers = [];
function atexit(ptr) {
  atexitHandlers.push(ptr);
}

const buf = await readFile(process.argv[2]);
const { instance } = await WebAssembly.instantiate(buf, { env: { write, atexit } });
ctx.memory = instance.exports.memory;
instance.exports._start();
while (atexitHandlers.length > 0) {
  instance.exports._call_atexit(atexitHandlers.pop());
}
