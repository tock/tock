# Tock Meeting Notes 2026-10-07

## Attendees

- Brad Campbell
- Branden Ghena
- Amit Levy
- Pat Pannuto
- Alexandru Radovici
- Leon Schuermann
- Johnathan Van Why

## Updates

## 2.3 release
- Leon: There's a release notes PR for 2.3. Once that's merged, I can tag the
  release.
- Brad: Any reason to not do that right now?
- Leon: Merging, sure. I can't tag for a few hours.
- Brad: Aren't we going to end up with commits that are on 2.3 but not on 2.3
  because of the delay?
- Pat: Because of the version number change?
- Leon: Version bunch is on release-2.3 branch. Nothing we merge now goes to
  that branch.
- Brad: I guess that was always a problem that we'd have commits on 2.3 but not
  release-2.3.
- Leon: Master is on 2.4-dev now.
- Brad: So for all purposes, the release is done?
- Leon: Yes
- Brad: So we can merge the PR whenever.

## QEMU boards
- Brad: We discussed it last week, said "yeah go ahead", then nothing happened
  since. What's the outlook?
- Branden: They just need reviews, it looks like. One has been reviewed.
- Pat: Leon, our CI lead, are you happy with it? If so, lets move forward.
  Changes since I last looked seem benign.
- Leon: I haven't looked recently because I've been focusing on the release.
  It's been a while, we can move forward.
- Pat: I clicked merge on #5101 because Leon and I have approved in the past.
- Brad: #5155 hasn't had many comments.
- Pat: I'll echo Leon's frustration that we're accumulating a lot of runners. We
  have one in libtock-rs too. I'd rather have duplicates testing things than
  have less testing, we can deduplicate later.
- Leon: I can't look at the PRs and approve at the moment, but I'm not opposed
  on principle.
- Pat: #5155 looks fine. Anyone opposed to merging?
- Branden: I looked at the last two. #5162 says all its checks are failing, I'm
  not sure why.
- Brad: Sorry, I just tried to re-run it. I don't know why.
- Branden: #5213 is super easy. Just some test definitions, looks fine. Can't
  merge because stacked.
- Brad: I have to rebase it because you can't have trees of stacked PRs.
- Branden: I'm happy to click merge when GitHub/Brad are ready.


## HIL updates
- Branden: I tried to merge the SPI one, and it has a real error. A
  documentation nit you didn't notice. I think I2C can be merged, but no-one
  else approved it. If anyone has opinions, they can take a look, otherwise I
  can hit merge.
- Branden: I hear no opinions, I will hit merge. Same thing with SPI — Brad, if
  you have a second to touch it, please do. If not, I can try to fix it on the
  call here.
- Brad: Sure, go ahead. I'm a little confused why it failed in the merge but not
  in the PR.
- Branden: Yes, I expected it to fail sooner. But it is a real bug.
- Brad: Yeah.

## NRF52 fixes
- Brad: These are part of my mission to convert the NRF crates to Rust 2024 and
  to split the chip crates into safe and unsafe crates. I haven't fully figured
  out how to do that latter, it turns out to be a lot to get to the point where
  we can do that. This is another step in that direction. I don't know what's
  useful here.
- Pat: Unless anyone wants to look at a lot of mechanical changes, we can click
  merge on #5170.
- Brad: It unfortunately doesn't merge anymore. Hopefully easy enough to fix.
- Brad: What about the DMA one, #5179?
- Pat: Frustrating it's 20 files. I know why, but.
- Brad: Because we never invented chip components. This is the penalty for using
  `static mut` all these years.
- Branden: I don't understand your DMA comment.
- Brad: I fixed it in the series. It's inconsistent in `master`, a later PR
  fixes it. But you're drawn to look at it because of a similar line.
- Branden: I just saw three files, two looked different from the other, and I
  wondered why.
- Brad: #5211 is the most interesting of this series, it tries to fix some
  unsafe. We have two constructors: one for default peripherals, one for chip
  itself. It's unclear why the chip one should be `unsafe`. The constructor is
  unsafe because the MPU constructor is unsafe. If we had safe constructors for
  the MPU and syscall implementation, nothing would force it to be `unsafe`. I
  believe the MPU constructor is correctly marked `unsafe`, because a lot of the
  soundness guarantees come from the MPU working correctly. However, I think
  that the chip is the perfect place to discharge that safety requirement,
  because the nrf52 knows it is a cortex-m4 with floating point, so it knows the
  MPU  will work correctly.
- Leon: All of this makes sense if you have the guarantee you are running on
  that chip. We also need to make this a singleton.
- Pat: Yeah. I think the MPU parts are fine.
- Brad: The other part is the syscall implementation. This one is not an
  unsafe trait. I don't think there are any Rust safety concerns with creating
  something that does context switching stuff. Using that would be unsafe, but
  constructing it seems like it should be safe.
- Leon: Which PR?
- Pat: It's the same PR. Two lines of change in `arch/cortex-m/src/syscall.rs`
  and `chips/psoc62xa/src/chip.rs`. It seems safe, the SysCall struct itself is
  a nothingburger. It doesn't hold any state, all the interesting bits are on
  the functions that do the work.
