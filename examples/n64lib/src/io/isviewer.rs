use core::{
    fmt::{self, Write},
    mem::size_of,
    ptr::{read_volatile, write_volatile},
};
use no_stdout::StdOut;

struct Stream;

const IS64_MAGIC: *mut u32 = 0xB3FF_0000 as *mut u32;
const IS64_READ_HEAD: *mut u32 = 0xB3FF_0004 as *mut u32;
const IS64_WRITE_HEAD: *mut u32 = 0xB3FF_0014 as *mut u32;
const IS64_BUFFER: *mut u32 = 0xB3FF_0020 as *mut u32;

// Based on Cen64
const BUFFER_SIZE: usize = 0x10000 - 0x20;

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
        write_volatile(IS64_READ_HEAD, 0);
        write_volatile(IS64_WRITE_HEAD, 0);

        read_volatile(IS64_MAGIC) == magic
    }
}

/// Print a string to IS Viewer 64.
///
/// # Panics
///
/// Asserts that the maximum string length is just under 64KB.
fn print(string: &str) {
    assert!(string.len() < BUFFER_SIZE);

    // SAFETY: It is always safe to get the write head; static memory-mapped address.
    let read_head = unsafe { read_volatile(IS64_READ_HEAD) } as usize;
    let mut write_head = unsafe { read_volatile(IS64_WRITE_HEAD) } as usize;

    // Ensure there is enough free space in the ring buffer to store the string.
    let free_space = if read_head > write_head {
        read_head - write_head
    } else {
        BUFFER_SIZE - write_head + read_head
    };
    if free_space < string.len() {
        return;
    }

    let word_size = size_of::<u32>();
    let mask = word_size - 1;

    let bytes = string.as_bytes();
    let start = write_head & mask;
    let align = (word_size - start) & mask;
    let len = align.min(bytes.len());

    if start > 0 {
        // Slow path: Combine string bytes with existing word data in the buffer.
        let shift = ((align - len) * 8) as u32;
        let (val, data_mask) = match bytes[..len] {
            [a, b, c] => ((a as u32) << 16 | (b as u32) << 8 | (c as u32), 0xff00_0000),
            [a, b] => (
                ((a as u32) << 8 | (b as u32)) << shift,
                0xffff_ffff ^ (0xffff << shift),
            ),
            [a] => ((a as u32) << shift, 0xffff_ffff ^ (0xff << shift)),
            _ => unreachable!(),
        };

        let offset = (write_head & !mask) / word_size;

        // SAFETY: Bounds checking has already been performed.
        unsafe { combine(offset, data_mask, val) };

        write_head += len;
    }

    // Get the string remainder, this aligns the output buffer to a word boundary.
    // It may be an empty slice.
    let bytes = &bytes[len..];

    // Write one word at a time
    // It's ugly, but it optimizes really well!
    for (i, chunk) in bytes.chunks(word_size).enumerate() {
        let (val, data_mask) = match *chunk {
            [a, b, c, d] => (
                (a as u32) << 24 | (b as u32) << 16 | (c as u32) << 8 | (d as u32),
                0x0000_0000,
            ),
            [a, b, c] => (
                (a as u32) << 24 | (b as u32) << 16 | (c as u32) << 8,
                0x0000_00ff,
            ),
            [a, b] => ((a as u32) << 24 | (b as u32) << 16, 0x0000_ffff),
            [a] => ((a as u32) << 24, 0x00ff_ffff),
            _ => break,
        };

        let offset = (write_head / word_size + i) % (BUFFER_SIZE / word_size);

        // Combine existing word data in the buffer when writing an incomplete word.
        if chunk.len() < word_size {
            // SAFETY: Bounds checking has already been performed.
            unsafe { combine(offset, data_mask, val) };
        } else {
            // SAFETY: Bounds checking has already been performed.
            unsafe { write_volatile(IS64_BUFFER.add(offset), val) };
        }
    }

    // Write the string length
    let write_head = ((write_head + bytes.len()) % BUFFER_SIZE) as u32;
    // SAFETY: It is always safe to update the write head; static memory-mapped address.
    unsafe { write_volatile(IS64_WRITE_HEAD, write_head) };
}

/// Combine a word value with the existing word data at the given offset.
///
/// # Safety
///
/// The caller is responsible for bounds checking the offset.
unsafe fn combine(offset: usize, mask: u32, val: u32) {
    let word = read_volatile(IS64_BUFFER.add(offset)) & mask;
    write_volatile(IS64_BUFFER.add(offset), word | val);
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
