Tock Core Notes 2023-11-03
==========================

## Attendees
 - Alyssa Haroldsen
 - Amit Levy
 - Brad Campbell
 - Branden Ghena
 - Johnathan Van Why
 - Hudson Ayers
 - Leon Schuermann
 - Phil Levis
 - Tyler Potyondy

## Updates
 - Leon: Revisiting the RISC-V PMP update. This changes a bunch of critical
   infrastructure, so we want a few rounds of good reviews. More importantly,
   it's not ready for merge just yet. While switching to the kernel memory
   protection, I found the VexRiscV CPU has an integer overflow bug. I proposed
   a fix upstream. Hopefully it'll be merged soon and we can pull it in to
   exercise these code paths in our CI.
 - Brad: Update to the libtock-c build system discussion from a month-ish ago.
   It started from an upgrade of the newlib version, and has graduated to an
   option to not use the users' libc headers and instead use our own newlib
   headers. Now we don't rely on package managers shipping them, just need a
   toolchain. It's in place but the RISC-V GCC toolchain is still new and
   fragile. Works with the newest toolchain but not older versions. Plan is to
   compile libraries with the older version and hope the work with the newer
   version.
 - Branden: It's really exciting to get us off from "whatever happens to be on
   somebody's system".

## Networking WG Updates
 - Branden: Spent a bit of time discussing buffer management, and a lot of time
   talking about what it will look like to have an OpenThread application-land
   library. Particularly, if we can take C libraries that exist and package them
   into an application, what would that look like -- do we have the right kernel
   support for it? What will it stress? Not an immediate concern, but
   brainstorming for that process. Leon had good papers to share on latency and
   measurement, which Tyler is looking into.
 - Brad: It'd be interesting to have a stack like that.
 - Phil: This is what we did for the original 15.4 stack, which I ported to a
   userspace process. It turned out to be valuable, because it gave us a working
   stack. Made it easier to do A/B testing as we reimplemented it.
 - Leon: Also merged the STM ethernet support into the Tock ethernet branch.
   Hopefully in the next week or two we'll have a big PR from the ethernet
   branch towards Tock to merge in the infrastructure.

## More updates
 - Amit: I've been discovering more Tock users, including a Microsoft team using
   Tock in a secure enclave-type use case. Sounds like there is some duplication
   of efforts. Promising -- means maybe there's more incentives to work across
   and with upstream.
 - Phil: What's the group at Microsoft?
 - Amit: Folks working on Chariot, not the same as Caliptra. They also announced
   at the event that they are making an enormous investment in Rust.
 - Hudson: That's exciting. Do you think you'll get them to join calls soon, or
   just hoping for repo contributions?
 - Amit: One of the two.
 - Hudson: It would certainly be cool to see something from them. Maybe a
   presentation like what Alyssa's done -- what are the pain points, what do you
   like -- but I realize that's a big ask.

