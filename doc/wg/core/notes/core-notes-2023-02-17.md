# Tock Core Notes 2023-02-17

Attendees:
- Branden Ghena
- Amit Levy
- Johnathan Van Why
- Philip Levis
- Hudson Ayers
- Alyssa Haroldsen
- Leon Schuermann
- Brad Campbell


## Updates
### Hudson
 * Hudson: We merged a few PRs pretty quickly this past week to support Alex and his RustNation tutorial. All reasonable. One of them: adding reset to the processconsole is maybe worthy of further discussion. The RP2040 doesn't have a reset button, and they didn't want to have to unplug/replug all the time, especially with virtualbox and USB passthroughs.
 * Phil: I had to do this for RPis. USB is robust, but it's not that robust. Easy way to break things.
 * Leon: Short remarks on the PR, the question is whether the reset function is actually safe.
 * Hudson: It's memory safe, I think. Just shouldn't be exposed to capsules.
 * Phil: Seems like a clear case for a "capability"
 * Amit: From a process console standpoint, if you can restart applications, then restarting the whole board is similar.
 * Hudson: We're going to open a new PR after RustNation to talk about stuff more.
### Leon
 * Leon: I'm working on instruction tracing infrastructure for RISC-V on Tock. This is on LiteX now, but should work on OpenTitan stuff too. Should allow us to dump instruction traces from real running boards, including registers and stuff. Could work with context switches too! Unfortunately not compatible with ARM, but hopefully a very useful debugging tool.
 * Amit: The state is that it ran all night long, generating more data then the workstation could handle. Couldn't trigger the bug though.
 * Branden: What bug?
 * Leon: There was a race condition in the RISC-V process switching code. We're hoping to detect these in a more automated fashion.
 * Hudson: Does tracing affect timing?
 * Leon: No, although it can slow the simulator based on disk latency for storing results.
 * Phil: You might reach out to Dawson, as he does stuff with bare-metal OS development for his classes. He might be good to talk to about this stuff. He is very interested in detecting bugs, and going deep on hardware.
 * Amit: Good idea, we'll reach out.
### Alyssa
 * Alyssa: The function pointer thing for dynamic dispatch. I plan on next week finishing an implementation that would work for any of our traits. Look out for it.
 * Hudson: So the benefit for arbitrary traits is that we don't have to track a bunch of things in the vtable like destructor that we don't need.
 * Alyssa: Hopefully it should always be inlined. Also looking into static vtables.


## Proposed Edits to TRD01
 * Hudson: Current text of TRD01 says any changes must deprecate the document and create a new one. But something frustrating is that changing syntax of something unrelated requires deprecating the TRD in order to update the code example.
 * Hudson: I had to do this for the Time TRD, because it had an example of a deferred call and we changed that API.
 * Hudson: Phil might disagree with me. But I think there are some clear spots where the current approach has issues. We link to TRDs all the time, and deprecating and replacing keeps these links to outdated documentation.
 * Phil: That's the moment in time though. You could end up with issues talking about a TRD, but the linked document doesn't match the discussion. It could cut both ways.
 * Hudson: You're right. They could end up linked straight to a code example that's deprecated and not notice.
 * Hudson: Generally, there's a clear history of the document in Git. It totally makes sense to deprecate and change if we're changing the API, but for unrelated example code, that seems bad. We've had people implement things based on the old time TRD before, because they didn't realize.
 * Phil: People don't read. I do see your point, but it's a question whether the documentation is the current state of the system, or if the documentation is a point in time. In order to see the history and prior versions, should I need a Git repo instead of just needing the documents.
 * Alyssa: Git does store that history though. And having multiple copies of documents is less elegant than having links directly to specific versions of the document on certain commits.
 * Hudson: I think Phil's argument is that tooling like Git changes. So you don't want history to rely on it.
 * Alyssa: We'll move our history to that new system.
 * Phil: The issue here is that if documents should never be finalized because people don't understand which version they're on...
 * Alyssa: I do get that, but for unrelated changes, it seems good to make those in place. You could keep a changelog in the file.
 * Phil: That does go back to the "people don't read problem". This is based off of RFCs and Python Enhancement Proposals. Maybe Tock changes more in part because of evolution of Rust and our understanding of how to use it. So maybe that's a bad model for Tock.
 * Hudson: I think it's reasonable that none of the rules in the TRD are changed once the TRD is finalized. I just think that for code samples, I think some of the finalized TRDs have non-compiling code in them, which seems bad.
 * Phil: Let me toss out an alternative. I think part of the challenge is that the code in Tock changes and improves in a way that's rare in systems. So why don't we do versioning? So we could have multiple versions of TRDs, and the most recent link points to new versions.
 * Hudson: How does that work?
 * Phil: You keep file copies for old versions. Maybe this is an age thing, but the idea that I must use Git to see history feels wrong to me, like no one will actually do that.
 * Alyssa: On the other hand, will people not submit changes because the process is too hard?
 * Hudson: Actually, people do submit updates and don't update the TRDs.
 * Phil: It's the code examples that touch other parts of the system.
 * Hudson: It's hard to check that the code examples in the documents compile.
 * Alyssa: We could do doctests
 * Hudson: We end up having to ignore more things that are architecture-specific.
 * Alyssa: Unit tests everywhere!
 * Phil: The reality is that the TRD stuff was copied from TinyOS, which had a very different development model. We'd write a TEP in TinyOS and it would persist for twenty years. That is NOT the reality in Tock.
 * Hudson: I will open a PR with proposals
 * Branden: I do strongly want to have Finalization remain part of the idea
 * Phil: I do see the issues here. I think you could have different versions JUST for code API changes.
 * Hudson: I am opposed to new copies of files. I am okay with some kind of change in the document to signify that a change has been made. But I don't want old stuff hanging around. That way we don't have links to possibly outdated docs.
 * Phil: My point is that you would have copies of prior versions. So the duplicate is only for deprecated/outdated files. Maybe related to releases.
 * Hudson: Okay, I'm not so opposed to that.


