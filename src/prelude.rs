use core::panic::PanicInfo;

/// This function is called on panic.
#[cfg_attr(target_vendor = "nintendo64", panic_handler)]
#[no_mangle]
fn panic(_panic_info: &PanicInfo<'_>) -> ! {
    #[allow(clippy::empty_loop)]
    loop {}
}
