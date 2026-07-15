# Tock Meeting Notes 2026-06-17

## Attendees
 - Branden Ghena
 - Leon Schuermann
 - Amit Levy
 - Johnathan Van Why
 - Alexandru Radovici
 - Brad Campbell


## Updates
### RISC-V 64-bit Support
* Brad: Team update. RISC-V 64 bit apps work in QEMU complied with libtock-C. I had previously closed an issue saying we'd never get to this, but here we are! Good proof-of-concept that end-to-end works. But the QEMU 64-bit uses the same memory map as the QEMU 32-bit version. So I suspect the top 32-bits of all of our registers are zeroed out.
* Leon: VirtIO is DMA and that would use 64-bit addresses
* Brad: It's mounted in the lower 32-bits of the address space.
* Leon: We could move the memory map to test that
* Brad: Moving userspace to the upper 32-bits would be a good test. I suspect that doesn't work
* Leon: Good job getting the medany memory model working!
* Amit: What's the memory protection mechanism? Is it virtual memory or an MPU?
* Leon: Tock is still running in machine mode without virtual memory. We could implement virtual memory, but that would be tangential.
* Amit: Do all RISC-V boards have machine mode without virtual memory, and some have supervisor mode which does have virtual memory? (yes) And the QEMU emulator has both modes, but Tock is running in machine mode (yes)
* Leon: You can ignore supervisor mode safely if you don't use it. The assembly for moving to supervisor mode wouldn't be too bad, but the virtual memory support would be
* Leon: So pleased this got working. I tried this for years but never managed to push it past the line
* Brad: Still not there yet. The CRT0 header in libtock-c, that's 32-bit still. Linkers aren't designed to just handle "register-sized" values. So definitely more work to do.
### DMA Soundness
* Johnathan: The OPSEM working group has been discussing volatile atomics, which they might add to Rust. Maybe our DMACell use case actually requires volatile atomics to be sound. That conversation is ongoing to understand. We could possibly use inline assembly to do that. Could add extra effort to tock-registers if we wanted to support it.
* Amit: Any feedback from them so far?
* Johnathan: Mixed feedback so far. One says it's good, and one says it's not, but didn't clarify why yet.


