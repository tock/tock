# Tock Meeting Notes 2025-07-02

## Attendees
 - Branden Ghena
 - Hudson Ayers
 - Alexandru Radovici
 - Leon Schuermann
 - Johnathan Van Why


## Updates
### Naked Functions
* Hudson: PR for bringing back naked functions now that Rust 1.88 has been released. One thing that came up on the PR was Brad pointing out that back when we had naked functions, each of them would have exported names that matched the function names with a macro. It seems that just marking the function as naked is sufficient now, so I'm going to not keep the old macro. But I wanted to check if anyone here remembered why we needed it at the time?
* Johnathan: That could have been something stabilized within naked functions
* Leon: I think we used to use the symbol names literally in the assembly text. That's been removed as part of ongoing refactors. Same for Cortex-M where we have a trait that defines those functions. So proper rust functions resolve names
* Hudson: Okay, I'll go ahead and remove those then.
### Network WG
* Branden: Network working group update is that we're discussing IPC for the rest of the summer. We had a meta-discussion about it and decided we want to try to come up with a high-level design with interfaces and goals to bring to the group. The plan for next meeting is to have a discussion about the current design of IPC, IPC designs in different OSes, and industry requirements.


## MMU Support
* https://github.com/tock/tock/pull/4465
* Branden: Big PR on MMU support that touches a lot of stuff in the kernel. This is the kind of intensive PR that lingers quietly for a long time because it's a lot of work to consider and comment about. So I want a path for considering it instead of that.
* Alex: For background, we ported Tock to ARM64. It boots on a RPi and works! And that needs MMU. We can't open-source the ARM64 port, but we want to open-source what we can and stay in sync with upstream, so we're using x86 as a guinea pig.
* Alex: We could use a simple MMU like Microsoft does, but we want paging to allow Tock to scale from smaller to large chips. This is particularly meaningful as chips are just going to keep improving moving forward
* Alex: Having paging could also support a way more capable IPC system with memory sharing. The use case is that many USB chips for AI processing don't need Linux but also don't have another choice. Tock could be a certifiable OS that could be useful for them. This could be a future selling point for Tock.
* Alex: For performance, Tock isn't competing right now. Slower than Embassy or RTIC. But we could do way better and more deterministic than Linux with support here. We hope anyways.
* Branden: Design document
* Johnathan: That's a good idea. We had a PR changing the x86 ABI and when the Pluton developers originally sent code for x86 implementation we merged it without a TRD. But then there was the recent PR that changed the ABI and there was no documentation requiring things to be a certain way. Without documentation, it's unclear what the goals are.
* Johnathan: When I hear that Tock is running on bigger systems with more resources, I worry that you'll generate conflicts with different projects having different priorities. Writing down how to handle differences and how to improve Tock without hurting things that already exist
* Leon: I agree with that. This PR is really awesome, but it's a lot of invasive changes which are likely to totally break things for existing users.
* Leon: I'm actually worried that just writing a TRD based on the existing implementation might ignore the high-level considerations of how virtual memory could work and how it would change Tock design constraints
* Alex: My goal here is "don't pay if you don't use it". I haven't closely followed this PR. We do want to use Tock without penalties
* Leon: That makes sense and aligns with the PR's stated goals too. I'm not focusing on size or performance overheads, I'm mostly concerned about cognitive load and churn. Even assuming a zero cost implementation in size and performance
* Alex: Can you explain?
* Leon: Tock has always had a flat user space between kernel and userspace, which has manifest itself in many implementations. For example ProcessBuffer assumes there is not difference in addresses and that there is a linear equivalent in memory from the process. That could break if we introduce paging. Those kinds of changing in reasoning everywhere is a possible cost.
* Alex: Okay. For capsules this should be transparent. For the kernel it's a thing though.
* Alex: What I would like to get out of here is "what is a roadmap to start discussing this?" Just the PR isn't enough
* Johnathan: We need a medium for discussing this. Some kind of design doc is probably the right thing. Maybe we could produce concerns we might address in that doc, rather than just having the PR author guess at it.
* Leon: Agreed. Amit wanted to kick this off by getting all stakeholders together and talking about at a high level what the benefits, problems of this change are. One of the first things we need are experts of subsystems this would touch to get together and compile a list of requirements and potential issues to reason about at a high level
* Branden: Example of a different design we could discuss. We could have MMU translation, but still require apps to have contiguous memory. That would allow them to be position independent but also allow Tock to assume that their memory is contiguous. I'm not saying we for sure want that, but it's the kind of thing we might weigh the pros/cons of.
* Leon: Agreed. That's the kind of thing we need to discuss in a design document
* Leon: Another thing I want to raise from Johnathan's point is that this PR is highly localized to the kernel. There are a few other changes, but mostly not major (a few might be). To me this raises a question of the guarantees the kernel makes to the other parts of the system about how it ties things together. What this proposes is an evolution of the kernel. So I'm wondering whether this should go hand-in-hand with a clear design about what interfaces the kernel ought to provide. And maybe this could even live as a separate kernel implementation.
* Alex: I'm afraid it would diverge a lot.
* Leon: Really for now just reasoning about which interfaces the kernel provides could inform us on which changes are appropriate and which aren't.
* Alex: So far I'm seeing that we should make a list of concerns. And a list of interfaces the kernel provides. These would be prequels to making an MMU design document. I can make an issue out of this. I want to make an issue out of this and have a clear path on what steps to take. Even if we don't have MMU stuff in the end.
* Leon: I think getting a group together about people with MMU thoughts would be a great start too. I think that's Alex, Amit, me, and some others. My concern is that we haven't had an authoritative group about how to go about things. We need to have a meeting or email thread where we give everyone a chance to chime in about kicking off these efforts and decide on path.
* Alex: What about an issue? That records the discussion.
* Leon: That's fine. I just don't want people chiming in way later saying "oh no, this isn't what we want".
* Alex: We just opened the PR for now, but we're totally happy to start a process and consider how to get to a better result
* Branden: To play devil's advocate, I could see the argument that we don't want MMU support in Tock. It adds a lot of code and a large surface for attack that won't be exercised on our most-supported and widely used chips.
* Alex: I do see that too. I see the Cortex-R which is like an ARM64 but without an MMU. It's otherwise quite similar to Cortex-A. They even have chips that do both with deterministic interrupt handling. So it's possible the next generation of Cortex-R has MMU capabilities. Just a hunch
* Branden: Okay, so action items. For now Alex will make an issue to discuss MMU design. We hold off on following up on the PR for now other than to link to the issue.
* Alex: I'm perfectly comfortable taking time to work on a path towards MMU support, even if that isn't this specific PR. I just want to make sure that we keep moving down the path of considering it fully and not let things linger without discussion.

## Cortex-M33 Merging
* Alex: We have an implementation and want to merge this upstream, but it has a different MPU. We want to implement the ARMv8 MPU which is different but not that drastic. So I'm wondering if I should open a PR on it. The RPi Pico 2 and an NXP chip.
* Branden: What's the question here?
* Alex: Should we open a PR or start with a design document?
* Branden: Are the changes internal to that chip/arch?
* Alex: I think there's a change in ProcessStandard as that allocates memory.
* Leon: I think the MPU region allocation is in arch. So I think that's all internal.
* Leon: Be sure to fork from the tip of Tock. There were some things shown broken with verification that we fixed.
* Branden: So if this is self-contained, I think that's okay to start with a PR.
* Alex: Okay, we'll add a Cortex-M33 arch PR with the MPU changes

