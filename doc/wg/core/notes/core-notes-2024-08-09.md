# Tock Meeting Notes 2024-08-09

## Attendees
 * Branden Ghena
 * Amit Levy
 * Leon Schuermann
 * Lawrence Esswood
 * Hudson Ayers
 * Ludovic Vanasse
 * Brad Campbell
 * Alyssa Haroldsen


## Updates
 * Ludovic: Embedded software dev in Montreal. Working on a startup building altitude platforms for detecting weather or fire events. I'm evaluating platforms for the microcontrollers in the project.
 * Johnathan: No longer on OpenTitan team. I'm in ChromeOS security. Still have some time to work on Tock, but a little reduced
 * Hudson: Amit merged my initial memop syscall PR to libtock-rs, so heads up to Jonathan. Still working on dynamic allocation and hoping to have something people can look at soon.
 * Johnathan: I was aware of it. The comment guessing at my thoughts on the issue were correct.
 * Lawrence: I ended up calling to C malloc in my libtock-rs implementation. At the end of the day, the Rust code is pretty darn unsafe.
 * Johnathan: It's actually more about how it's simpler to build a rust-only system then compile both Rust and C
 * Leon: On the CI front, things are moving along. We should soon have the platform in the state where we can have some users writing tests. We're still working on usability and a CI tool to request a board. 
 * Amit: I just opened a big draft PR I've been working on that changes the SPI TRD to allow chip select lines to be either active low or active high. Right now we assume active low always, which is 99% true, but I found a platform where it isn't. This PR adds polarity of the CS line as a configurable parameter. It has little changes to everyone who uses SPI, but seems like a good change. We'll have to update the TRD as well.
 * Brad: I'd be happy to add that to the agenda if you want to talk more about it.
 * Amit: Wasn't sure I'd have a draft PR by the meeting originally
 * Branden: Note about a post in the general channel in slack. Someone working on libtock-rs apps on the Nucleo and having issues. It'd be great to get a response if anyone with knowledge has time.


## Outstanding PRs
 * Amit: the main stale PR right now is checking in on the process slice buffer API. Alex's PR: https://github.com/tock/tock/pull/4023 I think this is waiting on author?
 * Brad: Yes, that's more or less correct. The bigger value there has been the exploration of the design space. I think we've come to a spot where we have a design in mind that would work very well and just needs to be implemented. Anyone could implement it. Shouldn't be that involved. Doesn't require changing the kernel.
 * Amit: It's just something that individual capsules would adhere to?
 * Brad: A capsule could choose to include a library to manage a data structure
 * Amit: Okay, so really we could think of this PR as a design that anyone could pick up. Let's put a comment on the PR that spells that out.
 * Amit: We don't need to discuss the VeeR PR https://github.com/tock/tock/pull/4118 as the author just responded that they're going to work on the feedback.


## CHERI Upstreaming
 * Amit: Most people know that Lawrence et al. have been porting Tock to a CHERI capability-based platform, and the hope is to upstream that. Lawrence has done the work of extracting the non-proprietary stuff into three repos in the Tock org
    * https://github.com/tock/tock-cheri
    * https://github.com/tock/libtock-c-cheri
    * https://github.com/tock/libtock-rs-cheri
 * Amit: I also made a tracking issue on the upstreaming here: https://github.com/tock/tock/issues/4134
 * Amit: CHERI is different from existing systems in Tock in a lot of ways. So upstreaming will be a big dump of a lot of orthogonal additions that might be hard to think about.
 * Lawrence: Some background: CHERI frames itself as an MPU but also has some MMU implementations in there.
 * Amit: And one of the main differences is that CHERI pointers are not word sized, usually twice as big. And in the Tock code base, we often assume they are usize or `* const somthing` or whatever. So the first step in the port makes a new type to hold pointers, that's either usize or holds a CHERI pointer. That needs to be placed in all the right places in Tock that interface with userspace and exchange pointers. So that could be the first thing, to clean up the various interfaces where pointers occur. It should have no functional difference for non-CHERI platforms.
 * Hudson: How much does this work for CHERI also simplify the general 64-bit support?
 * Lawrence: I was doing both at once, actually 64-bit AND CHERI in one motion, because it really was the same problem.
 * Amit: We do implicitly assume 32-bit throughout Tock and almost certainly confuse usize and u32 in various places. So being precise about what we really mean in all cases would advance support for both
 * Brad: I'm not sure I understand. The usize and u32 makes sense. But the places where we're using an actual pointer type like `* const something` shouldn't need to change right?
 * Lawrence: There are bare addresses and addresses plus metadata. So we need to be clear about which one each pointer is.
 * Brad: Okay. So the next question, which we don't need to answer right now, is why not just only use one? And how would people reason about which one to use if they had the choice?
 * Lawrence: I think we could use metadata everywhere, but there are things like break water marks that are just numbers and don't need authority.
 * Leon: On a more practical note, we have a CHERI QEMU target, so we should be able to test if people are doing the right thing with CI
 * Brad: Something I'm wondering about, is that the type is a different size and should be generic, and that the type has metadata or not. So we might have sort of four options here.
 * Lawrence: Using a pointer with metadata everywhere would be fine but just waste space. Using a bare pointer everywhere would cause failures when attempting to dereference.
 * Amit: My view has been that pointers with metadata really sort of mean something different. They're pointers that userspace cares about being related to some existing object, so there should be rules about how the kernel interacts with it. And we should be careful to only manipulate these pointers while being in the same object. So in a sense this is differentiating pointers we have to be careful with versus pointers like memory breaks that we can move around.
 * Brad: So encoding that semantic understanding in the codebase is the first part. Then by being generic and explicit, it should be easy to make these the right size.
 * Lawrence: Also this happens to be the size of the register file in CHERI
 * Amit: Okay, then there are some changes to the MPU trait that would provide hooks for bounds on function pointers
 * Lawrence: And alignment related things. Both MMUs and CHERI have very different alignment requirements than general RISC-V MPUs. There are weird rules in CHERI for alignment, for example, based on region size.
 * Amit: So we should decouple that from the pointer thing, as it's more specific to CHERI
 * Amit: And then actually supporting a CHERI architecture would involve toolchain stuff. Neither LLVM nor Rust-C have upstreamed things.
 * Amit: Finally, there's stuff in userspace that it felt like we could decouple. Having stuff in the kernel could come first.
 * Brad: It would be fantastic to figure out the meaning and semantic and size of different pointers. That feels like a real step forward.
 * Leon: I agree. Tripped over this trying a 64-bit port a while ago and it was a mess
 * Hudson: How far along is Lawrence's port now? I imagine he hasn't touched all of the capsules
 * Lawrence: I think that because everything comes through the grant interface, capsules should be good already? Libtock-rs needed lots more changes. Libtock-c is worse and hasn't been updated much yet. Right now we're just lying and passing pointers that C doesn't exactly expect. The biggest thing for all of these is that pointers shouldn't be treated as u32s
 * Hudson: I do suspect there might be some old capsules that are messed up. I think we'd need to read through all of these manually.
 * Lawrence: In libtock-rs many of these are type errors, which I fix by casting back to u32 to make things happy and lose metadata
 * Leon: One of the things I'm worried about in CHERI is that because it's not upstream in a compiler yet, and hardware isn't readily available, is that we need to have a CI workflow immediately to test for issues and as an example of how to set stuff up locally
  * Lawrence: I do have a set up steps for what to do to update toolchains


