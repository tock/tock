# Tock Cryptography WG Meeting Notes

**Date:** 8-25-26

**Participants:**
  - Tyler Potyondy
  - Hans Martin
  - Roy Bachynskyi
  - Irina Bradu
  - Alex

## Updates
- Hans: One clarification from last week on my end, the driver mutex doesn't define the composition model. Both the higher level API and composition model require careful consideration. 

## Digest HIL - [branch](https://github.com/frihetselsker/tock/tree/digest_hil)
- Roy: I removed the dma buffer method as we discussed before. Now the HIL looks more abstract. 
- Roy: Now we allocate a buffer and pass this to the init function for the driver so it is stored in the driver itself if the driver needs DMA. This made the capsule code much more clean.
- Tyler: What do the client methods for passing data to the driver look like? 
- Roy: Now we just pass &[u8].
- Tyler: This looks great and is much closer to the shape of Bobby's design for data movement in the aes hil redesign.
- Hans: I'm curious what the driver's implementation of this HIL looks like.
- Roy: This is in the hash capsule. 
- Tyler: And this uses the driver mutex in the capsule? 
- Roy: Yes, the biggest difference between this updated capsule and the upstream hash capsule is that we cannot enqueue multiple operations. It will just return BUSY to the app if some operation is already in progress.
- Roy: This is as opposed to something like the HMAC capsule which enqueues operations. Bobby's aes branch also didn't use enqueuing so I based the work in progress version 
- Tyler: This may be a side effect of the aes capsule/redesign being a work in progress, simple example to showcase the new HIL. We haven't extensively discussed this, but I imagine we will want to support queuing of operations in the capsule.
- Roy: Okay I can work on adding this.
- Tyler: Does Pluton support queuing in crypto capsules?
- Hans: Generally in Pluton we default to not using queuing work and just return with a BUSY failure to the app.
- Tyler: That makes sense, I think there are reasons for both. In the case of knowing the specific applications that are running on your kernel, avoiding queuing operations seems reasonable. However, in the more general Tock case where a number of different apps may be run on a kernel image, it seems very useful and important for us to support and allow multiple apps to queue up work and not just bubble up the error to userspace.
- Tyler: Overall, this is great though!
- Roy: The only difference in my capsule was how I handle some offsets. I'm not sure if this will cause issues for some users of the interface. Remember, I mentioned for the STM driver that you have to write the key twice (from the capsule). 
- Roy: There is a potential maybe for a vulnerability here. You can pass from the capsule a buffer of arbitrary size and this seems maybe a way someone could leak key information. This doesn't happen in the current implementation, but seems to maybe be a cause for future issues.
- Hans: If I understand correctly, the HMAC capsule resets the key offset to zero after each pass. I wonder if it would be a good idea to track what the key phase is.
- Roy: Maybe we can impose some enum in the capsule to restrict the number of times we could allow a driver to read the key.
- Hans: This would help prevent an error from silently replaying a key. I can take a look at this.
- Roy: I'm looking now to what I should do next for upstreaming this.
- Tyler: One thing that may be awkward with the timing is that this is now ahead of our AES work and would be the first to introduce the driver mutex pattern. This is something we will need to discuss more once Bobby and Amit are back to finish discussing the driver mutex and also getting this upstreamed. 
- Tyler: In terms of changes, is it just going to be the updated digest HIL and then the driver/capsule updates?
- Roy: Yes, I think that's everything. I think it might also make sense is to keep the current HIL in Tock and have the new one plus the old one that is marked for deprecation.
- Tyler: Why would we keep the old one as well? Is the new one not a superset of the functionality of the old HIL?
- Roy: OpenTitan uses the old HIL and I don't have a way for testing the changes to those drivers.
- Tyler: There should only be the one HIL and any new HIL should replace the old one for this. In regards to testing on OpenTitan, we can likely ask Kat for some help with this. In general too, I think that implementing the HIL for another set of hardware is a good test of the HILs design and whether it is truly hardware agnostic. 
- Hans: I concur generally and think we should fully replace the old HIL. 

## State of Public Key Crypto in Tock
- Alex: We are working on some digital signature infrastructure. I saw there was a capsule made last year by Kat on some of this.
- Alex: What I've been doing is looking at elliptic curve/modular arithmetic for a HIL. I am hoping to have a capsule/draft of how this would work in an ecdsa capsule signing. 
- Alex: The idea for modular arithmetic would be having a calculator that is stored in memory and trying to use the data movement pattern the working group has been looking at.
- Hans: If I understand correctly, this is a general modular arithmetic HIL?
- Alex: Yes.
- Hans: Would you consider a more minimal HIL for something like modular exponentiation?
- Alex: Yes you could, but I was thinking about algorithms that have more complex operations.
- Tyler: What does existing hardware support for these cryptography modes and operations proposed in the HIL look like in tock? 
- Alex: Some of the modes ares supported on the STM board I've been working with.
- Tyler: So this would be a subset of the operations in the enum?
- Alex: Only division would be missing in this implementation but we could implement division from some of the other modes that are supported.
- Tyler: One feedback I have on this proposal is that we should encode more of the hardware properties into the type system. We have been discussing this in the working group generally, but this is a great example where you will have many runtime errors of NOSUPPORT.
- Alex: Using traits would allow for this type enforcement. I can switch to this.
- Tyler: Other thing I'd like to draw attention to is the need for software-cryptography support here. This is another area we've spent a lot of time discussing and introducing some of these more complicated cryptography algorithms will provide an increased need for having a good story for this.
- Tyler: We have previously discussed moving all software cryptography from the kernel and into userspace. Something like https://github.com/tock/tock/pull/4869 provides a way to expose userspace services to the kernel.
- Tyler: All of this is perhaps less relevant for your specific HIL here, but this is something that'd be useful for us to keep thinking more about and that is important as we expand support for more complicated, and potentially less well supported in hardware, cryptography algs.
- Alex: I'm concerned as well of software implementations of cryptography. Since I found accelerators that work, I think this is a decent place to work on adding hardware support for this in Tock.
