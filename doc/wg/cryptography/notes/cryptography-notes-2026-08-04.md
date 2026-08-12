# Tock Cryptography WG Meeting Notes

**Date:** 8-4-26 

**Participants:**
  - Tyler Potyondy
  - Bobby Reynolds
  - Amit Levy
  - Hans Martin
  - Kat Fox
  - Eva Cosma
  - Roy Bachynskyi
  - Irina Badu
  - Alex

## Updates

## OxidOS + Tock Cryptography WG Introductions
- Eva: We are working on an HSM module that has a heavy focus on cryptography.
- Eva: We are using TockOS on this, it's a cortexm chip we work with.
- Eva: The existing digest HIL does not support multi-modal hardware and does not support when hardware needs to know the message length in advance.
- Eva: We have come up with solutions to this and  would like to add support for both of these things upstream. We would like the cryptography-wg's feedback on this work.
- Amit: For some context on what the cryptography-wg discussions have been about: we have been tackling two things at once (1) looking at the shape of AES hils and trying to fit the differences in blocklength and other aspects to hardware that might support a variety of these things (2) the other focus has been thinking about data movement across the kernel/hardware and thinking about who owns these buffers.
- Amit: Another relevant point is that we have looked into figuring out how to support hardware that is multimodal and hardware that is discrete for different modes. This was more looking into this for AES.
- Amit: At a higher level beyond AES though, we want to create HILs that allow users of the HIL and also applications to write primitives ontop of these that are hardware agnostic. The other side of this though is not wanting to force every implementer of this HIL to need to implement a lot of software crypto if they aren't actually using it.
- Amit: Most recently and last week we discussed using a new async lock mechanism in chip crates that have multimodal hardware to allow implementing the HIL in a way that avoids different operations (e.g., cbc or ctr) from causing issues (since the lock mechanism will queue the work).
- Amit: Trying to use multimodal hardware concurrently in different modes fundamentally requires a lock since these operations must occur sequentially. By using the lock allows us to remove a lot of complexity from the HIL to avoid runtime errors.
- Bobby: One option for this call would be running through some of proposals and show some examples in code.
- Bobby: This would also include some discussions for the locking mechanism as well.
- Bobby: Goal would be to hear if these proposals would work for your platform as well.

## Digest HIL
- Roy: In stm32 and lowrisc, there is multimodal hardware which differ in digest length.
- Roy: Currently, the digest HIL length is set once and is encoded into the type which makes it impossible to switch between different modes. This prevents using multimodal hardware where we change the digest length at runtime.
- Roy: To introduce some flexibility into digests, we propose using an enum where we construct an enum from the slice that validates the slice's length. This allows us to remove the const generic length.
- Roy: We like Bobby's proposed solution but we would prefer splitting the HIL to have a multimodal hardware HIL that allows for runtime changes and a separate HIL for compile time enforcement.
- Alex: The second issue we encountered is that the current digest HIL does not allow you to pass the length. This is needed in some hardware platforms.
- Alex: We propose adding another optional function to the HIL that allows you to pass the message length.

