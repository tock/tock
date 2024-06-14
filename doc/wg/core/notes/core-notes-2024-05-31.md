# Tock Core Notes 2024-05-31

Attending:
- Hudson Ayers
- Branden Ghena
- Alyssa Haroldsen
- Philip Levis
- Amit Levy
- Tyler Potyondy
- Alexandru Radovici
- Leon Schuermann
- Johnathan Van Why

## Updates

### UART TRD Rebase

- Leon: Working on rebasing the port to the UART TRD & HIL. Can't really use
  git-rebase, because it's been pending for too long and the codebase has
  changed significantly.

  Not sure whether this is breaking any UART implementations, as sometimes the
  changes are substantial. Can catch that during release testing?

- Hudson: Might be a while to the next release, if we do one soon?

- Leon: Could try to squeeze it in.

- Hudson: Otherwise, might test a few of the platforms ourselves. Otherwise,
  users can always report issues to us.

## `tock-register-interface`: Keep `RegisterLongName` and Register Type Separate?

- Hudson: Johnathan posed the question of whether we want to keep
  `RegisterLongName` and the register data type separate.

  Let's read https://github.com/tock/tock/pull/4001#discussion_r1619542913.

- Johnathan: Analogous to the current syntax. We could have the register types
  take just one argument, such as either the `RegisterLongName` or the integer
  type.

  If Amit's argument is correct (that we had both to make it explicit how long a
  register is, as we did not have explicit offsets), then this should no longer
  be necessary with this proposal.

- Branden: This seems better, now that we have explicit offsets. But we can
  still use just regular integers without `RegisterLongName`?

- Johnathan: Yes.

- Phil: Amit's analysis as to why we had both previously is correct.

  Do we still want to support non-uniform sizes? Trying to think of an MMIO
  register that is larger than the word width, but we need to be able to access
  it atomically.

- Branden: So you want a bitfield, and have it be of different size than the
  other bitfields?

- Johnathan: Not supported in tock-registers right now.

- Branden: Could have two bitfield declarations.

- Hudson: New design looks good.

- Alyssa:

  In chat: > +1, also the size of the field itself can be looked up at the >
  `register_bitfields!` declaration location
  >
  > `#[repr(transparent)] struct AlignedTo<T, U>([U; 0] T>;`
  >
  > could also do this by wrapping the original register bitfield?

  Concerning alignment.

- Johnathan: In the new design, there's a disconnect between the register type
  and the register size. The actual size is communicated through a new trait.

- Alyssa: Makes it more conceptually confusing. Having a thing of length, e.g.,
  4, but the actual type be zero-sized.

- Leon: Right now, it seems like we just rely on the alignment working out. Are
  you validating that the alignment is corresponding to the types' size.

- Johnathan: No, but should be easy to add.

- Leon: Should probably do that by default. If we ever don't want that, we can
  add an additional parameter to override.

- Jonathan: On LiteX, does the alignment of a field always correspond to its
  size?

- Leon: The platform ensures that any given register is always exposed to
  natural alignment, or a more coarse-grained alignment. For us, we should be
  able to, e.g., rely on the fact that a u32 will be always be placed at an
  offset aligned to 4 bytes or greater.

- Johnathan: Okay, so a u32 on a LiteX system where it takes up 16 bytes of
  address space -- if core::mem::align says its alignment is 4, then the
  register's alignment constraint is still 4, even though the register takes up
  16 bytes of address space?

- Leon: Correct.

## 64 bit Timer Proposal

