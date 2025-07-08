# Tock Core Notes 2021-07-30

## Attending

- Hudson Ayers
- Brad Campbell
- Branden Ghena
- Philip Levis
- Pat Pannuto
- Anthony Quiroga
- Alexandru Radovici
- Leon Schuermann
- Johnathan Van Why

## Updates

### Time TRD (TRD 101 -> 105)

- Phil: Jett's updates to the time TRD are upstream. Currently holding
  off on finalizing 105 (new time TRD), we can let it sit for a while
  and make sure we've got everything right. There is no rush to
  finalize it, we can wait a couple of weeks.

- Branden: Is it ought to be finalized by the time 2.0 is released?

- Phil: Don't think so.

- Leon: This would mean for others that once they are compliant with
  TRD105 as implemented in Tock 2.0 -- which is now a draft TRD -- the
  interface could change again with the next minor Tock
  release. That's okay, but something to think about.

- Phil: The interface is pretty dependable. We want to finalize it,
  when we feel really confident that we don't foresee any needed
  changes. If we want to finalize it before the 2.0 release, we should
  feel confident that there aren't any changes for 2.0.

- Leon: Right, it's a pretty stable interface. Although technically a
  draft TRD is not finalized and thus not dependable.

- Phil: Yes. The IETF uses a similar process, where there is a very
  late draft which is rather stable, to make sure it's correct.

### elf2tab v0.7

- Brad: Just released `elf2tab` v0.7, having the new changes with
  being able to specify permissions as well as the required kernel
  version. It's on `crates.io`.

- Phil: Great, first dependency in the chain.

### Hudson's panic tracing tool

- Hudson: Working on a tool that can take in an `elf` Tock kernel
  binary file and find all of the locations of panics that come from
  the Tock source code, as opposed to the standard library. If there
  is a panic in the core library, the tool backtracks that to the
  point where it is called in Tock. Gotten it to work for a single
  `elf` file and got it to find all the calls which can indeed panic
  (between 100-200 panic locations in our source code). There are
  still a few hacks, hoping to clean that up and submit it such that
  it's usable upstream. This is a good way to build binaries which
  reduce panics.

- Jett: Do we want to reduce the panics upstream?

- Phil: Generally yes, we'd like to reduce them. It's too high a bar
  in that there can be no panics. We want to get rid of them because
  they're bad, could potentially be replaced by functions which don't
  panic and produce a good chunk of embedded data.

- Branden: What are common sources for panics? Is it mostly array
  bounds checks?

- Hudson: Yes, that's the primary source. Sometimes, the compiler is
  smart enough to infer that there cannot be an out-of-bounds
  access. For slices, most of the time there can be panics.

  Out of 140 panics from Tock code (some of non-upstreamed capsules),
  - 75 caused by bounds check
  - 40 caused by explicit `panic!`, `unwrap!`, `expect!`

  For some reason, calls to `Result::expect` failed are separated
  out. Three explicit `Option::unwrap` failed.

  Will send a breakdown of the panic sources to the tock-dev ML.

  Decent chunk from the formatting code from `#[derive(Debug)]`. Is a
  procedural macro, creates formatting code, about 10 panics.

- Leon: Do `#[cold]` functions make a difference? The Rust library
  tends to split out panics in functions which can fail into functions
  marked `#[cold]`, presumably to keep the compiler from inlining them
  and keep the success path efficient as opposed to the failure
  path. Does this make a difference for your tool?

- Hudson: I don't think this makes a difference for my tool.

### OpenTitan

- Phil: query for an update. Johnathan: is there an update
  w.r.t. OpenTitan and code usage?

- Johnathan: can give an update. Previous stance: OpenTitan's work on
  Tock would be out-of-tree in the OpenTitan repository. We're
  pivoting to using the chip crates (earlgrey and lowrisc) of upstream
  Tock and contributing to those more than planned. Will probably keep
  board crates out of tree.

- Phil: Makes sense. What does this mean for the OpenTitan working
  group and membership? We can talk about this offline in the coming
  weeks.

- Johnathan: Possibly. Might end up joining the OpenTitan working
  group meetings.


## Remaining Tock 2.0 PRs

