# Tock Core Notes 03-19-2021

## Attending
 - Pat Pannuto
 - Amit Levy
 - Leon Schuermann
 - Hudson Ayers
 - Alistair
 - Brad Campbell
 - Johnathan Van Why
 - Philip Levis
 - Vadim Sukhomlinov

## Updates

- Johnathan: Rust async working group requested feedback on developing in async
  rust. I am going to submit a report describing why I decided not use futures
  in libtock-rs
- Hudson: Testing out a newer Rust nightly revealed that LLVM-12 partially fixes
  the stack size / `static_init!()` issue I had identified. In some cases, LLVM is
  now smart enough to not stack allocate peripherals, then copy them into the
  global, instead allocating directly into the memory of the global.
  Unfortunately this does not seem to fix the issue on every board, for reasons
  that are still somewhat unclear to me. In general, this "fix" relies on an
  optimization that is not guaranteed to trigger, so we still will need to decide
  whether the fix we had originally agreed on is worthwhile.
- Alistair: I submitted a PR for the SwerVolf Risc-V board, it allows for
  verilator testing. Hopefully we could use this in CI eventually, as it is
  better than QEMU in that it is actually a simulator.

## Tock 2.0 Alpha 1
- Amit: Should we merge Alpha1? Waiting on 3 approvals. Pat / Branden / Niklas.
  Regarding Niklas, he messaged me saying he does not have cycles to review
  things regularly, so I don't think we should block on him.
- Branden: I am not blocking on anything, though I have not had time for a super
  deep review. I may be able to spend some time this afternoon, but I do not
  expect to find anything others have not already.
- Amit: Are the userland libraries *ready enough* for this merge?
- Amit: I assume that libtock-rs is not, but I think we had decided we are okay
  with that.
- Johnathan: Yeah that is correct, I am trying to get testing in place before
  sending PRs upstream.
- Amit: libtock-c seems to be ready, though we may find stuff as we work through
  and test
- Leon: IME I used to find major issues in libtock-c 2.0, and I think more
  testing would be good.
- Hudson: I think the "Imix" app works, at least
- Phil: Yeah that is a starting point but definitely need more stress testing
- Leon: I am concerned about error handling as I saw different people use
  different approaches there. We had some guidelines in the TRD and I am not
  sure every userspace library uses them correctly.
- Amit: Well the question: is libtock-c *ready enough* that we should merge the
  kernel PR. It does not make sense to wait to merge into the development branch
  until everything is ready for release. My personal sense is yes.
- Phil: I agree with that, but we should run some tests. I can do that this
  afternoon
- Leon: I think it would be a good idea to skim over the commit history and
  remove some of the automatic commits by github and squash them into the actual
  commit. I would be willing to do that. I would need 1-2 days
- Brad: Not opposed
- Leon: Will try to get that done tonight
- Branden: Were we gonna do an alpha-1 PR for libtock-c?
- Phil: Yes
- Amit: I don't think these things have to happen in lockstep
- Johnathan: In libtock-rs I am not using a branch, I am using separate crates
  for 2.0. At some point I was going to wipe out the 1.0 branches and rename
  stuff to get rid of "2" suffixes. I was imagining doing that when 2.0 is
  released.
- Amit: I see this alpha merge as not a release but necessary to keep
  development proceeding reasonably. I think that it is fine to keep working on
  a 1.x release in userspace until 2.0 is ready
- Johnathan: That was my assumption but I am not sure anyone is using libtock-rs
  against 1.0 right now.
- Branden: From the libtock-c standpoint I am just thinking about randos who
  find Tock and try to install something, and it won't work.
- Leon: I think Brad had an idea about having tockloader prevent this?
- Brad: I think that effort basically failed. The only way I could think of to
  get this to work required everyone have the newest version of everything so I
  don't think any automatic checks are feasible.
- Leon: If we want those guarantees in the future would it still be good to have
  a mechanism for this?
- Brad: Yeah sounds great, I just don't think we have found a very good
  mechanism yet
- Amit: I agree with Branden's concern, but think a few days is not a huge deal.
- Hudson: We could push a big warning to the front of the libtock-c README for
  those few days
- Branden: I think that would be fine
- Amit: Moving forward, maybe we should generally update documentation to
  encourage people to download releases instead of cloning the git repo.
