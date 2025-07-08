# Tock Core Notes 2023-04-28

## Attending

- Hudson Ayers
- Branden Ghena
- Alyssa Haroldsen
- Pat Pannuto
- Tyler Potyondy
- Alexandru Radovici
- Leon Schuermann
- Johnathan Van Why

## Updates

### Async/Await Update for `libtock-rs` (and potentially the kernel)

- Alyssa: might have a method to have smaller async/await support in
  Tock. Can use one global vtable instead of individual vtables per
  future. Primarily for `libtock-rs` for now.

- Pat: no dynamic memory? Problem with many of these is that,
  per call site memory must be allocated for the future's state.

- Alyssa: Planning on using a static buffer.

- Johnathan: not a hard problem to solve. Tricky to avoid reducing the
  bloat around "what woke you up" and "what signal to handle".

- Alyssa: [RTIC](https://rtic.rs/1/book/en/preface.html) looks like
  they have been adding async support, so looked at some of the
  innovations.

### Availability for TockWorld 6 Meeting

- Hudson: Poll has been sent around via email, please respond!

### nRF52 - Support New Access Port Protection Mechanism (PR #3422)

- Hudson: Brad introduced support for some new JTAG access port
  restrictions introduced on recent nRF52 chips. Seems ready to go,
  people should take a look and test.

  This is nice, as it makes recent nRF boards usable out of the box
  again.

### Updates to the `ExternalDependencies` Documentation (PR #3312)

- Hudson: Brad updated the document according to our discussion last
  week. Would appreciate if people take a look.


## MaybeUninit / Write-Only Allow Buffers

- Alyssa: Discussion from 2 weeks ago. I'm replacing some of our
  storage read calls with calls that are able to take `MaybeUninit`
  data as their input. This is so long as the syscall does not read
  from it.

  Considered the impact of reading from it, should not be worse than
  just reading garbage data.

  Wanted to know whether there was any interest in a write-only
  process slice / write-only allow.

- Leon: Where we left off last week: we viewed this proposal in the
  context of chips having ECC memory, which can cause faults if read
  before initialized (written to).

  Seems that now we're mostly focused on reading uninitialized memory
  in the sense that it may hold arbitrary data, but not that it could
  fault the chip?

  Seems very important to disentangle these two issues:
  - seems hard to use such infrastructure to handle "dangerous"
    uninitialized memory (e.g. ECC memory) which can cause faults
  - much more reasonable when we're talking about memory which just
    hasn't been initialized to known-good contents, but is nonetheless
    readable.

- Alyssa: So should we consider uninitialized memory as part of our
  threat model?

- Leon: What precisely does "uninitialized" mean here?

- Alyssa: Precisely the definition of `MaybeUninit`.

- Leon: Tricky to use this Rust-focused definition when talking about
  memory shared across the system call boundary. We essentially only
  take in an arbitrary slice of bytes; can't rely on this slice to
  contain well-formed data as required by the system call handler. In
  practice, we thus validate the contents of that slice in a system
  call driver.

- If passing in a buffer to e.g., read data from flash into that
  buffer, there is not any validation apart from the buffer's length.
  Just want to make sure that the contents aren't read when they are
  not intended to.

- Leon: Seems reasonable. Still unsure whether `MaybeUninit` is the
  right tool to use here. We're still operating on a slice of
  arbitrary but fixed bytes.

- Alyssa: This is what freezing across the system call boundary means.

  Maybe an out-reference is a better choice here.

  For systems where reading uninitialized memory can cause a fault,

  Values are frozen across the system call boundary from a Rust safety
  perspective. Let's say we have a system which faults when a memory
  location is read before it is written. When userspace shares such
  data with the kernel, it could still fault. Don't think it's a
  problem for Rust safety, but rather system resilience.

  Out-references that wrap `MaybeUninit` perfectly wrap write-only
  memory.

- Leon: I'm focusing on the issue that there does not seem to be a
  reasonable approach for us to track whether a given memory location
  has been initialized by userspace. As a result, if we were to give
  userspace some properly uninitialized memory on these systems, we
  can't reliably determine at runtime whether the kernel may accept a
  readable or write-only slice of memory.

- Alyssa: Today, an allow requires passing in a `&mut [u8]` slice,
  which captures that it is initialized. Of course, this doesn't hold
  in C. So right now, such shared memory is always readable and
  writable.

- Hudson: But if userspace lies about that and claims that some memory
  is initialized, when it actually isn't?

- Alyssa: Same risk as we have today.

- Hudson: Leon's trying to get at the fact that running Tock on a
  system where the memory model is designed in this way is not a valid
  use of Tock. Tock can't trust the applications.

- Alyssa: Depends on definition of trust. If the MMU faults that
  specific app, then it's fine.

- Leon: Would fault the kernel though, given it is reading
  uninitialized data passed in by the application.

- Hudson: Sounds like we're getting way outside the scope of current
  Tock. Currently have no means of unwinding such faults. Seems weird
  to design a system to such hypothetical changes.

- Alyssa: More thinking of preventing bugs in the kernel.

- Leon: A write-only system call for that purpose seems fine. Just not
  for handling such faulting memory. We would work around the latter
  by just initializing all memory on app startup.

  Would still need a motivating example.

- Hudson: Seems like this is mostly about making it easier to write
  correct code. There may be some vague security arguments, but that
  might be pretty contrived.

- Alyssa: Would at least like to see some documentation regarding the
  safety & security considerations of sharing uninitialized memory
  as part of a readable allow operation.

- Johnathan: Could extend `libtock-rs` to pass a `MaybeUninit` into an
  allow, but can't take it back, would effectively operate as a
  `Result`.

- Alyssa: If we we to introduce a new `WriteOnlyAllow`, should that be
  a new system call variant or just a flag?

- Leon: Don't know. There's a code-size concern with adding a new
  system call variant, similar to the userspace-readable allow system
  call.
