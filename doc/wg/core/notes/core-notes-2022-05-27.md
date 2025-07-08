# Tock Core Notes 2022-05-27

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

# Updates

## Unsoundness in `VolatileCell`

* Alyssa: Discovered some unsoundness in `VolatileCell`. It gets
  allocated read-only when used as a static field and is placed into
  the `rodata` section in the final binary; writing to it causes a
  fault.

  The undefined behavior occurs because it can be modified with just
  an immutable reference to it, and it does not have interior
  mutability.

  The only reason why we haven't seen a lot of this UB is because
  we're using volatile reads and writes, so these are not going to be
  reordered or elided, even if Rust thinks that the memory has not
  changed.

* Leon: the fix for this would be to just have it wrap an
  `UnsafeCell`?

* Alyssa: yes. And technically it should be made unsafe in the way
  it's current used, or creating it should be made unsafe.

* Hudson: right. You can create a `VolatileCell` with any type inside,
  and `get` and `set` are just safe wrappers around the unsafe
  `volatile_{read,write}` internally. There's nothing stopping you
  from using this on something other than MMIO registers.

* Amit: what specifically does `UnsafeCell` do in this particular case?

* Alyssa: this is the only way to tell Rust that some memory may
  change behind a shared reference. You cannot mutate behind a shared
  reference, unless its interior mutable.

* Leon: it still does not take care of volatility, just that the
  memory is allowed to change.

* Phil: and volatile makes sure that they can't be elided.

  Also, code seem originally taken from
  https://github.com/hackndev/zinc/tree/master/volatile_cell. Might be
  worth informing them. It looks like their implementation changed a
  lot.

* Alyssa: `VolatileCell` is usually not what you want; what you
  actually want is a pointer to volatile memory.

* Leon: talked about this as part of the discussion around
  tock-registers, and why what we're doing there is unsound currently.

  Rust is technically allowed to insert arbitrary reads to references,
  even if the value isn't accessed explicitly.

* Alyssa: yes, LLVM is technically allowed to insert spurious
  reads/writes to references, but I cannot conceive why it should.

* Amit: so the problem is that a spurious read over MMIO can have side
  effects?

* Alyssa: references in Rust are declared as dereferenceable in the
  LLVM-IR. A requirement of that is that reads have no side effects
  and the accesses have to behave like regular memory.

  This also holds true for `VolatileCell`, for which we technically
  violate this requirement.

  However, in our codebase all reads and writes that _should_ happen
  are declared as volatile, so in practice this (spurious reads) might
  not lead to issues for us.

* Leon: does sound like an important issue though. In the previous
  discussion about tock-registers we concluded that we'd like to
  redesign its API and we should fix these issues.

* Phil: should be a full agenda item.

* Leon: or good discussion for the mailing list.

* Hudson: we should mark the constructor `unsafe` and have it wrap
  `UnsafeCell` in the short term. For the more intricate issues, I
  would rank this as less important than some of the other soundness
  issues we have currently.

## WiFi trait & driver

- Alex: Alex (student) has been working on WiFi interfaces and a
  driver for Tock. We are able to send and receive UDP packets. It's
  not complete yet, but working towards a fully functional
  driver. Question: what should the data channel look like? It works
  on the Arduino RP2040.

* Amit: what's the WiFi hardware for that?

* Alex: WiFiNINA by U-Blox, probably an ESP32 underneath.

* Hudson: were you able to reuse the IPv6 UDP stack, or was that too
  tightly coupled to 6LoWPAN?

* Alex: no, that is too tightly coupled. Also, currently we can only
  use one UDP socket at a time.

* Amit: does the device implement the IP/UDP stack itself? Can you
  send raw IP packets?

* Alex: yes, it's a network coprocessor. Not sure whether we can send
  IP packets directly, we had to reverse-engineer the interface from
  the Arduino source code.

  Project for another student is to use an STM32 with a userspace
  network stack, which is going to be more tightly integrated with
  Tock.

  Most of the chips around have a fully integrated IP stack. I have
  been struggling to find a chip which has just a bare-bones WiFi
  radio inside, not a full processor.

* Leon: Problem with these devices is that the implementation will get
  significantly more complex. You will have to provide an entire WPA
  supplicant, etc.