- Branden: Phil sent a list as part of the agenda:

  1. Kernel version compatibility:
     a. [elf2tab PR #30 "Add kernel version header
        implementation"](https://github.com/tock/elf2tab/pull/30)
     b. [libtock-c PR #221 "Added kernel version and automatic update
        for elf2tab"](https://github.com/tock/libtock-c/pull/221)
     c. [tock PR #2669 "[rfc] Comptibility header
        (v1)"](https://github.com/tock/tock/pull/2669)
  2. TRD104: [tock PR #2962 "trd/104-syscalls: clarify guarantees
     w.r.t. returned Allow
     buffers"](https://github.com/tock/tock/pull/2692)
  3. SPI errors: [tock PR #2566 "Make SPI return buffers upon
     error"](https://github.com/tock/tock/pull/2566)

- Phil: These are PR which we decided we want to have into 2.0.

  For 1a, 1b and 1c we just need to work on them.

  Number 2 is important, as we would like to get TRD104's text
  right. Hudson had some comments.

  Number 3 seems to have some open questions, might not want to have
  this PR be on the critical path for 2.0.

### SPI errors PR

- Alexandru: Number 3 includes changes to be compliant with the
  HIL. The open questions are related to the configuration and the
  `init` function.

  First question: the `init` function  uses `configure`, what if that
  fails?


  Second question: SPI mux is broken. It can execute configuration
  commands such as `set_polarity` and `set_baudrate`, however it's
  completely useless. If there is only one device using the mux, it
  works. If there are two or more devices it should store the
  configuration and restore them when a device performs some
  operation. It currently just sets the configuration once.

  I can fix the mux early next week. But as Phil points out, won't
  work if devices need to do transfers back-to-back.

- Phil: Two questions.

  `init` / `configure` can not return an error? It's a design
  question. I'm more worried about this one.

  For the mux, I can fix it today. If we want to fix this bug, we can.

- Alexandru: Technically, `configure` should never fail, unless the
  parameters are wrong (such as an invalid baudrate). I'm not exactly
  sure about the purpose of `init`. Is it actually `enable`, such as
  the `enable` function for I2C?

- Phil: It's not documented at all. If you look at the SAM4L
  implementation, `init` puts the hardware into either the master or
  slave mode.

  `configure` is setting the configuration of the bus itself.

   Another example would be having a USART hardware and `init` would
   put it into either UART or SPI mode.

- Alexandru: If one calls `init` on the slave mode after having it in
  the master mode, it would put the hardware into that mode again?

- Phil: There might be some cases where that could be desired, yes.

- Alexandru: Can it fail?

- Leon: Based on the previous examples, yes. I suppose there might be
  hardware which is only one-time configurable, or one is talking to a
  device which has its clocks powered down.

  Because its such a generic function and entirely dependent on the
  actual hardware in use, there is an infinite number of failure
  cases. Not all of them can be meaningfully handled, however we
  should be able to return them.

- Phil: Do you know of any such hardware Leon?

- Leon: Yes. For instance, on the STMs peripherals are commonly on
  different clocks. If they are powered down, register files can't be
  read (cause some recognizable behavior, e.g. give all zeros back or
  cause an exception).

- Alexandru: Can confirm this is an issue.

- Phil: I see. Does this mean that all peripheral's `init` should be
  able to return errors? Because if its the case for this one, we
  should gravitate towards including error handling for these types of
  operations.

- Alexandru: I2C does not have `init`, has `enable` and `disable`.

- Phil: Written by different people.

- Branden: To add another example, UART has neither of those. But it
  is common to have a USART, combining UART and SPI.

- Leon: With `enable`/`disable`, if I have such a shared-hardware
  device as a USART, I would expect `enable` on the disabled mode to
  fail. Silently changing the device's mode seems very bad. So this is
  an additional error case.

- Phil: We should probably standardize this and have a trait for
  it. But that is not for today. It sounds like `init` should return a
  `Result`, as should `configure`.

- Alexandru: What should happen in a component?

- Leon: Given that components are these ready-to-use blocks on which
  we are building board definitions and those are generally
  initialized on board startups, it seems fair to panic there. More
  advanced users, who might for example convert a USART in UART mode
  to a SPI bus later on, they can always choose not to use the
  component and handle these cases themselves.

- Alexandru: So you are suggesting they should panic?

- Leon: Yes, for components I think that would be fine.

- Phil: I'm generally in favor of fail-fast. There might be cases
  where one might want to not panic, just print a debug message.

- Alexandru: Right, we could just not panic and fail.

- Leon: To give an example, as a user instantiating two components,
  one UART and one SPI bus over the same USART hardware, would
  definitely want my board to panic immediately.

- Branden: Yes, same thing if clocks aren't initialized. You want it
  to break as soon as possible.

- Phil: Would have saved some debugging in the past.

- Branden: These are a different kind of panic -- issuing a panic in a
  board initialization is a different concern than issuing a panic in
  the middle of a capsule.

- Phil: Right, but from a code-size perspective, it's still the same.

- Branden: We can have the component call some function instead.

- Leon: Yes. Components could call to a single `component_init_failed`
  function marked as `#[cold]` as the standard library does. When
  panicing in a component instantiation, we generally don't have the
  UART initialized anyways and thus just blinking LEDs is fine.

- Phil: That is true. In a component initialization we don't need a
  full panic dump. We just need to print a message that a certain
  component failed to initialize.

- Alexandru: Should it continue after that function or hang and blink
  a LED?

- Branden: Probably actually hang.

- Phil: For now just call to `panic!` and we can sort out the details
  later.

### Tock 2.0 PR triage

- Brad: We need the compatibility header,
  [#2669](https://github.com/tock/tock/pull/2669).

- Phil: 1c in the list above.

---

- Phil: Brad has done some cleanup changes
  (e.g. [#2710](https://github.com/tock/tock/pull/2710)). We should
  include those.

---

- Phil: [#2709](https://github.com/tock/tock/pull/2709)?

- Branden: Based on the immediate responses, this will take some time.

- Leon: It appears to be a small fix, but it is an important bug, very
  annoying and hard to pin down for people new to Tock.

- Brad: That should be a blocker release-candidate 2.

- Jett: We want to get this in before 2.0?

- Phil: If it's a bug, we need to fix it, yes.

- Jett: This PR fixes the bug.

- Phil: But there's issues with it.

- Brad: Going to be a lot of PRs like this one once we start testing.

---

- Phil: [#2707](https://github.com/tock/tock/pull/2707). We can wait
  on 2.0 for this one.

---

- Phil: [#2705](https://github.com/tock/tock/pull/2705)?

- Leon: Appreciate if we make it in.

- Pat: Bors already started.

- Phil: Should distinguish things we'd like to get in and things we
  need to block on.

---

- Phil: [#2701](https://github.com/tock/tock/pull/2701).

- Alexandru: Would appreciate to have this in. We merged drivers
  upstream with errors because we haven't seem them. Only discovered
  because of this PR.

- Pat: Do we want to have this `enum Callback` as proposed by
  Alistair? I think it's interesting, but probably leads to more
  confusion.

- Leon: And requires importing the type then.

- Phil: Should we block on this one?

- Brad: We should block on this one, either do it or don't. This
  affects every single driver.

- Leon: It only changes kernel-internal APIs and is mechanical. It
  wouldn't hurt to get it in but not a necessity.

- Hudson: One reason we had blocked on the larger kernel
  reorganization was that it requires many changes for downstream
  developers. Ideally with 2.0, we would not immediately follow this
  release with other updates which again require massive changes for
  downstream developers. That is one reason why it makes sense to get
  these far-reaching changes in before the release.

---

- Phil: [#2633](https://github.com/tock/tock/pull/2633)?

- Pat: I don't expect to reach a state of maturity before 2.0, so
  probably merge shortly after the release.

---

- Phil: [#2566](https://github.com/tock/tock/pull/2566). Sounds like
  we're going to want to include this.

---

- Branden: [#2381](https://github.com/tock/tock/pull/2381)?

- Hudson: Explicitly decided to be merged after Tock 2.0.

- Leon: Probably justifies a bit more discussion as well.

---

- Leon: [#2248](https://github.com/tock/tock/pull/2248)?

- Phil: There seems to be a lot of design work there still.

## Release process & testing

- Jett: Should bugs to be included in Tock 2.0 also be flagged with
  the `tock-2.0-include` label, or just `bug`?

- Leon: There is at least one bug
  ([#2637](https://github.com/tock/tock/pull/2637)) which turns out to
  be pretty complex and thus will be fixed after 2.0 as long as
  testing doesn't reveal any impact. Judging based on that, bugs to be
  fixed in 2.0 should be marked as such.

- Hudson: As a general rule, a release does not have to fix every
  outstanding bug, just has to fix the regressions from previous
  releases.

---

- Phil: Brad, can you walk people through the release process?

- Brad: Once we included all pull requests which we want, we're
  tagging a release candidate 1. Usually some people will start
  testing. It's usually better for people to not test all at the same
  time, as fixes will come in. Going to keep tagging release
  candidates as boards come in passing all of the tests.

  As for managing the tests, we have comments on the [Release 2.0
  issue](https://github.com/tock/tock/issues/2429), where we have a
  template of different tests and userspace apps to run.

  Once all boards have been tested and seem to be working, the core
  team signs off on the release.

- Jett: Do we create a 2.0 branch?

- Brad: We usually just tag it. If there turns out to be a major issue
  with the release, we can create a branch from that and do a patch
  release.

- Jett: Do we have a document outlining what type of issue (security,
  etc.) justifies a maintenance release like that?

- Leon: Suppose that would make a great TRD.

- Brad: Have a document which starts that. Mostly outlines the tasks
  as part of a release.

---

- Leon: Question regarding release testing -- would we also block on
  bugs which aren't exposed to userspace? There are some boards which
  have drivers for HW, which aren't necessarily exposed to userspace
  or supported in the rest of the ecosystem yet. For example, the
  LiteX boards have an Ethernet drivers which I'd like to test.

- Phil: I think we would block on these. We use userspace tests
  because they make testing easy, but there are also kernel
  tests. Also, it's hard to trigger edge cases from userspace, for
  example with the Alarm virtualizer.

  Although, it is okay to find bugs which we decide not to fix for the
  release. We can have known limitations in a release.
