QEMU ARM MPS2 AN386 (Cortex-M4) Platform
=========================================

QEMU's `mps2-an386` machine, emulating ARM's "MPS2 + AN386" CMSDK reference
platform. Built for `thumbv7em-none-eabi`.

The AN386 does have an FPU available, but this board does not yet exercise it
(i.e., it uses the `cortexm4` architecture crate, not `cortexm4f`).

See the [family README](../mps2_base/README.md) for the peripherals these
boards support and for how to run them.

- **`run`**: start Tock under QEMU:

  ```
  $ make run
  [...]
  Running QEMU emulator version 10.2.1 with
   - kernel target/thumbv7em-none-eabi/release/mps2-an386.elf
  To exit type C-a x

  QEMU MPS2 AN386 (Cortex-M4) initialization complete.
  Entering main loop.
  tock$
  ```

- **`run-app`**: start Tock with one or more apps loaded.
