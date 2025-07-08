# Tock Core Notes 2021-06-11

Attending:
- Hudson Ayers
- Arjun Deopujari
- Branden Ghena
- Philip Levis
- Amit Levy
- Pat Pannuto
- Alexandru Radovici
- Leon Schuermann
- Vadim Sukhomlinov
- Johnathan Van Why

## Updates

* Johnathan: There might be more people from Chrome OS teams
  contributing to Tock in the future.

---

* Phil: It is becoming apparent that the kernel's code size is a
  difficult problem, especially for various people trying to use Tock
  in production. Part of the challenge is: if we are hitting the code
  size limits, then it's likely that essentially every new feature
  which adds to the kernel's code size will be rejected. The challenge
  is: this might be a correct decision strategically, but on the other
  hand we should really carefully choose which features make it in and
  which do not.

  If we look at the executable functions themselves, these look very
  efficient. However, dyn-traits and constant data adds significant
  flash usage. There might be opportunities in the core kernel to
  optimize the flash space requirements. Capsules are optional
  components (except some core capsules such as Timer/Alarm
  infrastructure), whereas the kernel is always present.

* Hudson: I'm currently looking into these issues. I will hopefully
  get back with some results in the next few weeks.

  Preliminary results: There is a lot of embedded data, it cannot be
  easily reduced / removed while still retaining useful panic
  strings. It is possible to build a very small kernel without most
  capsules and peripherals, and without panics. As capsules are added,
  that adds some significant code size. I suspect that there is some
  overhead because of the abstractions that the core kernel presents
  to capsules.

* Phil: One example could be our current queuing implementations in
  the kernel. Currently, we have individual queuing implementations
  for the different use cases.

## Alexandru is writing a book

* Amit: Alexandru is writing a book. He would appreciate feedback on
  what to include, and also some info on the time line for Tock 2.0.

* Alexandru: Used Tock in teaching, seems like a good project to learn
  Rust & embedded system security. Started to write a book with a
  colleague, to explain how to get started with Tock OS and writing
  some simple applications. Started out in November with Tock 1.6,
  however it is going to be outdated rather quickly as soon as Tock
  2.0 is released. We were hoping it would converge towards a release
  in July, but this might not happen.

  One of my questions is whether someone can foresee a release /
  stabilization time for Tock 2.0? That might influence the deadline
  of the book.

  Also, some feedback on the book would be great.

* Phil: Is this going to be a physical or online book?

* Alexandru: It is published by Apress. It is going to be available
  online and printed.

  [sent preliminary outline in Chat]

  Motivation is to introduce people to embedded systems, the Tock
  architecture, and why and how Tock is different. Further, we are
  showing people how to write an application. We are using the
  MicroBit and Raspberry Pi Pico boards for their ease of use and
  popularity respectively.

* Amit: Is the goal of this book to be usable as a textbook for a
  class?

* Alexandru: We were debating on whether it would be a technical or
  hands-on book. Given the clients Apress has, it might be a better
  idea to be a hands-on book. In my experience, a lot of theory is
  rather difficult for some students. We are probably going to start
  out with some technical details, and then continuing with how to
  write capsules. Rust is rather difficult, so this might help people
  unfamiliar with the language.

* Phil: You are saying that Rust is rather difficult, but only get to
  it later in the book, by starting out with applications. If the book
  is about Tock and Rust, it might make more sense to introduce this
  earlier. It might make sense to have a comparison between common
  embedded OSes / frameworks, such as FreeRTOS and Arduino, and
  Tock. This can serve as a basis to show differences and advantages,
  also for the rest of the book.

* Alexandru: Thanks for the feedback. We are talking about Tock and
  its differences early on. It a good question whether to introduce
  Rust earlier.

  Another question: is `libtock-rs` usable for applications already?
  I am not sure whether the 2.0 release of that is out before the book
  is going to be released.

