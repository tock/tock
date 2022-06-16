# Tock Core Notes 2022-04-22

Attendees:
- Hudson Ayers
- Branden Ghena
- Alyssa Haroldsen
- Philip Levis
- Amit Levy
- Leon Schuermann
- Vadim Sukhomlinov
- Johnathan Van Why

## Updates

### Porting implementations to new UART HIL

- Hudson: students at Stanford porting over the UART code to the HIL have
  gotten basic functionality implemented on the SAM4L chips.

  Found one case on a board where there are upcalls being issued
  within a downcall. Hopefully should have some fixes there shortly.

- Alyssa: What exactly is a downcall?

- Hudson: Just a regular call issued from an application, versus a
  call from the kernel in response to an interrupt.

### Additional CPU registers in app switching

- Leon: Been looking at the RISC-V specifications and noticed they
  specify a `uscratch` CSR as part of an optional extension. It turns
  out that this is not supported on any of the boards in-tree, but it
  made me worried about different ISA variants potentially exposing
  further registers to userspace which we should store & restore in
  response to a syscall / switch apps. If we don't watch out there we
  might introduce covert channels.

- Amit: Good question. Wonder if there is something like this for ARM
  chips with TrustZone-M.

- Phil: Student of mine got to know TrustZone really well. Might be
  worth to reach out to him.

### Embedded code size paper

- Hudson: Paper on embedded Rust code size was accepted at LCTES
  (co-located with PLDI). It uses Tock for many of its examples.

- Amit: Saw a draft of that. Can you send that around?

- Hudson: Camera-ready is May 6th, sending it around afterwards.

- Phil: Two other papers at PLDI on Rust. One of them is from Will
  Crichton about using ownership for information flow control.

## Converting ProcessBuffer to use raw pointers

