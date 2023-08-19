# Tock Core Notes 2023-08-11

Attendees:
 - Branden Ghena
 - Hudson Ayers
 - Alyssa Haroldsen
 - Johnathan Van Why
 - Amit Levy
 - Pat Pannuto
 - Chris Frantz
 - Alexandru Radovici
 - Leon Schuermann
 - Tyler Potyondy
 - Alistair Francis
 
## Updates
* Tyler: we had some issues with Thread, OpenThread not acknowledging things due to a lack of acks. Nordic boards don't have HW support for acks. Doing a software implementation.
* Leon: first networking WG meeting, discussed a bunch of things.  Deliberately left the WG open for anyone to join. Mostly started with bureaucracy, PR to formally establish WG, talked about status of existing subsystems.

## Agenda Items
* Hudson: last week we decided to defer discussing OpenTitan until Alistair could make the meeting, he's here.
* Leon: This can be short. We had a conversation about how we want to bring OpenTitan support up to date, as silicon goes to tapeout. We are interested in having a version of Tock run on the latest tapeout chip. We will support our board for the current hardware. Thank you, Chris, for all of your work.
* Leon: With respect to porting efforts, documentation efforts, using the downstream OT environment, e.g., on silicon, verilator, or the chip, the key takeaway is we don't want to delete anything in the upstream repository that isn't anywhere else. But once the downstream has instructions on Tock, we want the upstream documentation to point to the OpenTitan documentation.
* Leon: We also had a discussion on how to keep upstream Tock up to date with the downstream code.
* Leon: We didn't decide on anything, it was just a high-level discussion keep Alistair's concerns in mind.
* Alistair: This is pretty high level, sounds fine. It would be nice to keep the instructions in Tock on how to do this. It's nice to have it within Tock, run make, rather than go here.
* Hudson: It does sound like we don't have complete instructions. 
* Leon: OT is in the process of using Tock as its own OS. A series of PRs are trying to get a version of upstream Tock, rather than pointing at a commit, to be built in the downstream toolchain. The issue with having the information in Tock is it's dependent on a downstream repository.
* Leon: The goal is to have OT using its downstream repository use its own system while using CI against upstream, to make sure that it always works.
* Chris: Two conflicting needs here. Tock wants to depend on a stable version of OT. You don't want every commit in OT to possibly move registers, change peripherals. Earl Grey isn't moving anymore. OT thinks of itself as a model repo for the project. It has hardware, software, test code, all of the code for the testing environments. We want a reference OS to be in that repository as well. The kernel as it should be constructed for that reference OS. Linking in the apps that have the base functionality we think is necessary.
* Chris: The kernel and apps want to refer to a fixed version of Tock. That's one of the needs. We also want to take advantage of all of the testing infrastructure, both for kernel driver and userspace things, and dispatching those things to our test environment. This lets us qualify a fixed version of Tock that we can verify for a project. We don't want Tock to drift and break something without our knowing about it. We want to execute all of the tests, don't use the fixed version, test against the HEAD. This is what we're trying to achieve. OT wants to depend on a fixed version of Tock. Tock wants to depend on a fixed version of the hardware.
* Phil: That's a great way to put it -- each side wants the other to be fixed. So it suggests each side should do that, it's not that Tock should refer to the OT documentation.
* Hudson: It's great that while OT is working against a fixed release, it's testing against HEAD. It sounds like you'd prefer Tock does the same thing, so working against a fixed release but testing against the HEAD, so we can detect problems?
* Chris: That would be great, but I don't know if it'll work. There are resources Tock depends on in Earl Grey. If we discover bugs in that chip that we want to fix for the production version of the silicon, when we go to silicon for production we are going to take those improvements in the master branch and tape that out. Nothing in master with respect to Earl Grey is planned to change, but there will be at least one IP might change a little, we will disable something we don't have a use case for.  We don't want to contend with what's changing there.
* Hudson: Just wanted to make sure we don't remove docs, then don't have instructions.
* Phil: How do we handle CI failing against HEAD in the other repository?  Whose fault is it? That seems like a long debug cycle.
* Leon: We will be able to trigger a downstream test in response to an upstream commit, and make it a non-required check. E.g., if something 
breaks and it's not upstream's fault.
* Phil: I'm worried about the debugging cycles.
* Hudson: This is getting a little into the weeds, it should be pushed to the OT group.
* Phil: Agreed.

* Hudson: How about the pull request review document?
* Brad: Nothing to talk about, let's just read it.

* Hudson: Yield for?
* Brad: This is complicated and difficult unless we are talking about precisely the same version. We really need to stick to specific names and terminology or it's too confusing. These are just codenames.
* Phil: Unsheathe your dagger definitions! (James Joyce)

* Brad: There's also fixed offset apps. This makes libtock-rs a lot easier to use, and RISC-V on libtock-c.
* Hudson: High-level, what's the summary?
* Brad: Major change with libtock-c and libtock-rs is that libtock-c is for CortexM and so works for a lot of platforms. libtock-rs has very different addresses. The loader has to choose a useful RAM address. Don't have a way for the loader to know "what's a valid RAM address". Now the kernel can tell it "these are the addresses I'm going to use for app RAM."
* Hudson: And you are on-board, Johnathan?
* Johnathan: Yes.
* Leon: What's the status of the elf2tab linker madness?
* Brad: I've been using it, and I'm pretty happy with it. There seems to be a behavior in the linker, where it can actually put a segment before flash, it'll move it, relating to alignment.
* Hudson: This isn't just a bug in our linker scripts?
* Leon: Pretty sure it's not.
* Brad: We can tell the linker to use a different alignment, that helps, but doesn't solve the problem.
* Leon: Let me test this PR with one of the broken ELFs.

* Hudson: For yield-for, I sent you a bunch of messages on Slack, Alyssa.
* Alyssa: I'll take a look.
* Hudson: I was trying to get libtock-rs to looking a bit closer to what Ti50 does, e.g. uformat. Looking at different implementations of yield-wait-for and how they affect code size. I had a couple of questions.
* Alyssa: Some syscalls would benefit from centralizing subscribe ID, others wouldn't. I'll answer your questions on your branch.
* Alyssa: The thing that comes to mind is our storage reading, that's always inlined. That's repeated in a bunch of places.