* Johnathan: it is difficult to give reliable time estimates. I don't
  see myself hitting a July deadline.

  Furthermore, the 2.0 `libtock-rs` looks very different compared to
  the 1.0 `libtock-rs`. It does not really make sense to write the
  book based on the previous version.

## Tock 2.0 time line

* Amit, Phil, Hudson: The remaining things for 2.0 seem to be:

  - reorganizing kernel exports
  - AppSlice aliasing issues / discussion
  - Upcall swap prevention

    Amit: What exactly is blocking this one?

* Leon: There has been a different proposal on how to solve the Upcall
  swapping issue by Brad. He has been working on getting his solution
  running.

  The proposal Hudson and I presented (#2462) is ready to merge. We
  probably just want to wait until Brad gets back to us, on whether
  his approach works.

* Hudson: Could we get Tock 2.0 out without this merged?

* Phil: I think the conclusion has been yes. Whether or not the kernel
  enforces these swapping semantics, we can always integrate that
  later. The ABI will not change.

  We are going to have to block a release on the AppSlice aliasing
  discussion. We are going to have to figure out whether aliasing
  Allows are permitted.

* Leon: There has been a discussion synchronous discussion with Phil
  and Johnathan, organized on the mailing list. It likely justifies a
  follow-up discussion.

* Phil: It might make sense to raise the question regarding the Allow
  aliasing checks: whether they are always enabled, never present or
  configurable.

* Amit: It seems that these issues are resolvable by the end of July?

* Hudson, Leon: Seems realistic. It's the highest priority issue for
  Tock 2.0 currently.

* Phil: As a group, we are sometimes very divergent in our
  tradeoffs. It might take time to settle on this.

## OpenTitan integration test & scheduler loop changes

* Amit: Embodied in [#2599](https://github.com/tock/tock/pull/2599).

* Phil: The idea of #2599 is to be able to write integration tests at
  a per-board level. Essentially, it allows to put tests in the main
  kernel scheduler loop.

  This PR refactors the main scheduler loop, such that one can run
  some tests in between individual iterations of this loop.

  However, it changes the main scheduling loop, just for performing
  tests. My perspective: running tests is a compelling reason to make
  such modifications.

  With these changes, one can decide on whether to run `kernel_loop`
  and `test_kernel_loop`, whereas `test_kernel_loop` allows to
  intersperse running tests with running the scheduler loop.

* Hudson: the only thing which changes w.r.t. the kernel loop as it is
  written today is that an argument is introduced to avoid the kernel
  loop to sleep. That is required, as in order to have these tests
  working we do not want the kernel to sleep if there are no more
  tasks. Instead, we want it to return and run the next test.

* Pat: why do we need the [hard-coded number of kernel loop
  iterations](https://github.com/tock/tock/pull/2599/commits/71916045f9b0d6eed74785f7c7ab813e719a691e#diff-250e0d9aceb5946a44203066a07b9d89b08411c8741afb443d6d8bfa5669178eR582-R595)?
  For instance, the kernel loop is run 200 times, and then 100_000
  times. This seems odd to me. Why is the decision made in the kernel
  at all? Supposedly the test runner should decide that.

* Phil: Yes, this makes sense.

* Amit: What is the controversy about this?

* Phil: Not necessarily any controversy about this. It does involve
  changing the core kernel loop. The core working group should look
  over it.

* Amit: I requested a review from the core working group. This seems
  like a good idea for such changes. It is more reliable to reach
  people, as GitHub notifications might be filtered.

## AppSlice aliasing discussions

* There have been a lot of discussions regarding AppSlices, in
  particular how a kernel representation should look like. I would
  like to focus on the question of whether the current restrictions of
  TRD104 should remain or be lifted:

  ```
  The Tock kernel MUST check that the passed buffer is contained
  within the calling process's writeable address space. Every byte of
  the passed buffer must be readable and writeable by the
  process. Zero-length buffers may therefore have arbitrary
  addresses. If the passed buffer is not complete within the calling
  process's writeable address space, the kernel MUST return a failure
  result with an error code of INVALID.

  Because a process relinquishes access to a buffer when it makes a
  Read-Write Allow call with it, the buffer passed on the subsequent
  Read-Write Allow call cannot overlap with the first passed
  buffer. This is because the application cannot write that memory. If
  an application needs to extend a buffer, it must first call
  Read-Write Allow to reclaim the buffer, then call Read-Write Allow
  again to re-allow it with a different size. If userspace passes an
  overlapping buffer, the kernel MUST return a failure result with an
  error code of INVALID.
  ```

  Essentially, the text currently says: if userspace allows a buffer,
  and subsequently allows a buffer which overlaps with the currently
  allowed buffer, the kernel should return an error. The issue here is
  that the kernel would have to do the check of two allowed buffers
  overlapping.

  Do we need this? These checks will add about ~180-200 cycles to any
  allow system call, and then about 30 cycles for each additional
  allowed buffer per process.

  It currently adds about 0.5kB of code, when fully integrated will
  probably amount to 750 bytes.

  This ties back to the question of whether we allow the kernel and
  userspace to concurrently access shared buffers, does the kernel
  need to protect itself of userspace changing this memory, and does
  the kernel need to protect itself from having multiple references to
  the same memory region.

  Other part of the discussion is how we can build abstractions in the
  kernel which would not violate non-aliasing assumptions in the
  kernel.

  Even assuming we have a sound Rust abstraction for this, should we
  return an error to userspace?

* Leon: One additional point might be: even if we have sound
  abstractions for overlapping buffers in the kernel, this might still
  increase complexity. For example, when performing a SPI read-write
  operation, with a transmit and receive buffer, if these overlap, the
  kernel could potentially overwrite the transmit buffer by receiving
  data. This would send garbage over the SPI bus, which userspace did
  not intend. Also we then must make sure that the kernel does not
  assume buffers not changing even within one capsule invocation, for
  example to cache certain computations.

* Phil: I would summarize this as, it seems rather clear that we do
  not want to permit userspace to Allow overlapping memory regions in
  the kernel. This would require significantly more programming
  defensiveness. It appears rather obvious for read-only and
  read-write Allow to not support overlapping buffers. The question
  is: would like the kernel to return an error?

* Vadim: I think this might be best done whenever one tries to use the
  AppSlice in question. When `AppSlice::take` is called, it would be
  checked at that instance. This would mean the scope of checks would
  be limited to the currently in use buffers. Furthermore, one might
  choose different behavior when dealing with overlapping buffers. For
  instance, if there is any overlap, we might either fail or handle
  this gracefully, for example by only accessing the non-overlapping
  range.

  My use case is encrypting a large block of data. If one cannot
  overlap, one has to provide a separate buffer for input and output.

* Phil: Why do you have to pass two buffers? If one does a read-write
  allow, one can just put encrypted values back into the same buffer.

* Vadim: Correct, but this would mean one has to implement two APIs:
  one to encrypt in place, and one to encrypt into a separate buffer.

* Leon: I suppose runtime checks (`AppSlice::take`) of overlaps would
  give us the same problem as we have with multiple Grant enters:
  there we do employ runtime checks to ensure we don't enter a Grant
  region twice. This is rather confusing and difficult to handle in
  practice. A user of an `AppSlice` does not necessarily know where in
  the code an overlapping `AppSlice` is already taken, and so it would
  require runtime checks and handling failure cases each time an
  `AppSlice` is used essentially anywhere in the kernel.

* Phil: From a code size standpoint this would also be tricky.

  I acknowledge that the restrictions on overlapping allowed buffers
  are tricky in some cases, in particular high performance edge
  cases. After circling around this for many times, we have come to
  the conclusion that we are going to need to have a separate Allow
  for those kinds of special cases.

* Vadim: It is not blocking for me, I can work around that. If there
  are checks, I was just thinking those could be more efficient at
  _take_-time, rather than on the Allow syscall.

* Amit: I do not think that I am convinced this is something the core
  kernel should worry about, except for type-safety and soundness
  issues. I thought that this were an issue that we know how to deal
  with.

  I understand that there are cases where a capsule might prefer --
  for it's own logic -- for an AppSlice to not be shared
  elsewhere. I'm not convinced that there are cases where this would
  break a capsule for other applications. If that is the case, it
  seems that this could be a restriction that a system call capsule
  imposes by convention or requirement on applications, and
  applications can cause capsules to misbehave for servicing their
  calls. Userspace libraries could assist in helping applications to
  prevent these issues.

  Otherwise, this sounds to me like adding a lot of complexity to make
  some cases for applications to not do the wrong thing for themselves.

  If this is not really a cross-application or kernel level safety
  issue, this is not required.

* Phil: I do not think these checks are a lot of complexity, but if
  it's unnecessary we shouldn't do these. It seems clear to say that
  in general, one should not have overlapping Allow
  buffers. Especially, system call capsules should not require
  applications to use overlapping Allow buffers, because of the
  challenges that would ensue. But if userspace does this, we are not
  going to throw an error.

* Leon: The only safety issue this could cause in practice is the case
  of poorly written capsules. These could read a value from `AppSlice`
  A, rely on it not changing, and write to `AppSlice` B, in turn
  changing A's contents themselves. That this _can_ happen is clearly
  communicated by using a slice of `Cell<T>`s. Hence capsules will
  have to be slightly more careful in how they deal with buffers to
  avoid these semantic issues.

* Amit: How do these issues manifest themselves? What is the way this
  would be bad?

* Leon: For instance, a capsule is writing some serialized data --
  which it verified to be correct -- to a device. In parallel, it
  writes the device's response to a second buffer. This could
  implicitly overwrite the source data, invalidating the invariant
  that the source data is valid and will remain valid for this
  invocation of the capsule.

* Amit: I see. This seems like a slightly odd case, but I see how this
  could be problematic. An example could be a process sending a
  packet, having the header and payload in two buffers. The header
  would be validated first, then the capsule writes to the payload and
  sends it. Because the buffers overlap, the capsule would implicitly
  also modify the header. This could be used to, for example, send
  packets from a port one is not supposed to have access to.

* Leon: Correct. There is not necessarily any unsoundness caused in
  the kernel, but it might not be the behavior userspace would expect
  or could lead to these security implications.

* Vadim: But we can check for these situations in capsule code. It is
  very similar to my case of encrypting/decrypting, whereas there it
  is actually desirable.

* Phil: The cases where one wants buffer overlap are very
  narrow. Checking in capsules, both from a code size and bug
  standpoint, it might be worse to just preventing overlaps.

  One other thought has been to have these checks during development,
  and disable them in production. This seemed appealing at the time,
  but is actually problematic. Whether the kernel throws an error
  would change depending on the kernel build.

  I think that we do not want these checks, as we figured how to keep
  the kernel sound. Also, there is the scaling issue, where a larger
  number of Allowed buffers would make the checks more
  expensive. Should we remove the TRD104 requirement which states that
  the kernel should return an error?

* Amit, Hudson: Yes.

* Amit: Biased, as initially proposed the slice of cells approach.

* Phil: These are not mutually exclusive. We can also do these checks
  in the user libraries.

* Amit: Correct. Johnathan, if we are using `libtock-rs`, it seems
  that this is a great thing to enforce there if one cares about it.

* Johnathan: When I thought of the `libtock-rs` API, I came up with
  one that naturally ruled out overlapping buffers. The design could
  support overlapping buffers if we wanted to. This is except for the
  case where a buffer is truly in flash, which can always overlap.

* Amit: Right, this is the user library helping to enforce a property
  which is useful for the application itself.

* Phil: My conclusion is that the arguments against doing the checks
  are stronger than the ones in favor. Especially the fact that
  userspace can perform these checks to be on the safe side helps.
