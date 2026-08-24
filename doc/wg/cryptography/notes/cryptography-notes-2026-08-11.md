# Tock Cryptography WG Meeting Notes

**Date:** 8-11-26 

**Participants:**
  - Tyler Potyondy
  - Bobby Reynolds
  - Amit Levy
  - Hans Martin
  - Kat Fox
  - Roy Bachynskyi
  - Irina Bradu
  - Alex

## Updates
- Tyler: There was a PR from Roy adding the optional preset message length to the digest HIL (https://github.com/tock/tock/pull/5032). 
- Roy: The PR was a bit of a hot patch fix and is a bit awkward with the current HIL design.

## Driver Mutex - Reference Counting
- Bobby: Last week we discussed the downcasting and some workarounds, today I'll go through reference counting with the driver mutex.
- Bobby: As a case study, I'll go through the software CCM capsule. This is a software implementation of AES CCM which is backed by a hardware implementation of hardware ECB. It uses the ECB primitive for hardware based operations and uses this to implement a software CCM.
- Bobby: Hypothetically, AES CCM is built on a number of midlevel primitives. It stands for CTR with CBC Mac (that's the CCM). If you had hardware support for CBC Mac accelerator you could use ECB + CBC Mac for more performant implementation of CCM.
- Bobby: In this scenario, you'd have a driver mutex for CBC Mac + ECB hardware primitive. You'd have logic for retrieving smart pointers to the two hardware engines you need to build the CCM suite. 
- Bobby: Tying this back to what we discussed last week -- multimodal vs single modal hardware -- let's say one chip has distinct peripherals and peripheral drivers and on the other you have multimodal engine that supports ECB + CBC Mac in the same driver. This would be guided by a single mutex.
- Bobby: In the multimodal case ECB and CBC Mac types could boil down into the same type (this is where the need for downcasting comes from).
- Bobby: Reference counting comes into play to prevent deadlocking when there is desire for consumer to use this in a concurrent manner.
- Bobby: Without reference counting, you would need to write your driver to serialize driver accesses that go through mutexes.
- Bobby: On some platforms where multimodal hardware is present, you'd need to serialize or else you would deadlock. 
- Bobby: For instance, you have a capsule that consumes 2 AES interfaces (use perhaps for secure flash storage). For the mutex design, each could be guarded by a mutex.
- Bobby: In the multimodal case, performing aes request first then hmac request, hmac request never returns ready because first aes request is holding.
- Tyler: Just to make sure I'm understanding correctly, does the reference counter return an error when you attempt to acquire the mutex/be added to the mutex queue if the capsule/driver already is in the queue/holds the mutex?
- Bobby: No, both requests are granted and then increment reference counter. 
- Tyler: And how does this avoid the deadlock?
- Bobby: It does not add the client to the mutex queue (to avoid it being placed in the queue twice and deadlocking).  
- Tyler: So it seems this essentially shoves the complexity to the capsule. We now would have two references, one to say ECB and the other CBC in this example. If you are doing an ECB crypt operation, I imagine it would be undefined to say alter the key material or perhaps some configuration registers via the CBC reference you hold to the driver. Is the capsule needing to make sure it doesn't perform these invalid operations?
- Tyler: This seems to leak some of the hardware specifics across the HIL? 
- Bobby: Yes, this is a fundamental hazard.
- Bobby: Ideally, we don't want to rewrite the capsule for different hardware, that's what we are trying to encapsulate with traits/the HIL.
- Bobby: The capsule fundamentally needs to serialize it's hardware operations regardless of a virtualizer or a mutex, with this design, we need the capsule to serialize due to the constraints of multimodal hardware you mentioned earlier Tyler.
- Bobby: Let's say we don't have reference counting, if we have the rule that you can only hold one mutex at a time, this driver would then need to aquire ECB operation relinquish ECB then aquire CBC Mac and relinquish. In the time between here, there is a chance that some other operation from a different driver gets enqueued that results in a different deadlock. 
- Bobby: Reference counting adds a layer of complexity that needs strong justification before becoming the default implementation. For Pluton, we use it and it has worked in production to avoid deadlocks.
- Tyler: This is an interesting approach and it seems useful for some cases. It seems though that this is separate from the Mutex and in practice serves as a bit of a convenience feature to make the mutex usable (at the cost of some complexity and potential hazards).
- Bobby: Internally, we didn't use reference counting to start and ran into situations where we needed it very quickly. I'm in favor of not adding this complexity if and when it is needed. 
- Tyler: Is this code in your wip branch for the HIL changes?
- Bobby: Yes, I can share a link post meeting. 

## Digest HIL Thoughts + First Draft
- Roy: This is a first draft/rough proposal for the digest HIL. We would like your feedback on this.
- Roy: Small reminder about the problem -- Tock offers the digest HIL with const generic N that results in the HIL only being able to operate in a specific mode. 
- Roy: Removing this const generic size guarantees are transferred to the developer, which is also not ideal.
- Roy: There are currently 6 supported hashing algorithms in Tock and supporting all of them is not scalable either (in terms of placing all of them into one struct).
- Roy: We started by separating the algorithm from the digest. 
- Roy: After this, we then can split current traits into general and algorithm specific. There is a general trait, DigestAny, that would be provided to the capsule. This links with the Mutex idea from Bobby well. 
- Roy: Our main question is if we want to keep DigestData, DigestHash, and DigestVerify traits? 
- Bobby: This is a question that was touched on a few months ago. We concluded that Digest Verification is usually more of an application concern and is inappropriate at the HIL layer. So the first recommendation is that DigestVerify should be removed from the HIL.
- Bobby: Can you give a quick refresher on DigestData nad DigestHash.
- Roy: DigestHash only provides the "run" and DigestData is how you pass the buffers in.
- Bobby: Is single vs oneshot operations representing the difference between these?
- Roy: No, digest data accepts all the data and digest hash calls run once the data is loaded and receives the digest. This is used for oneshot and streaming operations.
- Bobby: Then are these traits inseperable? Are there situations where a driver implements one without the the other?
- Hans: It seems pretty rare to have one without the other, can these be combined?
- Roy: The current HIL clients/users don't seem to justify separating these. I personally have not encountered any hardware that have encountered this.
- Alex: When I looked at the PR that introduced these traits, a comment mentioned wanting to separate these to keep the privileges at a minimal level and to expose just data or hash to the capsule. I am also in favor of these being combined.
- Bobby: So to make sure I'm understanding correctly, you would separate these to allow you to just verify from a capsule and also just supply data separately. If we remain in agreement on removing Verify, it seems there is no longer a justification for keeping these traits separately. Are others in agreement?
- All: Yes.
- Roy: Continuing on, the main idea is to pass the digest token to any peripheral but to introduce a DigestMode that allows a capsule/user to set a specific mode. You would call a `verify_mode()` and receives a token.
- Roy: ClientDigestAny would not need to make any changes. 
- Roy: Unfortunately, in Tock upstream there is not any hardware that supports multimodal digest lengths. 
- Roy: Our future work is looking into designing a DigestSlice that can be shared by both client and driver with size guarantees. 
- Roy: We also want to experiment hashing drivers with the driver mutex.
- Tyler: You mentioned there not being upstream examples in Tock. Is this referring to the example code you are showing here or the PR that is open. 
- Roy: This is specifically about the example code and how passing data can be awkward and incompatible for some hardware types.
- Bobby: It would be interesting to see how things intersect with the callback driven data patterns. I think this would be applicable here as well. Rather than using a SubSlice or a DigestSlice or a raw static u8 buffer, the data movement pattern we are trying to put together is a callback issued to the client that can avoid this.
- Bobby: I'm curious how this could work for the digest hardware you are using. This would be a thought experiment that would be good to see. 
- Tyler: At a high level, the data movement is essentially at it's core altering the buffers to not be static buffers (or some abstraction of static buffers like SubSlice) but instead just passing standard u8 slices. 
- Bobby: Yes, this data movement pattern makes it much simpler as we don't need statics and also removes some of the ownership concerns that can be challenging to reason about.
- Roy: I'll go ahead and look into this.
- Bobby: One other piece of feedback that we've been discussing is how to deal with modality in the type system and how to have a single object that can support multiple variations of operations hardware can perform.
- Bobby: For a while we danced around doing this and this single object returning errorcode no support if it is unsupported. We ended up settling on wanting to move away from these runtime errors and encoding more of this into the type system. What is being proposed here for the Digest trait seems to be moving a bit against this direction we settled on where more of these errors are runtime vs in the type system.
- Bobby: It might make sense for this to be different in Digest, but I just would like to call out here that this is inconsistent compared to the AES design discussions. 
- Hans: One thing that is important to keep in mind, from my perspective, is that mutex's shouldn't model modality and conflating those two seems like rough waters.
- Hans: On the digest mode slide, you mentioned not having a large struct containing everything, it seems like the enum you propose is large and it seems the large enum just moves this large struct to the enum. 
- Roy: Yes this is true.
- Hans: I don't have targeted feedback at the moment on this, but would like to explore this a bit more to see if there is a design pattern we could use to better encapsulate this. 

## Next Steps
- Tyler: Next week, Roy will continue with the proposal for DigestSlice and also incorporating some of our existing wip data movement patterns.
- Tyler: One other note or comment that would be helpful, is confirming the longer term plan for the digest HIL, specifically in the context of the open PR adding the length setting method. 
- Tyler: Roy, you mentioned a few times throughout the call that this is very much a hot patch and an awkward fix. Is this something we want to get merged in now or wait on?
- Roy: I am in favor of just waiting for the redesigned HIL to get this change in and closing this for now.
- Tyler: What are other's thoughts with this?
- Bobby: If the OxidOS folks need this and not having this upstream is a major blocker, I am in favor of merging this since it isn't a very large change and we don't want to block on waiting for the improved interface. 