- Leon: Follow-up to the alarm capsule rework, given the imminent merge of
  [#3975](https://github.com/tock/tock/pull/3975).

  What's our take on Alistair's proposal to extend the userspace timer interface
  to 64 bit:
  - [timer: Add support for 64-bit
    timers](https://github.com/tock/tock/pull/3343)
  - [Expose 64-bit timers to the userspace alarm
    capsule](https://github.com/tock/tock/pull/3355)

  This is related to the problems we had in porting OpenThread to
  `libtock-c`. Summary: platforms with a hardware-timer of 24-bit had their
  Ticks value directly passed through to userspace. Thus, the timer exposed to
  userspace did not wrap at `2^32`, but `2^24` ticks. This meant that it was
  impossible for userspace to predict when the timer was going to wrap, and thus
  it could not build a "virtual" timer that reliably worked for longer than the
  relatively frequent `2^24` ticks rolled over.

  Many different proposals discussed, such as implementing wrapping in the
  kernel.  With the new fixes, we're left-justifying Ticks values exposed to
  userspace, so they always wrap at `2^32`. Turns out that this is exactly what
  userspace always expected, documented in a comment from Amit in 2017 in the
  userspace alarm implementation.

  The PR that we're about to merge makes sure that the kernel delivers on those
  assumptions, but does not break the interface.

  A couple years back, Alistair proposed to pass 64-bit alarms to userspace. Our
  current PR solves at least one of those issues -- userspace can now build
  timers that do last for longer than the exposed `2^32` ticks.

  Question now: what's happening with Alistair's PR?

- Amit: With your PR merged, we would support arbitrary bit-width timer
  infrastructure in the kernel, including 64-bit, right?

- Leon: Yes.

- Amit: Only question is whether the system call driver should be 32 or 64
  bit. In my opinion, it seems perfectly fine for some niche use-cases to have
  their own system call driver that have their own API, but the common case is
  32 bit. This seems sufficient for most use-cases, and so it should be the
  canonical upstream one. There may be other that lived upstream, for those
  other usecases.

  We would want to have a compelling reason to either complicate the existing
  driver to everyone, or switch to 64-bit which would make things more expensive
  (for some users, uneccessarily so).

- Phil: Agree with that. One perspective is that on RISC-V we do have those
  64-bit timers, so we should be able to use them. However, reasoning about the
  interactions with 32-bit is complex. It's understandable that Alistair wants
  this to be supported for his platforms.

  Given the complexities around timers smaller than 24-bit, where we
  left-justify to work around these issues, would that mean that for all
  platforms we now left-justify to 64-bit? That doesn't make much sense.

  Agree that 32-bit should remain standard, and 64-bit can be its own driver.

- Leon: Also answers my question in which direction to guide this PR; seems like
  we want to suggest pushing for just another driver.

- Amit: Do all RISC-V platforms have 64 bit?

- Phil: Yes, mtimer, the standard RISC-V timer is 64 bit.

- Leon: Not all platforms have mtimer though...

- Phil: For new PR, looks like you wrote a lot of tests. With the old version,
  one of the challenges was to cover all of the crazy edge cases.

- Leon: Wanted to bring this up -- would like to see more eyes on the alarm
  driver rewrite. It's hard, and I don't claim that I got it right. However, the
  new design learned a lot from the old one; none of the actual wrapping logic
  is different. It just splits it into individual functions that are testable,
  and removed some other quirks around event handling and state machine
  progression.

- Phil: Run a test where you have multiple kernel timers and multiple userspace
  timers, and let it run for a few days.

## PRs #3252 and #3258 (Add syscall notification mechanism / attributes to shared buffers)

- Leon: System call notification mechanism proposed by Alex two years
  ago. Hudson commented on it, and it got my attention because of a slightly
  related issue in the alarm rewrite.

  For the alarm driver, we have this optimization where we count the number of
  alarms that are set. However, this does not handle the case of process faults
  and restarts correctly. For now, removed that optimization.

  This is tangential to Alex's initial motiviations for this PR.

- Alex: Not for notifying about faulting apps, but for knowing whether a buffer
  was swapped, because the capsule should know whether it should reset its write
  offset in a buffer.

- Leon: Reason I brought this up is that presumably the implementation of any
  given notification mechanism is going to be quite similar. Didn't mean to
  divert this discussion from the original use-case.

  Was looking for a fundamental discussion for as to whether we want to have a
  notification mechanism for certain kernel events at all.

- Alex: I would support for a capsule to at least have a way of determining
  whether such an event occurred between any two commands (like buffer swapped /
  application crashed, etc.).

- Hudson: Push back against adding application crashes into the scope of this
  PR. Might open pandoras box, where capsules are much harder to reason about
  and test if they can run arbitrary code in response to these events.

- Alex: Could figure out whether an app crashed depending whether the grant is
  empty or not.

- Hudson: But not if it restarted.

- Alex: Can set a magic value in the grant logic.

- Leon: Other case not covered -- peripherals that are only enabled when an app
  is using their respective system call drivers. When an app faults, the
  peripheral will never be turned off.

  This is tangential the main question -- want to see whether there is
  interested in having any notification mechanism, for any type of event. Should
  avoid further stagnating this PR.

- Hudson: The ability to potentially extend this PR to cover these other
  use-cases would seem like it's better than #3258, which would not support
  this.

  Alex, have you used either of these implementations in a while?

- Alex: No, but can port it.

- Leon: Before we spend time on this, should clarify whether we do not want
  #3258 instead. This one instead adds an attribute to buffers to indicate that
  they have been swapped.

- Alex: In favor of #3258 more than #3252. Problem is: right now, a capsule
  needs to rely on an application for it to be correct.

  There was pushback against having a capsule be able to react to
  allow/subscribe events, and #3258 is the only way to prevent that and be able
  to tell whether a buffer was swapped.

- Leon: I do agree with the fact that, in the general case, capsules should not
  be able to react to allow/subscribe events. However, it does seem like the
  change to the alarm driver tells us that sometimes this is genuinely
  useful. It's not a system call but a process state change, but the pricinple
  is the same.

  Right now, we have incorrect capsule code that is hard to get right without
  notification mechanisms, whereas we did not want to introduce those mechanisms
  also to avoid capsules becoming too complex.

- Alex: We should decide whether capsules should support notifications, with the
  caveat that with notifications the capsules might do arbitrary things.

- Amit: Share the intuition that the attributes approach is lighter-weight and
  less intrusive to drivers that do not care about this aspect.

- Leon: It's simple, and light-weight, but seems like it's a modification to
  some core infrastructure to achieve its goals. Not sure I like that it's not
  very general.

  The implementation is also slightly hacky, by using a bit in the buffer's
  length field as a niche, etc.

- Amit: Not tied to this implementation.

- Leon: Even then, I'm feeling uneasy about this as I cannot directly see how
  this feature would generalize to user use-cases?

- Amit: Examples?

- Alex: Any network, any streaming capsule will have these types of issues.

- Leon: Given that this is so specialized, would it make sense to add a
  dedicated allow system call, similar to the userspace-readable allow?

- Amit: Sounds like it would add a fairly significant overhead.

- Leon: With userspace-readable allow, the implementation is shared with regular
  R/W allow, just different semantics in the design document -- not enforced.

  Here we could do something similar, but the buffer would be wrapped in a
  special ring-buffer interface that allows you to place a packet into, or pull
  one out of the buffer.

- Amit: Does it alleviate your concerns if we mark this interface as unstable?

- Leon: Sure.

- Phil: Whether we generalize this to other attributes, I'm a little skittish on
  as well, but this does seem like a basic piece of information that a capsule
  should have access to.

  Leon, understand your point about ring buffers -- we were using them for
  high-speed ADC at some point -- but it's pretty hard to get those right,
  especially for a very simple problem: I want to know whether something was
  changed.

- Hudson: We keep mentioning that this is necessary for capsules to be correct
  -- not convinced. These seem like optimizations. PR contains example for touch
  driver outlining interactions between allow and command. None of these seem
  like they'd actually be different from app misbehavior, such as re-allowing a
  buffer that has not been fully consumed.

- Alex: Problem: you have an offset in a buffer. The capsule writes to that
  offset. It needs to know when it can reset this offset (start writing at zero
  again), which is when the app provides it a new buffer to write into.

- Phil: Streaming stuff into buffer from UART -- one approach is to just swap
  out the buffer as soon as there's contents in there. But then the capsule
  would resume writing in the middle of the new buffer. You can swap the buffer
  and tell the capsule, but there's a race condition there.

  It's really about having an index pointer into the buffer.

- Alex: Solution we have right now is to maintain offset in the buffer. If the
  app fails to zero it, the capsule is incorrect.

- Leon: I don't understand this very last part, why would "the capsule [be]
  incorrect"? Except for the guarantee by the kernel that it won't lie about
  whether a buffer has been swapped, you could represent this by a one-bit
  "ready" handshake in the buffer.

  Capsule should _always_ check that their buffer indicies are in bounds, and if
  userspace supplies an invalid offset, it would simply cause the capsule write
  at an incorrect index.

- Phil: Correctness in the sense of "you want the capsule to write data at some
  offset, and know whethere that offset is".

  Let's say we're doing high-speed ADC, sampiling 10KHz, I'd like to pass a new
  buffer for the capsule to continue streaming into. How do I do this such that
  the capsule knows it should write at index zero, not index, e.g., 43.

- Leon: For example by reserving the first byte, and having the app ensure that
  every new buffer it shares zeroes this byte. The capsule then writes a
  non-zero value to that before it writes any data. When it contained zero
  previously, reset your offset.

- Tyler: This is essentially what I implemented for 15.4.

- Alex: This is what we have in the CAN driver, but I'd like to avoid relying on
  the app.

- Phil: In the sense of "if the app doesn't zero it, I don't want to check".

- Hudson: If the app doesn't zero it, all that happens is that the capsule
  writes to the middle of the buffer. This doesn't affect other parts of the
  system.

- Phil: What if it's shorter.

- Leon: We'd have to do a length check every time we write (which we'd likely do
  anyways).

- Phil: Now we're passing more structured data in the buffer (length/value). Not
  convinced, but this does seem like a viable alternative.

  I agree that the added complexity from a notification mechanism is a tougher
  sell.

- Leon: We might want to just have a thin wrapper around existing allow buffers
  that expose these semantics.

- Phil: Makes sense. If the abstraction works well for our three use-cases (ADC,
  15.4 and CAN), then it's probably pretty good.

  Seems like a good thing to add to guidelines for writing capsules.

- Amit: What does this mean for #3258?

- Leon: Seems like we can relatively easily get all of the benefits of this
  approach without modifying any of the core kernel infrastructure. So I'd vouch
  for exploring that path.

- Amit: We should set a deadline on this. Let's revisit around TockWorld.
