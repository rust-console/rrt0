#![cfg_attr(target_vendor = "nintendo64", feature(alloc_error_handler))]
#![cfg_attr(target_vendor = "nintendo64", feature(asm_experimental_arch))]
#![no_std]

mod math;
mod platforms;
pub mod prelude;

pub use crate::platforms::*;

#[no_mangle]
fn panic_main() -> ! {
    panic!("Main cannot return");
}
