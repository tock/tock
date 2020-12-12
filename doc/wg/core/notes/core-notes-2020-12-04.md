# Tock Core Notes 2020-12-04

## Attending
* Hudson Ayers
* Brad Campbell
* Branden Ghena
* Philip Levis
* Amit Levy
* Pat Pannuto
* Leon Schuermann
* Vadim Sukhomlinov
* Johnathan Van Why
* Alistair

## Updates

### Rubble-based BLE support

* Alistair: Starting to work on the [BLE capsule based on
  Rubble](https://github.com/tock/tock/pull/2233) and trying to get it
  upstream.

* Amit: Great! What's the state of that?

* Alistair: The last concern with David's original work was that it's
  contained in the `components` crate, which then could be pulled in
  for other boards that do not have BLE / Rubble support.

* Hudson: David moved it to another directory, so now it seems fine. I
  tried to use the PR on the Nano 33 though and it did not work, there
  don't seem to be any BLE advertisements.

### Arduino Nano 33 & Adafruit CLUE bootloaders

* Brad: Related to the Nano 33: played around a bit with the
  bootloader, trying to figure out a better bootloader system.

  Motivation for this: just merged the Adafruit CLUE, which has a
  different bootloader, but neither of them support reading back
  flash.

  Trying to figure out a viable path forward. The best option appears
  to be to implement the Tock bootloader on top of the USB stack.

* Amit: Does it seem like once that happens people will need to
  reflash the Nano 33 with some JTAG pins soldered / attached or is
  there a way to bootstrap using the existing bootloader?

* Brad: the leading idea is to take the existing bootloader, which
  flashes a stage 1 on the board, which replaces the existing
  bootloader.

  Also, we should have a backup bootloader (the usual one) to sit
  there dormant (which sits at the end of flash). If someone does not
  want to use Tock anymore, they could recover to a state which works
  with other SDKs and toolchains.

  I hope that this can be made simple enough to be usable.

* Amit: How are the boards used now?

* Brad: One of the pre-shipped bootloaders allows writing to any
  address, whereas another one allows writing to a specific region
  only.

  Without read support, we cannot list installed apps, etc.

* Amit: Might be useful for other boards as well.

## Porting capsules for Tock 2.0

* Amit: Phil and Leon have got the system call ABI to a point where it
  should be ready to start porting capsules. Phil sent an email asking
  about feedback on the porting document.

* Phil: Leon and I have been working on the Tock 2.0 system call
  interface. It's not completely finished, but will get polished in
  the next few days.

  I have ported the LED capsule (being trivial) and the Console
  capsule, which is much more difficult.

  The tricky part is understanding the semantics of the drivers and
  how the system call interface is managing buffers, especially given
  the fact that AppSlices and Callbacks can disappear between any two
  Driver operations. This has been true previously as well, but is
  explicitly acknowledged in the interfaces now.

  I've tried to write up a document explaining the structure of ported
  capsules and the reasoning behind the changes.

  I think that we are mostly ready to go with respect to porting
  capsules. There is still work going on in the kernel, but that is
  mostly isolated from the interfaces capsules are using.

  Porting some capsules will be very easy, whereas others might be
  harder to get the buffer and callback semantics right.

* Amit: It seems like we should enumerate the capsules and divvy them
  up amongst volunteers. Perhaps take a straw poll of who has spare
  time to port some?

* Phil: I think we should create an issue to keep track and cluster
  the capsules by difficulty. Especially with difficult capsules, the
  original authors might find it easier to understand the internal
  structure and state machines, so they can focus on the new Driver
  trait.

* Leon: As part of that the people porting capsules should also cover
  the respective transitions in `libtock-c`.

  This will allow us to test the changes directly, as the transition
  is architected in a way such that old and new drivers can be used
  side-by-side, with minimal breakage of the old ABI for unported
  capsules.

  Phil and I are going to create the wrapper functions around the new
  interface in `libtock-c`. For every ported capsule in the kernel,
  userspace can then be updated to use these new functions.

  A good strategy might be to have pull requests against the
  respective `tock-2.0-dev` branches in both the kernel and
  `libtock-c` repositories.

* Phil: The `libtock-c` changes are very mechanical. The most
  important change is to use read-only allows where previously
  read-write allows were used.

  When we actually want to improve `libtock-c` to make sure we are
  utilizing all the new return value variants, that will be a separate
  change.

* Leon: Right. My motivation is to take advantage of the additional
  features of the new ABI to make capsules work more efficiently.

* Phil: I see this as a second step. Let's first port things such that
  they work and improve the designs as part of a separate change.

* Johnathan: `libtock-rs` is going to be delayed at least several
  weeks, as I'd like all PRs to go through prior to porting drivers.

  After porting kernel capsules and updating documentation, I'll start
  porting the `libtock-rs` drivers, which I'll send to the respective
  author of the Tock 2.0 rewrite for review.

* Amit: In summary, Phil is going to enumerate the work to be done.

* Leon: We should create a tracking issue where people can tick off
  their work using checkboxes.

* Hudson: This shouldn't touch any of the virtualized capsules at all?

* Leon: No, just things which previously implemented `Driver`.

### Implementing the threat model for Tock 2.0

* Johnathan: We were aiming to completely implement the threat model
  as part of Tock 2.0. The argument was made that Console, should be
  muxed between different applications. Is that something we should do
  after porting all capsules to the Tock 2.0 system call ABI?

* Leon: Did not have it on my radar. There's a lot of things to do for
  Tock 2.0 once the ABI is established and ported to. It's easy to add
  things on the Todo list, but that might delay Tock 2.0.

* Phil: How does muxing the Console relate to the threat model?

* Johnathan: When data is sent to the chip which every application can
  read, that data is not tied to a particular application.

  When we had discussions on the threat model, we decided that any
  networked protocol where possible, including and in particular
  Console should be muxed so that messages are intended for specific
  apps and routed exclusively to the respective apps.

  Do we still want to do that, and where do we put that on the
  roadmap?

* Phil: It's different from how consoles behave in UNIX systems.

* Johnathan: It is. If the answer is to change the threat model, I'm
  fine with removing that requirement. I don't remember who the
  proponent for this was.

* Leon: If we introduce such a Console mux, we should make it optional
  and for boards to decide.

* Amit: Yes, it should be implemented as a different
  capsule. Differentiate between a "regular" (root) Console and a
  muxed Console. Specify that the regular Console should not be used
  for production.

* Phil: An effective way without an explicit unmuxing layer on the
  host side would be to use a console over USB with different virtual
  console devices on the host.

  Let's first port the Console over to the new interface and if we
  want to implement the muxing behavior, do that as a second step.

* Johnathan: If we decide not to do this, I'll update the threat
  model in a PR.

  Most OSes don't have this particular level of isolation.

* Phil: To be fair, on Linux systems you can create an arbitrary
  number of virtual consoles (pTTYs) to use respectively.

* Leon: I think such a statement in the threat model would be more
  far-reaching than simply saying that we have one transport
  supporting this (for instance, Console via USB). If we include it,
  we should support it for every transport including UART.


## (A)synchronous implementation of TickFS

* Amit: TickFS is implemented for a synchronous flash interface. This
  makes sense for on-chip flash, as this is practically always
  synchronous. This is not true for off-chip flash though. We should
  talk through the trade offs and decide on the approach.

* Hudson: TickFS presents a synchronous interface which is unusual in
  the context of Tock. Currently, as it's designed, it's intended only
  to be used for applications. All the logic that is required to use
  this synchronous implementation is encapsulated in the userspace
  driver which uses TickFS.

  There is cause for concern, as TickFS then falls into a different
  category of virtually every other API in Tock today.

  One particular example would be the `get_key` method in the userspace
  driver. The general flow of how it works is that
  1. the user (being an application) calls `get_key`, which returns a
     result immediately, or a "success state" indicating that the
     request couldn't be completed now
  2. the Driver is set up to receive Flash callbacks and receives an
     interrupt that the operation finished
  3. based on a state machine in the Driver, it will call into TickFS
     to complete the last operation based on the completed Flash
     operation
  4. return the result to the userspace app.

  This works fine as long as the only user of TickFS is an
  application, but makes usage in the kernel and virtualization hard.


* Alistair: The reason the design is like this, is that it should be
  possible to use the codebase outside Tock as well.

  If we implement the usual Tock callback-style interfaces, it can
  only be used in the Tock kernel itself.

* Hudson: In order to present this synchronous interface for usage
  outside Tock, there perhaps are a number of inefficiencies for usage
  inside of Tock.

* Alistair: If TickFS was made asynchronous, the design would be very
  similar, except that the state machine keeping track of requests
  would be inside TickFS itself.

* Hudson: The upside is that virtualization in the kernel would become
  much simpler.

* Alistair: Couldn't we virtualize the current capsule?

* Hudson: Yes, but that would require us to have all the logic of
  the current userspace driver inside the virtualizer.

* Phil: If you are using TickFS on on-chip flash the operations will
  often be synchronous, but not on every chip (for instance, on a chip
  with two flash banks, which blocks if code is currently executing
  from the accessed flash bank).

  It is in many cases easier and less wasteful to make a synchronous
  operation look asynchronous than the other way around. If you
  provide a synchronous API for an asynchronous operation you will
  need to spin.

* Leon: For this exact reason the DynamicDeferredCall infrastructure
  has been introduced.

* Alistair: I don't think that the current implementation differs
  significantly from an aysnchronous implementation, as we would have
  to wait for a read operation to finish until we can operate on the
  data. So we would either have to spin, wait for callbacks or employ
  deferred calls.

* Phil: The question is whether one can do anything else during that
  wait.

* Alistair: Currently that is possible, we do not stall in the current
  implementation.

* Hudson: And that works, because TickFS is not actually presenting a
  fully synchronous API. Even though it can be used as such, when
  results are queued you still receive callbacks, which the userspace
  driver uses as a substitute for spinning.

* Brad: There aren't callbacks, it's retry based. On a call, the
  file system may return that a retry is necessary.

  The callback from the flash layer to the userspace driver is a
  result of the current implementation of the flash driver. There is
  no guarantee that the software layering is constructed this way.

* Alistair: Right, but it's the way it works right now in Tock.

* Brad: I agree that the general flow would be approximately the same
  in terms of overhead, but when looking at the architecture, the
  current implementation is much more complex instead of the usual
  approach in Tock.

  It's more a software engineering question rather than a question of
  complexity. The retry approach takes some time to understand, as the
  software layering is not as clear.

* Phil: Is the reason for this design that TickFS does not put a
  wrapper around the low-level callback for the retry function?

* Alistair: It does use wrappers, however I don't think we could wrap
  it much more.

* Hudson: What makes this design weird is that TickFS does not deliver
  callbacks up, but rather that the actual user of TickFS also has to
  be directly subscribed to the callback of the lower flash
  layer. Whereas the alternative would be that TickFS received the
  flash callbacks.

* Alistair: The disadvantage being that the logic and state machines
  essentially don't change, whereas the design becomes
  Tock-specific. I did look into making it callback-based first, but
  it seems quite complex and very similar.

* Brad: Then it seems like it would be complex either way.

* Alistair: Right, but a file system implementation shouldn't be
  complex. This allows to move the complexity in part into the upper
  layers.

* Brad: Is Tock the only asynchronous OS out there? I thought this was
  a more common paradigm?

* Alistair: Flash operations on other OSes are most commonly
  synchronous, if not for special cases like multiple flash banks.

* Phil: That's true only for on-chip flash. Off-chip synchronous
  operations would be wasteful.

  In TinyOS there was a device which was implemented synchronously. It
  might be fine for a single device, but when there are multiple
  instances this causes high latencies and reduced responsivity.

  I think it's good that we are trying to find a middle ground between
  something that should be asynchronous for Tock but also work on
  other operating systems implementing flash accesses as blocking
  calls.

* Amit: As a thought experiment, if we had our own equivalent of
  crates.io, would TickFS be its own separate package or rather a
  basic component of some or all the boards that we have upstream?

  If it were published in such a registry instead (being an optional
  addition to Tock), would we be as concerned on whether it is
  specifically synchronous for on-chip flash?

  In essence, unlike `tock_registers` which is included in
  `libraries/`, but is tailor-made for and integral part of Tock,
  could TickFS be an instance of something which is not necessarily
  made for Tock, but just used by some boards?

* Alistair: TickFS would be an external library instead of an integral
  part of Tock, that is also why it currently resides in the
  `libraries/` directory.

  If someone had already written a file system matching the
  requirements (for example, not having external dependencies), we
  could probably use that instead.

* Brad: Looking at the pull request, adding the asynchronous interface
  to TickFS requires additional complexity. I think it's unlikely that
  there is a good option which is both synchronous and asynchronous.

  Also, looking at pull request more, there is a state machine both in
  TickFS and in the capsule.

* Alistair: Right, but making it fully asynchronous makes the state
  machine in TickFS itself more complex. Also, it would make
  unit-testing the file system much harder.

* Brad: But then we are unit-testing the version which we are not
  using?

* Alistair: I think most of the complexity is not in the state
  machine, but in the file system mechanics (writing to flash, making
  sure checksums match). That would be tested using unit tests. I
  think the file system ends up being well-tested because of unit
  tests.

  What if I split out some logic from the capsule into something else?

* Brad: This is essentially a question of whether we are okay to
  include an example of returning a retry on a function call as a way
  to write asynchronous code.

  If that is fine, this is a reasonable implementation. It gives us
  the same concurrency model, while being better suited to be
  unit-tested.

* Amit: It's one thing to scrutinize the way how TickFS is implemented
  because it might be incorrect, unsafe or buggy, and another sthing
  because it would not be a canonical example of building
  things. Perhaps we should separate these concerns. If we are
  concerned about the latter and TickFS is a highly optional
  extension, it could be fine.

* Phil: The challenge is that this is a library that is included in
  the main repository. We noticed that platforms maintained by the
  core group tend to be very clean, whereas other platforms (such as
  the STMs) contain an unusual amount of unsafe code. The concern is
  that someone uses this as an example to follow.

  The code should explicitly say and reason why this is not the
  canonical way to do implement asynchronous code in Tock, which could
  be sufficient.

* Alistair: I have some documentation contained in the pull request
  already, I can extend that.

* Brad: I think it is a reasonable conclusion.

* Phil: I'll be happy to take a very detailed and careful look at the
  design for correctness, but I need a very clear statement about what
  guarantees it is trying to provide.

* Brad: I'm mostly interested in the interfaces, but this is hard to
  review because the two PRs are different.

* Alistair: The second one just adds more functions.

* Phil: Can you include the API in the design PR?

* Alistair: Yes, I can add those on top.

## Tock 2.0 allow buffer swapping semantics

*There was insufficient time for an in-depth discussion, so this issue
will be brought up again in the next call*

* Leon: The new allow semantics support passing back the previously
  allowed slice to the userspace application as part of the return
  value.

  We are currently getting these values based on the AppSlice
  instances a capsule passes back into the kernel as part of its
  return value on the Driver methods. This implies that a capsule
  could swap two AppSlices and return one which was not previously
  allowed under this allow number.

  In the kernel, we can either tolerate these misbehaving capsules, or
  intervene accordingly (e.g. by blocking the capsule as it is clearly
  incorrect or panicking the kernel).

  If we want to ensure that a capsule must always pass back the
  previously allowed buffer, we must keep track of `(driver num, allow
  num, ptr, len)` tuples in the kernel, whereas otherwise we would
  only need to track `(ptr, len)` tuples to prevent app slice
  aliasing.

  With some data structures (to store currently allowed memory
  regions) this information would be inherent in the structure, while
  for others it would come with additional memory requirements.

  This is a blocking issue in the design of the allow-table right now,
  so it'd be great to reach a conclusion soon.

* Phil: Perhaps it's best to write this up as an Email, along with an
  example.

* Leon: Sure, will do!