## `tock` PR #3597
 - Johnathan: PR 3597 is stalled on what I think is a QEMU inaccuracy. No-one
   working on the PR has QEMU development expertise or the time to debug QEMU.
   My inclination is to just disable the test for now, but looking for others'
   input.
 - Leon: Had similar issues with QEMU-hardware discrepancies while working on
   PMP. Switching contexts to fix QEMU is a lot of overhead. I don't think it's
   reasonable to do this for every bug. The other issue we'll be facing is I
   know OpenTitan is maintaining a QEMU fork with a more accurate hardware model
   of OpenTitan. Those two models will likely merge, and the upstream OpenTitan
   development will target their version of QEMU, which is not the upstream QEMU
   version.
 - Hudson: Why wouldn't the upstream version try to take in the changes?
 - Johnathan: Aware of a difference in technical design that may not be
   acceptable to upstream.
 - Amit: We only care about QEMU to the extent it is a good representation of
   the real hardware. If emulator quirks are really getting in the way, it seems
   counterproductive to stall progress on supporting the hardware because the
   emulator is incorrect and is failing the CI.
 - Hudson: Does this PR passes the forked QEMU's CI?
 - Johnathan: That has not been tested.
 - Amit: One of the FPGA boards is becoming available to potentially stick in a
   test rig. My proposal would mean to not block on this, if that means
   disabling the test. Using the forked QEMU seems fine in principle, but that
   meant compiling QEMU as part of the CI which takes a long time.
 - Leon: We do that anyways. We're tracking latest head.
 - Amit: I would say if we're confident the problem is with the emulation and
   not the code, we should move forward with the PR, and if that means disabling
   the test, then disable the test.
 - Leon: The one thing that makes me skittish, is that right around the time
   this PR came up, is we fixed an actual driver bug thanks to a QEMU-based
   test. While there may be some divergence between the hardware and the model,
   there seems to be a benefit to having those tests.
 - Amit: Two questions. First question: would that have been caught on a
   hardware-based test as well?
 - Leon: The issue with this bug is we encountered it on every 10th or 20th CI
   run. Me updating QEMU to include the PMP fix changed timing and made it
   consistently reproducible. I don't know if hardware would've let us reproduce
   it in the same way.
 - Amit: Sounds like there's nothing special about QEMU.
 - Leon: There is some benefit to heterogenity in our infrastructure. I think
   we'd lose something if we disabled them.
 - Hudson: The magical thing about QEMU is we have it set to run on every PR.
 - Amit: I'm not suggesting the long-term solution is to not run QEMU tests, I'm
   suggesting a temporary solution of either disabling that test or to ignore
   the QEMU CI for now. I don't the solution is to not merge the PR.
 - Hudson: Johnathan, is it one test or all of them that aren't passing?
 - Johnathan: If I remember correctly, it's just one test, that checks if the
   kernel boots.
 - Leon: We run one big integration test, then if that passes run a bunch of
   smaller tests.
 - Johnathan: Oh, it's the initial test that fails.
 - Hudson: We've had some issues be caught by the hifive tests, but opentitan
   seems to have more tests. It sounds like it would be easy to switch to the
   QEMU fork. If we start ignoring the QEMU CI, then it'll become hard to turn
   it back on. If it's hard to switch to the fork, then probably not worth it.
 - Leon: I'm concerned about a temporary fix becoming a permanent one. What is
   our policy on this?
 - Brad: Nothing. We're Tock developers. We don't expect people to contribute to
   QEMU whenever we hit issues. It's a tool. If it works, great, but if not then
   we can't always fix it.
 - Amit: It would be great to test every PR, but if that's not practical, then
   what Brad said. In the medium term, I think the thing is to try using the
   OpenTitan fork of QEMU. I'm acknowledging that is a nontrivial effort, and
   this is a trivial PR. What should precede what?
 - Hudson: I was trying to get a feel for the level of effort of this. Is it as
   simple as pointing to a different repo, or is it significantly more involved
   than that?
 - Leon: My knowledge base it the fork exists, but I have not used it so far.
 - Hudson: Is it publicly downloadable?
 - Leon: It's a public repository. lowrisc/qemu. It's in a branch called
   ot-earlgrey
 - Amit: May be as simple as pointing to that. Could try that real quick, make
   it part of this PR.
 - Hudson: That's my stance. If it's that easy, then great, but if not, then we
   shouldn't ask the PR author to fix CI.
 - Leon: I agree. My one remaining question is we have someone working actively
   on Tock who is an upstream QEMU maintainer. Would we want to consult Alistair
   on this?
 - Amit: Ideally, this would've been discussed within the OpenTitan WG. I think
   we should be able to do things temporarily that are not permanent statements.
   I don't think we should block individual PRs on QEMU bugs. Whether a fix is
   upstreamed is a separate question and we should talk to Alistair about that.
 - Leon: That makes sense to me.
 - Hudson: I think we should certainly have a discussion with him. He's been
   given a couple heads up on this.
 - Brad: I think we can do things in parallel. Get on PR through, and have
   another that tries to make improvements. The more we stack things serially,
   the harder it is on us.
 - Johnathan: So, try pointing to lowrisc's QEMU in the PR, if that works
   trivially, great. If not, disable the test in the PR. Either way, discuss it
   in the OpenTitan WG.