- Phil: I agree with that as well. If master is "current development" we should
  shelter users
- Leon: Is there always a userspace release corresponding to each kernel release
- Alistair: There has never been a libtock-rs release
- Brad: We try for libtock-c
- Branden: We failed for 1.6
- Phil: Sure, but that is part of the point of separating these things.
- Hudson: I think it is reasonable to say that each userspace release specify
  what kernel version it was tested on
- Brad: My vote against pegging releases is that it does not work for newer
  hardware, as almost always the first step on newer hardware is "try the newest
  code"
- Leon: I suppose that relates to the whole issue of board maturity. Maybe not
  the time to discuss this, but we should clearly specify which boards are
  experimental
- Phil: I see your point, but stepping in the feet of someone trying to get a
  board working. They boot a board, something fails, they get on slack, someone
  tells them to try the newest code. They try it, it works.This seems better than
  following the prescribed instructions out of the box and something does not work
  at all.
- Amit: The pattern usually used for this is separate documentation for users
  and devs, where "devs" is a euphemism for anything on the bleeding edge.
  Anyway, most people will probably use a board like the nrf52dk where a release
  version will work fine.
- Amit: Back to the OG question, what's next after merge? Just testing?
- Several: There are a few steps left on
  https://github.com/tock/tock/issues/2429 , most notably callbacks passing
  errorcodes instead of returncodes, and then callback/appslice swapping
  restrictions.
- Phil: I think this touches on how there are certain userspace APIs which IMO
  are fundamentally broken -- values are not passed back that userspace would
  need to be able to properly handle errors. This issue came up in the whole
  `map_or()` vs `map()` debacle, and it would be good to add that as an item on #2429.
- Leon: I will update my rename AppId to ProcessId as soon as the alpha is
  merged.
- Brad: So when do we merge alpha 1?
- Hudson: I vote as soon as Leon cleans up the commit history, and Branden/Pat
  check their boxes
- Branden: My box will be checked before Leon has cleaned up the commit history
- Brad: Is anybody opposed to the merge and business as usual until we are ready
  for a release?
- Phil: I am opposed to merging until I have done a little libtock-c testing
  that I will do today. Always a danger to merge without integrating tests
- Brad: Sounds good, no rush.
- Amit: Ok, I think we are in agreement.

## Bump Rust nightly?
- Amit: Brad says we should bump the rust nightly now
- Brad: normally we update every 2 months, we are there now. The LLVM-12 stuff
  is interesting. But we are in a transition period so wanted to check
- Johnathan: There is a fix that allows you to use clobbers on ARM in asm!()
  that is only available on new nightlies
- Amit: I think we should do it either now or right after alpha-1
- Leon: Note: I saw a slight size increase on LiteX
- Hudson: I saw a 6kB decrease on Imix, these both might be mostly noise
- Phil: I propose we merge alpha 1 on Monday, and then do new nightly right
  after.
- *general agreement*

## https://github.com/tock/tock/pull/2452
- Brad: Rust provide flags for separating code based on OS / arch, but they are
  coarse grained. We have gotten away with this, but an open PR for a port to a
  Raspberry-Pi that is Cortex-M0+ requires finer resolution than rust gives us
  using the base set of `cfg` flags. There is a feature that adds much better
  resolution, letting you base stuff off the exact architecture and features
  available on a given chip. This PR adds that feature and lets us specify that
  some of the asm in the cortex-m crates is only for certain versions of
  arm-thumb. The downside, of course, is that its a new nightly feature.
  Decision: are we okay with this? Am in support for 2 reasons. 1. This is really
  just a change in the build system, so I see it as less risky. 2. There is
  another PR which is a different implementation of the same thing that
  accomplishes the same goal in a messier way. If everything else gets stabilized,
  we could revert to the manual way, so this won't stand in the way of our path to
  stable long term.
- Leon: This feature could be great to optimize the use of compressed
  instructions for RISC-V architectures that support them, which performs much
  better on platforms without caching.
- Johnathan: They should be automatically replaced, you have to be careful
  because only some instructions are compressible.
- Amit: Any dissent?
- Johnathan: No, but we should verify the feature flag is actually needed.
- Everyone: cool
