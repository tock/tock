# Tock Core Notes 2022-01-21

## Attendees
 - Amit Levy
 - Alexandru Radovici
 - Brad Campbell
 - Leon Schuermann
 - Jett Rink
 - Philip Levis
 - Johnathan Van Why
 - Hudson Ayers
 - Vadim Sukhomlinov
 - Pat Pannuto
 - Branden Ghena
 - Alyssa Haroldsen


## Updates
- JVW: libtock-rs. New script for running elf2tab and running binaries via QEMU
  and tockloader. Useful for testing with 2.0.
- Alyssa: libtock-rs. Updates to build system for assembler and archive to work
  with specific addressing requirements.
  - Could we just use LLVM tools?
  - Might be missing assembler.
- Ran into assembler bug with comments on move instructions.


## ufmt experience

- Hudson: considered vendoring ufmt into libtock-rs to replace core formatting
  libraries to reduce binary size and increase performance in some cases.
- Ported ti50 applications to use ufmt on libtock-rs.
- Experimented with using it in the kernel as well.
- Findings
  - Doesn't support all of the same format specifiers. Supports only normal,
    debug, and :#.
  - No support for hex numbers.
  - No support for many number formatting and padding tools.
  - Difficult to write independent code that works with both core and ufmt.
    Would have to limit to just the very limited interface of ufmt.
  - For ti50 apps, saved ~13 kB!
- Now working on ufmt-extended that adds some important features, see what code
  size effects are.
- Is this viable for apps?
- Looking to support most formatting for hex numbers.
- Seems like it could work for most apps.
- Issue: if library doesn't use it, might have to include both corefmt and ufmt.
- ufmt is not "pay for what you use", the entire binary size is added even if
  you don't use certain features.
- If you use a feature that doesn't exist, does it fail at compile time?
  - Yes.
- Any attempt to do the formatting in the kernel?
  - Debug hard to do since the type is complex.
  - For simple types, would be feasible, but would result in more syscalls.
  - Could overlap with console syscall?
    - Still might require many more syscalls.
    - Might want to use formatting more generally.
    - Would need to pass the structure of the thing to be printed.
  - Macro implementation, how much could be done at macro invocation time?
    - Not clear yet.

## AppID

- Phil: state machine in place for loading multiple processes with credentials and without.
- Dummy implementations for async checks (placeholders until RSA implementations exist).
- Three changes to process:
  - New queue init task fn
  - Two credentials functions: mark as pass, mark as fail
- What does toolchain look like for working with credentials?
  - Brad: Adding to tockloader would be straightforward.
  - elf2tab can add a header specifying the footer, or tockloader can insert that.
- Hudson: if someone does not want to use this, what do they pay for in code size?
  - Goal is code is structured so all unused features are elided.

## Process Slice

- Jett: remove ProcessBuffer layer in exchange for a check on access that makes
sure no aliasing.
- Any invalid mutable access to a process slide would fail.
- Checks would be at runtime.
- Phil: we thought about this at allow time, not necessarily use time. Doing at
  use time could lead to unusual failures in certain cases. Checking at
  allow-time adds some overhead (100s of cycles).

## Testing for Tock

- Integration testing? Unit testing for capsules?
- Litex simulation in use.
  https://github.com/tock/tock/blob/master/.github/workflows/litex_sim.yml
- QEMU in use.
- libtock-rs with 2.0 support coming.
- Hardware CI in progress.
