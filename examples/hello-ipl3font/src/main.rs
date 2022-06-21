#![deny(clippy::all)]
#![no_main]
#![no_std]

use n64lib::{ipl3font, prelude::*, vi};

// Colors are 5:5:5:1 RGB with a 16-bit color depth.
#[allow(clippy::unusual_byte_groupings)]
const WHITE: u16 = 0b11111_11111_11111_1;

#[no_mangle]
fn main() {
    println!("It is safe to print without initializing `stdout`, you just won't see this!");

    let io_backend = n64lib::io::init();

    println!("I/O initialized with {:?}", io_backend);
    println!();

    println!("Now that `stdout` has been configured...");
    eprintln!("These macros work about how you expect!");
    println!();
    println!("Supports formatting: {:#06x}", WHITE);
    dbg!(WHITE);
    println!();

    vi::init();

    ipl3font::draw_str_centered(WHITE, "Hello, world!");
    vi::swap_buffer();

    println!("Panic also works :)");
    println!("Returning from main will panic and halt... Let's do that now!");
    println!();
}
