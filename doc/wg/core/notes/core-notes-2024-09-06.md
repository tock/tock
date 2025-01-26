# Tock Meeting Notes 2024-09-06

## Attendees

- Branden Ghena
- Brad Campbell
- Johnathan Van Why
- Amit Levy
- Ben Prevor
- Tyler Potyondy
- Pat Pannuto
- Hudson Ayers


## Updates

- Tyler: Doing some research in to low power Tock, specifically using the type
  system.
  - Default kernel using 1.2 mA on nRF52. Doing nothing except running UART for
    process console. Otherwise it is about 200 uA.
  - Using a $90 power monitor (Power Profiler Kit 2).
- Amit: Hacking on SMA Q3 smartwatch. (https://shop.espruino.com/banglejs2)
  - Exposing some issues around buffer management in HILs and screen stack.
  - Screen is unique with its own special color format.
  - Question: where to bring up issues with screen?
    - Brad: Ideally, the tracking issue should be the hub. But any screen
      changes are fairly difficult, so helpful to know what the goal/intent is
      before starting a bunch of changes.
- Hudson: Still working on nightly asm const PR.
  - Still working on a build.rs/cargo test solution that is suitable.
  - Difficult to manage the precedence of how options are selected.
  - Brad: I still think a proper build crate for Tock would make a lot of
    subtlety much more clear.


## Leasable Buffers

- SubSlice in upstream kernel, PacketBuffer in the works.
- Would be helpful to use SubSlice in SPI HIL.
- Question: what is the relationship between PacketBuffer and SubSlice?
  - API: more adjustable slice and unslice with PacketBuffer. Can take/hide
    arbitrary bytes with PacketBuffer.
  - PacketBuffer provides guarantees about space at compile time.
- Question: what is the underlying representation of PacketBuffer?
  - SubSlice is just a Rust slice and an active range.
  - PacketBuffer is similar to SubSlice, has slice and active range.
- How far are we from being able to try out PacketBuffer?
  - Unclear.
- How aggressively should we switch to Leasable Buffers?
  - Brad: Let's do it! https://github.com/tock/tock/issues/3504
  - Branden: We don't need to wait for PacketBuffer.
- Hopefully easier to switch SubSlice -> PacketBuffer than [u8] to SubSlice.
- It can be tricky to switch to SubSlice in a meaningful way. Easy to escape
  hatch.
- Fairly easy to switch SPI, but not getting full benefit from SubSlice.
- Two aspects: enabling new capsules to be able to use SubSlice, and having good
  reference code to copy from.
  - Hopefully switching the HIL is the key to allow new capsules to be able to
    embrace SubSlice.
- There are three versions: SubSlice, SubSliceMut, SubSliceMutImmut to support
  different mutability of buffers.
  - Callbacks make using SubSliceMutImmut hard, need to ensure you get a mutable
    slice back. Leads to runtime checks.


## Updating Rust

- CHERI needs out-of-tree compiler. That version is slightly behind latest Rust.
- How often do we update Rust?
- At the point of removing nightly features.
- We could want stable naked functions.
- Our current cadence is we update roughly every two months plus when something
  new is added we want.
  - Compiler changes might be slowing down, but there are a lot of other tools
    we use that see more frequent changes.
- Could use nightlies that correspond to Rust stable versions.
  - https://forge.rust-lang.org/
  - Seems like a reasonable option.


## Release???

- How close are we to using treadmill for testing for a release?
  - Basic functionality there (reading console output).
  - Issues around starting jobs.
  - Some basic tests are there.
- Seems like a good time based on the current PRs. More features than fixes or
  low-level changes.
