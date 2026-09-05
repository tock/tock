QEMU ARM MPS2 AN385 (Cortex-M3) Platform
=========================================

QEMU's `mps2-an385` machine, emulating ARM's "MPS2 + AN385" CMSDK reference
platform. Built for `thumbv7m-none-eabi`.

See the [family README](../mps2_base/README.md) for the peripherals these
boards support and for how to run them.

- **`run`**: start Tock under QEMU:

  ```
  $ make run
  [...]
  Running QEMU emulator version 10.2.1 with
   - kernel target/thumbv7m-none-eabi/release/mps2-an385.elf
  To exit type C-a x

  QEMU MPS2 AN385 (Cortex-M3) initialization complete.
  Entering main loop.
  tock$
  ```

- **`run-app`**: start Tock with one or more apps loaded.
