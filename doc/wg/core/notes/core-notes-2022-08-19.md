# Tock Core Notes of 2022-08-19

Attendees:
- Hudson Ayers
- Brad Campbell
- Branden Ghena
- Alyssa Haroldsen
- Philip Levis
- Amit Levy
- Pat Pannuto
- Alexandru Radovici
- Jett Rink
- Leon Schuermann
- Johnathan Van Why

## Dialpad Meeting Automatic Recording

* Hudson: Any reason to not turn on automatic recording for Tock
  meetings? Room used for other things?

* Amit: Very rarely, can still turn it off.

* Johnathan: OpenTitan WG uses it, but recording should not be an
  issue. Will ask next OT meeting.

## Updates

### Console Debug Print Ordering

* Phil: In response to ordering of console prints. Looked at how
  queuing works in the kernel. The only non excessively complicated
  method to provide ordering of console prints is to use a single
  output buffer. The best solution to this problem seems to be a
  second system call, which would essentially invoke debug and copy
  writes from userspace into the global kernel debug buffer.

* Alyssa: I think this should work for our use cases.

* Phil: These operations would be non-blocking; no flush can be
  implemented because we'd only be able to run top-half interrupt
  handlers. If it cannot run any bottom-halves, it cannot invoke the
  UART peripheral's interrupt handler to continue printing.

  Debug buffers should be sufficiently large to work around that.

* Alyssa: Would like a userspace `flush` operation, which blocks the
  issuing app.

* Phil: That should work.

### `tock-registers` Stable Release

* Leon: User commented on issue #2982 that `tock-registers` still does
  not have a release usable on stable Rust. It works on stable Rust
  internally for a while now. We might want to do another `v0.Y.0`
  minor release with what's in the repo currently.

* Hudson: If external people are blocked on a release, just do a minor
  release.

* Pat: Johnathan, what is the latest update on the register interface
  PR? Is that expected to land in the short term or requires some more
  time?

* Johnathan: It is going to be a couple of weeks still. Also, it would
  manifest in major non-backwards compatible changes to the API
  surface. Would not be a great release for users who just want to use
  it on stable.

  Also, we should wait a while with these changes in to polish it.

* Leon: Also, these changes might be rather controversial. Revamping
  the entire register structs infrastructure should require some
  elaborate discussions.

* Pat: Preparing the release now.

## State of Tock 2.1

* Hudson: Current state seems that some boards have been tested, but
  there is more testing to do.

  Discovered failures in the log tests during `imix` kernel tests;
  interestingly not the linear log tests. One of the bytes written is
  not read back correctly on `imix`. Looking into changes since the
  last release.

  Some of the 6LoWPAN tests fail. Specifically ones which combine
  in-kernel capsules using 6LoWPAN along with multiple
  applications. Failures are strange, just at the application level
  and only occur with multiple applications. Order of app operations
  seem to matter, and applications have time-quantum expirations.

  Alex has a potentially unfair number of boards assigned to him. Can
  take over the Nano 33.

* Phil: There is still one release blocker (PR #3139, Redboard Artemis
  Hardfault Exception).

* Branden: Tock kernel issue or just for specific board?

* Phil: Just for this board. Brief summary is that it gives a
  hardfault exception, because the stack is corrupted and the saved
  link register ends up as `0`. It then tries to jump to address `0`,
  which does not contain a Thumb instruction and hence it gives a
  hardfault.

  Inlining a function "fixes" this issue. The root of the problem
  seems to be that, to perform some FPU configuration checking, you
  have to trigger an `svc` handler and the code was not triggering
  this handler correctly.

  Solveable, but requires a very deep understanding of ARM exception
  handling.

* Hudson: Default bootloader of this board enables the FPU, so it is
  entirely board-specific.

  We have not gotten to the bottom of this in a week. Do we merge
  Alistair's fix, which is not actually solving the underlying
  problem?

* Phil: Answer has to be no, this fix is coincidental with the stack
  corruption (which is happening) does not then also corrupt return.

* Hudson: Defer 2.1 until we have a fix for this, or do we release
  with a known-broken board.

* Phil: Release with a fix or pull Redboard Artemis from the release.

* Alyssa: Can we list this under known issues?

* Phil: Hardfaults on boot, unusable. Just exclude it from the current
  release, but it back in later.

* Hudson: Hope for this release was to tag it during the next meeting
  (2022-08-26). Not sure whether that seems realistic. We could at
  least have an attempt at testing for the individual boards by then.

  `weact_f401ccu6` board was assigned to `@yusefkarim`, have not heard
  back from that person. We might need to consider deprecating
  it. Will put a post on the issue stating that, if we have not heard
  back until a release is tagged, it will be deprecated.

  (Note from chat: `weact_f401ccu6` seem no longer current and/or
  available, so deprecating seems like a reasonable outcome)

## PR Review

- Hudson:

  - Merged (non-trivial):
    - https://github.com/tock/tock/pull/3140
      boards/redboard_artemis_nano: Fixup app loading
    - https://github.com/tock/tock/pull/3136 Fix secondary alarm from
      firing immediately during alarm callback
    - https://github.com/tock/tock/pull/3134 use deferred calls to
      report aborted reception in sam4l uart
  - Opened (non-trivial):
    - https://github.com/tock/tock/pull/3149 boards/litex: update
      pinned tock-litex release to 2022081701
    - https://github.com/tock/tock/pull/3148
      boards/esp32-c3-devkitM-1: Prepare for the 2.1 release
    - https://github.com/tock/tock/pull/3139 boards/redboard_artemis:
      Fixup Hard Fault exception

  Hope would be that each goes in before the release, except for
  perhaps the last one.
