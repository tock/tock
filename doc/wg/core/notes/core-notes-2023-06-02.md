# Tock Core Notes 2023-06-02

Attending:
- Hudson Ayers
- Brad Campbell
- Alyssa Haroldsen
- Amit Levy
- Pat Pannuto
- Tyler Potyondy
- Leon Schuermann
- Johnathan Van Why

## Updates

### TockWorld 6 Tutorial

- Brad: have a registration link for the Tock tutorial, will link that
  from our page on the tutorial.

### New Memory Model Implemented in Miri

- Alyssa: New memory model has been integrated into Miri. It's "tree
  borrows" instead of stacked borrows. It appears to have been
  endorsed by the original author of stacked borrows.

  Still investigating whether it solves our read-only process slice
  problem (as what we're doing there is technically unsound).

- Johnathan: Ralf Jung's latest blog post explicitly calls that out as
  allowed now.

- Amit: High-level overview of the problem?

- Alyssa: Read-only process slice contains `Cell` internally, and is
  primarily used as a reference-only type. You're supposed to only use
  it to read process memory that is not supposed to be changed, and
  the kernel cannot legally write to. But it shouldn't be undefined
  behavior if C were to mutate it, or from within Rust using `unsafe`.

  However, in stacked borrows, converting a `&u8` into a `&Cell<u8>`
  is considered an invalid operation, as doing so asserts that you
  safely have write access to that memory. The idea in our case is
  that we don't expose APIs to mutate it externally, so we should be
  able to perform this cast.

  This fixes that, as it was not a mutation that caused these issues,
  but really the `transmute`.

- Amit: This is a proposal for a new aliasing model?

- Alyssa: Always been proposals, there has never been an official Rust
  memory model.

  Just for better understanding of what's needed in the language, and
  to have a tool which can verify correctness dynamically.

- Amit: Assuming that this adopted, we wouldn't change Tock code?

- Alyssa: Big part is the practicality to run Miri. We can use Miri to
  verify that we're not violating the memory model. Making sure
  everyone understands the same rules.

- Hudson: There was agreement that there are some set of things which
  should be deemed safe in Rust, but were not allowed under the
  stacked borrows model. As a result, Miri would throw errors. And
  because Miri is the best tool we have to check whether code is
  unsound, it was treated as a specification.

- Leon: This is very exciting. We had a solution for this (using a
  slice of units) that would probably have been sound under the
  stacked borrows model. However, I was hesitant to push this over the
  finish line, as it would have added a lot of new unsafe code into
  the process slice infrastructure. Keeing the current code and
  waiting for tree borrows to stabilize is much nicer.

- Alyssa: References are also much more ergonomic.

### Thread Networking

- Tyler: Made some progress on Thread networking. Design decisions
  with help from Hudson and Pat. Initially was planning on having the
  Thread capsule be implemented on top of UDP; Hudson and Pat thought
  it may be better to use the UDP mux which already exists and have it
  sit on the same layer as UDP itself.

  Working on decrypting Thread messages. Should hopefully have some
  PRs ready soon.

- Amit: Where is this implementation going to exist? Capsules,
  userspace?

- Tyler: Current plan, capsules/extra, along the network
  stack. Similar to UDP.

### (Informal) Networking Working Group

- Leon: Seems to be growing interest in Networking (6LoWPAN, Ethernet,
  Wi-Fi) on Tock. Maybe it would make sense to have an (informal)
  networking working group? That could help reason about interfaces &
  consolidate efforts.

- Amit: Sounds like a good idea. Could unify around some interrelated
  common goal, such as a gateway device.

  Could also think about single userspace interface vs. different
  ones, etc.

- Leon: Exactly. Reason for bringing this up is a WIP design on how to
  handle (allocate, pass around) network packets in the
  kernel. Currently still in an early stage with crazy type
  signatures, but might be good to tune people in and get feedback.

  In other news, with the work by Alexandru's student (Ioan Cristian),
  we now have a physical board, FPGAs and simulators/emulators running
  a userspace network stack with an HTTP server. Maybe interesting for
  app updates?

### External Flash on nRF52840DK

- Brad: For using the nRF52840DK, if we want to use it for the
  tutorial, we may want to use the board's external flash to transfer
  data from a participant's laptop to the board (as opposed to
  compiling it into the kernel). However, the Nordic tools don't seem
  to be able to read the flash correctly. Chip is very
  complicated. WIP.

## Tock Matrix Server and Slack Bridge

- Amit: We want to deploy a Tock home-server on Matrix. This will be
  an additional option for people to join the Tock chat community. It
  would be bridged to Slack, so folks on Slack won't be affected.

- Hudson: People would still be able to join Slack as usual, but now
  they could also join via Matrix? And we'll announce this everywhere
  we currently direct people to Slack?

- Amit: Correct.

- Hudson: Are there concerns that we're violating Slack's TOS?

- Amit: I have no such concern.

- Hudson: One issue we had with another user using a Matrix
  integration is that replies in a thread wouldn't generate a
  notification. Maybe that's fixed?

- Amit: Not a thing anymore. Leon and I use Matrix internally. It's
  very seamless for everybody involved.

- Leon: Full disclosure: DM's don't work, but both parties will
  receive a message stating that this is unsupported.

- Amit: Could potentially solve that with puppetting, but yes.

- Hudson: Does this mean that there's going to be two instances of
  people? Amit and Matrix-Amit?

- Amit: If somebody has accounts on both, there might be two
  accounts. They can be bound together. But in practice, people will
  be just using one or the other.

## Open Pull Requests

- Brad: Long-standing issue of removing `'static` lifetimes from HILs
  for clients. Can't get the `sam4l`'s SPI & I2C drivers to compile
  with that. Maybe people can have a look at that? PR #3460.
