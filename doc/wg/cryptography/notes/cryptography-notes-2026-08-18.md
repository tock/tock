# Tock Cryptography WG Meeting Notes

**Date:** 8-18-26 

**Participants:**
  - Tyler Potyondy
  - Amit Levy
  - Hans Martin
  - Roy Bachynskyi

## Updates
- Amit: Moving forward, we should have a call for agenda items prior to the meeting to help structure the meeting.
- Tyler: This is a good idea and would be useful now that we have a larger group with more discussion items. We can maybe setup a reminder in the matrix channel?
- Amit: Yes, I can set this up.
- Hans: Do we want to have a shared doc for agenda items?
- Amit: In core/network-wg we have had a slack reminder that goes out before the meetings. That's what Tyler was referring to.
- Hans: I'm in favor of an email thread, this seems easiest and good.
- Tyler: We previously have used the cryptography working group mailing list email thread, but this didn't have much engagement. Matrix seems to have had more engagement.
- Hans: I'm good with using the matrix channel as well.
- Amit: One other idea is to use a hackmd shared file for agenda items. Another thing I've seen for the Rust working groups is using github discussions/issues for agenda items.
- Tyler: I like the idea of using a github discussion + a reminder in the matrix channel linking to the github discussion. 
- All: Agree use github discussion + matrix reminder.

## Digest HIL - Adopt AES Data Movement Patterns
- Roy: Experimenting with the aes HIL data movement patterns, I like the design and it worked well here.
- Roy: One drawback of the aes hil redesign is this is not compatible with DMA since for DMA we would need to store the buffer.
- Roy: I added a SubSlice type for when we need to use DMA and dma specific functions/traits.
- Roy: What I've seen is that the key is usually not sent using DMA and is loaded in one run. The only exception to this was a specific STM chip that requires sending the key as the first block, calculates the hash, then you start sending data. After sending the key again, you then finally get the outputted digest.
- Roy: For the HSM module we are currently working with, you load the key into registers (don't need dma).
- Amit: So the `read_key` method here is fine to think of as blocking since often it is a non-dma blocking implementation. In the "weird" case you mentioned above where you need to send the key twice, the low level implementation of the hil should probably just store the key and write/send it twice. So for the user of the hil it seems this method would work.
- Roy: Yes, this is correct. When implementing this before, we used a mapcell to store this key.
- Tyler: I'm a bit confused why we are needing a dma specific implementation here for the digest hil. Does the aes hil redesign not support dma hardware? For Bobby's new design, we just pass &mut [u8] via the client methods. Does pluton not use dma hardware for crypto operations and is this something our aes hil redesign is missing?
- Hans: Generally we've tried to avoid drivers/hils holding stateful operations since this can be a vector of attack. This makes me a little hesitant to introduce this.
- Hans: Taking a step back here too with the driver mutex, I have some concerns with the mutex design as well that it may be too low level of a primitive to safely use in Tock generally.
- Hans: We generally try to avoid any stateful operation. 
- Amit: I think the question more specifically is in aes you pass some big blob and request hardware to go encrypt the blob and that is async.
- Amit: So for the client when it is time for the client to know that it needs a new key, a callback is issued and passes a slice of u8s that most likely is static but this lifetime is dropped.
- Tyler: So my question is more so about the feeding data down into the hardware / driver specific implementation. It seems to me like there is some discrepancy here since Roy's implementation is using SubSliceMutImut with a static lifetime instead of just &mut [u8] for the data movement.
- Tyler: This tells me that either (1) there is something fundamentally different with the digest crypto hardware (like using dma) that is different than the aes hardware or (2) the digest hil design being presented here is inconsistent with the data movement patterns we've been using.   
- Roy: So the challenge here is that we need to store the buffer since we need a static lifetime for dma operations/store the buffer during the dma operation.
- Amit: So what do we if we drop the feed dma buffer and dma buffer done from these interfaces?
- Roy: We would need to add the static lifetime to the read input.
- Amit: I don't think the interface would need a static lifetime and it could maybe just be stored as a SubSliceMut/Imut.
- Amit: The implementation of this HIL is going to have some static buffer. When you call read input it's going to essentially subtype this to &mut u8 to the client even though it is a static buffer.
- Tyler: Right, I want to really drill into why the HIL method though looks very different and is using SubSliceMutImut instead of &mut u8 to feed data to the hardware/driver.
- Roy: So my understanding is that for AES it is only synchronous 1 shot operations?
- Amit: I think this would be driver dependent.
- Amit: Ideally the low level driver doesn't copy, the client copies into the input parameter which can be used for dma since it has a static lifetime under the hood. This however is only one possible pattern. You could also have many static slices under the hood, one long static slice under the hood and you could pass in half of a slice the first time to read input, read input would return and you fire off an async operation. 
- Amit: You then would call read_input again to pass in more data for perhaps increased throughput for double buffering. 
- Roy: Okay, I will explore this pattern more with the lifetimes as discussed here.
- Amit: If it seems you need to use a SubSliceMutImut, my advice would be to have the low level driver hold a SubSliceMutImut and then pass the needed subslice up to the user of the hil.
- Roy: This sounds good. This will help to cleanup the user of the HIL. The current capsule implementation is a little messy with this HIL proposed today since you from the capsule need to track if you are using dma or non dma hardware (and call separate functions accordingly). This obviously is not great.
- Tyler: You already mentioned this is suboptimal but just to emphasize this point, we really should avoid hardware specific details leaking across the HIL into the capsule.
- Hans: So this divergence from dma mode vs cpu mode is indicative of the driver mutex being the right shape or tool but the higher level api/design around it seems to conflate a lot of different issues.
- Hans: What this indicates is that there is no sole unit of ownership in the driver mutex model that I think should be resolved before we upstream the driver mutex.
- Hans: The driver mutex is a useful primitive to have for drivers and is more broadly useful outside cryptography operations. Perhaps we can design an interface for using the mutex that would provide some safeguards for locking access. This is an abstract criticism of our current design in pluton and challenges applying it to other interfaces.
- Amit: Where does the mutex come in/connect to the discussion from the data movement? 
- Amit: I think this is useful as a reminder that there are two orthogonal things happening here, there is the data movement challenge/solution that avoids each layer owning a static buffer which leads to memory waste and makes it difficult to know how to size things. What Roy was showing here is specifically for data movement. Separately from this, there is also the mutex for allowing multiple clients to use the same hardware without needing to have a handwritten virtualization layer for each piece of hardware.
- Tyler: Yes, these are orthogonal. 
- Amit: There is overlap with the digest hil / mutex is multimodal so either you build a specialized virtualizer or you use something like the mutex to expose something that looks like separate pieces of hardware which under the hood mutually exclude each other (how to do this is still unclear).
- Hans: I was probably over emphasizing the mutex side of things, but I see them as very related to how you would design a compositional model for heterogenous hardware. I suspect the driver mutex design would influence the data movement. 
- Tyler: For next time I think it would be really useful for you Hans to give us a rundown of these specific concerns with the mutex model and the challenges you see as where potential footguns are. 
- Hans: Sure, I can do that for next time. The biggest concern for me is the unit of ownership in the driver mutex model. 
