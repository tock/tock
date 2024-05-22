# Tock Meeting Notes 2024-05-17

## Attendees
- Branden Ghena
- Brad Campbell
- Amit Levy
- Hudson Ayers
- Pat Pannuto
- Johnathan Van Why
- Alyssa Haroldsen
- Phil Levis
- Alexandru Radovici

## Updates

- Amit: Synthesizing RustNL takeaways.
- Pat: CPS/IoT Week tutorial: small but engaged audience.
  - Tutorial went well, good starting point for TockWorld7.
- Amit: re: TockWorld 7.
  - Talks to fill schedule.
  - Not (currently) including googlers.
  - Call to register coming soon.
  - We can pay for speakers.
- Brad: soil moisture sensor demo app in progress. Works for sensing and screen.
  Todo: connect to Thread and compile libtock-c out-of-tree.
- Alex: work ongoing to simulate Tock. 
  - Compile kernel on webassembly. Run the kernel with applications. Swap out
    hardware at HIL-level.
  - Plan is to upstream.
  - JVW: new registers intended to support similar goal. Allow using Tock
    peripheral drivers and swap out at the hardware layer.

## PRs

- https://github.com/tock/tock/pull/3343
  - How to proceed?
  - Somewhat stale at this point.
  - Connects to https://github.com/tock/tock/pull/3975
  - Might have issues for non-64 bit use cases.

- https://github.com/tock/tock/pull/3067
  - Needs an owner moving it forward.
  - Several screen-related PRs and issues all outstanding.
  - Close for now without prejudice (bradjc)

- Perhaps we need more structure on how to propose designs
  - Alyssa: I can provide a template
  - Corporate environment more of a group decision making process to prioritize
    development (or drop support)
  - Amit: is the clarity from the process going to outweigh the higher burden to
    contribute?

- https://github.com/tock/tock/pull/3258
  - Never reached agreement on how to implement notifications about how capsules
    can tell if buffers are swapped.
  - Issue: callback misused.
    - Might be worth it if the functionality is important
    - Goal of removing callbacks was to make PRs easier to review
  - How urgent?
    - Have a workaround, so a better approach would be usable.

- https://github.com/tock/tock/pull/3256
  - Waiting on Leon

- https://github.com/tock/tock/pull/3258
  - Likely harder to misuse

- https://github.com/tock/tock/pull/3268
  - Assigned to Pat

- https://github.com/tock/tock/pull/3549
  - Waiting on Leon

- https://github.com/tock/tock/pull/3696
  - Brad to look at

- https://github.com/tock/tock/pull/3867
  - Needs review - tockbot assignment

- https://github.com/tock/tock/pull/3964
  - Waiting on Pat

## libtock-c Versions

- Users of libtock-c and stabilizing the library
- Affects on downstream users
  - Could try to create a policy/versions
  - Could say "not a priority"
- Can do a libtock-c release
  - Also fine to have new interface unreleased for now
- Reasonable to separate libtock-c releases from kernel releases
- Brad: I think it is hard to stabilize the libtock-c interface without
  corresponding stability in the kernel ABI for a specific driver.
- Need to do release testing.
- Idea
  - Release testing
  - Mark in libtock-c that stability matches the kernel

## libtock-rs

- JVW likely spread thin (confirmed)
- Many users outside of core team
- What ask can we make of users to help contribute upstream?
- https://github.com/tock/libtock-rs/pull/540 is one approach?
  - Might help, probably not for everything
- Are there out-of-tree drivers for libtock-rs that could be upstreamed?
  - Brad: It's hard to tell if users of libtock-rs are using a forked/divergent
    version, using just the existing upstream syscall drivers, or have drivers
    which would be upstreamed.
  - There are many good reasons not to upstream.
  - Asking for upstream drivers could be a good way to encourage contributions
