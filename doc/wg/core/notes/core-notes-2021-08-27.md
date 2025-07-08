# Tock Core Notes 2021-08-27

## Attendees

- Pat Pannuto
- Alexandru Radovici
- Philip Levis
- Branden Ghena
- Jett Rink
- Hudson Ayers
- Gabe Marcano
- Leon Schuermann
- Vadim Sukhomlinov
- Brad Campbell
- Amit Levy

## Agenda

- Updates
- Tock 2.0 final release
- Changing C application behavior on return from main

## Updates
- Leon: CI target testing app integrations with simulator
- Pat: Unergrad working on CI infrastructure, NRF CI working, verifies GPIO, I2C, bluetooth advertising
Should CI run every time?
- Pat: Gate them to run only on board changes?
- Leon: Simulator doesn't take that long
- Amit: Run nightly?

- Philip: Students looking into binaries. 2 results. Paul found weird cases where inlining/no-inlining can lead to 30-40 instr code size change. Many more loads/stores. Looking into why. Gabe has been looking data in read-only variable. ARM, lots of different strings aggregated, making this hard. RISC-V doesn't do this, part of the LLVM IR -> ASM . Trying to do an accounting graph of what methods are responsible for image bloat.
- Brad: tockloader has quite a few recent changes, and it's going to be necessary to do a release alongside Tock 2.0. Let me know if anything breaks.
- Amit: Some of the changes?
- Brad: Major changes is it now does a lot more autodetection. Some internal changes, optimizing some commands so it runs faster. Can also just write to a file on the computer now. Can edit local tbf and tab files.

## Tagging final release of Tock 2.0
- Amit: Half of the signoffs on the issue. What's the status?
- Branden: Going to test stuff later today, and check off.
- Alexandru was also planning on testing later
- Alexandru: Any way to include shell prompt on version 2? Planning on including tag in book about Tock 2.0. Can we release shell in 2.0.1?

Sounds like it's possible. Only concern is that shell should be added in a month or two, not in a year.
So, waiting on Branden to do more testing today, and ping Jonathan to see if he has a problem, or just busy. And Pat.
Should we commit to 2.0.1 timeline? Sure, why not.
- Amit: Any additional work we'll include in the next release?

Bug fixes.
- Philip: Feels weird to test now for 2.0, and then do more testing again, in short order, for 2.0.1?
- Jett: Seems like a lot of rework and testing for just a shell prompt.
- Hudson: Outstanding concerns from Brad on how to adjust/fix the PR
- Leon: Reasonable to have 2.0.1, might help people move to 2.0
- Philip: A lot of stuff we have queued up. 2.0.1 might have more than we expect.
- Brad: Can just cut off new features when testing picks up
- Brad: What we're really describing is 2.1
- Amit: Alex, purposes of the book/class, usecase, is it a particular board, or many boards?
- Alexandru: Simulator, RPi, bit[sp?]. Having the shell would be nice, it's not a blocker, but it would be nice to have.
- Amit: Could make it a patch release, can use a capsule that specific boards can use.

Discussion on whether to block or continue 2.0. No one wants to block 2.0. Concerns seem to be around how long 2.0.1/2.1 will take, and how much testing that will entail, and how are patches/PRs managed.

Hopefully CI helps catch issues/increase confidence on release new versions (regression testing is nice).
- Brad: Wrap up 2.0 thing. What's our cutoff?
- Amit: Waiting on Pat and Branden, and maybe waiting on Jonathan? Probably cut it off today or tomorrow. If no issues crop up, release Tock 2.0 tomorrow.

## Changing C application behavior on return from main
Pull request from Brad
- Brad: Backstory here is what happens, in libtock-c, when main() returns. I typically think that your program is done. In tock we didn't have a way to do this, until recently with exit syscall. Previously, sat in while(1) loop that yields. If process has pending callbacks, yields would allow upcalls to execute. Recently, since we have exit, why not use it? Thing I didn't think of, processes have been relying on this behavior of while yield loop. I went through example folder that are relying on this behavior. Main change from programmer's point of view is having an explicit loop in program.
- Branden: Good point in PR, by exiting, when returning from main, we're fulfilling expected C program behavior. Question to consider, do we expect Tock applications to follow this? Should C tock applications do this? Or more special embedded thing? Making sure that we realize that changing this will change default behavior.
- Jett: As a downstream user, we don't depend on it.

Some people remarked surprise on current behavior when first encountering it. Discussion on whether we want to allow exit or not.
- Leon: We don't have a fork call, but that's the main issue, we don't have a process supervision interface.
- Philip: So, should we just not call it main()?
- Alexandru: Suggestion was to have one main() and another setup() or something. main() would be more traditional, and setup() would be what's right now.
- Alexandru: 3rd option, adding a way to change loop function

Discussion on whether to match POSIX main() semantics, or go our own way. Tock is different than POSIX, so it can have different exit semantics when returning from main. Not uncommon for embedded systems to do their own thing on exiting from main(). Also, in many cases, exiting from main doesn't make sense. Question of what to do on exit-- kill application? Restart? What should be the default?
- Amit: Past time, seems like we're not going to resolve this. Basically all agree that current interface is not great. Need to figure out something better. Brad's suggestion seems to have some support.
- Leon: If we were to change this, it should be before libtock-c support release.
- Pat: We don't have to release libtock-c 2.0 at the same time as tock 2.0, but it's not necessary.
- Philip: Sounds like we need to figure out the execution model
- Amit: Noting that none of this is changing the semantics of how processes
- Philip: Strange that application can end, with no real way to restart. Worth having a discussion