* Alex: Tried it on the RISC-V ESP. The WiFi hardware is not
  documented, and the IDF provided by Espressif uses a binary
  blob. Tried to integrate this with Tock, was not able to call more
  than one function and have it crash.

# TockWorld 5 Invites

* Hudson: The dates are set and hopefully everyone who wanted to
  recommend people to invite has sent these recommendations over. Amit
  and I talked about who would be reasonable to invite.

  *[list omitted for privacy]*

* Branden: is anyone going to invite more students?

* Brad: thinking about it.

* Pat: probably not. The student working on hardware CI is doing an
  internship during this time.

* Branden: from my perspective, adding a couple more people is not
  going to hurt.

* Hudson: I assume there is no concern we are going to have a strict
  (e.g. < 25) limit on the room capacity?

* Branden: don't think so.

* Johnathan: according to Hudson's email, the plan is for the first
  day to have everyone involved, and the second day be just for the
  core team. Is that final?

* Hudson: that seems like a reasonable approach for it. It allows
  people who are not super involved to more easily book travel on the
  second day.

  It makes sense to keep project governance to just the core team, and
  this is currently planned to be on the second half-day.

* Phil: if we choose to do that, it seems very important to hear the
  people who are not in the core team what their thoughts are on
  project governance (e.g. from lowRISC, OpenTitan at Google,
  etc.). We can have an extended discussion within just the core team
  after that.

* Amit: I agree. Differentiating between two days makes it easier for
  people who are not on the core team, but other people should be
  allowed to attend still.

* Phil: agreed, it's not secret.

* Leon: so that would lead to essentially a differentiation between
  organizational and technical discussions over the two days.

* Pat: we have not talked about whether we want to support remote or
  hybrid participants.

* Amit: we should support hybrid if it were the case that key people
  cannot attend in person, but that does not seem to be the case. If
  we're supporting people attending remotely, it might make the
  experience worse for everyone else.

* Leon: could be on a per-meeting granularity? E.g. dzc-self had some
  very interesting project-inspired feedback on Tock. It would make
  sense to hear this feedback and incorporate it into the
  discussions. dcz-self cannot attend in-person, unfortunately.

* Phil: benefit of having occasional in-person meetings is to create a
  social fabric, as well as it being a form of high-bandwidth
  communication. Developing the security model at the previous
  TockWorld worked really well. Other organizations (IETF) made bad
  experiences with a hybrid model.

  Having virtual events to talk with users would be great as well,
  just not hybrid.

* Pat: agree with all of this, just want a finite answer to this
  question.

* Brad: would be open to having someone give a remote talk. Trying to
  engage in a hybrid model over the entire day seems unreasonable.

*settled on enabling remote presentations, but not hybrid in general*

* Alyssa: hybrid is a lot easier for people to participate (just
  live-streaming).

* Braden: in this case it seems more about deep discussions from the
  people who can make it, instead of lots of people attending.

* Amit: if there are talks permissible to share widely, it seems
  absolutely reasonable to live-stream these. Live-streaming
  discussions is tougher to do technically and may have an influence
  on the discussion culture. It is probably a good idea to have an
  additional virtual-only event.

* Branden: we can get a classroom which is set up with the appropriate
  equipment for live-streaming (using a tool we run, e.g. Zoom or
  Google Meet).

## Mutable & Immutable Buffers for Digest and other HILs

* Hudson: Leon and I have an update for our approach to implementing
  mutable and immutable buffer passing with the Digest trait, and Phil
  wants to give an update to his approach as well.

