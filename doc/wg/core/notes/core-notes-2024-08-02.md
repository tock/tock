# Tock Meeting Notes 2024-08-02

## Attendees
- Alexandru Radovici
- Alyssa Haroldsen
- Amit Levy
- Benjamin Prevor
- Brad Campbell
- Leon Schuermann
- Max Cura
- Tyler Potyondy

## Updates

- Leon: Updating treadmill to make it more maintainable. Close to working on
  real hardware.
  - Goal: next few days be able to introduce the system for use.

## OpenTitan Working Group

- Brad has been the chair. He has no direct connection to OT.
- There seems to be some background activity that will lead to PRs.
  - A chip exists.
  - Firmware is happening behind the scenes with plans to upstream.
- What to do with the WG?
  - Could go dormant in the meantime.
  - Could try to create WG around the external contributors.
- Brad: focus hasn't been on OT for a long time.
  - Many people have stopped participating in the meetings.
- Leon: Discussions were good, but not necessarily OT-focused.
- Meeting times unpredictable and inconvenient.
- Would a general technical call be more useful than an OT-specific call.
- An OT group could decide on OT PRs independently of whether there is a call.
- Is it even viable to bring people into the working group? How likely is that
  to happen?
  - Group working on an open implementation wants it to live upstream.
  - Unclear how fragmented the development is.
- Is it ok if the WG is primarily from one organization?
- Maybe we should wait until PRs start happening?
  - If the WG did a first-pass review on PRs that could help a lot (but don't
    actually merge).
- We could announce the status of WG and try to build it out once things ramp
  up.
  - Hard to get things rolling.
- What is the value proposition for potential WG members?
- There is clear benefit to the project for understanding the needs of OT and
  how that can match with Tock.
- Suggestion: keep reaching out and see if things can happen, but otherwise wait
  until there is concrete work to be done (i.e. reviewing PRs).
- TODO: Housekeeping on the WG document.

## Storage Permissions Implementation (#4031)

- Updates: Removed `Option` from the kernel implementation. Update TRD to match
  implementation.
- Marked last call.
