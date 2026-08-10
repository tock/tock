# Tock Meeting Notes 2026-07-22

## Attendees
- Brad Campbell
- Hudson Ayers
- Johnathan Van Why
- Pat Pannuto

## Updates

Updates on 64-bit RISC-V and tock-registers progress were shared as part of
their agenda items. No other updates.

## 64-bit RISC-V
- Brad: The fact the PMP PR doesn't pass CI is a big deal. I asked Claude and
  there's a PR open with what Claude suggested, but I don't have interest in
  learning how that works. I feel like we either just go with it or wait for
  Leon. After a major round of feedback from Pat and Amit, everything is back in
  the place they were in last week. They work, though I haven't tested it
  because there's so many PRs. We just need one PR later to make it compatible
  with the TRD, whatever that settles out to be.
- Pat: Do they just need one last round of review and click the merge button,
  or?
- Brad: Yeah, then it's just "what do we do with this PMP"?
- Pat: We can poke Leon offline and get everything else merged, so the only
  thing blocking by next week is the PMP.
- Brad: One other option is to go back to an earlier version of the PMP PR.
  Unfortunately the commits are squished, so we lost history, but I did find an
  older commit.
- Pat: I'm happy to let it ride for a week-ish and hopefully it will sort itself
  in the next week. We can do a more dramatic option next week.
- Brad: One other thing, unfortunately Amit isn't here. We have two files —
  `clic` and `machine_timer` — which are in the rv32i crate not the RISC-V
  crate. To use these on 64-bit, Leon copied them verbatim into the rv64 crate.
  I don't know why they work as-is, and I don't know why Leon copied it. Amit
  raised the issue on the PR and I don't know what to say.
- Pat: I'll send a message asking Leon to look at the PMP, and Amit and Leon to
  look at the CLIC and machine duplicated files.

## tock-registers
- Johnathan: All but one of the PRs from the original design have been merged.
  During review, came up with changes that I'd like to implement before we use
  it. The only one that is concerning is I need to figure out fake peripheral
  ownership for unit tests. That is duplicating what Brad's currently working on
  with `once_init`, and interacts with safe DMA API design. Hoping to make
  progress in the next week, ideally solve in two weeks. Then there's a handful
  of cleanups. Everything tracked in [tock-registers issue
  45](https://github.com/tock/tock-registers/issues/45). There is one thing that
  would be nice for someone else to do: `register_bitfields!` doesn't generate
  doc comments. I'm not very familiar with that code so it would be better for
  someone else to implement that. Tracked in [tock-registers issue
  8](https://github.com/tock/tock-registers/issues/8).
- Pat: I have been working through the backlog of things that have happened in
  tock-registers since I left. I have 24 notifications left. Hopefully I'll
  catch up soon. I can think about doc comments for register_bitfields.
  Otherwise my impression is things are moving and you're blocked on time to do
  things.
- Johnathan: Yeah, at this point it's blocked on me figuring out ownership.

## Cortex-M 2024 PR
- Brad: We don't have Brandon, but Pat approved the Cortex-M 2024 PR. It'd be
  nice to decide on that. We may be close to doing a full Nordic board in 2024,
  I'm curious what is left.
- Pat: I'm comfortable hitting merge on it. Brandon approved, then I added
  concerns, which we've since approved. None of the changes since affect
  Brandon's concerns. At the end of the day it's a refactor that's cleaning
  things up, so it's relatively safe as they go.
- Brad: It also puts us in a good place to think about our unsafe blocks and
  safety invariants.

## STM32 U5 peripheral driver PRs
- Brad: Large burst of people working on these. Some amount of duplication, some
  serialization required to merge them. But they seem to be good overall.
- Pat: Examples: [4934](https://github.com/tock/tock/pull/4934)
  [4942](https://github.com/tock/tock/pull/4942)
  [4950](https://github.com/tock/tock/pull/4950)
  [4951](https://github.com/tock/tock/pull/4951)
  [4885](https://github.com/tock/tock/pull/4885)
