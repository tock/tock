# Tock Meeting Notes 03/29/2024

## Attendees

- Amit Levy
- Andrew Imwalle
- Brad Campbell
- Leon Schuermann
- Johnathan Van Why
- Pat Pannuto
- Tyler Potyondy
- Hudson Ayers
- Alexandru Radovici
- Alyssa Haroldsen
- Vishwajith
- Branden Ghena


## Updates

- Amit: Code size analysis. At a completion point. Writing up results now.
  Describing where code size is going, and some recommendations. Initial doc in
  progress.
- Some results: TBF header parsing not so bad compared to other features. GPIO
  init is tricky. Some small but surprising things.

- Tyler: OpenThread: libtock-c PR open. 6LoWPAN bug and alarm are two remaining
  issues. 6LoWPAN bug addressed by trait updates (in progress).
- 7 minute timer issue. nRF can handle a prescalar. Need more comments on this.
- PRs to look at: https://github.com/tock/tock/pull/3933,
  https://github.com/tock/tock/pull/3940

- Alex: Packet buffers with [u8].
- Leon: Migrating console API down to UART to new interface. Adding room for
  headers and footers in buffers. Useful for protocols. Right now just a byte
  slice interface.
- Brad: This didn't work for subslice.
- Leon: Many design iterations, now have a design with is a superset of
  subslice.
- Alex: Right now use 12 bytes overhead. Once `const` instructions in Rust is
  stable this overhead goes away.
- Alyssa: Will likely be a while until that is stabilized. I recommend nesting
  in another type with a fixed footer.
- Leon: Current implementation does not need nightly features.

- Hudson: Rebased https://github.com/tock/tock/pull/3934. Need libtock-c driver.
  Interface with debug!() macros? Only works on cortex-m right now.
- Brad: Is there a riscv equivalent?
- Hudson, et al.: Unsure.

## Bar for Capsule PRs.

- https://github.com/tock/tock/pull/3881 is for userspace support for NMEA
  devices.
- Issue arising: "is this the right interface?" or "is this interface and its
  capabilities appropriate for upstream Tock".
- How do we determine if something is suitable for upstream? When do we say no?
- Brad: I2C some open questions. NMEA: is this suitable?
- NMEA: unclear
- Hudson: could this be specific to one chip?
- We don't know the exact chip.
- Downside: others would come around and think they could use something, and it
  doesn't quite work right. Some design decisions which weren't great get
  copied.
- Our bar should (basically has to) be lower for brand-new
  support/drivers/chips.
- NMEA support is good.
- Part of the logic is outsourced to userspace. Difficult to reason about the
  driver.
- NMEA model is like UDP. Have to wait until data is available. Odd fit with
  I2C.
- What is the process for handling this? Hard to just say "no". Want to avoid
  randomness in the response.
- Can write down what we want or what the expectations are. Hard to draw the
  line when the metrics are more subjective.
- Just might not want to include things in upstream.
- Can we cite previous examples?


**ACTION ITEM**: Amit: propose a plan for normative documentation.

## Timer and Prescalar

- OpenThread needs a global time clock. On nRF wraps at 512 seconds.
- https://github.com/tock/tock/issues/3938 to change the prescalar.
- `time` HIL has overflow functionality. Could notify userspace on overflow and
  keep track of wrap arounds for current time.
- Not sure even what the right timer to use is.
- Might be better to use a timer with higher resolution.
- Can't tie a networking stack to a specific timer.
- Might not want to wake up every 7 minutes in any case.
- Could ensure that the counter provided to userspace always wraps at 32 bits.
- Could shift left 8 bits to make a 24 bit counter look like a 32 bit counter.
- Brad: what is the simplest approach that really solves this problem?
- Always have a 32 bit counter. Userspace sets a timer to check more frequently
  than the timer wraps around.

## Libtock-c rewrite

- https://github.com/tock/libtock-c/pull/370
- 10 left + apps
- Assign to Amit
