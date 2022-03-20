# Tock Core Notes 2022-02-11

## Attending

- Hudson Ayers
- Brad Campbell
- Branden Ghena
- Philip Levis
- Amit Levy
- Alexandru Raduvici
- Jett Rink
- Leon Schuermann
- Vadim Sukhomlinov
- Johnathan Van Why

## Updates:

### libtock-rs Rust stable

- Johnathan: using libtock-rs does not require any unstable features anymore.

  Migrated away from external assembly on RISC-V. On ARM, it generated
  substantially different instructions from the external assembly. I'll create a
  PR that does this transition and pass it over to someone more comfortable with
  ARM assembly.

- Hudson: I'll be happy to test it on one of my boards.

### Tockloader / elf2tab issue with libtock-rs

- Hudson: It'll also allow me to look into the Tockloader issue. Issue is with
  the cargo runner that Johnathan built.

- Alex: Yes, this is because the name of the TAB file is incorrect because it
  names the TBF files not by the architecture but for the name of the app.

- Johnathan: not sure whether elf2tab actually knows the CPU architecture.

- Brad: Why does it not know the architecture name?

- Alex: within the TAB file there used to be multiple TBF files with the name of
  the architecture. Right now, the TAB file has only one TBF, which is named by
  the app.

- Amit: Why would it be the case that elf2tab does not know the architecture it
  is translating for?

- Johnathan: Architecture is something like "cortex-m0", is that encoded in the
  ELF file?

- Alex: It just names the TBF after the name of the ELF file.

- Brad: Right, so if the ELF name is changed, it generates a different TAB.

- Amit: Why not have the cargo runner name the ELF file the way we expect it to,
  or rename the TBF file after running it through elf2tab. We can add a `-o`
  argument for the output file name.

- Brad: it needs this for `n` inputs.

- Leon: maybe add a modifier for each of the input files. This would avoid us
  having to rely on filenames.

- Amit: Yes, add essentially two options for every input.

- Leon: Right, I've built such an interface before, I can have a look at it.

- Alex: I can also try to implement an interface that uses an architecture
  specification through a comma-separated string.

## Stable Rust progress and planning

- Brad: relevant issue:

  There are developments in the Rust compiler which help us reach our goal to
  target stable Rust. The `asm!` macro is now stable. There are extensions we
  use which are not yet stabilized. Because they are related, there is a push to
  get that done.

  Other one is `naked_functions` (at least a preliminary version will be
  stabilized).

  Furthermore, `const_fn`, or more specifically `const_fn_trait_bound`, which
  tock-registers uses.

  This is looking really good for our path to work on stable Rust.

- Phil: could we work around the extensions to `asm!`?

- Hudson: not sure about `asm_const!`. libtock-rs works around it by just
  matching on the const and having a duplicate of that value in the assembly.

  Tricky part of this feature is that it is blocked by some other inline-const
  features.

- Leon: `const_fn_trait_bound` being stablized means that all required features
  of `tock-registers` would then be stable. With the prospect of eventually
  splitting this out into a separate repository, this would guarantee
  compatibility with Rust nightlies and stable versions chosen by Tock.

- Hudson: Great thing about `const_fn_trait_bound` is that Rust realized that
  the lack of this feature inspired various workarounds made possible through
  mistakes by the Rust developers. They decided to rather stabilize the feature,
  than to provide backwards compatibility for people relying on the workarounds.