## Zero-copy Grants
 * Amit: In the port to CHERI, there's stuff Lawrence and company did to change grants. One non-controversial and one maybe-controversial
 * Amit: Lawrence, would you be willing to explain the non-DMA related zero-copy grant stuff?
 * Lawrence: The idea is roughly smart pointers that point into userspace that also point to a liveness tracker for safety. So when a process dies or allow changes, the pointer becomes dead. And the type system would have options for turning that into a pointer that can be used ephemerally between system calls, while the app can't change. This actually allows a bunch of state machines to be removed and simplify things.
 * Lawrence: This changes types from grants to include shared references instead of owned ones. You can still use the legacy version, but exclusively with the new version. One or the other. This also fixes the double entry issue with new types.
 * Amit: Do you have a concrete example off the cuff?
 * Lawrence: Being able to pass arbitrarily sized buffers from userspace to a driver without having to copy them, or write that logic. Because the pointer is a part of a smart pointer, you can do things like check alignment and it can never change underneath you. So checks about whether the length of the buffer have changed aren't necessary. So you can have a smart pointer to more complex types and trust that they'll continue to be aligned in that way. Or you can use one allow slot to pass in multiple pointers.
 * Leon: I'm slightly confused. There's a smart pointer idea and also we're changing the grant type to be immutable?? That's two separate things, right?
 * Lawrence: It's because they're shared things. There used to be a one-bit lock for entering. Now any of them could be used at any time.
 * Leon: But effectively, they're two separate API changes that might be rooted in one architecture change.
 * Lawrence: Yes
 * Leon: So, we have this concern about buffers and this concern about the type T we place in grants. And you're proposing changing both of those APIs, but they're separate changes, right?
 * Lawrence: They're both memory tied to the lifetime of the process. Allows just move more frequently than app lifetimes
 * Leon: Is there a proposal for this yet?
 * Amit: It's in the Tock CHERI repos, but not pulled out yet
 * Leon: Cool. I just want to find something concrete to look at that's not buried in massive unrelated changes


## DMA Capable Grants
 * Amit: And the more controversial one. The goal is to be able to take a large buffer from userspace and stick it directly into DMA. This has always been forbidden, so how does this work?
 * Lawrence: It's a capability-locked interface that gets a reference-counter interface via refcell. It will leave the application as a zombie until the DMA has finished. So then we could safely know that the memory will still exist until the hardware is finished. Right now I'm just intending this for DMA use cases
 * Amit: I think the mechanism here is straightforward. This does break something we always cared about, that if a process runs out of grant space we want to clean up the process right away. So this is in a sense dangerous if a capsule never cleans up grant memory. The way that this becomes okay is that the ability to use grants like this is gated behind a capability. So only "trusted" capsules could do this
 * Leon: This does fundamentally change our trust level of capsules. Presumably some upstream capsules would require this. And it's unlikely that we'd bother with two capsule version where one copies and the other is zero-copy. So this does make me skittish, as we discussed this for Tock 2.0 and decided we didn't want this.
 * Lawrence: So maybe Tock core capsules should be forbidden from using this
 * Leon: Although if we don't use it for core capsules, who's going to test this and make sure it works for upstream?
 * Leon: I do think that if we were to introduce this, we'd have to think through the consequences carefully.
 * Brad: How does the capsule know if DMA is being used?
 * Lawrence: The HALs I have needed to change to requiring a DMA-able reference.
 * Leon: It's not required to use DMA. Even a driver that copies one byte at a time might hold onto a queue of pointers for buffers
 * Lawrence: Generally, when a driver needs to hang on to a pointer in a way that's not quickly stopped