- Leon: The way that makes sense to me superficially, without looking at it for
  hours, is the kernel should not rely on any state the process can modify for
  soundness concerns. The syscall implementation should encapsulate the process
  behavior, and not give guarantees the kernel can rely on, so it should not be
  an unsafe trait. Does that make sense?
- Pat: Yeah. We could re-litigate it if we wanted to re-litigate unsafe on the
  UserspaceKernelBoundary trait. Currently every method there is unsafe, so
  there's unsafe reasoning, but that's also kind a true. We could also do a blob
  we-implemented-this-correctly thing, but I don't think that constructing that
  blob is unsafe.
- Leon: I'm much more worried about removing the unsafe conditions from the
  individual functions.
- Pat: Yeah. One of the reasons why this PR is sound is because every individual
  trait method is unsafe.
- Leon: I disagree with the reasoning, I agree with the conclusion.
- Brad: I just caught up with what Leon was saying, so I'm not sure if I agree
  or disagree.
- Leon: As far as why the constructor shouldn't be unsafe, you can make the
  argument that the kernel loading an untrusted arbitrary userspace process that
  goes through an authentic syscall interface — the kernel should be as
  resiliant to it as... I think I need a bit more time to think it through.
- Pat: I approved it, but left a note that it's up to Leon to merge.
- Brad: Even if we came to the opposite conclusion, and decided there is a
  soundness issue, it feels like it would fall to the same argument. It would
  either work for the chip or not. So it would still be the chip that would have
  to encapsulate that unsafety. Therefore, the chip constructor remains safe.
- Leon: My gut feeling is a safe constructor is fine. I don't have a convincing
  argument yet.

## Default peripheral unsafe docs
- Brad: #5212
- Pat: The more real question I might ask is: we're making a choice that is not
  crazy, but maybe not necessary. Instead of using a singleton macro that is
  enforcing this, we're using a safety thing saying we're only constructing it
  once. I guess that's only half, the other half is nothing else points at this
  underlying memory.
- Leon: We litigated that extensively.
- Brad: So extensively.
- Leon: The conclusion is that even with the singleton, we need the safety
  comment.
- Brad: 5212 doesn't really change anything, it cleans up technical debt on
  making all versions of the nrf52 have the same safety docs. Because our boards
  are on 2021 and non 2024, it doesn't require unsafe blocks anywhere. This puts
  them in. It's just a clean-up, it doesn't change policy or make anything new.
- Leon: I approved it, this is clearly better than the status quo. The one
  safety invariant we should maybe add is that this should be called on the
  correct chip, but I'm not going to block merging this on that requirement. I
  can leave a comment on the PR if that helps.
- Brad: I don't think that would help *me*.
- Leon: What does that mean?
- Brad: That's a whole different can of worms. That's true with Tock writ large.
  That's a separate issue -- how should we address it, how should we document
  it. Holding up this PR is not the place.
- Leon: I don't think this call is a good place to kick off this discussion
  again. I will leave a comment that this is missing a safety invariant, and I
  may or may not open a PR that adds that other comment.

## Driver num function
- Brad: Can we merge the function we discussed last week?
- Branden: You made the assert changes we wanted, I think we're good. Any
  complaints? If not, I'll click merge.
- Branden: Oh, it might need rebase. It's failing CI due to the EPSILON thing. I
  think if I click merge it'll work.
- Pat: Yes.
- Leon: Yes, unless GitHub prevents you from clicking merge on it.
- Pat: *reads docs* I don't think it'll do it, I think you have to rebase it.
- Brad: Okay, I can do that.

## #5177 Creating a CDC test board.
- Brad: Creates a test board to use the nrf52 to test the CDC stack on the
  nrf52dk. We don't use it by default because we use JTAG. But there's a second
  USB port, so this is a common board we can easily test the stack with.
- Leon: I've done this exact thing downstream for the hardware CI. I clicked
  approve. I'm concerned about the increasing number of board configurations,
  but that's a problem to solve in the future.
- Pat: It feels like configuration boards are for configs we don't use.
- Brad: #5180 is the same thing but for PWM testing.
- Leon: I rest my case.
- Brad: I don't think there's any doubt. I'm a fan of configuration boards, I
  think they're the only way to manage it, but I'm not married to them. If we
  had a scalable device-tree-like thing for Tock, we'd replace this in a
  heartbeat.
- Pat: Agree
- Brad: LLMs have made this easier.
- Pat: I'll push back because reviewing sucks.
- Johnathan: The amount of time it takes to review those PRs is proportional to
  the number of boards.

## #5125 More QEMU boards
- Brad: I'm waiting, I don't know what's up with this.
- Leon: I don't know when I'll have time to look through. Can dismiss my
  reviews.
- Brad: If you're saying we can move forward with it, we can move forward with
  it.
- Pat: Does it need a rebase?
- Brad: I assume everything needs a rebase.
- Pat: Okay, I'll rebase it.
- Leon: I won't approve without looking at it. If you want me to verbally agree,
  I'll say sure. I had some concerns in the past about it diverging from other
  boards, but it doesn't diverge any more.
