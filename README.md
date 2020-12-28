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

The `panic` function must be imported with `pub use`, or you will get missing-symbol errors at link time.

```rust
pub use rrt0::panic;
```