## `libtock-c` PR #353
 - Brad: The question is where we put all the pre-built things. The main reason
   we're pre-building things is so we can compile with PIC. Today we're saying
   "we only have hardware boards with these architectures, so lets cherry-pick
   these ones". However, if we're building them all, why not include them all?
   That drives up the size. What do we want to do moving forward?
 - Amit: What are the options?
 - Brad: Continue to commit the raw compiled libraries into git as we're doing
   today, we could commit zipped (or otherwise compressed) versions into git,
   could use gitlfs, or we can package it and host it on whatever server we want
   and have the build system fetch and use it. The PR does the last one.
 - Amit: GitLFS essentially looks similar to what you're doing manually here.
   Leon has a better technical sense of this, but it is essentially committing a
   hash of the artifact into the actual repository, with a pointer to the
   artifact on another system. It tracks changes that way. Indeed the goal is to
   avoid tracking large files in git which git is not particularly good at.
   GitLFS is pretty specific to GitHub, which is not ideal. I think it's not
   supported it git natively -- you need to install an addon, although I think
   most git distributions do that anyway.
 - Alyssa: It also involves billing.
 - Leon: I think that's an accurate description. I've tried it a few times and
   never got it to work.
 - Branden: That was my opinion. I tried it 6 or 7 years ago, and I remember it
   not working at all.
 - Alyssa: I recall it working, but we had issues with billing and quota.
 - Leon: It's really expensive. GitHub has insane charges for it.
 - Alyssa: How big are the actual files?
 - Brad: About 150 MB for one version.
 - Leon: The issue with GitLFS is you're paying for traffic quotas, and we don't
   have any control how many times a user tries to download it. Alistair's
   concerns are valid that files we host elsewhere may be gone, but that'll also
   happen if Tock stops wanting to pay for LFS. Most of us are at universities
   with web space and bandwidth available for free, so I don't think that'll be
   much of a concern, especially with multiple mirrors.
 - Branden: Users can also build it for themselves, we presumably have
   instructions.
 - Amit: It's unlikely we'll run up tens of thousands of charges for this; we
   can almost certainly find a way to afford GitLFS or run it on our own. The
   main advantage of GitLFS I see is that by default, we would get a history
   view. Maybe with this strategy, that wouldn't be the case, as the mirrors may
   want to remove the old versions. I think that is almost entirely ameliorated
   if this is a replacement for building something locally. Provide hosted
   versions for people using real releases, with a fallback of building from
   source.
 - Leon: We could also enforce a policy of tagging uploads with the date, and
   not delete them.
 - Amit: Ultimately, this is what package managers do.
 - Brad: We definitely could be doing a better job of explaining how we got to
   the precompiled thing. It will not be easy to recreate one of these things
   back in time, because toolchains and whatnot. We could better document that.
   Certainly the intent is there that you could recreate it.
 - Amit: Is it not something that we could put into a script?
 - Brad: What you get is highly dependent on the toolchain you use when you run
   the command.
 - Amit: Leon, sounds like a job for Nix.
 - Leon: Yes, but also I was going to suggest Docker.
 - Johnathan: Second Docker.
 - Leon: I love reproducible artifacts as much as the next guy, but I don't
   think it'll be critical if we lose year-old toolchains at some point.
 - Amit: In the long run, it is important. In general, an important thing in
   practice is that people may need to reproduce and patch an old version
   deployed in the field. They just need the particular version used to build
   an application. That's also maybe big-boy problems.
 - Leon: You would hope they did due diligence on their part and saved the
   binaries they needed.
 - Amit: The question is how attractive is our setup for people who have those
   concerns.
 - Brad: Realistically, are we going to pay for GitLFS when we can have it for
   free without doing anything?
 - Branden: I think it's fine to host at another location.
 - Brad: Is there a service that will run a Docker container for you?
 - Amit: Azure? Google Cloud Run? Lambda?
 - Brad: Because I don't have the RISC-V binaries, I'm motivated to run this in
   a specified environment.
 - Amit: We could do this in GitHub Actions.
 - Brad: That sounds hard.
 - Amit: GitHub actions is just Docker.