- Pat: Rebase applied cleanly.
- Brad: Does anyone have issues with this? It's been open for a month and a half
  and I think it should be good.

## #5187 No-mux console
- Brad: Most components virtualize everything. If you want to use something
  without a mux, you have to make two components -- one with, one without. We
  didn't have a console component without a mux.
- Branden: I approved this. I left a comment saying it would be nice to explain
  why it exists, but I'm not holding it up for that.
- Leon: I have feelings on this. Ultimately, it makes a fundamental race
  condition slightly less bad. Possibly this un-breaks some real use cases. It
  doesn't genuinely insure it has lossless UART reception.
- Brad: Did I send the wrong PR? This is 5187.
- Leon: Oh, I was thinking of a different PR. This is fine.
- Leon: *clicked approve, added to merge queue*.

## Call organization
- Branden: The reason this call and last call are like this is people are too
  busy to organize, and the speed of PRs has gone up. It's been problematic to
  keep up with PRs in Tock. This is a problem. I don't have a solution.
- Leon: I agree it's a problem. I have to drop.

## #5210 no_mangle PR
- Brad: Johnathan because you're here. Johnathan and Pat have commented.
  Johnathan had a macro idea, I had Claude prototype it, and it didn't do what
  we expected. Symbols don't follow dependency tree.
- Johnathan: I read your comment yesterday. I came to the conclusion the chip
  crate can export the macro. Or maybe the arch crate defines a macro that the
  chip crate uses to define the macro. It's a lot of nesting and boilerplate,
  which may be worse than the bit of unsafe.
- Pat: We're run into a few places where safety invariants cannot be discharged
  in the Rust source alone. It depends on Cargo, it depends on the linker, it
  depends on something else. I think we can establish a convention for external
  safety invariants, or build safety invariants, or something else. Then we can
  collect them. So I think that instead of adding a clause, we should add a
  separate sentence that is safety-we-can't-discharge or cargo-safety-invariant,
  and collect those. So for the moment, we can make them a safety TODO.
- Brad: Okay. I guess, yeah, this is tricky without Leon here. What are you
  proposing more concretely?
- Pat: I propose using a particular keyword that we can search for later.

## #5218 Imix tests edition 2024
- Brad: I don't want to do this work. It's tedious and annoying. Forget 2024,
  lets just do it for the crates we care about and leave the other crates alone.
  If we want the whole repo to be 2024 that's cool too, but we have a lot of
  code that nobody takes care of. Pat you pointed out that the network stack is
  not implemented the way we think it is. Where do we draw the line? I don't
  want to fix this.
- Pat: It's a good time to do a survey of boards that should move to the
  archive. Only the people on this call plus three have Imixes.
- Branden: I don't have one.
- Pat: If these boards aren't being maintained, we should move them to the
  archive.
- Brad: Okay. I think that will work for some. I suspect some are in a middle
  ground.
- Pat: I think with those, hopefully we can be forgiving on letting Claude do
  things. I think there is value in getting the whole repo to 2024. I left some
  suggestions, and hoped you can just feed them to Claude. But lets archive the
  boards to reduce the scope of the problem.
- Brad: I hope these tests are not copied to other boards.
- Pat: I sure hope not.
- Brad: Every time I find this, I'm thinking "if we had just enabled lint this
  from the beginning, this wouldn't be such a huge project".
- Johnathan: Can the `static mut` lint be enabled in Rust 2021? The
  unsafe-in-unsafe-fn lint can be. If we can turn those both on, then we can
  solve the main problem without dealing with the other Rust 2024 changes.
- Brad: Those are the two main challenges with 2024. If you solve them, then you
  can easily bump to 2024.

## #5228 Doc builds check changes
- Brad: Can push full docs to nightly.
- Pat: My instinct is it's not worth the complexity. A simpler change is to make
  push-to-Netlify a less problematic change. The docs just have to build, not
  push to Netlify.
- Brad: How do you separate them?
- Pat: They're already separated. Push to Netlify is the last line. Just don't
  make that a failure. Make the deploy-to-netlify step not required.
- Brad: I'm saying that in the PR, there's only one row. docs-ci. That will have
  a red X.
- Pat: Why will it have a red X?
- Brad: If something fails, it doesn't give a red X?
- Pat: If we go back two months ago, we used to build the docs on Netlify's
  servers. Netlify's CI is way slower. I adjusted it so the docs build on
  GitHub, then GitHub pushes to Netlify. I'm now saying the ci-docs job should
  be green if the build succeeds, then after the build it will *try* to push to
  Netlify. But if the push fails, it will fail silently.
- Brad: I think we should separate out building docs from checking docs, period.
  Then there's the question of whether we should do it the fast or the slow way.
  Docs CI can stay as build and push, then we can add a quick check for docs.
  Then people can check it locally as well.
- Pat: I get that. But the PR looks Claude-y.
- Brad: Look at my new PR I haven't opened yet.
