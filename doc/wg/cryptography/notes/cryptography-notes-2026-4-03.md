# Tock Cryptography WG Meeting Notes

**Date:** 2026-04-03

**Participants:**
  - Tyler Potyondy
  - Bobby Reynolds
  - Kat Fox
  - Amit Levy

## HOTP Tutorial Updates 
- Tyler: I had a chance to trial run the tutorial. Some functionality seems to maybe be a bit broken, but the app builds, flashes, and runs fine. 
- Bobby: So what would the steps be to "productionize" this tutorial?
- Amit: You could fairly easily do this if we had some hardware source/baked in key rather than the "oracle" we are using the capsule.
- Bobby: I like the idea of having in the tutorial a step where you "provision key". Could be a neat add on at somepoint.
- Amit: That would be neat. The reason for the preprovisioned key makes it easier for tutorial purposes of debugging, but it seems reasonable to have a key.
- Bobby: So as far as the tutorial is concerned, it seems that there is perhaps a point in the tutorial that we can stop at that would serve as a good starting point for our work for testing and prototyping these changes.
- Tyler: For my todo, I can put together a condensed doc that synthesizes the key points of setup and replication for the tutorial so we can all be on the same page for starting this work.

## Crypto HIL Updates:
- Bobby: Haven't had a chance to make much progress since last week. The kernel/userspace interface and syscalls are ready for feedback.
- Bobby: The work that remains is just going ahead and implementing those interfaces.
- Bobby: I know the one item of feedback was streaming vs oneshot interfaces and the need to have both to support various crypto hardware.
- Bobby: We had also talked about sync vs async and wanting both.
- Tyler: Which do we want to do for the purposes of the tutorial extension/prototype we are planning to work on?
- Kat: I think we should start with one shot for the purposes of the prototype.
- Bobby: I agree from an implementation perspective. Oneshot sync is the least amount of work.
- Bobby: The part I'm not sure about is if that is sufficient for the tutorial needs.
- Tyler: I don't remember. Would need to check.
- Amit: Just to clarify, what is the sync vs async interface you are mentioning?
- Bobby: This is from the userspace interface's perspective.
- Bobby: We discussed last time what this might look like. There is a bit of nuance with regards to how the interface/callbacks work for the async model.
- Kat: For some crypto interfaces that need say FIPS compliance, it is common to split interface into `start` and `finalize` operations. Having control over when to call the `finalize` operation is important. Potentially having more control at the user level would be helpful.
- Amit: With this update vs finalize, what gets split between software and hardware?
- Kat: The difference I was thinking about in a one shot sync interface is you would have the upcall happen after the finalize operation. However, if userspace is doing something very timing sensitive, you might want the upcall to go to the kernel and then call finalize from userspace. 
- Bobby: I propose we set up the contract for oneshot to be (1) invoke the tock api, give all input at one time (2) send syscall to kernel, does all work in kernel (including finalize) (3) issue upcall.
- Bobby: For the streaming api (1) call init method (2) call update method (async/kernel syscall) (3) upcall for each update operation indicating end of update (4) app then decides what to do next (when update completed) (5) app decides when to issue finalize.
- Bobby: For a scenario where an app needs more fine grained control for responsiveness, the expectation is that the app would use the multipart version of the api. Would this satisfy the concerns?
- Kat: Yes, I think so.
- Amit: This makes sense to me. Can you clarify what hw/sw state needs to be preserved across operations? 
- Bobby: The state info of the operation in progress tends to vary in size/composition depending on which crypto hw you are running on. 
- Bobby: There are tradeoffs involved here and the state structure might not be applicable to all hardware.
- Amit: Do you have an example of this state you pass to hardware? Perhaps bitlength/payload passed to hardware?
- Bobby: If you are thinking just about aes and streaming cipher modes, in general, it is often true the individual hardware ops are identical to one another. 
- Bobby: There are other crypto operations where the state between operations is more stateful (e.g. hmac/message digest with sha1).
- Bobby: There might be some endianness constraints for hardware or bitextension or padding. These are examples of nuanced differences here with what is the hardware/software boundary. We have an aim for different hardware backends to be compatible. 
- Amit: I'm a bit confused on this. It sounds like the specific bits in some fixed length array might mean different things to different hardware. Are there cases where the shape of the state is different?
- Kat: This is true for hmac and kmac for opentitan. In cryptolib there are state structs that are bit more amorphous. 
- Amit: And cryptolib or the user of cryptolib tracks this?
- Kat: Some state is passed from cryptolib back to capsule.
- Bobby: So say a userlib app starts some cryptolib op. The intermediate update isn't a blob of data that is exposed to usermode, but is stored in the grant then?
- Kat: That would fit the needs for cryptolib.
- Bobby: So interleaving crypto operations between two apps, there is now kernel state. How does the app control which hmac operation it wants to update next? 
- Kat: I think usually you can't kick off another hmac (or other streaming operations) until the other completes.
- Amit: But cryptolib could handle concurrent ops, right?
- Kat: Yes this is true.
- Amit: So something where you call init which initializes some blob in my memory other times it is a file descriptor essentially that is a handle to a particular operation.
- Kat: I can't think of cases in opentitan where we would want/need concurrent streaming operations, but is certainly possible to do.
- Bobby: One of our design principles we have for reliability is to keep firmware operations stateless. In order to achieve this, we need to sometimes export state to a higher layer. 
- Amit: You could imagine handling all this, but now you are shoving persistent storage into the encryption capsule. 
- Bobby: If the context occupies say 256B, that's memory that would be occupied by the grant for this application if in the capsule, but placing this in an application, the app can control how this is placed to avoid wasted space.
- Tyler: To confirm, you take the "store userspace" approach. 
- Bobby: Yes.
- Amit: So both needs have their benefits. I think this is a +1 for the shared library idea we've talked about. For basic usecases, the kernel interface is sufficiently general, but your application should preserve the ability to be more hardware specific.
- Amit: So there is perhaps a cryptolib specific kernel interface that doesn't make promises about exposing to userspace. The cryptolib capsule then would expose a nuanced interface vs the pluton capsule interface. 
- Bobby: The only revision I'd make to this is that it isn't necessarily a difference in hardware. It is more a matter of the threat model/defense in depth posture that might differ. In one case, intermediate state might be exposed to apps to manage vs wanting the kernel to manage this.
- Amit: So this isn't different hardware it is a different kernel.
- Amit: So in both cases, you ideally want to run some set of generic applications. 
- Bobby: So everything then could be kernel managed, but we leave a syscall that essentially allows you to import/export state. Opentitan then would disable this, pluton would use this.
- Amit: A downside of this is trusting the capsule to do more than you need it to.
- Tyler: So wrapping up, I'll make a doc for the minimal steps for the tutorial setup that we will use as a starting point.
- Bobby: I'll take the doc and starting playing around with the tutorial/our working example and how we can map our discussions here onto it.
- Bobby: Perhaps we should start writing docs for what these different interfaces could look like for say oneshot vs streaming.
- Kat: I can flush out an example of the kernel/stateful version.
- Amit: I'll try to perhaps import this to qemu and think about a completely software implementation in qemu. 
- Bobby: The virtio crypto for qemu could be neat.
- Amit: I'm talking about a fully sw crypto.
- Amit: The goal would be to try and make sure the interface works without accelerators in hardware. 
