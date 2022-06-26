pub use crate::{dbg, eprint, eprintln, print, println};
use core::panic::PanicInfo;
pub use no_stdout::stdout;

/// This function is called on panic.
#[cfg_attr(target_vendor = "nintendo64", panic_handler)]
#[no_mangle]
fn panic(panic_info: &PanicInfo<'_>) -> ! {
    eprintln!("Application: {}", panic_info);

    #[allow(clippy::empty_loop)]
    loop {}
}