## Split Capsules
 * PR #3396 https://github.com/tock/tock/pull/3396
 * Leon: We have talked many times about separate capsule subcrates. We oscillated between which kind of crates can pull in external dependencies. Our solution that we've been converging on is that different subcrates of capsules could pull in dependencies, so you could depend on a set of essential core capsules, which should never need external dependencies. Then also optional capsule crates which might have dependencies you have to check on. Crypto is one example. Or TCP network stack is a good use case.
 * Leon: Hudson proposed an initial division. Instead of nit-picking how many crates we want, for now we can just do core and not-core (extra). And we can pull things out of extra if it makes sense to separate them because they become core or rely on dependencies.
 * Leon: So this PR implements that division. The only remaining question is where virtualizers should go. A separate module of the core crate, or not?
 * Hudson: I think a separate module is nice. But I don't feel super strongly
 * Amit: I lean in the same direction. I like namespacing. But it seems like a "whatever" decision.
 * Leon: The reason to bring this up on the call. This PR touches 300 files and is a nightmare to rebase. So we should decide and then merge very quickly.
 * Brad: A question. It is `core_capsules/` right now. Could we do `capsules::core::`?
 * Amit: I don't think so. We can't have namespaced crates like that. Maybe we could have a capsules crate which re-exports crates, but that would remove the benefits of this change. We could just call it core and extra. But there's a global namespace of crates.
 * Hudson: For example, we CAN'T use core as a name
 * Leon: Yeah, we talked about the core kernel crate, and more specific names seemed better, even if verbose. I don't care about the exact name, but I don't think we can more reasonably name it
 * Brad: Let's flip the words then so it's sorted reasonably
 * Leon: The folders are already called core or extra only. Folders are hierarchical like that. For the term, I can do a find/replace to make it `capsules_core/`.
 * Brad: So this name is weirdly board specific. Boards could rename it.
 * Leon: The cargo.toml defines the name, then the cargo.toml in the boards pulls that in
 * Brad: Thanks, that makes sense. So the bigger question for me is just the folder name, and then what boards do.
 * Hudson: I do think that "capsules*" would be good, so the imports are adjacent in the files
 * Branden: Is anyone against making this change quickly?
 * Phil: I agree with it.
 * Hudson: Should be safe. If it compiles, it almost certainly works.


## Success vs Success32 for Command Zero
 * Issue #3375 https://github.com/tock/tock/issues/3375
 * Hudson: This came up in the PWM PR. We have these capsules like GPIO, LED, and ADC return Success32, as they declare the driver is present and the number of resources available. The number of pins, for example. This technically violates TRD104, which says they should return Success. https://github.com/tock/tock/blob/master/doc/reference/trd104-syscalls.md#431-command-identifier-0
 * Hudson: This mistake prevents us from generically checking for whether a driver is present. We don't have a use case for that command right now, so maybe it doesn't matter
 * Hudson: A couple options: we could change these drivers, but they're stable interfaces, so we'd have to bump Tock's major version. And this doesn't seem important enough for a Tock 3.0 bump.
 * Hudson: Another option would be to change TRD104 to not make this statement. That's finalized too, so we'd have to do something there.
 * Hudson: Pat also suggested that we could add a new driver number for every capsule that does this, and maintain both versions. I think that's not a good idea.
 * Phil: Yeah, it looks like LED was Leon and I and we just weren't careful. It think it's fine to just say we're sorry about it, do the right thing moving forward, and update them on the next major revision. It's a consistency question, it's a wart, but it's not hurting people.
 * Hudson: Sounds good to me
 * Phil: Going forward, should definitely follow TRD104
 * Hudson: Where do we put this?
 * Branden: Tock 3.0 initial issue!
 * Phil: Also a comment in the files noting that it's out of compliance
 * Johnathan: A label for "fix in Tock 3.0" too
 * Phil: I just want to avoid someone assuming that they follow TRD104. So some documentation seems enough
 * Alyssa: Is there a route that doesn't involve changing major versions for changing TRD104 to allow success with any variant?
 * Phil: I think it would be to make a new LED driver with a different API
 * Branden: But if it's just inconsistent, and not hurting anybody, just leave it for now seems fine.
 * Alyssa: Could we change TRD104 though to broaden possible returns?
 * Phil: The idea of Command Zero was just "are you present". Not information too. That it was overloaded was a little messy.
 * Alyssa: I was thinking that maybe the intention should change, since someone made this mistake
 * Phil: I think it wasn't intentional in this case.
 * Brad: Notably this doesn't matter in Tock 2.0 for libtock-c. https://github.com/tock/libtock-c/blob/master/libtock/tock.c#L693
 * Brad: I think just documenting it sounds good for now
 * Phil: We could also maybe just look at multiple return variants and detect success anyways.
 * Johnathan: We could maybe add it to libtock-rs. Gotta check exact wording of TRD104
 * Hudson: It would need to be at a different layer in libtock-rs to implement it.
 * Phil: `is_success` in the kernel already returns true for any success type. https://github.com/tock/tock/blob/2be6fdb00fc4d34f4902746a9e8360abc8a0447b/kernel/src/syscall.rs#L325
 * Johnathan: It's different in libtock-rs right now. Actually, according to TRD104, the libtock-c implementation is buggy right now. See section 3.2, return values. There's no promise that future return values aren't up there.
 * Johanthan: Although, we can probably rely on it in libtock-rs since it would already break libtock-c
 * Hudson: Okay, I'll open a PR that adds comments, and I'll make a note that it's broken right now on an issue.

