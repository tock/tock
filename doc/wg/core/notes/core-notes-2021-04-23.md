# Tock Core Notes 2021-04-23

Attending:
- Hudson Ayers
- Brad Campbell
- Branden Ghena
- Philip Levis
- Gabriel Marcano
- Pat Pannuto
- Leon Schuermann
- Vadim Sukhomlinov
- Johnathan Van Why

## Updates

### Brad - Tockloader fixes

* Brad: Did a couple of small fixes on Tockloader.

  There were some exceptions thrown when Tockloader exited. That
  should be fixed on latest Git HEAD. It might have only happened on
  Mac.

  Going to release soon, but more testing required to ensure that no
  new bugs creeped in.

### Phil - Looking into kernel code size

* Phil: A couple of students are looking into code size of the kernel
  binaries. They have been experimenting with inlining, such that we
  can decompose where the cost of functions are. Next up: if we
  removed all debugging statements, how large is the effect?

  There should be some results in the next couple of weeks.

* Leon: Are the results published once at the end or continuously? We
  could make great use of the feedback, to tune the binary size prior
  to releasing Tock 2.0.

* Phil: Depends on where the results go. Students should also focus on
  their report.

* Leon: Sure. The [mailing list thread about system call
  overhead](https://groups.google.com/g/tock-dev/c/FPTmNe4BAq0) might
  be related.

  Thanks for investigating!

### Phil - Updates to the HIL TRD (ErrorCodes in callbacks)

* Phil: Opened a [PR](https://github.com/tock/tock/pull/2550) for some
  updates to the HIL TRD.

  There was a disconnect where the document said that callbacks would
  have to return an `ErrorCode`, but also that HILs could define their
  own error types if `ErrorCode` would not fit (example: I2C).

  Changed the text to say _SHOULD_ return `ErrorCode` and the types
  _SHOULD_ be the same whether returned as part of a callback or
  synchronously.

## Agenda Item #1 - libtock-rs macOS CI

* Johnathan: For `libtock-rs`, Hudson, Alistair and I have been
  reviewing all of the PRs. We have a CI which tests building RISC-V
  on macOS. It works sometimes (approx. 1/3), but often times out
  (taking ~6 hours). Unfortunately, none of us seem to use a Mac.

  There are some changes I'd like to make, for instance have it
  support ARM as well. We can probably cache the results of the
  homebrew command which installs the RISC-V toolchain to make it more
  reliable. However, none of us uses a Mac and can efficiently develop
  this.

  The options are:
  - `libtock-rs` doesn't officially support macOS builds, at least
    until someone with a Mac can repair the CI.
  - someone on the core team using macOS can become the maintainer of
    the `libtock-rs` macOS CI.

* Leon: I've not seen macOS builds time out, but rather see them not
  start for several hours recently. Could this be related?

* Johnathan: There is a per organization limit for macOS builds on
  GitHub actions. I suspect `libtock-rs` might be saturating that
  limit with its 6 hour timeouts and hence Tock might not be able to
  run its CI.

  Also, the limit for macOS is lower than for other platforms.

* Leon: That's great to know!

* Pat: Potential compromise, since we only care about the fact that
  builds work eventually: can we reconfigure the macOS CI to only run
  on the Bors staging branch? Such that it only runs as the final
  stage when merging. Also, the probability for macOS breaking when
  Linux works is low.

* Johnathan: We don't currently block on macOS CI.

* Hudson: Might have the effect that merging any PR takes several
  hours.

  I might be able to get access to a Mac.

* Brad: Can we just test ARM on macOS? That will get rid of the
  toolchain issues of RISC-V, but still allow us that nothing
  unexpected happens on macOS.

* Johnathan: I will try that. Have not tested to see whether it's
  faster.

* Brad: Don't know of any issues with getting an ARM toolchain running
  on macOS.

* Johnathan: Will consider the following options in this order:

  1. Try building ARM on macOS.
  2. Having Hudson maintain the macOS CI for `libtock-rs`.
  3. Not testing on macOS.

* Brad: Option 1 and 2 are likely orthogonal to each other.

## Agenda Item #2 - Tock 2.0 status update

Tracking issue for Tock 2.0:
[#2429](https://github.com/tock/tock/issues/2429)

* Phil: Only a few things left to do: update changelog and prevent
  callback swapping.

  One thing missing on this list: the system call TRD introduces
  `BADRVAL` in userspace, representing the case where the kernel
  (capsule) returned a system call return variant which did not match
  the expected one. Much of `libtock-c` is not doing this. Do we want
  to block the release on this?

* Brad: As part of the update of `ReturnCode` and switch to
  `StatusCode`, `libtock-c` now does this.

* Phil: Fantastic.

* Brad: One additional thing: reorganizing kernel crate exports. It's
  an old issue. Usually I don't like these changes, but now seems like
  a good time to do it.

  Until now, the kernel crate has experienced organic growth and hence
  leads to some oddities using it. For instance, sometimes one needs
  to import a long path, whereas other times one can import from the
  crate root. Sometimes imports are descriptive, and other times
  aren't really.

  Do people agree? Also, the PRs are quite invasive.

  Relevant PR: [#2545](https://github.com/tock/tock/pull/2545)

* Phil: a good way to approach it might be to just list what the
  exports should be, and then change it accordingly.

  It can ensure that the end-result will be consistent and it's clear
  exactly where we're going.

* Brad: I like that.

* Leon: I think it's better than a lot of iterative changes, where
  there is also a lot of inline discussion. Knowing the final exports
  could help.

* Phil: Just make this a draft PR with no code changes? Once we've
  settled on it we can make the changes.

* Brad: Sometimes I find it good to prototype at least some of the
  changes to get an idea of how it's going to look like. I agree that
  we should look at it holistically.

* Phil: Let's add as a bullet on the Tock 2.0 tracking issue. Any
  other issues?

* Leon: AppSlice aliasing unsoundness - we can't have two mutably
  borrowed slices pointing to the same memory region in the kernel, as
  that's undefined behavior. We talked about it a little last
  week. Amit, who's unfortunately not here today, gave some pointers
  to volatile slices. I looked into this and could not find a good,
  non-intrusive way to do it. If anyone has any ideas here, that would
  be great! All other options don't look very promising as well.

* Phil: Amit should probably be present for this discussion. We might
  want to move it to next week's call and give it a high priority.

* Leon: Absolutely. If someone had a lot of time on their hands, I'd
  like to invite them and look into this issue. It gets complex
  quickly, so is probably best approached in a collective effort.

* Hudson: Leon, I'll reach out to you and talk about this. I'll look
  into it prior to next week's meeting.

## Agenda Item #3 - Prevent Upcall swapping

PR [#2462](https://github.com/tock/tock/pull/2462)

* Hudson: It seems like all that is left -- in order to introduce the
  mechanism preventing swapping of Upcalls by capsules -- is to change
  all non-virtualized capsules to use Grants.

  Leon, Brad and I have been working through those. If anyone else is
  interested into migrating a driver, that'd be great! There is a list
  in the [PR description](https://github.com/tock/tock/pull/2462). Add
  your name to the list if you take one on.

* Phil: Please pick one. It's a change, and migrating a capsule can
  help understand how non-virtualized capsules work from now on.

* Leon: Changes aren't as difficult as they might look at a first
  glance. There's a good template in
  [#2521](https://github.com/tock/tock/pull/2521) one can easily
  translate to the other capsules.

* Hudson: One question on my latest PR. With the first capsules we'd
  only enforce the single-process limit/reservation on a `command`
  system call. If a process submitted a `command` system call to a
  capsule that's already reserved by another process it would return
  an error, whereas `allow` and `subscribe` would succeed.

  In my most recent PR I applied it to all of them. If an app does use
  the driver via an `allow` or `subscribe` system call _before_ trying
  `command`, it will have its Grant region allocated. Downside: we add
  some text to the files and small amount of code size.

* Brad: Question is: can an app have its capsule Grant region
  allocated, but then not be able to use this capsule. In my opinion,
  non-virtualized drivers are not particularly useful anyways, except
  for testing. I'm rather concerned with safety and soundness of the
  kernel, this does not seem to be an issue here. The capsules don't
  provide the interface we want them to anyways. Hence I don't think
  we should be concerned about this.

* Leon: One thing to think about is that for other processes, we'd
  also block unallow and unsubscribe. If a process managed to share
  some resources and another process could acquire the lock (when
  thinking about also being able to release a capsule reservation),
  the process could never get it's resources back.

  Semantically speaking, a capsule holding process state but refusing
  an operation seems fine.

- Hudson: Makes sense. I'll change my PR accordingly.
