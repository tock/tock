# Tock Meeting Notes 02/02/24

## Attendees

- Amit Levy
- Leon Schuermann
- Branden Ghena
- Jonathan Van Why
- Andrew Imwalle
- Tyler Potyondy
- Brad Campbell
- Philip Levis
- Pat Pannuto


## Updates

- Leon: Some progress on auto PR assignment
  - Do we want this?
  - https://github.com/tock/tock/pull/3822
  - How to test? Just run it live?
- Amit: I think we want this, testing live is fine.
  - Need some expectations for what the assignee is supposed to do.
- Leon: Assignee is responsible for PR.
- Leon: Bot could automatically send stale PRs.
- Amit: We could have script for what to do with stale PRs. What do we want?
  - Just merge stale PRs? Get more comments?
- Brad: How does it work? 
- Leon: Runs on github actions. Run into github limitations if use too much.
  - Can run in dry-run mode.
- Brad: Testing live seems fine.

- Leon: Working on hardware testing.
  - Slack: #ci-hw
  - Working on tests running on RPi with boards attached. USB and then GPIO connections.
  - Test spins up netboot RPi to execute tests.
  - Start on hook with github actions.

## Process Checking PRs

- #3772
  - Adds another process checker.
  - Phil: I can take a look.
  - This doesn't need to be in the kernel crate. We could move to some other crate.
  - Could mismatch hash function.
- #3818 - AppID based on process name.
  - Just need(ed) a review.
  - In kernel crate, but doesn't really affect core kernel APIs.

## Compile on Stable

- #3803 - merged

## Cortex-M Crates

- Rename to the actual names (eg Cortex-M3 -> v7m)?.
- Brad: Not in support. Marketing names are more familiar to developers.
- Phil: Agree, chips say "Cortex-M4".
- Amit: Cortex-M define ISA + default peripherals?
  - Pat: Not sure.
- Brad: ok to have both, nice to expose the familiar names to boards
- Leon: complexity with hierarchy/subsets
- Phil: People coming to the repo looking for specific cortex-m.

## Legal TRD

- Merged.

## Documentation Working Group

- Proposal on how to handle PRs across areas for documentation
- Amit: Concretizing process for creating working groups.
  - How do people join WGs? Join DOC WG? Ex: join OT WG?
- Amit: Use WGs to help manage PRs.
  - We have PRs in a series of repos (userpsace, kernel, book, etc)
  - These PRs don't get looked at quickly enough, or get looked at but not merged
  - Suggestion: having groups with purview of specific parts of repositories or parts of repos would help
  - Amit: as example, feel like so many PRs is overwhelming
  - In contrast, if my purview was more limited it would be more scalable.
  - Also would get best people to look at PR to be more likely to
- Brad: general support
- Phil: sounds reasonable
- Brad: this proposal seems like it implies that it would grow involvement. That implies giving up control. That is somewhat significant.
  - Leon: Problem of divergence between groups / code conflicts between groups.
  - Phil: Chairs could help. Perhaps from core WG.
  - Phil: Help ensure continuity and connection
  - Phil: Filesystem divisions can help. Separate folders maintain distinctions.
  - Amit: Yes, but there will be conflict points (HILs, for example)
  - Amit: Another challenge: where does code size monitoring live? That would have to be umbrella.
- Amit: Action Item for me: write up a proposal on this
- Amit: How do people join WGs?
  - OT? Pretty open.
  - Net? Open to join.
- Amit: Someone else could lead OT?
  - Brad: OT really is both OT and RISC-V, I'm more involved with RISC-V
  - Brad: but if someone is primed to do more with OT WG, that would probably be better for Tock
- For the net WG, Alex is chair but not in core.
- Amit: WG chair two functions: administrative (ie scheduling) and technical leadership
  - More important for the technical leadership to be involved with core
- Amit: Don't want working groups to grow uncontrollably
- Leon: Name of working group is important. Shows what the purview is.
- Branden: Does every WG need to have meetings?
  - Amit: no.
  - Role of core WG evolves in the future
  - Phil: WG focus could evolve and change
- Brad: So, DOC WG?
  - Leon: clicked merge.