## Tock Registers Transition Plan
* Johnathan: Replacing various traits and systems in Tock Registers with new stuff. So we're going to have to delete old code at some point.
* Johnathan: We talked about version numbers in Github, based on transition plan. Lots of differing opinions there. If I'm planning multiple releases in the future the plan could still change, but I want to align on an initial plan at least
* Johnathan: Action plan I proposed right now: 1) Soft-deprecate old API and release as 0.11.0. 2) Point Tock to main branch so we can start moving Tock over to new APIs. 3) Eventually when we feel ready delete old stuff and the push a 2.0 release with just new APIs.
* Branden: Confusion, you said we'd point at master, oh but that wouldn't be updated yet, it would just use deprecated stuff. I see
* Leon: Sounds okay. We'd want to force this to be some git revision pin, not just the main branch as a moving target.
* Johnathan: Lock files pin an exact commit.
* Leon: If we're not pointing to a crates.io release, I'd still like to have a git revision hash and not just rely on a lock file
* Brad: In this plan, how would you tell what's using the old code and what's using the new code?
* Johnathan: You'd have to look at the code to see.
* Brad: I'd like to change the name of the old package temporarily. And change code to match. So then we could just grep to see where the old version is used. Different names means no ambiguity.
* Johnathan: Where would the rename exist? In the kernel crate?
* Brad: In the kernel crate. So that would propagate to every crate that uses tock registers version 0
* Johnathan: I think that's feasible.
* Amit: The benefit would be having a different package name? Instead of a different version?
* Brad: It would be very obvious what we have ported and what we haven't ported. Then once the porting is complete, the project is done. No further renaming. I guess both avoid that.
* Branden: I was thinking of this too. Two packages: tock-old-register and tock-new-registers or something like that
* Amit: What if we move tock registers from being a dependency of the kernel to being a dependency of each chip individually? That would be easy to change. Then we could still depend on a fixed commit hash of Tock registers. So 2.0 would totally get rid of the old API, and chips that haven't been ported yet would depend on Tock register pre-2.0
* Brad: I agree, except that I don't see the value in changing all the chips to depend on Tock-registers version 0. Then whoever is the last person to remove Tock registers v0 from a crate, has to remove that dependency. And it'll get annoying as things are in parallel. So limiting it to the kernel crate would be better
* Amit: In my version, I don't think you can reasonably update a chip piece-by-piece. I'm thinking the whole chip gets upgraded. So each chip would either have pre or post 2.0
* Brad: I'm worried about the porting work. That would be another hurdle where the whole darn chip has to be updated at once.
* Branden: I do think we should be able to port half a chip
* Amit: You could alias a dependency and still do that
* Johnathan: I do like using the old dependencies through the kernel crate, then updating to new dependencies chip-by-chip.
* Amit: That would defer the pain of downstream chips which use Tock registers through the kernel. I guess if they delay porting and the kernel updates to remove it, they could always just import the old tock registers directly at that point. It would sort-of couple the transition.
* Johnathan: Unless we made automatic porting tools, I think this transition could take a very long time. Maybe we need a Tock kernel release before we start porting. We probably don't want to release the kernel mid-update
* Amit: That's only if these things need to be coupled. I don't know if they do.
* Leon: That's a fine suggestion. It's been a while since a release and there have been significant changes. But I don't see it as strictly necessary.
* Amit: Release could remove tock-registers from the kernel crate.
* Leon: Removing it as we update seems better to me. Less churn just for churn's sake
* Brad: Coupling things to releases is hard. Hasn't gone well
* Branden: Delays stuff a lot
* Brad: We could have a hybrid release which doesn't totally switch to new registers but at least flagship chips have changed. Wouldn't require a release strictly before or after
* Leon: I'm concerned about any release depend on a non-released tock registers version
* Amit: Agreed. It would have to be a released Tock registers version.
* Leon: Okay so that hypothetical release would have some chips depend on tock registers 0.11 and some depend on 2.0
* Johnathan: If we want to iterate without constant crates.io releases, we need some intermediate stage dependent on a git commit instead. But then later we could still release 2.0 before the next Tock kernel release
* Amit: The more I think about it, just removing Tock registers from the kernel and moving to individual chips would be a huge win. Then that would let us update one chip at a time, pointed at a different git version.
* Johnathan: But if we're pointing to git commits for tock-registers chip-by-chip, then we'd have to update a whole bunch of these at once.
* Amit: Eventually it's going to be stabilized. Either the APIs are changing in which case we have to update the chips, or else they're not so we don't have to update anything. You could put it in the workspace
* Johnathan: Yeah, the one that's tracking current development should be the workspace dependency
* Brad: I am starting to disagree. Changing all the chips and having chips potentially have multiple imports sounds like a huge tracking overhead. Hard to tell which ones are updated and which aren't. If we just change it in the kernel, that'll force that change everywhere. And we'll know exactly who is using the old version. We'd rename the export to tock-registers-v0 and update the name everywhere.
* Amit: You could have two dependencies in the workspace, one aliased to tock-registers-v0 and one tock-registers-v2. Remove from the kernel. And then all the chips use those dependencies. And you can find who is using which via a grep.
* Brad: That's fine. I just want it standardized in one place
* Johnathan: Why add to workspace, and not rename in the kernel crate?
* Amit: Keeping in kernel crate defers moving. Also it makes it an issue to determine when that dependency needs to removed altogether. And it would still ship a kernel with a known-unsound dependency.
* Brad: It'll still need to be removed sometime. But agreed
* Amit: Good to discuss this, but isn't this partially informed by how hard it is to port things? Should we revisit this when the new interface is merged and we're closer to actually using it in Tock?
* Johnathan: Something I'm working on is a tool that finds all register blocks in Tock. This is helpful for understanding arrays and slices in registers anyways. The progress was slow, but I think I was using the wrong tool. If I make good progress on that over the next few weeks, maybe I can make an automated port. But I'm not sure that's going to result in a reliable tool.
* Leon: Something that stuck out to me: understanding how register structs use array suboptimally now. I don't think this is viable for a port, but for just understanding LLMs are good at extracting semantic patterns.
* Johnathan: I thought about that, but I was worried that it would switch all PRs to "I used LLMs on this" and track that. An LLM would be helpful for understanding though.
* Leon: Makes sense, I can see the taint issue.
* Johnathan: So release 0.11 on crates.io. Then add tock-registers-v0 workspace-wide pointing at 0.11. Migrate all crates to that version. Delete the kernel exports. Then add a new dependency that's the new tock registers that points at main on the new tock registers, tagged to a commit.
* Johnathan: Question, do we immediately remove the old APIs from main after releasing 0.11?
* Brad: Yes
* Leon: Did we decide on a 0.11 not a 1.0? (general agreement)
* Amit: We don't want to call something a 1.0 that's broken
* Johnathan: So 0.11 then move to 2.0. Sounds good


## Removing Static Mut
* https://github.com/tock/tock/pull/4870
* Brad: In my quest to remove static mut everywhere. This PR switches ARM exception handler state tracking from static muts into assembly variables with labels, which we only access from assembly.
* Leon: The latter part is most important here. You can technically still have something be a static mut, but if you ever access it in a race-condition-y way, all accesses have to be from assembly as the Rust rules become undefined. As far as I understand, you can still have static muts in the code, as long as you don't access them from Rust.
* Brad: Oh, okay. So I could leave those alone and implement getters that use assembly to access them?
* Leon: As far as I understand, yes. A load instruction that's word-sized on an aligned address is semantically equivalent to a volatile atomic load, which is sound. You just have to do it in assembly. Sounds slightly more elegant than having a linker script dependency.
* Brad: It wasn't in the linker, but it is just a lot of assembly.
* Branden: But you have to remove static mut totally, right? For Rust 2024
* Leon: It's not the presence, it's the access. I think? Just defining is okay, but a dereferenceable reference is the issue.
* Johnathan: You can do `static UnsafeCell<u32>` instead.
* Leon: I'm all for migrating to that too. No issue. But apart from that, glancing at the PR this looks good.
* Brad: The other context here: I have branches in various states of switching crates to Rust 2024. The kernel is already done. The major change is that you have to put unsafe everywhere. I'm starting with ARM first, then I'll do RISC-V. x86 is a whole other rabbit hole. Once we have a strategy we're happy with, I'm hopeful that strategy will apply to these too

