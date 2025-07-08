# Tock Core Notes of 2022-09-02

Attendees:
- Brad Campbell
- Chris Frantz
- Branden Ghena
- Alyssa Haroldsen
- Amit Levy
- Pat Pannuto
- Alexandru Radovici
- Jett Rink
- Leon Schuermann
- Vadim Sukhomlinov
- Johnathan Van Why

## Updates

### Machine-Readable Tockloader Output

- Brad: revisited an old Tockloader issue of adding machine-readable
  output. Now some commands output JSON, for use in scripts.

  Broader goal was to use Tockloader in testing, e.g. checking the
  current state of a board. Could be useful in a CI setting.

- Alyssa: Commitments to stability of output and/or tests? Might we
  worth a tracking issue.

- Brad: Stability of output is a concern, no good solution yet. Maybe
  we should get somebody using it first? Just recently introduced.

- Johnathan: In the long run might make sense to rewrite in Rust,
  given as of now it's the only reason for a Tock contributor to learn
  Python. Maybe it should be architected as a library?

### Progress on Capsule Testing

- Branden: Working on capsule testing. Commands worked with Alyssa's
  code. Trying to get allows and subscribes working; they require
  Process and Grant space to work. Made a whole mock implementation of
  Process. Next step is figuring out how to allocate Grant
  space. Dynamic allocation / big array?

## Tock 2.1

- Leon: Catching up on release tests. Remaining board is the newly
  introduced QEMU RISC-V 32bit "virt" platform, to support VirtIO
  devices. Tested it with apps before the PR was merged, no longer
  works. Will take the weekend to figure this out, if we don't tag the
  release.

- Alex: All STMs have been tested (all boards that I have). STM-F3
  reboots continuously on a fault. Tried to debug this with Hudson,
  have not found the issue yet. In a panic print, reaches the process
  memory table and reboots immediately then.

  Faulting due to memory issues was related to incorrect memory sizes
  in the linker script.

  On the RP2040 IPC faults in an unexpected place. It seems to fault
  randomly if the IPC callback function has any code in it. Other than
  that, all works.

- Branden: Do we find these kinds of failures acceptable in a release?

- Brad: IPC one would be good to solve, but should not hold up the
  release because of that. ARM Cortex M0 support has not been tested
  extensively. Other issues seem acceptable.

- Pat: There once was a Tier-{1,2,3} board support list.

- Leon: Recurring theme, established multiple times that this might be
  good to have again.

- Branden: Is there a place to document issues with "second-class"
  boards then?

- Leon: GitHub issue [#3141](https://github.com/tock/tock/issues/3141)
  collects breaking changes and generally content release notes. Might
  be a appropriate place to document this.

### PR #3175 (syscall: Fix SuccessU32U64 format)

- Leon: As part of Tock 2.0 multiple system call return variants have
  been introduced, documented in TRD 104. By this definition, they are
  part of our ABI contract. For the `SuccessU32U64` variant, it has
  been implemented as to return a 32-bit value in the first, followed
  by a 64-bit value in the following registers, whereas the TRD
  specifies those values to be passed the other way around.

  PR #3175 changed the kernel implementation to match the
  TRD. `libtock-rs` implemented the specification as documented in the
  TRD. Even though we are not using this particular variant upstream,
  it is exposed in the kernel crate's API surface and could have been
  used by downstream users of Tock.

  Issue is that this technically breaks our ABI compatibility. How
  closely are we following semantic versioning, what's our plan here?

- Alyssa: Was this API actually used?

- Brad: We do not know for sure, but likely not.

  Strong proponent to sticking to stability guarantees. We want to
  guarantee that the low-level ABI is backwards compatible. That said,
  the documentation is as intended. `libtock-rs` was written against
  that documentation. We have not evidence of it ever being used. We
  should update our code. It is not a breaking change, in the sense
  that it would not break anything in practice.

- Leon: Supposedly the TRD 104 has been written (and actually adjusted
  during development) to reflect the target platform's calling
  conventions, so adhering to this might actually be more
  efficient. Looked into the generated assembly, but no results yet.

- Amit: We made the same mistake twice, once in the kernel and once in
  `libtock-c`? If we hadn't made the mistake in `libtock-c`, this
  would clearly be just a bugfix.

- Johnathan: Did not actually make the mistake in `libtock-c`, as it
  does not have any code to decode that particular return variant.

- Leon: Touches on the root of my question: at what layer would we
  like to guarantee stability? ABI aka. register interface or
  userspace library ABI as provided with `libtock-c` and `libtock-rs`?

- Amit: If an application is compiled (binary artifact with userspace
  library included in that) for 2.0, it should work on 2.1.

- Leon: That would make our guarantees decoupled from the exact
  userspace library used in compiling applications.

- Johathan: If this breaks someone, it is code that has been written
  against the kernel implementation in code that we cannot see.

- Amit: Seems like this is a bugfix. The API is defined through the
  TRD. If there is someone out there who wrote their own system call
  library, it was probably someone who would have written it against
  the TRD and noticed this bug.

- Leon: Seems fair. If we conclude to view the TRD as the ultimate
  governing document and define stability in its terms, we should be
  consistent in that. Otherwise this is going to be arbitrary.

- Alyssa: Natural for decisions to feel arbitrary in the beginning of
  a project (at the current stage).

- Branden: Thought our argument would be more along what Amit said: if
  an app worked on 2.0, it should work on 2.1. We are breaking that
  here. Maybe this app does not exist, so we are fine now. But it
  seems we did have a decision and we are deciding against it in this
  case.

- Alyssa: Even Rust stable has made decisions which go against its
  theoretical guarantees of stability, for bug fixes or security
  fixes.

- Amit: We always have to make some sort of judgment call about what
  is 2.0 in that particular case. Is it the implementation which has a
  bug, and perhaps someone has written an application to rely on
  that. Or is it the idea / specification of it, which a reasonable
  app has been written against.

- Brad: It is the code, as this is what people compile against.
  However, this is a good example of one of the limits of that
  stance. Certain that there is some ambiguity in our documentation,
  and we cannot use that as an argument to vouch for changing the
  implementation.

- Alyssa: Documentation should be the final decider. In the case of
  ambiguity, the code is what determines the interpretation.

- Branden: If we had used this extensively, would we decide the other
  way around and change the TRD?

- Leon: Third option: new release. Breaking changes are not
  necessarily bad, we just have to communicate them properly.

- Pat: If we would have used this in `libtock-c`, we would have caught
  it, because it would not have worked with the way the TRD is
  currently defined. In this specific instance, it is very unlikely
  that any code has relied on that; some level of pragmatism.

- Amit: For other kinds of mistakes, we can also deprecate a
  implementation and introduce a new one which implements the desired
  approach.

  Does not make sense in this case, because there is inconsistency and
  someone's code would break anyways.

- Brad: Final decision? Leaving #3175 in?

- Branden: Seems like the decision. Anyone opposed?

### Remaining TODOs

- Brad: Waiting on release notes, RP2040 SPI busy issue and last of
  Leon's testing.

- Leon: I've tested all platforms which are actually in use. The
  `qemu_rv32_virt` seems broken, but it's a newly introduced platform
  and not particularly useful without VirtIO support yet, so we can
  leave it out if I don't get it to work before the other issues have
  been resolved.

- Brad: Might do a corresponding Tockloader and elf2tab release as
  well.
