# Tock Core Notes 2023-05-12

Attending:
- Hudson Ayers
- Brad Campbell
- Branden Ghena
- Alyssa Haroldsen
- Pat Pannuto
- Tyler Potyondy
- Alexandru Radovici
- Leon Schuermann
- Johnathan Van Why

## Updates:

- Alex: Ethernet -- got RX/TX working on the STM board. Will progress to HILs
  from there.

  - Leon: Now have 3 Ethernet MACs supported, good basis to progress towards HIL
    design from here. Tried to continue port on NXP chip, can't receive yet.

- Tyler: Getting up to speed with the UDP stack. Made some good progress on
  getting Thread to work. Managed to join an open Thread network, but facing
  some issues w.r.t. to IPv6 addressing. Would appreciate talking to someone
  with experience with the IPv6 stack.

  - Hudson: Touched the addressing part a few years back.

  - Tyler: Issue relates to requests within the larger subnet of addresses in
    Thread. Not a specific IP.

  - Hudson: Addressing portion is very bare-bones. Not any discovery mechanisms
    implemented, etc. Seems like Leon is going to run into this soon with his
    Ethernet-based stack.

  - Leon: Looked into the existing stack, quite limiting and purpose-built for
    6LoWPAN stack. Maybe need to consolidate and redesign it.

  - Brad: Is there a tracking issue?

  - Leon: Concerning Ethernet and higher-level layers, there's an Ethernet
    tracking issue.

  - Hudson: There is a very old 6LoWPAN tracking issue. Should be updated.

## Merge the External Dependencies doc (#3112)

- Hudson: Brad wanted to merge the external dependencies document. Has approval
  from Leon, me, implicitly from Brad. Last-call label attached. Merging.

## libtock-rs API PRs

- Hudson: Alex requested looking at a few libtock-rs PRs.

  *List posted in chat, examples:*
  - [NineDOF API](https://github.com/tock/libtock-rs/pull/468)
  - [Sound Pressure API](https://github.com/tock/libtock-rs/pull/469)
  - [Buzzer API](https://github.com/tock/libtock-rs/pull/470)

- Branden: What's the approval policy again?

- Hudson: Believe it's one week and Johnathan's approval for significant PRs.

- Johnathan: These are not significant though. One approval, and usually try to
  keep them open for at least one workday.

## TockWorld 6 Planning

- Brad: sent around link / template page. Decided on starting Wednesday, July
  26th. Plan was to do a day-long tutorial on Tock. Alex, you've done some
  workshops recently. Can you give some insights?

- Alex: latest workshop we held was at a Rust workshop. Was a 2-day event. We
  tried to show students how to contribute to Tock.

  In London, we had 2 hours for our workshop. We designed a small application
  where attendees were asked to write a driver. Things were displayed on a
  screen, where that infrastructure was written by us is well. If we have a day,
  this could be done step-by-step.

  We need to choose the right hardware though.

- Brad: Goal is to get a description together quickly, such that we can invite
  people. This way we give people enough time to plan ahead.

  Which hardware should we use? What topics are interesting?

- Leon: Hardware availability -- anyone have experience? STMs are impossible to
  get right now.

- Alex: Can confirm. RP2040 devices + MicroBit are available. However, Raspberry
  Pi has issues with Windows and AMD processors. Could use help debugging those
  issues.

- Hudson: Does it work on Windows when using a Linux VM?

- Alex: Likely, although not with an AMD processor. It works on most bare-metal
  Linux machines.

  We can also use two RP2040s, where one programs the other.

  Could get a "Pico Explorer base" board. Has a screen, some buttons, a small
  breadboard; educational board.

  MicroBit doesn't have a screen, difficult to attach peripherals. Good for
  kids.

  Adafruit Clue boards work very well.

- Branden: Brad and I have some MicroBits, as long as we're not giving them away
  we should be fine.

- Leon: What about nRF devboards? Slightly more expensive, but they have a
  full-fledged JLink on there, which is nice.

- Branden: Teach a class with them. Have 40.

- Alex: MicroBits are available on Mouser, can dispatch immediately.

- Hudson: Disadvantage of MicroBit?

- Brad: Marketed towards kids, looks like a toy. We have industry professionals
  come in.

- Branden: But it does have sensors and outputs on there, which the nRF board
  does not (just 4 LEDs).

- Alex: On the Adafruit Clue, the Tockloader / Tock Bootloader integration works
  very well.

- Hudson: When we use a board with a screen, it seems like we're emphasizing
  that particular subsystem of Tock. If our support isn't great (e.g., updating
  the screen is slow) then that might give bad impressions. Haven't actually
  used Tock's screen support.

- Alex: We used an STM Discovery board for our first workshop. Looks
  professional, but flash page size is an issue, Tock bootloader doesn't work
  with it.

- Leon: Setting up the STLink with openocd is a pain. JLink or Tockloader
  support would be much nicer.

- Alex: May be able to flash this board through a browser.

- Hudson: Even if we can do that, Tockloader is nice and we should try to use
  it.

- Hudson: Summarize. We want a board which
  - we could get enough of
  - should be supported by Tockloader
  - maybe(?) has a screen, which could be used to run the existing tutorial
    applications.

- Brad: Maybe not just have one board. If we keep the MCU the same, we may be
  able to use different boards.

- Leon: Many of the interesting tutorial applications do depend on the partiular
  inputs and outputs available on a given board, though.

- Brad: Maybe first think about the types of sessions we'd like to have, then
  evaluate which board works best.

- Pat: Demonstrate Tock as a secure operating system, e.g., code signing?
  Loading applications?

- Brad: Surprising to me is that most of our contributions over time don't seem
  to be capsules. From a "what's the most utility" perspective, is capsules
  actually right? Is doing a tutorial on the core kernel / chip drivers too
  complicated?

- Branden: We could be having people reimplement chip drivers, demonstrate MMIO,
  etc.

- Leon: When talking about the core kernel, there's a risk of this becoming a
  lecture instead of a tutorial. Also, talking about chips / drivers, those are
  not the most consistent, some of them do inherently unsound DMA copies, not
  particularly well structured right now. Might not want to highlight that.

- Alex: What about a practical project? E.g., coffee machine running on Tock?

- Pat: Think about differentiating factors of the target audience. E.g.,
  demonstrate isolation by trying to break one app from another. Have it be
  open-ended, where people can get creative.

- Brad: Second that. Maybe introduce mechanisms such as signing apps, syscall
  filtering, etc. Need to get those mechanisms supported on the nRF platform
  then, though.

  In theory, we can also write up some sensors to the nRF boards.

- Leon: May be able to solder up some Arduino shields beforehand.

- Pat: Good idea by Hudson - two types of registration: one with the board
  included, one where we provide boards for the workshop.

- Leon: When we're trying to break applications / the security model, and we're
  using nRFs anyways: we could be running OpenSK, and people could take that
  home and play around with it.

- Pat: Other applications could be something like an occupancy sensor. A third
  party app could try to forge that count.

## I2C Implementation Generics (#3431)

- Alex: SPI used generic, I2C used trait objects. Generics allow more
  gateway-optimizations such as function elimination, but code size is not
  affected much. Disadvantage: type signatures look more complex.

  Also, now we have an additional type, because we need to have a type to plug
  into to generic for I2C implementations which don't support an SMBus.

- Leon: In the past we've almost always decided that generic are preferable, so
  think this is a good change.

- Branden: Are we worried about the churn of this change?

- Leon: Here we have actual benefits of making the change.

- Brad: Also brings this HIL in line with the others.