* Phil: update relates to the current state of [PR
  #3041](https://github.com/tock/tock/pull/3041). Digest needs to take
  both immutable and mutable slices: mutable for data in RAM,
  immutable for data in flash such as an application binary.

  Prior approach was designed to use two traits for both mutable and
  immutable buffers respectively. Went through with this approach and
  decided that it is not workable. For RSA this worked well, but
  Digest is more complex.

  Current approach uses two functions and two callbacks for immutable
  and mutable buffers respectively. When calling the mutable function,
  the client receives a mutable callback, restoring the buffer
  reference.

  This seems strictly better than the two traits approach as well: for
  instance, the mutable function can refuse an operation when the
  immutable operation is currently processing a request. With two
  traits, each interface must either cache requests for them to be
  multiplexed onto a single underlying hardware, or refuse operations
  because a different unrelated interface is busy.

  With the single trait, it is also possible to compute data which is
  located partly in RAM and flash (does not have to be contiguous).

- Leon: Hudson and I have been experimenting with a different approach
  described in [the comments on that same
  PR](https://github.com/tock/tock/pull/3041#issuecomment-1130655228).
  Implementing a variant of this proposal immediately revealed some
  issues with it. To recap: the proposed approach defines two new
  types, one for holding a mutable and one for holding an immutable
  `'static` slice. We would only pass down a `'static` immutable
  reference to this container to the hardware, which locks the
  container to prevent the buffer from being replaced or accessed by
  other layers. When the requested operation completes, the container
  is unlocked prior to invoking the client callback.

  One issue with that is that capsules have been written in a way
  relying on checks whether a buffer is present in a capsule-held
  container (e.g. `TakeCell`). However, with the proposed approach a
  buffer would always be present in this container, and the lock
  indication might not be immediately set when requesting an
  operation, for instance when it is queued by a mux.

  We transitioned to a second approach, still using two containers
  holding an immutable and mutable buffer respectively. However, when
  passing down a buffer to lower layers, both of these types lock
  themselves internally and synthesize a buffer handle, with a type
  independent of the original buffer's mutability. This handle only
  allows immutable access to the underlying memory (something which is
  sound for both mutable and immutable slices), for as long as this
  handle is in scope. To restore access to the original buffer, the
  handle needs to be consumed by the buffer container again.

  This infrastructure is analog to concepts already used around Tock:
  the containers (`MutableDMABuffer` and `ImmutableDMABuffer`) replace
  `TakeCell`s maintained in the capsules, whereas the
  `ReadableDMABufferHandle` takes the place of `&'static mut [u8]`
  slices passed down to peripherals and back up in
  callbacks. Therefore, it does not change any semantics in existing
  capsules and can be integrated easily.

* Amit: how could this relate to `LeasableBuffer`?

* Leon: the concept of `LeasableBuffer` can be trivially integrated
  into this solution. Instead of handing out
  `ReadableDMABufferHandle`s over the entire slice held by the
  container, we can simply create a `ReadableDMABufferHandle` over a
  window in that slice.

* Hudson: the current interface does not support this. We would add a
  function similar to `borrow_readadble`, which returns a handle over
  a subslice.

* Leon: we can even go ahead and define a function akin to
  `return_readable`, which would move the borrowed window forward in
  the buffer. This would be particularly useful for chunked
  operations.

* Amit: that seems like it could make a lot of our internal interfaces
  much simpler. Rather than being a reasonably complex type
  infrastructure to solve a narrow problem, it seems this has the
  potential to become a generally useful (and similarly complex)
  buffer type to pass around the kernel.

* Leon: if we were to integrate `LeasableBuffer` into this natively,
  given that we only need LeasableBuffer to maintain a subsliced
  region on a `&'static mut [u8]` buffer across call stack
  invocations, we can get rid of that infrastructure entirely.

  There is one major drawback with this approach: given that we coerce
  both `MutableDMABuffer`s and `ImmutableDMABuffer`s into a single
  `ReadableDMABufferHandle` type, we need to keep track of the
  container which created a given handle, and to only unlock locked
  containers with matching handles. For this we use a static counter
  incremented once for container allocation and panic when that
  wraps. Should not be a problem, given we have only a constant number
  of containers allocated at board initialization.

* Branden: for the nRF52, DMA only works from RAM. All peripherals
  have their own DMA engine, but all of these are limited to RAM.

* Leon: excellent question. If we were to change these functions
  changed across our HILs, it might well happen that the nRF52 is
  exposed to buffers over flash memory. Not sure whether we want to
  incorporate these restrictions into the type system, but we should
  return a runtime error in such cases.

  Plan is to extend this implementation to the `Digest` HIL to have a
  direct comparison, extend it and write tests.

* Hudson: should be noted that this is currently only for operations
  which don't mutate memory. We might want to extend this to work for
  mutable operations as well.

* Phil: it sounds like these approaches are solving different
  issues. Leon's and Hudson's approach looks down at the DMA level,
  whereas my issue is much narrower, just for the Digest
  implementation.

  For me the Digest is blocking for end-to-end verification of
  processes.

* Leon: the two-methods approach seems like a good short-term solution
  just for `Digest`, whereas our approach might be something we'd like
  to integrate in the long term.

* Phil: Agreed.
