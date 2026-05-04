/**
 * Support function for `link-section` crate.
 */
export function readCustomSection(
  wasmModule: WebAssembly.Module,
  wasmInstance: WebAssembly.Instance,
  namePtr: number,
  nameLength: number,
  targetPtr: number,
  targetLength: number,
): number {
    const memory = wasmInstance.exports.memory as WebAssembly.Memory;
    const nameBytes = new Uint8Array(memory.buffer, namePtr, nameLength);
    const sectionName = new TextDecoder().decode(nameBytes);

    const sections = WebAssembly.Module.customSections(wasmModule, sectionName);
    if (sections.length === 0) {
        return 0;
    }

    const section = sections[0];
    const need = section.byteLength;
    if (targetLength < need) {
        return need;
    }

    new Uint8Array(memory.buffer, targetPtr, need).set(new Uint8Array(section));
    return need;
}