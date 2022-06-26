# Tock Core Notes 2022-06-24

Attendees:
- Brad Campbell
- Branden Ghena
- Alyssa Haroldsen
- Philip Levis
- Amit Levy
- Alexandru Radovici
- Leon Schuermann
- Vadim Sukhomlinov
- Johnathan Van Why

## Updates

### Digest HIL and Software SHA-256 Implementation

- Phil: Digest HIL updates and software SHA-256 implementation are now
  merged. There was some discussion last week about the way
  `LeasableBuffer` is to be used. Opinion was that it's okay the way
  it's implemented now, but might be revisited in the future.

  Fought this for over a week because of Tock OpenTitan instructions
  had all types of errors in them, hope to iron this out.

### UART HIL Chip Implementations PR:

- Phil: Also, there is the draft PR for update to UART HIL:
  https://github.com/tock/tock/pull/3046.

  Students at Stanford started porting this HIL to some. <...>

- Leon: Ported the `sifive` chip, was reasonably straightforward.
  Couldn't really test whether my port works, given there seems to be
  some other crucial infrastructure missing (IIRC DebugWriter). Should
  we work on that first, so that people porting chips can test their
  changes immediately?

- Phil: Absolutely. You might want to explicitly write down what the
  issues are on the PR, so we can get them fixed.

### Bootstrapping USB on the RP2040

- Alex: Trying to figure out how to bootstrap USB on RP2040 chip. Does
  someone have experience with this and could join a call?

  Registers seem to setup correctly, if we boot nothing works. If we
  boot the RP2040 with the official SDK, and then booting Tock without
  power cycling, USB CDC works.

- Phil: I have some experience working with that on OpenTitan. Let's
  chat after the call.

- Amit: Also happy to help.

- Alex: Enumeration works, the communication is jammed when we need to
  properly send data. We send some bytes but just get zeros.

## Potential System Call Encoding Abstractions / Refactoring

- Amit: Vadim wanted to talk about refactoring of the system call
  handling and integration to improve performance.

- Vadim: Currently focusing on code size rather than performance, but
  this could ultimately also aid in performance.

  For instance, to perform cryptographic operations, we need to pass
  different kinds of buffers to the kernel, execute an operation, and
  subsequently unallow these buffers to be able to reuse them.

  Commonly, we need to provide 5 different buffers. This will then
  translate to 5 `allow` operations, one `subscribe`, one `command`,
  one `yield`, and then 5 `allow`s again. This results in almost 500
  bytes of just system call overhead in the binary.

  It would be possible to change the system call ABI to a more
  efficient one, when it would be easier to swap out the current
  system call handling implementation in the kernel. We cannot
  currently provide our own syscall enum, given there are
  interdependencies in the kernel for various system call variants in
  the enum.

  Maybe it's worth looking into redesigning the kernel to just provide
  functions for handling system calls and allow the system call class
  to be sourced from different location. So let the kernel provide all
  infrastructure for handling system calls as a library and allow a
  custom system call class to be sourced from outside the kernel. Our
  own implementations would use the exposed kernel infrastructure, but
  we can change how system call information is transferred.

  This would allow us to easily run experiments to see what type of
  encoding would work best for us.

- Leon: Remember well working on that particular part of the kernel
  during the Tock 2.0 system call interface redesign, which
  significantly changed the way system calls are handled throughout
  the kernel.

  We did try to make it as flexible as possible to introduce new
  system call encodings. By defining the `Syscall` enum in the first
  place, we have a standard interface to inject system calls into the
  kernel regardless of their encoding. However, this infrastructure is
  insufficient if you want to introduce entirely new classes of system
  calls without modifying the core kernel code.

  Question are:
  - can we make it as efficient to have an additional layer of
    indirection to route system calls through the kernel _without_ a
    fixed enum defining the different types of system call variants in
    existence.
  - do we want to allow unbounded flexibility to introduce new system
    call classes externally (somehow) in the first place?

- Vadim: Routing should be up to the system call implementation rather
  than the system call structure, but would be a significant change
  throughout the kernel. I will need to change the ABI, to come up
  with more efficient ones specifically for my target platform. From
  the application POV, this would be hidden by the usual system call
  abstraction. On the kernel side, it should be up to the system call
  implementation what to do with a specific set of registers.

  Instead of having the kernel handle the system calls, have the
  architecture call functions within the kernel in response to system
  calls. This can then also decide on how to route system calls.

- Leon: What you are describing is that you want to define your own
  ABI. This is deliberately supported in the kernel as of today's
  implementation. If you look at `syscall.rs`, it contains some
  reference functions to create the `Syscall`-enum variants based on
  passed register values. However, these functions are not mandatory
  to be used, and they are called from the architecture itself. You
  can define a different architecture, and just decide not to call the
  kernel-provided marshaling and unmarshaling functions for encoding
  and decoding system call and return parameters.

- Vadium: if you look at the `Syscall`-enum, the ABI is hard coded
  within this infrastructure. If you look at `process.rs`, I cannot
  change something there as it is intertwined with the kernel.