- Hudson: Leon, do you want to give an update on where PR
  [#2977](https://github.com/tock/tock/pull/2977) stands?

- Leon: Was busy writing my thesis, hence did slack off a bit
  w.r.t. this PR. However, it seems we agreed on the general sentiment
  and approach proposed there.

  This PR does introduce a lot of new unsafe code in the core kernel
  which demands careful review. This is hard to get right.

  The capsules should at least all be compiling. These changes
  introduce new calls to `unwrap`, which makes the index-operator
  panics explicit (so doesn't introduce new panics). We want to reduce
  calls to `unwrap` and eventually entirely get rid of panics in
  response to accessing userspace memory. We can use iterators and
  proper error handling for that. People should look at capsules,
  happy to get new commits improving the code quality there.

- Alyssa: I will take a look at this.

- Leon: Thanks!

- Hudson: Is there a list of the capsules that have been done?

- Leon: In a sense all have been done. Did a mechanical replacement of
  square-bracket index operators `[$IDX]` to `.get($IDX).unwrap()`
  calls. To remove these panics, people can look at the diff and see
  where `unwrap` calls have been introduced.

- Amit: Ideal thing would be to avoid doing this and just use entirely
  non-panicing accesses.

- Leon: That's right. Ideal thing would be to handle the various
  errors properly to compose a coherent capsule ABI. Even if we were
  to just return a generic error, still better than panicing.

- Hudson: We wanted to identify a capsule which serves as a good
  reference for what this porting process can look like. Did `console`
  end up being that?

- Leon: Good question, not sure. Essentially any capsule containing a
  panic-free way to access a sequence of elements using iterators as
  well as accesses to known offsets will serve as a good example.

- Alyssa: Yes. there are a lot of `unwraps`, but many of them are in
  functions that return `Result`. We could use
  `.ok_or(ErrorCode::$ERR)?`.

  Do you want to delay this transition to fallible operations? Or
  preserve semantics.

- Leon: For capsules where we do have a specified ABI (perhaps just
  `console`) we want to preserve semantics. For all others, we can
  just do this transition as part of this PR.

- Alyssa: Turning a panic into a return code is always ABI safe.

- Hudson: Well, the console API specifies what `ErrorCode` will be
  returned when. If we start returning arbitrary error codes in
  response to various internal error cases, we would be misleading
  users regarding the precise cause of the error.

- Leon: re whether we should do this now or later. Once this PR is
  merged, people will forget and delay these transformations. As long
  as this PR is open, at the risk of it getting rather large, it might
  be good to do these transformations all in one go.

  The reason why this new ABI does not return a `Result` but rather an
  `Option` is that we want to avoid blindly throwing up
  kernel-internal errors to userspace. We need to be careful of the
  errors we return and deliberately choose them individually, as this
  will compose our offered ABI.

- Alyssa: Once again, the alternative is panic.

- Hudson: Agreed that panics are even less debuggable than improper
  returned errors, but we should adhere to our ABI contracts. If
  something is specified to return `EINVAL` and we return `ESIZE`,
  that's not good.

- Leon: In terms of ABI compliance, panicing is technically better
  than returning an incorrect error code, as the app will never be
  exposed to that. Doesn't help us though and we don't want to panic.

- Alyssa: Many of these panics should be optimized out anyways.

- Hudson: Yes, but it's still a lot easier to grep for calls to
  `unwrap`, `expect` or `panic` than it is to analyze the resulting
  binary. We can't really rely on this being optimized.

  (...after the below discussion around error codes...)

  It sounds like for this PR, we want to get this in, possibly with
  some more transformations of capsules. Secondly, we want to look
  into introducing a new internal error code and possibly some others.

- Leon: Want to emphasize that people should review `processbuffer.rs`
  and the unsafe code there. Going to reach out to Alyssa for ways to
  automatically assure that the pointer transformations there don't
  end up in invalid memory.

- Alyssa: Like a Miri test? Can help with that.

### Side discussion: ErrorCode extension

- Alyssa: I think an internal error code which never needs to be
  documented would be valuable. It would be good to not panic, but
  still preserve ABI compatibility.

  I'm generally a fan of canonical error codes and
  [Abseil](https://abseil.io/docs/cpp/guides/status-codes).

  An internal error code would specifically be an error indicating
  that something is broken within the kernel, i.e. worthy of a bug or
  outage report. If a user runs into an `EINTERNAL`, they should
  report that.

- Phil: historically used `EFAIL` for that.

  Agree that _fail_ encompasses other things as well. Making that
  distinction would be valuable.

- Leon: Generally agree with the sentiment that an internal error can
  be useful, but is separate from the ProcessBuffer PR. This would
  require an update of TRD 104.

- Hudson: Happy to work on a PR that adds this.

- Alyssa: while we're at it, _deadline exceeded_ is also a really
  useful error case.

- Leon: it might be worth keeping in mind that for specific error
  cases, or when the predefined `ErrorCode`s aren't sufficient, a
  capsule can always use `FailureWithU32` to introduce custom error
  cases in its ABI.

- Phil: the error codes are directly inherited from TinyOS. There's
  clearly more things which come up. Agree with Leon that we should
  think about this and introduce additional error codes in a
  systematic approach instead of adding them one-by-one.

- Alyssa: Now that I am looking at this, there's also
  _unauthenticated_ and _unimplemented_. Also four different variants
  of _unavailable_.

- Phil: Don't think _unimplemented_ should be included, as it should
  be caught at compile time.

- Alyssa: Useful during development when using partial implementations
  of components.

- Amit: Useful for development, but presumably not wanted in a shipped
  kernel.

- Alyssa: Many kernels do ship with unimplemented
  functionality. E.g. RSA signing only supported for 2048 and 4096 bit
  keys, not 3072 bit.

- Phil: for system calls, we can use `NOSUPPORT` for that. My argument
  was for kernel space.

- Alyssa: oh, missed that!

  How do we feel about overloading `ALREADY`? Is defined as an
  operation is already ongoing. Been using it for _already exists_.

- Leon: I have been doing the same (e.g. port listener already
  registered).

- Alyssa: Might want to modify the meaning to include this.

  Also, something indicating violation of security invariants.

- Phil: We don't have an `ENOACCESS`, as in TinyOS this was not a
  concept. Everything would be determined at compile time.

- Amit: Might be good to share particular use cases for error
  codes. On the one hand, we have enough space to introduce more error
  codes. On the other hand, there is an overhead in dealing with and
  interpreting error codes.

  For something like `ENOACCESS`, the decision we made at the time was
  to return `ENOSUPPORT` or `ENODEVICE`. It does not leak information
  about the device. Also, then applications don't need to handle these
  cases separately: for an application, it does not make a difference
  to distinguish between it not being allowed to access the device, or
  it not being present.

- Phil: Agreed, we should come up with examples for new error codes. A
  new error code should much better articulate an error case than the
  existing error codes.

- Leon: We should also embrace the fact that the new ABI allows us to
  associate additional discriminators with returned errors. Might be
  worth documenting how this can be done, or provide a canonical
  example. For instance, this is useful when two error conditions were
  to collapse onto the same error code.

- Alyssa: It could be nice to have a better mechanism for custom error
  codes in the future.
