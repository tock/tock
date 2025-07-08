# Tock Core Notes 2023-08-18

## Attendees
- Amit Levy
- Brad Campbell
- Alyssa Haroldson
- Alistair Francis
- Chris Frantz
- Leon Schuermann
- Pat Pannuto
- Johnathan Van Why
- Hudson Ayers

## Updates
- Amit: Ethernet over USB implemented and working on nRF52.
- Brad: Expanding clippy lint allows. Clarify which clippy lints we need to
  allow for clippy to pass.
  - Will be PRs addressing some. Feedback will be helpful.
  - Chris: Which group is not a good fit?
  - "restriction"
- Pat: Tyler opened PR for software acks for nRF52. Next up is support for Tock
  as sleepy end device for Thread.
  - Hudson: will send imix, but


## PMP Redesign
- https://github.com/tock/tock/pull/3597
- Two implementations: PMP and ePMP (OT). Some issues and duplications.
- Goal: separate implementation for pmp logic from specific impl of tock process
  regions.
  - Simple PMP implementation
  - OT/earlgrey implementation
- Alyssa: should we get a review from Vadim? A: yes.
- Current implementation not matching hardware as riscv has evolved. Boot stages
  can affect how the PMP is configured.
- Downside
  - No longer a single implementation that can be used on any chip.
  - On complex chips, have to implement bottom half of ePMP yourself.
  - ePMP moved to chip folder, must be duplicated for each custom chip.
    - But we currently only have 1 (OT) which is very unique
    - There may be a generic ePMP implementation for future chips. ePMP is
      standard.
    - earlgrey version of ePMP: board or chip? all earlgrey will use the same
      bootrom that sets up regions.
- New modular design sets us up to support ePMP in the future.
- Amit: need to move low level conversation to issue or separate discussion.
- Brad: let's implement ePMP when we have an ePMP chip. This seems like good
  progress.
- OT essentially customizes the ePMP by setting up locked regions.
  - How much does this configuration complicate the implementation?
  - Difficult to do and ensure correctness.
- Open tracking issue.
- Move to OT call.


## Maintaining 3.0 Changes
- https://github.com/tock/tock/pull/3622
- What is our version policy?
- How do we maintain changes for major version changes?
- Action: read policy.


## Command 0
- https://github.com/tock/tock/issues/3375
- TRD104 specifies that command 0 returns Success if driver exists. A couple
  stabilized drivers return Success(u32) with the number of available resources.
- What to do?
  - Change to match TRD104: breaking change.
    - Makes the simple check clear and easy to do.
    - Can implement the check in the kernel.
  - Update TRD104.
    - Makes sense to get the same functionality plus more.
    - Seems more elegant.
- Advantage to changing TRD104: no code needs to change. Stabilized drivers stay
  the same.
- Brad: what is the motivation to change TRD104? Are there new use cases?
- TRD104 written after drivers.
- Command 0 is a safe check to determine if a driver is present.