- Alyssa: I think we'd like a concrete example of what it might look
  like to implement custom syscalls. I think it is possible in the
  current design.

- Leon: When you do not want to introduce proper new system call
  classed, but just have a more efficient representation of system
  calls in registers, or represent multiple system calls being passed
  as a batch, this is all possible within the current infrastructure.

  Our `arch`s are essentially scheduling a process, and upon returning
  from the process in response to receiving a system call use a
  kernel-provided helper function to decode this system call, and then
  finally call into the kernel with the constructed `Syscall` enum
  variant. By not using this kernel-provided helper function, you can
  define an arbitrary encoding of system calls and even schedule
  multiple kernel-syscalls (e.g. `allow`, `subscribe`, `command`,
  `memop`, etc.) in response to a single hardware system call.

- Alyssa: Would this require us to carry patches against the Tock
  kernel?

- Leon: No, because you'd choose to just not call these functions. All
  these functions do is they get passed a few registers, and they
  return you a variant of the `Syscall` enum.

- Vadim: That is what I want to change.

  For instance, `process.rs` mandates that a command system call gets
  passed parameters one and two. I want to also pass parameters three
  and four.

- Leon: There is a distinction to make between the kernel-concept of
  system calls, of which there is a defined set with a fixed number of
  arguments, and hardware system calls which can encode one or
  multiple of these kernel system calls.

  To change the set of kernel system calls itself is going to be much
  harder, as this is the one interface we carry around the kernel, up
  to the capsules as system call driver implementations.

- Alyssa: What helper function are you referring to, Leon?

- Leon: `syscall.rs`, there are the methods `from_register_arguments`
  and `encode_syscall_return`. These just parse registers to form a
  `Syscall` instance or encode a `SyscallReturn` into a set of
  registers respectively.

- Phil: This all seems like a technically subtle discussion. It might
  be best to have an implementation to talk about.

## `static mut` Globals in CI

- Phil: Currently, Alistair's OpenTitan code has standard Rust
  test. However, Rust's test cannot take any arguments. Thus anything
  you want to run a CI test on needs to be passed through a `static
  mut` global. However, this is something we have transitioned away
  from. Do we want to have boards which are being tested in CI have a
  global `static mut`s?

- Alyssa: Need to understand the use case better. There's other ways
  to pass data into tests, e.g. through context parameters and type
  parameters.

- Phil: How do you allocate the memory for these objects and where are
  they allocated? How would type parameters solve this?

- Alyssa: Easiest way to allocate memory returning a `&'static mut`
  reference without allocating a `static mut` global is to leak a
  `Box`.

- Phil: Use case is for tests to build on each other. For instance, we
  can have one test which tests initialization of the console, the
  second test uses this initialized console to print something.

- Amit: Generally, requiring these boards to have `static mut`s seems
  undesirable. If there is a way to move away from that, we should
  pursue this.

## Linked List Interface Redesign

- Leon: We had an issue at some point where people built linked lists
  in Tock, but accidentally inserted an element twice which resulted
  in a cyclic list.

  Linked lists are built in Tock as a list of references using
  traits. These traits can be used to define an implementation of the
  `next`-method, yielding the next list element, but this
  implementation is user-defined. This makes it impossible to, for
  instance, check whether a list would be cyclic if an element is
  inserted, given that the `next`-method can rely on arbitrary
  internal state (is not pure functional).

  This PR defines a generic list interface as a replacement for the
  current infrastructure, where list nodes are still built using
  traits, as well as a simple list with list nodes built through
  predefined types, giving more strict guarantees.

- Amit: Are there downsides to this new downsides?

- Leon: There might be more cognitive overhead in understanding the
  provided infrastructure, but there are also additional guarantees
  introduced for usages of the simpler list interface.

- Amit: Seems good generally. Probably needs just review, might ask
  Leon to walk me through the code.

- Phil: There is this comment in the PR: "However, it also makes
  implementing basic consistency checks as proposed in #2773
  impossible. For example, because the list is entirely dynamic and
  implementation dependent, and because nodes are not managed by the
  list itself, it's impossible to tell whether a list will, during
  runtime, result in a loop. This makes handling lists difficult,
  especially for simple cases where all of this flexibility is not
  needed."

  Can you explain this?

- Leon: If I recall correctly, the current interface returns list
  nodes using a trait, which then provide access to the underlying
  element. This is talking about this original structure, and is
  further relevant to the generic list implementation retained with
  this proposal. For instance, in the implementation of the `next`
  method, it is feasible to return a reference to `self`. To
  illustrate, this might be useful to represent a sequence of numbers
  through a list interface, without actually allocating individual
  list elements for each number, by returning an internal counter and
  incrementing it in for each call to `next`. This makes it impossible
  to implement basic sanity checks.

- Phil: The worry is that if people are implementing crazy lists, they
  will introduce bugs?

- Leon: Right, that's exactly what this PR should protect against.

- Amit: Okay, will go through this with Leon. One of those PRs where
  the diff is not very helpful, but comparing the two versions
  in-depth is.
