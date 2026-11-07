# Tock Core Call Notes 2026-10-28

Attending:
- Branden Ghena
- Amit Levy
- Alexandru Radovici
- Leon Schuermann
- Johnathan Van Why

## Updates

### `register_structs!` Migration

- Amit: I have opened 7 or so PRs to migrate bare register struct definitions to
  the soon to be deprecated `register_structs!` macro. Using a tool to do this
  deterministically; don't expect there to be errors. Done, except for
  reviewing. Would put us a lot closer to new and sound register map.

- Johnathan: Were we planning to review the code of the tool you made? Or some
  of its PRs in detail?

- Leon: I wrote a script to compare binary hashes before and after, posted to
  one of the PRs. I think it's good if the hashes are the same (modulo line
  number changes).

- Amit: We did not decide this last time. We might want to review the PR of the
  tool.

- Johnathan: Let's discuss after discussing the release. Tock registers is
  currently blocked on the same people reviewing these PRs.

### Network WG Updates

- Leon: Branden will have students working on IPC. We talked about my Flash HIL
  PR, maybe that should be under the purview of the network WG.

### Process Flash Padding

- Amit: Have been working on drastically reducing the amount of padding we're
  adding on ARMv7-M platforms. We can probably get down to only pad 25% of the
  TBF flash size with one MPU region, and with two or three regions we can get
  down to just 2 or 3%.

  Writing down the constraints of the MPU and a more optimal allocator in a TRD.

  Requires some cross-cutting changes to the kernel.

- Leon: How does the MPU behave with adjacent regions? The RISC-V PMP is weird,
  an access that spreads across two PMP regions might fail even if both permit
  the access otherwise.

- Amit: No. If two regions don't overlap but are adjacent to each other, it's
  clear. Otherwise higher priority wins.

- Leon: What about an unaligned load across two regions.

- Amit: Good question.

- Branden: Is there a way that a board could choose to not use more MPU regions?

- Amit: Yes.

  Right now we expect the TBF and its length to include any necessary padding.
  Which means that a compiled TBF needs to pre-determine how much padding is
  there and then the kernel either can or cannot cover that.

  Either you can cover the processes' flash without reaching into the next
  processes' region, or you can't and you need two regions.

  This can be entirely in the loader, the TBF doesn't need to be specified.

- Branden: don't save MPU regions because you think someone else might want to
  use them.

- Amit: for IPC, we either have a mechanism that allows for arbitrary sharing,
  or for exactly N (where N is small) regions.

- Amit: ARMv7-M is one category, but within that category, Cortex-M7 have 16
  regions, not 8.

## tock-registers 0.11 and 0.12 releases

- Johnathan: key part blocked on Pat, can't resolve all questions today. We're
  getting close to a release. Remaining blocker are thoughts that Pat has that
  he wants to change. There's minor fixup PRs:

  - Documentation fix: https://github.com/tock/tock-registers/pull/83
  - Cargo.toml fixups: https://github.com/tock/tock-registers/pull/84
  - #[must_use] fixups: https://github.com/tock/tock-registers/pull/85

  Blog post PR could use a second review:
  https://github.com/tock/tock-www/pull/142

  When you call `register_bitfields!` it generates zero doc-comments. I don't
  know if I'm the best person to document this. Marked as shouldn't block the
  release.

- Leon: I don't think this should block 0.11 or 0.12. This seems unrelated. We
  can decide whether this is something we should address before 2.0.

## Migrating Tock code to `register_map`

- Johnathan: do we want to mass-migrate code to `register_map`, or do it by
  hand?

- Amit: I'm in favor of mass-migration.

- Johnathan: Amit has seen good success with getting an LLM to write a tool, so
  that seems promising. This issue seems more tricky, because it doesn't just
  changes the register definitions, it also needs to change the code that uses
  those registers.

- Amit: Once you've changed the register map macro, is changing the driver code
  mechanical, or at least easy to review? Maybe the translation is
  semi-manually: `register_map` migration is done with a tool, and then the code
  migration might be done by a human or an LLM. But in that case, it needs to be
  easy to review.

- Johnathan: Things that were previously field references become function calls.
  This requires throwing in parentheses, etc.

- Leon: If the tool misses some things, how likely is it for that to result in a
  compiler error, or for that error to go unnoticed.

- Johnathan: Would think compiler error.

- Branden: The value of this migration is so high, even if we break one chip,
  that's still a good tradeoff.
