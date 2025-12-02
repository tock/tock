# Tock Meeting Notes 2025-09-25

## Attendees

- Brad Campbell
- Johnathan Van Why
- Pat Pannuto
- Amit Levy
- Alexandru Radovici
- Leon Schuermann
- Branden Ghena

## Updates

- Brad: tockloader 1.15.0 released
  - Adds probe-rs support
  - Much improves the --flash-file support with `tockloader local-board`
- Brad: our paper won best paper at EWSN
  - Develops a scheduler for energy savings with multiple apps in Tock

## Shared peripherals drivers in crates/

- There are shared peripherals across boards
  - eg, uart in rpi pico and rpi
  - eg, can in stm and infineon
- Question, where to put shared peripheral code? New crate or folder?
- We have some examples of this, eg, shared peripherals for nrf5x.
- We need something a bit more general, shared code even across different chip
  manufacturers.
- virtio has to do something similar. Has shared drivers, and multiple crates to
  handle arch-specific code.
- What do we call this crate with the CAN driver? Is there a name or vendor?
  - Could be "pl011" (?)
- Having a crate named this with these drivers that are used by chips makes
  sense.
- What about a top-level directory for these?
- Brad: I think these are going to be hard to name. We already have `sifive/`
  and `nrf5x/`, we can continue that. I think it will be confusing to have some
  peripherals in one place and some in others.
- Zephyr groups by peripheral type, eg, `adc/adc_stm32.c`.
- Example in rv32 where the same peripheral were subtly different in
  implementation, adds complexity.
- Ok where to put this? in `/` or in `/chips/`?
  - Idea: `/chips/shared/`
- General consensus on `/chips/shared/`.
  - Then crates inside that.

### Shared peripherals with different full implementations

- Some chips share the same IP block, but not all features. How to handle that?
- Could use features. If there is clear hardware distinction between versions
- Could use const generics. If that makes it easier to test and reason about the
  state machine.

## wg-crypto

- Add a member?
- Working groups decide their membership. Nothing needs to be decided now.
- wg-crypto will need to update the crypto stack in Tock.

## https://github.com/tock/tock/pull/4602

- Certain traits need their implementations to be correct for kernel soundness.
  Those `trait`s should be marked unsafe. Implementations must guarantee
  their correctness.
- Brad: I find it difficult to then have a direct counter example in the same
  file.
- The constructor is unsafe.
- But that should mean the actual description of why MPU is `unsafe` is more
  complicated. It actually is OK to violate this restriction if there is an
  unsafe constructor, which weakens the guarantee.
- What about making the constructor named something like
  `new_unsound_not_for_production()`?
- Or, put it in a new file that isn't available by default.
- JVW: We discussed at one point having the MPU methods be unsafe.
  - Probably complementary to this.
- Brad: I think we would benefit from being able to disentangle our uses of
  `unsafe` as much as possible. Help those new to Tock (and ourselves) to
  immediately understand _why_ something is unsafe.
- Leon: I can work on a way to not have the NoMPU be in such direct conflict
  with the unsafe trait.
