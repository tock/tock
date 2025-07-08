# Tock Core Team Call Notes 2024-08-23

Attending:
- Brad Campbell
- Branden Ghena
- Alyssa Haroldsen
- Amit Levy
- Pat Pannuto
- Tyler Potyondy
- Benjamin Prevor
- Leon Schuermann
- Johnathan Van Why

# Working Group Updates

## Network WG

- Branden: Irregular meetings over the summer, discussions mostly
  about moving forward on the PacketBuffer front.

  Plan: PR for PacketBuffer, and draft PR for console multiplexing.

## Documentation WG

- Branden: Never had a meeting.

- Brad: Work as been going into the book, and the mini-tutorials (to
  update them to the new libtock-c changes).

  Waiting PR to update docs of SPI HIL motivated by
  https://github.com/tock/tock/pull/4136

# General Updates

- Leon: Work on testing & CI front underway.

- Branden: Updated tockbot to assign people to all non-draft PRs:
  https://github.com/tock/tock/pull/4147

- Pat: Do we want to assign people who commented already?

- Branden: Would punish people who comment quickly.

- Amit: Working on a Raspberry Pi port.

# Revisiting TockWorld Breakout Discussions

## Non-XIP Platforms

- Amit: a lot of interesting things to do with these platforms, one of
  them is now widely deployed. We should think about doing this
  upstream.

  We should start with a QEMU-based platform for it.

  Bobby is working through pulling out non-propriety parts of x86 port
  for upstreaming.

## Tock Registers

- Johnathan: on another project right now, will be a couple months
  until I get back to it. Some minor todos.

  A couple bigger ideas: arrays of register structs, where each
  element is not a u32 register, but each is a whole bank of
  registers. Have not been able to think through whether its
  reasonable.

  Other idea: separating out register offset calculation into
  something that's not specific to MMIO registers. Seems less
  necessary for Tock, more helpful for the embedded community
  generally.

  A lot of things not documented about the design, so will be a
  ramp-up for somebody else to engage with the PR.

- Amit: Related -- one of the issues that Tock registers was meant to
  address are issues with VolatileCell. Leon, can you give a summary
  of the discussions we had around that?

- Leon: Went down the rabbit hole of safety & soudness interactions of
  using UnsafeCell and VolatileCell around the Tock ecosystem. Most
  importantly for tock-registers: VolatileCell is still unsound for
  registers, because Rust does not promise that it does not insert
  spurious dereferences (read operations) for anything that has a
  reference to it.

  It was unclear to me that, given a VolatileCell is dereferencable
  _and_ its underlying memory being mutated by something that isn't
  synchronized to the Rust thread, whether this is sound even for
  something like DMA. UnsafeCell is more permissive than initially
  expected, and it's fine to use on memory modified by other threads,
  so long as *accesses* are synchronized (like in Mutex). Thus,
  UnsafeCell + reads/writes for DMA memory seems sound.

  Current PR makes interactions between happens-before relation and
  volatile *less* clear.

  tl;dr: trying to nail down all these interactions and make sure that
  our basic operations to interact with HW are sound. Might incur some
  code changes such as to avoid holding references to DMA memory or
  inserting memory fences.

- Alyssa: While not perfect, inline assembly can help with some of
  these problems (as it inserts a memory barrier by default).

- Leon: That is true. In practice, inline ASM solves many issues, but
  we're also not running into any miscompilations right now. Inline
  ASM is even worse when it comes to explicit documentation of
  assumptions and interactions with other code. Kind of a magic wand
  we can use, but would rather actually have a correct solution that
  addresses our exact use-case and where we can reason about
  soundness.

- Alyssa: Received inline ASM as an answer to this before. Because
  LLVM does not have very well defined volatile semantics, and those
  have also changed over the years.

- Leon: Engaging more in upstream discussions right now.

- Alyssa: Inline assembly is a workable solution, even if a lot of
  Rust language assumptions change.

- Leon: Good fallback. Would be hesitant to sprinkle it through our
  codebase out of a fear of miscompilations.

- Alyssa: Not much traction for volatile memcpy.

- Leon: We don't really need that, we just need to be able to have a
  memory barrier between volatile and regular accesses. Rust
  documentation seemingly changing to more explicitly rule out fences
  as a solution for this. Will have to try and fix this documentation,
  and create assurances for us downstream.

- Amit: To clarify -- none of this would change how things are
  compiled, just change the documentation's semantic guarantees?

  Language around fences doesn't make anything worse in practice for
  now, but in the text implicitly volatiles.

- Leon: We'd like to re-introduce some language. Will be up to the
  experts to judge whether what we want is actually guaranteed by
  LLVM.

## Automatic Driver Generation

- Amit: automatically generate bindings for userspace system call
  libraries from kernel code.

  Brad drafted a proposal, I created a different one based on that
  design.

  Seems dormant -- not particularly high priority for either of us. To
  what extent is this something that we think is worth pushing on?

- Brad: adding to motivation -- matter of convenience, and also
  detecting whether the kernel is no longer in sync with userspace
  (CI?).

- Amit: backwards compatibility. For instance in
  https://github.com/tock/tock/pull/4144, the concern was raised that
  this driver doesn't bubble up errors. But those changes should not
  break userspace.

  This can be used to sanity-check backwards compability.

- Leon: Seems similar to what Rust and Cargo do upstream. For each
  test case include stdout and stderr files. If any single character
  changes, then the test will fail.

- Amit: At a high-level, nice to have. For prioritizing this, would
  this solve a more pressing problem? For me, the lack of userspace
  support for a lot of drivers in libtock-rs is one barrier for me
  using it more. Not so hard to write that I couldn't do it myself,
  but still.

- Brad: Part that I'd be interested in is from a code-review point of
  view -- structured way to enforce or check Tock conventions.

  For example, one paradigm for system call drivers is that they are
  owned by a single application. It's subtle how to do so
  correctly. Also, every command has to return exactly one variant of
  `SyscallReturn`. We should really have `ReturnCode` as the first
  argument to upcalls...

## Panic-Free Kernel

- Tyler: shouldn't be too much of a lift to get done. Action items:
  - RFC of some sort
  - remove panics

- Alyssa: basic idea -- a lot of places where this is not a reasonable
  way to continue. We want to replace the panic implementation with a
  fault invocation.

- Leon: can we publish a minimal example somewhere?

- Alyssa: yes, could be a good starting PR?

- Amit: can't you set panics to abort in Rust?

- Alyssa: that's orthogonal -- I want to prevent unexpected aborts
  from being inserted into the program. By ensuring that there are no
  intentional panics in the entire kernel, we can prevent accidential
  panics from sneaking in.

# Multi-Core Support

- Leon: labmate has been working on multi-core support for a
  tangential research project. Multiple separate kernel instances on
  different cores with a communication channel. Writing up an RFC of
  the various challenges, design constraints, high-level design, etc.

  Gongqi works on translating the communication channel from a single
  Mutex and locks towards queues, such that instances don't block each
  other.

  Implementation needs cleanup, RFC will come first, use the PR as a
  demonstration of what we describe in the PR.

# CHERI

- Amit: missing upstream Rust PRs, main action item: put together an
  RFC to Rust. Haven't published it yet, but RFC draft is pretty
  close.

  WIP Tock kernel port available on https://github.com/tock/tock-cheri

  In good shape overall.
