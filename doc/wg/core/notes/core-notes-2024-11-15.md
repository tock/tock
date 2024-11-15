# Tock Meeting Notes 2024-11-15

## Attendees
- Branden Ghena
- Brad Campbell
- Hudson Ayers
- Amit Levy
- Johnathan Van Why
- Ben Prevor
- Leon Schuermann
- Alex Radovici
- Kat Watson
- Pat Pannuto


## Updates

- Amit: First of several CHERI PRs ready to be merged.
  - Final call on #4174

## TockWorld 2025

- Planning to host it at Microsoft
  - Need to choose dates and spaces
- TW 2024 - three days
  - Developer-focused day
  - Community-focused day (talks)
  - Tutorial day (two tutorials)
- Brad: developer day last time was perhaps less effective than in the past
  due to larger audience. Might be nice to try to separate that more so we can
  have more in the weeds discussions.
- Amit: We could do a developer day at a different venue (UW or library, etc.).
  That seemed to work at RustNL.
- Branden: Could advertise third day differently. TW is a two day event.
  - Does the order matter?
- Branden: conference day in 2024 was great.
- Leon: tutorial day with so-so results. Topics may not have best matched
  audience.
- Amit: should we improve the tutorial or not do it?
- Leon: perhaps we could ask what people are interested in. Or have more of a
  choose-your-own-adventure path. Maybe more of a hack day.
- Amit: two days (one day conference, one day tutorial/hacking). Separate day
  for developer in-the-weeds talks.
  - Useful to do in July or later on
- Maybe end of August or beginning of September. Or early-ish July.
- Amit to send survey.

## x86 Upstreaming

- Blocked on how to handle the external dependency. Ready to do either vendored
  dependency or rewrite. But only want to do one.
- Some other work to be done, but main thing is the dependency.
- Mostly just need enum definitions. Would vendoring be that different from
  re-writing?
  - Rewrite might have different/new functionality and might use tock-registers.
  - Vendoring might need to pull in dependencies of dependencies.
- Did we ask if they could re-license?
  - We did ask, got some positive feedback, but would need more people to
    respond.
- Seems ok to rewrite, but hard to know what the code would look like.
- Vendoring seems like there could be some complexity, tree of dependencies,
  license issues.
- Consensus is to go with the rewrite approach.

## `usize` in Syscall ABI

- Brought up by recent `CapabilityPtr` pull request. Historically Tock syscall
  ABI has been 32 bit focused.
- Questions?
  - To what extent do we want support non-fixed-width values in syscall ABI?
  - If we do that in some capacity, how does that look in the kernel?
- Idea: we should avoid defining the syscall ABI in terms of Rust types like
  `usize`. We should use more general concepts. `usize` historically makes
  sense, but poorly fits with the complexity of hardware.
- Idea: base pointers and lengths are machine-sized.
- Options:
  1. Support 64-bit only capsules (not desired)
  2. Downstream users can support 64-bit only
  3. Tock does not support 64-bit only
- We do support 64 bit values in syscall, but they would be implemented
  differently on 32-bit platforms and 64-bit platforms.
- Why does length need to be "usize" like, why not just u32?
  - Some embedded systems might sit close to large amounts of memory and want to
    be able to do something with lots of memory.
- Implementation details on how Tock maps types to architecture-specific values
  - Tock types like u32, address, pointer, etc.
    - Map to specific rust types on specific architectures.
- Idea about using unions/enums
- Summary: generally agreement that the ABI _can_ talk about architecture-width
  values.
- Naming all of these are hard (e.g. Rust got it wrong).
  - We should use concrete definitions, either from Rust or C or hardware
    itself, rather than define our own types.






