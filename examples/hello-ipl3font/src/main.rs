#![deny(clippy::all)]
#![no_main]
#![no_std]
// #![feature(asm_experimental_arch)]

use core::fmt::Write;
use n64lib::{ipl3font, prelude::*, vi};

// Colors are 5:5:5:1 RGB with a 16-bit color depth.
#[allow(clippy::unusual_byte_groupings)]
const WHITE: u16 = 0b11111_11111_11111_1;

#[no_mangle]
fn main() {
    // println!("It is safe to print without initializing `stdout`, you just won't see this!");

    // let _io_backend = n64lib::io::init();

    // println!("I/O initialized with {:?}", io_backend);
    // println!();

    // println!("Now that `stdout` has been configured...");
    // eprintln!("These macros work about how you expect!");
    // println!();
    // println!("Supports formatting: {:#06x}", WHITE);
    // dbg!(WHITE);
    // println!();

    vi::init();

    let (value1, value2, value3) = test_pi_bus();

    // ipl3font::draw_str_centered(WHITE, "Hello, world!");

    let mut s = arrayvec::ArrayString::<32>::new();

    const HEIGHT: usize = ipl3font::HEIGHT + 2;
    let mut h = HEIGHT;

    let _ = write!(&mut s, "before: {value1:#010x}");
    ipl3font::draw_str(ipl3font::WIDTH * 2, h, WHITE, &s);

    s.clear();
    h += HEIGHT;
    let _ = write!(&mut s, "after:  {value2:#010x}");
    ipl3font::draw_str(ipl3font::WIDTH * 2, h, WHITE, &s);

    s.clear();
    h += HEIGHT;
    let _ = write!(&mut s, "dma:    {value3:#010x}");
    ipl3font::draw_str(ipl3font::WIDTH * 2, h, WHITE, &s);

    // let value4: u32;
    // unsafe { core::arch::asm!("mfc0 {}, $15", out(reg) value4) };

    // s.clear();
    // h += HEIGHT;
    // h += HEIGHT;
    // let _ = write!(&mut s, "PRId:   {value4:#010x}");
    // ipl3font::draw_str(ipl3font::WIDTH * 2, h, WHITE, &s);

    vi::swap_buffer();

    // println!("Panic also works :)");
    // println!("Returning from main will panic and halt... Let's do that now!");
    // println!();
}

fn test_pi_bus() -> (u32, u32, u32) {
    use core::ptr::{read_volatile, write_volatile};

    const HEADER: *mut u32 = 0xb000_0008 as *mut u32;
    // const HEADER: *mut u32 = 0xb800_02ec as *mut u32;
    // const HEADER: *mut u32 = 0xb800_0000 as *mut u32;

    // unsafe { write_volatile(HEADER, 0x12345678) };
    let value1 = unsafe { core::ptr::read_volatile(HEADER) };
    unsafe { core::ptr::write_volatile(HEADER, 0xa5a5_a5a5) };
    let value2 = unsafe { core::ptr::read_volatile(HEADER) };

    const PI_DRAM_ADDR_REG: *mut u32 = 0xa460_0000 as *mut u32;
    const PI_CART_ADDR_REG: *mut u32 = 0xa460_0004 as *mut u32;
    // const PI_RD_LEN_REG: *mut u32 = 0xa460_0008 as *mut u32;
    const PI_WR_LEN_REG: *mut u32 = 0xa460_000c as *mut u32;
    const PI_STATUS_REG: *mut u32 = 0xa460_0010 as *mut u32;

    unsafe {
        write_volatile(PI_DRAM_ADDR_REG, 0x0040_0000);
        write_volatile(PI_CART_ADDR_REG, (HEADER as u32) & 0x1fff_ffff);

        write_volatile(PI_WR_LEN_REG, 4 - 1);

        while read_volatile(PI_STATUS_REG) & 1 == 1 {}

        (value1, value2, read_volatile(0xa040_0000 as *const u32))
    }

    // println!("u64 from header: {value1:#018x}");
    // println!();
}
