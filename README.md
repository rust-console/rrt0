# `rrt0`

Simple cross-platform runtime / startup (like crt0).

## Supported platforms

* [Nintendo 64](./src/platforms/n64/)

## Primary goals

* Create a sane, platform-specific runtime environment
  * Set the stack pointer
  * Clear the `.bss` and `.sbss` sections (uninitialized static data)
  * Minimal hardware initialization (e.g. configuring the FPU)
  * Panic handler

## Usage

Here is a small template to get you started:

```rust
#![no_main]
#![no_std]

pub use rrt0::prelude::*;

#[no_mangle]
fn main() -> ! {
    // Do cool stuff, here!
    loop {}
}
```

If `main` returns, the startup code will panic.

See [examples](./examples) for more complete projects to get you started.