- Brad: Okay, why bring this up now? If this is the turning point, and if we
  want to go through with using stable in the short term, it does mean that
  dealing with "we use this unstable thing because it's going to be some time
  because we have to be on nightly anyways". Few things stand out:

  - intrinsics used by OpenTitan

    Amit: what are these used for?

    Brad: around atomic functions, atomic support is not provided by the ISA but
    by LLVM somehow. Used by the deferred calls in the kernel.

	Amit: presumably the LLVM intrinsics have some implementation for
    implementing the atomics with methods provided by the ISA and in principle
    we ought to be able to implement them ourselves in raw assembly.

    Hudson: not sure that we can necessarily emulate LLVMs behavior using
    assembly. A key part of the core intrinsics are going to be memory barriers
    at the LLVM IR level.

    Amit: we can 100% do that. LLVM has to be able to know about raw memory
    barriers. It is either going to be conservative or it has to look at the
    assembly.

    Hudson: maybe it is possible then. I remember in the last discussion we were
    talking about whether we actually need atomics for the deferred calls.

    Amit: right. Are we only using atomics because atomic semantics mean that
    they are allowed to be in shared globals? LLVM might be potentially just
    compiling that to regular atomic operations.

    Brad: first part, yes. Second part, not sure.

	Amit: if atomics are only used to have a cross-platform way of implementing
    this, we can look at solving it with assembly. Only tricky thing is if we
    were using this in the core kernel, shared across platforms.

	Phil: the comments say that this is for ARM.

	Brad: hard to say. Some of this code has been revived with OpenTitan was
    added.

  - `custom_test_frameworks`, which we've experimented with for testing.

    Leon: can we hide this behind a feature / only when we actually run tests?
    Might be reasonable to have a specific nightly just for the tests.

    Hudson: a little weird to test on a different compiler than we use to build
    the release binaries.

	Leon: Agreed. However, these tests are really concerned with testing
    semantic logic and implementations of capsules / algorithms across the
    kernel. We'd still be able to do other tests and validation on stable.

    Brad: Right. It's not all of our testing, it's just a recent subset of it.

    Hudson: is this used in the LiteX tests?

	Leon: no, LiteX just runs a regular Tock kernel and interacts with it
    through the console.

	Amit: is it the case that a stable compiler of Rust is essentially minted
    from a nightly version? If that is the case, we could use the nightly
    version corresponding to that stable version.

    Johnathan: not exactly going to have the same commit.

	Amit: seems like a reasonable trade off. There are additional tests which we
    can't run on stable. It is not negating all the tests that we can run using
    stable. This would give us more confidence in writing code, as we are not
    taking advantage of behavior differing between stable and nightly
    versions. A particular platform can always decide to use a different
    compiler versions, if they decide the benefits of the tests outweigh the
    differences between compilers.

	Leon: skimmed over Rust's release train model. We might be able to
    automatically derive the nightly version from which a particular stable has
    been branched off. There might be additional commits in there, but should be
    a good start.

  - builing core library ourselves.

    Hudson: this is just a size optimization. Upstream boards could simply not
    do this, although downstream boards can.

	Amit: Presumably, platform caring most about size optimizations is
    OpenTitan? Is it also the case that OpenTitan cares most about using stable
    Rust?

	Hudson: Likely, Ti50 will care more about size optimizations than
    OpenTitan. Believe that it uses are custom managed toolchain, which is a
    stable toolchain with some flags flipped.

	Amit: Downstream users don't necessarily care about upstream being stable,
    but using a version of Rust which is reasonably close to stable to make
    changes to.

	Johnathan: For OpenTitan, there is a good chance that Tock is going to be
    built with bazel. Building the core library is a `cargo` flag, not a `rustc`
    flag. In this case, this isn't even relevant to us.

    Amit: why does compiling `core` require a nightly `cargo`? Is this an
    arbitrary choice? Does `core` use unstable features?

	Johnathan: might be that building the core library is not a popular feature.

    Brad: according to the `cargo` documentation, this is in the very early
    stages of development. Perhaps they don't want to make promises about the
    feature.


## Code review policies for libtock-rs

- Johnathan: several months ago we introduced the temporary change in the Code
  Review document for Tock 2.0. There is no hurry to get libtock-rs to Tock 2.0
  anymore, so we might want to revert that again.

  PRs are divided:
  - upkeep pull requests
  - significant pull requests:

    Defined as _every PR which adds a new API (e.g. new function on a struct)
    that is public_.

	These have a full week of delay, which has been brought to me as a pain
    point.

  I propose to make the distinction at how much churn and turn it would be to
  revert a given change. If it is trival to take a PR out later, it's
  upkeep. However, when it would require significant reengineering it's a
  significant PR.

- Phil: I would trust your judgement on this. If you think that something is
  significant enough that is should wait for a week, that it'll wait for a week.

- Amit: I agree with that.

- Johnathan: You mentioned "you" a lot. A policy cannot privilege one of us over
  anyone else, if only for the fact that I might not be available always.

- Amit: we historically applied the same standards for this in Tock and
  libtock-rs. In Tock, generally a PR is considered "significant" if it touches
  security-critical code or works on major parts of the kernel or interfaces.

  It might be reasonable to choose a very different policy for libtock-rs, as it
  is used in a very different way.

- Johnathan: also, we might want to reduce the list of owners to the Tock Core
  Working Group and Alistair. The others on this list have not contributed in
  over a year. I feel like Hudson, Alistair and I have a good feeling as to when
  we review things.

- Amit: the list of contributors should be revised periodically to actually
  reflect the people showing up, including valid reviews, etc.

  Another option would be to state that if e.g. two people of this group
  approved a PR, then it does not have to wait any longer.

- Hudson: two people seems to be the magic number for libtock-rs because of the
  number of reviews on current PRs.

- Amit: An important difference between Tock and libtock-rs: for a user of
  libtock-rs it is entirely voluntary to update the library, versus for a kernel
  which promises to offer a given API. Applications cannot generally choose the
  kernel they are running on.

  It is much more reasonable to be liberal about breaking applications in
  intermediate commits. Every release should be explicit about what parts of the
  API have changed.

- Johnathan: that is true. A different discussion is how to actually version the
  libtock-rs crates.

  I am going to create a PR to shift the document to classify more PRs as
  "upkeep".
