extern crate alloc;

use alloc::vec::Vec;
use core::{ffi::CStr, mem};

// ===================================================================
// AIX loader constants
// ===================================================================
const L_GETXINFO: i32 = 0x0000_0008;   // from original ldr.h

// ===================================================================
// ld_xinfo – always uses 64‑bit pointer fields (__ptr64)
// ===================================================================
#[repr(C)]
pub struct LdXinfo {
    pub ldinfo_next: u32,
    pub ldinfo_flags: u16,
    pub ldinfo_filename: u16,       // offset to filename string
    pub _file: u64,                 // fd / core_offset union (always 8 bytes)
    pub ldinfo_textorg: u64,        // base of loaded XCOFF image (__ptr64)
    pub ldinfo_textsize: u64,
    pub ldinfo_dataorg: u64,
    pub ldinfo_datasize: u64,
    // remaining fields (tdatasize, tbsssize, …) omitted for brevity
}

// ===================================================================
// XCOFF 64‑bit file header & section header
// ===================================================================
#[repr(C, packed)]
pub struct Filehdr64 {
    pub f_magic: u16,               // 0x01F7 for 64‑bit XCOFF
    pub f_nscns: u16,
    pub f_timdat: i32,
    pub f_symptr: u64,
    pub f_nsyms: i32,
    pub f_opthdr: u16,              // size of auxiliary header
    pub f_flags: u16,
}

#[repr(C, packed)]
pub struct Scnhdr64 {
    pub s_name: [u8; 8],
    pub s_paddr: u64,
    pub s_vaddr: u64,
    pub s_size: u64,
    pub s_scnptr: u64,
    pub s_relptr: u64,
    pub s_lnnoptr: u64,
    pub s_nreloc: u32,
    pub s_nlnno: u32,
    pub s_flags: i32,
}

// ===================================================================
// XCOFF 32‑bit file header & section header
// ===================================================================
#[repr(C, packed)]
pub struct Filehdr32 {
    pub f_magic: u16,               // 0x01DF for 32‑bit XCOFF
    pub f_nscns: u16,
    pub f_timdat: i32,
    pub f_symptr: i32,
    pub f_nsyms: i32,
    pub f_opthdr: u16,
    pub f_flags: u16,
}

#[repr(C, packed)]
pub struct Scnhdr32 {
    pub s_name: [u8; 8],
    pub s_paddr: i32,
    pub s_vaddr: i32,
    pub s_size: i32,
    pub s_scnptr: i32,
    pub s_relptr: i32,
    pub s_lnnoptr: i32,
    pub s_nreloc: u16,
    pub s_nlnno: u16,
    pub s_flags: i32,
}

// ===================================================================
// Syscall declaration
// ===================================================================
extern "C" {
    fn loadquery(flags: i32, buffer: *mut u8, len: i32) -> i32;
}

// ===================================================================
// Public helper: find in‑memory address of a named section
// ===================================================================
/// Returns a pointer into the loaded image of the current process
/// where the section `name` begins, or `None` if not found.
pub unsafe fn find_section_address(name: &str) -> Option<(*const u8, usize)> {
    // The buffer may need to grow – start with 64 KiB and retry if full.
    let mut buf_size = 64 * 1024;
    loop {
        let mut buffer: Vec<u8> = alloc::vec![0u8; buf_size];
        if loadquery(L_GETXINFO, buffer.as_mut_ptr(), buf_size as i32) == -1 {
            // On failure, try a larger buffer (capped at 16 MiB)
            if buf_size < 16 * 1024 * 1024 {
                buf_size *= 2;
                continue;
            }
            return None;
        }

        // Walk the linked list of ld_xinfo entries
        let mut current = buffer.as_ptr();
        loop {
            let info = &*(current as *const LdXinfo);
            let base = info.ldinfo_textorg as *const u8;
            if !base.is_null() {
                // ---------- detect XCOFF bitness from magic ----------
                let magic = *(base as *const u16);
                match magic {
                    0x01DF | 0x01EF => {
                        // 32‑bit
                        let fhdr = &*(base as *const Filehdr32);
                        let scn_start = base.add(
                            mem::size_of::<Filehdr32>() + fhdr.f_opthdr as usize,
                        );
                        let sections = core::slice::from_raw_parts(
                            scn_start as *const Scnhdr32,
                            fhdr.f_nscns as usize,
                        );
                        for scn in sections {
                            let scn_name = CStr::from_bytes_until_nul(&scn.s_name)
                                .unwrap_or_default()
                                .to_string_lossy();
                            if scn_name == name {
                                return Some((base.add(scn.s_vaddr as usize), scn.s_size as usize));
                            }
                        }
                    }
                    0x01F7 => {
                        // 64‑bit
                        let fhdr = &*(base as *const Filehdr64);
                        let scn_start = base.add(
                            mem::size_of::<Filehdr64>() + fhdr.f_opthdr as usize,
                        );
                        let sections = core::slice::from_raw_parts(
                            scn_start as *const Scnhdr64,
                            fhdr.f_nscns as usize,
                        );
                        for scn in sections {
                            let scn_name = CStr::from_bytes_until_nul(&scn.s_name)
                                .unwrap_or_default()
                                .to_string_lossy();
                            if scn_name == name {
                                return Some((base.add(scn.s_vaddr as usize), scn.s_size as usize));
                            }
                        }
                    }
                    _ => {} // unknown magic – skip
                }
            }

            if info.ldinfo_next == 0 {
                return None; // end of list, not found
            }
            current = current.add(info.ldinfo_next as usize);
        }
    }
}
