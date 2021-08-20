# Tock Core Notes 2021-08-13

## Attendees

- Pat Pannuto
- Leon Schuermann
- Jett Rink
- Philip Levis
- Alexandru Radovici
- Vadim Sukhomlinov
- Johnathan Van Why
- Hudson Ayers
- Brad Campbell
- Amit Levy

## Updates

- Alex: Talks at OSPOCon
  - https://osselc21.sched.com/event/lBQZ/tutorial-managing-raspberry-pi-pico-applications-using-tock-os-and-rust-ioana-culic-alexandru-radovici-wyliodrin?iframe=no&w=100%&sidebar=yes&bg=no
  - https://osselc21.sched.com/event/lANg/tutorial-build-a-green-house-controller-using-the-microbit-v2-and-rust-alexandru-radovici-ioana-culic-wyliodrin?iframe=no&w=100%&sidebar=yes&bg=no

- Alex: Looking to use https://www.unicorn-engine.org/

## Changelog

- Phil: Take a look at https://github.com/tock/tock/pull/2744
- Add comments/suggested as needed.

## Panic Print Code Overhead

- PR: https://github.com/tock/tock/pull/2759
- Able to remove 7 kB from imix build, if you aren't going to use the debug
  print in panic handler.
- Works because variable is const.
- Still need change to kernel crate. Hard for out-of-tree boards.
- Jett: idea: control config struct from rustc features.
- Phil: hard to know what was actually built and what code is actually there.
- Johnathan: might be able to use const generic.
- Hudson: maybe not easy to use. Trait/object safety.
- Alexandru: unless returns `self` should be safe.
- Phil: still strange to have option set in kernel not boards. Maybe have some
  debug object.
- Amit: agree, but need LLVM to do the right thing to remove unused code.
- Amit: could have board define static variable that kernel is dependent on.
- Johnathan: would work, but gets awkward.
- discussion on compiler options, and how those compare or not to rustc config
- Amit: if we want to test all combinations of flags, easier on command line.
- Johnathan: hard to test all combinations.
- Brad: do not agree with using a kernel::config option for this. Core problem
  is Rust does not remove unused trait functions. This is not the same as the
  syscall tracing and process load debug features.
- Phil: pull this out of ProcessStandard?
- Brad: agree, could be in different trait.
- Phil: should be in different object.
- Amit: could use different impl of Process.
- Trait if never used would be elided.
- Phil: having printing object could be useful in other ways.
- Amit: is this a bug/limitation of LLVM/Rust that should fix this for us?
- Hudson: Dead virtual function elimination is what we want, LLVM supports it,
  Rust does not. Trying to see if we can get this added to Rust.
- Amit: are we all at least in support of the config in the short term?
- Brad: no, why not just a very small, easy to cherry-pick commit.
- Amit: strongly in favor of config option rather than that.
- Phil: opposed to config because it is a short-term fix.
- Hudson: config options at least all compile.
- Brad: that's a low bar.
- Hudson: traits can be a combination nightmare too.
- Brad: let's get 2.0 done.
- Phil: supportive of software redesign (decomposition) of process trait.

## HIL buffer lengths too short

- What happens if length longer than buffer in HILs?
  - Truncate length to size of buffer?
  - Return error (fail)?
- Which option should we go with?
- Leon: fail.
- Alex: fail.
- Amit: fail.
- Amit: `length` field is needed because we need to give up ownership of buffer,
  but we need to get the entire buffer back. Could use `LeaseableBuffer`
  instead.
- Leon: nice to use leaseable buffer so do not have to shift bytes around.
- Phil: disagree, harder to use leaseable buffer.
- Phil: SPI truncates in case want to read fewer bytes than write.
- Leon: LeaseableBuffer nice for users, maybe less so for implementors.
- Phil: it would be good to get more experience with LeaseableBuffer.
- Amit: seems consensus is fail fast.

## 2.0

- Hudson: goal still mid August for release?
- Amit: what testing are we still missing?
- Hudson: several boards missing tests.
- Alex: I will ping my student.
- Amit: shouldn't block release on untested boards we don't have.
- Brad: What stability guarantees do we want?
  https://github.com/tock/tock/blob/master/doc/syscalls/README.md
