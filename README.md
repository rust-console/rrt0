# `rrt0`

Simple cross-platform runtime / startup (like crt0).

## Supported platforms

* [Nintendo 64](./src/platforms/n64/)

## Primary goals

* Create a sane, platform-specific runtime environment
  * Set the stack pointer
  * Clear the `.bss` section (uninitialized static data)
  * Minimal hardware initialization (e.g. configuring the FPU)
  * Panic handler
* Provide basic I/O primitives
  * `stdout` and `stderr` with configurable streams
