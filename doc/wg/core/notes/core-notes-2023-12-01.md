# Tock Meeting Notes 12/01/23

## Attendees
- Branden Ghena
- Alyssa Haroldsen
- Jonathan Van Why
- Alexandru Radovici
- Phil Levis
- Brad Campbell


## Updates

- Brad: Work on https://github.com/tock/tock/pull/3653
  - New board with nRF52840 and LoRa (LR1110)
- Branden: NWWG: Update on OpenThread integration with Tock
  - C library from OpenThread foundation (everyone uses)
  - Plan is to re-use this library as well
  - Designs: service in userland or encapsulated in kernel (Leon's approach)
  - Phil: is code open, or compiled? Open, have all files.
  - Phil: meet with J. Hui? 
  - Observation: Linux includes Rust in C kernel, Tock includes C in Rust kernel!
  - Phil: RF233 started as userspace driver, eventually stack in kernel
  - Alex: Challenge with asynchronous kernel
  - Brad: Does OT library sit on existing 6LoWPAN in kernel?
    - Some of it. Not directly on HW, would use some of in-kernel stack.
- JVW: New Rust code size working group.
  - JVW planning on trying to participate


## libtock-c: newlib and libc++

- Brad: https://github.com/tock/libtock-c/pull/353
- Background:
  - Compatibility problems:
    - GCC10 compiler doesn't work with GCC13 headers
    - GCC13 doesn't work with GCC10 compiler
    - Newlib 4.3 doesn't work with GCC10
  - It's hard to get riscv compilers and riscv newlib
- We now compile newlib and libc++ for all architectures and pacakge headers
  - Docker files for reproducibility
- Upsides
  - Can update newlib since we retain old version of gcc10 (ubuntu)
  - Can compile riscv by default
  - No longer have hidden dependency on newlib (for headers)
  - Clean mechanism for compiling and distributing compiled artifacts
- Downsides
  - Compilation is now varied depending on the compiler version the user has
  - It's possible the first person to get gcc14 (or later) will run into compatibility issues
  - Larger binary downloads (one time cost)
- Thoughts?
  - Seems ok


## Dialpad??

- Could switch to google meet via OpenTitan
- Could switch to zoom
- We are using Dialpad via tock org
- Bummer to switch right after getting tock rooms for different meetings
- Not clear there is enough need to switch right now
