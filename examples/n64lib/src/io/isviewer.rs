use core::{
    fmt::{self, Write},
    ptr::{read_volatile, write_volatile},
};
use no_stdout::StdOut;

struct Stream;

const IS64_MAGIC: *mut u32 = 0xB3FF_0000 as *mut u32;
const IS64_SEND: *mut u32 = 0xB3FF_0014 as *mut u32;
const IS64_BUFFER: *mut u32 = 0xB3FF_0020 as *mut u32;

// Rough estimate based on Cen64
const BUFFER_SIZE: usize = 0x1000 - 0x20;

impl Write for &Stream {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        print(s);
        Ok(())
    }
}

impl StdOut for Stream {
    // Defer to the `Write` impl with a little reborrow trick.
    fn write_fmt(&self, args: fmt::Arguments) -> fmt::Result {
        fmt::write(&mut &*self, args)?;
        Ok(())
    }

    // The rest are not required for no-stdout to operate, but they are required to build.
    fn write_bytes(&self, _bytes: &[u8]) -> fmt::Result {
        todo!();
    }

    fn write_str(&self, _s: &str) -> fmt::Result {
        todo!();
    }

    fn flush(&self) -> fmt::Result {
        todo!();
    }
}

/// Check if Intelligent Systems Viewer 64 is available.
fn is_is64() -> bool {
    let magic = u32::from_be_bytes(*b"IS64");

    // SAFETY: It is always safe to read and write the magic value; static memory-mapped address.
    unsafe {
        write_volatile(IS64_MAGIC, magic);
        read_volatile(IS64_MAGIC) == magic
    }
}

/// Print a string to IS Viewer 64.
///
/// # Panics
///
/// Asserts that the maximum string length is just under 4KB.
fn print(string: &str) {
    assert!(string.len() < BUFFER_SIZE);

    let bytes = string.as_bytes();

    // Write one word at a time
    // It's ugly, but it optimizes really well!
    for (i, chunk) in bytes.chunks(4).enumerate() {
        let val = match *chunk {
            [a, b, c, d] => (a as u32) << 24 | (b as u32) << 16 | (c as u32) << 8 | (d as u32),
            [a, b, c] => (a as u32) << 24 | (b as u32) << 16 | (c as u32) << 8,
            [a, b] => (a as u32) << 24 | (b as u32) << 16,
            [a] => (a as u32) << 24,
            _ => unreachable!(),
        };

        // SAFETY: Bounds checking has already been performed.
        unsafe { write_volatile(IS64_BUFFER.add(i), val) };
    }

    // Write the string length
    // SAFETY: It is always safe to write the length; static memory-mapped address.
    unsafe { write_volatile(IS64_SEND, bytes.len() as u32) };
}

/// Initialize global I/O for IS Viewer 64.
///
/// Returns `true` when IS Viewer 64 has been detected.
pub fn init() -> bool {
    if is_is64() {
        let _ = no_stdout::init(&Stream);

        return true;
    }

    false
}