## Walkthrough of Proposed New HIL
- Bobby: Multimodality is definitely the stickiest of the challenges for these HILs.
- Bobby: This is a proposal for a new HIL for AES block ciphers. The way we model this is separate HIL traits for each cipher mode a device might support. 
- Bobby: For instance, we have a trait for AES CTR that can be generic over different cipher lengths. This gives you a narrow trait that can be implemented in any hardware that provides support for the CTR cipher mode.
- Bobby: We are prioritizing granularity to enable hardware that supports all of or some subset of the cipher modes to be able to implement the HIL.
- Bobby: If you have two distinct hardware peripherals on your chip, you would implement 2 HILs. If you have a multimodal hardware support, you would only implement the HIL once, but would need to use a locking/synchronization mechanism.
- Bobby: The HIL is very callback oriented. If you are a client, you initiate an operation, give an upfront parameter, whether you are encrypting/decrypting, etc. After that, the peripheral driver is expected to invoke a variety of callbacks to retrieve data from the client. This opens up a lot of flexibility and reduces the memory footprint of the driver. A more traditional Tock driver would pass a static mut buffer that results in unused buffers living in BSS.
- Bobby: One interesting thing here that might be interesting for digest is how we pass data to the hardware. We have a `read_input()` that your driver can call and request the number of input bytes it can support. 
- Bobby: We still need to nail down the exact semantics for this.
- Bobby: What we often see with hashing peripherals is two modes of operation (oneshot operation vs incremental) the idea being that in one mode you maintain state across invocations vs oneshot being passing a buffer once (state isn't maintained in this case).
- Bobby: Before we continue, for the OxidOS folks, is this something that would work?
- Eva: I like the idea of being truthful to the hardware in the HIL. I am most curious about the synchronization mechanism. 
- Bobby: Synchronization uses something called a driver mutex. The mutex holds a resource and some inner book keeping. The client model is somewhat similar to Tock's deferred call infrastructure. 
- Bobby: You create a new mutex, give it a reference to the resource you are guarding, and some empty buffers to store client book keeping in a queue. 
- Bobby: At runtime, when a particular client wants to request access to a driver, it passes a handle and gets added to a queue. When you get to the front of the queue you then get a reference to use the peripheral behind the mutex.
- Bobby: In comparison to Tock's mutex/virtualization, you get reference to an object that is virtualized and appears to not be virtualized/synchronized to the user.
- Bobby: This synchronization mechanism (via the mutex) flips this model on its head. We use this reusable mechanism for synchronization, but push some complexity to the client.
- Bobby: The two issues/challenges that appear with the driver mutex design is downcasting and reference counting. 
- Bobby: There is some setup (registering etc) then you request access to the mutex. When you receive access, there is a callback that is invoked (ready callback). You get a smart pointer to the resource then confirm the type erased guard is in fact the type you want. Eventually you drop the ecb smart pointer.
- Bobby: Upon dropping, the mutex then pops this client from the queue.
- Bobby: The Rust compiler will complain in a multimodal situation due to the generics provided when instantiating a capsule since for say a software AES CCM, you might require an AES ECB and GCM.
- Bobby: The workaround to this is downcasting.  
- Kat: The ambiguity for me is that we are turning this into a runtime error, correct?
- Bobby: Unfortunately, yes. It is technically possible to support different versions of the client trait. There could be an any version of the Driver Mutex that would need the downcasting and a version that could be strongly typed.
- Tyler: As you mentioned before, it is unfortunate that this downcast appears in the client. I wonder if we could maybe make this functionality/complexity for the generic more abstract/generic to hide this from the client that is using/would need to implement this.
- Eva: So with this proposal, you are suggesting we specify the length at HIL and the change to the length would occur at the synchronization level?
- Bobby: This example was for how a client would interact with the HIL. The capsule would need a mutex that is typed for say SHA256. If you had a driver that wanted to do multiple types of SHA operations, you would know at the time you request SHA mutex and that would inform how you perform downcasting. 
- Bobby: It is easy for me to envision an aggregation layer where there is one opaque object. Rather than all this tricky downcasting, you have a scalar parameter that specifies which operation you are doing. 
- Bobby: One of the benefits of granular HIL traits is that it makes it easier to think about how to stack or compose them together. 
- Eva: I think we could try to investigate using this synchronization mechanism for our use case. The next steps for us is trying to apply this to our usecase.
- Bobby: The data movement is also something I'd ask your team to see if works for your usecase.  
- Tyler: Based on the OxidOS usecase presented, I think the proposed data movement changes in the AES HIL that take a more client based approach would solve the length issue you are facing. 
- Tyler: I'd like to emphasize too that the synchronization mechanism for multimodal hardware is entirely separate from the data movement pattern. 
- Tyler: A good next step would be for you to maybe attempt to apply a similar shape to the AES hil's data movement to the digest hil and in doing that, also try to add the optional message length function you proposed.

## Next Steps
Continue discussion of synchronization mutex next Tuesday, focusing on the reference counting aspect.
