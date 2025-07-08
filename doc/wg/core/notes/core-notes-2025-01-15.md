# Tock Meeting Notes 2025-01-15

## Attendees
- Branden Ghena
- Leon Schuermann
- Pat Pannuto
- Hudson Ayers
- Johnathan Van Why
- Kat Fox
- Brad Campbell
- Alexandru Radovici
- Chris Frantz

## Updates
 * Leon: Alistair asked to add a TickV release tag to the Tock repo, which I added. That tag coincides with the Tock 2.2 release and marks TickV 2.0.0 release within Tock.
 * Branden: After a hiatus for holidays and conferences, we did a walk-through of all the Tock networking interfaces, what their state is, what their goals are, and what the blockers are. That might be interesting for others to look at, so see the notes from that call.
 * Branden: https://github.com/tock/tock/pull/4307
 * Hudson: What was the status of 15.4 for the release?
 * Leon: UDP remains broken, and has likely been broken for a few years now. 15.4 raw is still working.
 * Hudson: If I set aside time, is there interest in UDP in 15.4? Or is that deprecated?
 * Leon: I think either is good. It should probably be fixed or officially deprecated though. You might check in with Tyler about it
 * Hudson: I'll put that on my list then


## Syscall Drivers Testability
 * https://github.com/tock/tock/issues/4303
 * Alex: We need a testing framework for capsules and need to see what the capsule returns.
 * Alex: We need unit tests for sure. A problem with them is we can simulate things but we can't extract the CommandReturn's contents. The proper function for transforming it is within the kernel crate. So we can't use it outside the kernel crate
 * Leon: I can speak to why it's like that. For 2.0 we wanted CommandReturn to be separate from SyscallReturn. Command returns come from commands regardless of system.
 * Johnathan: Command can only output a subset of the syscall return variants (e.g., it shouldn't return pointers). CommandReturn intentionally hides that it wraps a SyscallReturn so it can guarantee that it only contains variants that Command can return.
 * Leon: You might need a functionality for testing that allows you to recover the internal enum
 * Alex: How would we do this? cfg(test) doesn't work because the capsule is in a different crate from the kernel.
 * Johnathan: I don't see any reason CommandReturn can't have public functions that introspect it. It seems to me it could support the same API as the libtock-rs CommandReturn.
 * Alex: Adding them seems plausible
 * Leon: That could be inconsistent if we had multiple different CommandReturns that mapped to the same SyscallReturn. But that doesn't necessarily make sense if we're shipping to userspace anyways.
 * Johnathan: If they're indistinguishable in userspace, they could be the same in capsule testing. That's fine
 * Leon: A concern if we have multiple CommandReturns. Let's say we have success A and success B and they both map to the same SyscallReturn encoding. We might not be able to distinguish them. This isn't something we have right now, but we didn't promise that there wouldn't be two different constructors.
 * Johnathan: Really CommandReturn is just the subset of SyscallReturn that we allow commands to return. So I don't think it's an issue
 * Leon: We could also promise that CommandReturn is always a subset of SyscallReturn and that would preclude the issues. Is that true right now? I'm not sure of it
 * Alex: I think it is
 * Leon: Currently it's a uni-directional mapping, you can't go back.
 * Alex: If I can test it as if I was in userspace, I don't care. That's sufficient for us
 * Johnathan: The actual implementation does guarantee uni-directionality. The comment doesn't guarantee it, but it always felt like the intent.
 * Leon: I'd be fine adding that constraint to the comment. Is there a way to mechanically enforce it?
 * Johnathan: I don't think there's an easy way to make sure we don't implement something that maps two to one. But I don't think it's so important. It's not a safety issue, just an understanding one.
 * Leon: So we'd add corresponding public is_success or is_failure methods. That doesn't fix getting the associated values
 * Johnathan: My proposal is to use the libtock-rs solution: https://github.com/tock/libtock-rs/blob/60c256168b965eb55d1ba4eeaa47f67c67bdb319/platform/src/command_return.rs#L73
 * Johnathan: We don't need the to_result, but the is_success_u32 and get_success_u32 seem basic. And it would be nice to have the same API in the kernel and libtock-rs
 * Alex: That's good for us
 * Branden: Alex, you probably don't have a full view yet, but this this issue is really that the kernel crate and capsules crates are separate, so you're considering which things are exposed from the kernel crate. Do you expect to run into more issues beyond this one example?
 * Alex: For sure. We'll have various PRs about this. Not entirely sure how many things we'll run into yet. We'll put a hook to see system calls coming in, but there's no hook upon system call return. And we'll need that as well in the process debug trait. There will be all sorts of things here
 * Branden: So this will be a continuous process of reconsidering what the kernel exposes
 * Johnathan: That's my experience too. The more testing you do, the more hooks you'll end up with for dependency injections and complete APIs for reading data types. Tock has a lot of types that are just never read, so we never developed them. For the most part, once you learn how to do dependency injection, it sort of becomes robotic to make a codebase more testable.
 * Alex: We could always fork it and have a custom kernel. But we do really want to upstream this
 * Johnathan: I would really like to see testability improvements upstream
 * Alex: I do foresee modifications to the kernel too, for testing upcalls and deferred calls. For now we're focusing on simpler capsules
 * Johnathan: I'll make a comment on that issue


## Stale PRs
### Non-XIP Document
 * https://github.com/tock/tock/pull/4081
 * Pat: Is this still waiting on external feedback? Should we merge the document and have updates go to that?
 * Leon: We never had anybody who actually uses XIP give us feedback beyond the initial TockWorld discussion. So this is a reflection of what we discussed, but we really wanted a domain-expert to look at this before we merge it
 * Pat: I think that last commenter uses XIP and gave a use case. And it feels like this isn't a priority for external people
 * Brad: Part of me wants to say this is just not done. But maybe it's better to have something to point at that's really in the repo. And if people find it "wrong" they can hopefully submit a PR.
 * Branden: It's nice to have something to point to. I guess a PR could do that as well as something in the doc folder
 * Pat: Discoverability of docs is better than of PRs. I'd vote that we merge this (general agreement)
 * Brad: I looked at it real quick and there's nothing flagrantly wrong right now. Seems fine
 * Pat: I'll merge that after the call
### ReadableProcessBuffer Trait
 * https://github.com/tock/tock/pull/4231
 * Leon: Thanks for reminding me about this. It wasn't on my todo list, but comments here are now on my list
 * Leon: This is something we want to have, and it addresses a downstream concern
### TRD104 Explicit Type Summary
 * https://github.com/tock/tock/pull/4228
 * Pat: This really just needs to be on my todo list
### DFRobot Rain Sensor
 * https://github.com/tock/tock/pull/4233
 * Hudson: No opinions here right now. I could do an actual review of this
 * Pat: The most important thing is probably just the actual update to the HIL
 * Hudson: I'll take a look at this
 * Alex: The thing that draws my attention here is the user features
 * Pat: That's just within the board file itself, and I think we're okay with that
### MachineRegister type
 * https://github.com/tock/tock/pull/4250
 * Johnathan: This might be worth a conversation. My opinion is that this needs to document provenance handling, and needs to follow some model to be sound and functional on CHERI
 * Brad: Responding to a recent comment, I'm still confused, but I think we're making progress. I think it boils down to at some level being complicated to balance: Rust, the future CHERI prospects, and a computer-engineer/hacker view of hardware and low-level software. So, I don't know what the right answer is, but whatever answer we land on we should be able to defend and describe for others in the future
 * Johnathan: Yeah, I wanted to make sure provenance made it in there for that reason. It's painful and not everyone understands it. C/C++ really discovered this in the early 2000s and still haven't described it in their standards
 * Johnathan: I could make a PR against your PRs with some updates. Would that be helpful?
 * Brad: That would certainly be helpful. Provenance is a property of a pointer, right? And MachineRegister doesn't necessarily hold a pointer
 * Johnathan: Correct, but it _might_ hold a pointer
 * Brad: So that's where I get stuck
 * Johnathan: So if there's a pointer in MachineRegister, we should maintain provenance. Something that only stores pointers always has provenance. And something that never stores pointers never has it. But something like this that can hold both gets really messy. Something that could convert either way sets off an alert in my mind about handling provenance in the casts. In my opinion, the correct answer is if you convert a pointer to a MachineRegister it should have provenance
 * Johnathan: I'll make some time to do a draft of that in text
 * Brad: It seems like what you just said is in line with what I'm thinking and feels logical. The question is how to implement that in Rust
 * Johnathan: The most important thing is documentation. There is a Rust way to do this...stabilized last week. Some of the functions got recently renamed during stabilization. We want to avoid some things that won't compile on CHERI which is on a different Rust version. If there's a function we really want to use, there's a backup with an as-cast with a comment about our limitations
 * Brad: That sounds like a great path forward to me
### MSRV Check with Cargo-hack
 * https://github.com/tock/tock/pull/4278
 * Pat: There was an extended discussion on this a few weeks ago, but then it kind of stalled
 * Brad: Right, I think we decided on a path forward for the release. The issue is that this PR doesn't build, so we have to decide if it works and how to test it
 * Johnathan: I think your implementation is fine. I don't love it conceptually, but it's not worth the fight, so we should merge it
 * Brad: Maybe it just needs a rebase. Let me do that real quick
 * Johnathan: I think we were just waiting on CI
 * Leon: CI is complaining about a Rust version missing in some Cargo.toml files. We might need to add that to all crates
 * Johnathan: That's gonna need a fix. But we should move forward after that
 * Brad: Also on this, getting Pat's eyes would be great. The diff looks pretty small but maybe there's a better way to integrate it into the Makefile
 * Pat: I'll skim it

