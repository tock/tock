# Core Working Group Meeting, May 13th 2022

## Attendees
 * Branden Ghena
 * Leon Schuermann
 * Phil Levis
 * Hudson Ayers
 * Alexandru Radovici
 * Johnathan Van Why
 * Brad Campbell
 * Alyssa Haroldsen
 * Jett Rink
 * Pat Pannuto
 * Vadmim Sukhomlinov


## Updates

- None

## TockWorld 5

- July 19-20 (Tue & Wed) rising to the top of good dates.
- Deciding on a location.
  - UVA
  - UCSD
  - NW <- winner
- Ranked choice vote.
- Invite list:
  - Cycle group.

## PR 3041

- Two versions of digest trait: one `mut`, one not `mut`.
- Useful to support non `mut` for data stored in flash.
- PR changes `LeasableBuffer` to `LeasableMutableBuffer` for mutable data. Adds
  `LeasableBuffer` for immutable data.
- Decision: expose mut/imut in interface, or handle internally?
  - Current idea is better to have this exposed in API for error handling.
- First attempt: wrap in `MutImutBuffer`. What happens if start with imut, but
  get mut data back? Can't do anything, basically have to panic.
- Need to separate mut and imut to match Rust's ownership model and asynchronous
  programming.
  - If you have a mut buffer, and pass it to imut interface, you lose the
    ability to ever mutate that buffer again.
  - Otherwise you need to do a copy to imut buffers.
- Why not combine mut and imut functions in same trait? Why two traits? Is there
  ever a case where you wouldn't be able to support both?
  - Need separate callbacks for sure. Could separate callback trait.

- Handle asynchronicity with suspend points in the kernel.
  - Would require multiple kernel stacks.
  - Async scheduler managing multiple kernel stacks.
  - Complexity and code size issues

- Users of Digest HIL want to be able use imut data
  - Also, nice to not have to implement for both imut and mut buffers.
- Should other HILs also use this method?
  - Not many HILs need to use large data.
- Reality is this solution is not great, but not clear there is a better
  solution.
- Need to work on PR for potential alternatives.

