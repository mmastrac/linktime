//! Minimal 16550 COM1 output and QEMU shutdown via raw port I/O.
//!
//! OVMF does not mirror its console to the serial port in every build, so the
//! runtime writes COM1 directly. That keeps captured output deterministic and
//! independent of firmware configuration.

use core::arch::asm;
use core::fmt::{self, Write};

const COM1: u16 = 0x3F8;

#[inline]
fn outb(port: u16, val: u8) {
    unsafe {
        asm!("out dx, al", in("dx") port, in("al") val, options(nomem, nostack, preserves_flags));
    }
}

#[inline]
fn inb(port: u16) -> u8 {
    let val: u8;
    unsafe {
        asm!("in al, dx", out("al") val, in("dx") port, options(nomem, nostack, preserves_flags));
    }
    val
}

#[inline]
fn outw(port: u16, val: u16) {
    unsafe {
        asm!("out dx, ax", in("dx") port, in("ax") val, options(nomem, nostack, preserves_flags));
    }
}

/// Configure COM1 for 38400 8N1 polled output.
pub fn init() {
    outb(COM1 + 1, 0x00); // disable interrupts
    outb(COM1 + 3, 0x80); // DLAB
    outb(COM1, 0x03); // divisor low (38400)
    outb(COM1 + 1, 0x00); // divisor high
    outb(COM1 + 3, 0x03); // 8N1
    outb(COM1 + 2, 0xC7); // enable + clear FIFO
    outb(COM1 + 4, 0x0B); // RTS/DSR
}

struct Serial;

impl Write for Serial {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for b in s.bytes() {
            while inb(COM1 + 5) & 0x20 == 0 {}
            outb(COM1, b);
        }
        Ok(())
    }
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments<'_>) {
    let _ = Serial.write_fmt(args);
}

/// Powers the machine off through the QEMU q35 ACPI PM1a control register, so
/// the QEMU process exits cleanly once the run is done.
pub fn shutdown() -> ! {
    outw(0x604, 0x2000);
    // ACPI shutdown normally ends the process here. Spin as a fallback.
    loop {
        unsafe { asm!("hlt", options(nomem, nostack, preserves_flags)) };
    }
}

#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => { $crate::serial::_print(format_args!("{}\n", format_args!($($arg)*))) };
}
