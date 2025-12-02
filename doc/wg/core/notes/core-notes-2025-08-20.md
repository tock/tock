# Tock Meeting Notes 2025-08-20

## Attendees
 - Brad Campbell
 - Vishwajith Govinda Rajan
 - Hudson Ayers
 - Johnathan Van Why
 - Pat Pannuto


## SingleThreadValue

- Brad: JVW, does it look good?
- JVW: I looked at it and it is sound. I trust that it is useful for Tock.
- Brad: I worked on a port of DeferredCall to STV. Looks reasonable.

## TockWorld Europe

- Lots of cool projects from Alex's team.
  - System tracing, RPi4 port, async support, etc.
- MMU TRD planned
- Hopefully there is a path to supporting the embedded rust ecosystem in Tock in
  a beneficial way.

## QEMU / VirtIO

- Working, very helpful for debugging.
- Would be good to get this in.

## VGA

- https://github.com/tock/tock/pull/4546
- Much less unsafe which is good!
- Still has the static_init, but that is a chip, problem, not a PR version.

## https://github.com/tock/libtock-c/pull/547

- Just need to include the libtock header from the .c file, not .h.

## Kernel Version Issue

- Right now, libtock-c apps require kernel 2.2, but even a slightly older kernel
  is (incorrectly) marked as 2.1.
- What to do?
- Hopefully not too big of an issue.
- Definitely fix for next release.
- Add check in tockloader.
- Do 2.2.1 release.

## https://github.com/tock/tock/issues/4562

- We have git version in the panic dump.
- Presumably we want to use the convention (e.g., "2.3-dev").
- What about adding `TOCK_VERSION_MICRO` to the kernel as a `i16`, where
  negative numbers mean unreleased. So -1 is dev, -2 is alpha, -3 is beta,
  etc.
