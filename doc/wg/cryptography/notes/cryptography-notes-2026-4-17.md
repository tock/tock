# Tock Cryptography WG Meeting Notes

**Date:** 4-17-26 

**Participants:**
  - Tyler Potyondy
  - Bobby Reynolds
  - Kat Fox

# Updates:
- Tyler: Unfortunately, wasn't able to make progress on getting a condensed tutorial guide together. Will complete, this upcoming week. 

# HOTP Tutorial - Motivating UseCase
- Bobby: What is purpose of encryption oracle capsule?
- Tyler: My understanding is that this provides a crypto interface for encrypt/decrypt bytes while storing the key in the capsule. This is meant to mock a usecase with say a RoT that has a pre-provisioned key. The capsule contains a hardcoded key.
- Bobby: What userspace changes would be needed as we begin modifying the kernel interfaces/HIL?
- Tyler: I recall their might be a finalize method? I think we settled on oneshot so this may be an issue.
- Bobby: Looking through the userspace application, it looks like there is just an oracle encrypt and decrypt call so this shouldn't be an issue.
- Tyler: One issue I see with this tutorial is that the encryption oracle capsule exposes a custom syscall interface to userspace. 
- Tyler: Our two aims are to 1. improve internal kernel crypto HIL interface, 2. provide improved syscall interfaces. HOTP tutorial only helps us explore the kernel interface.
- Bobby: We have many things that follow this key oracle pattern. I've avoided so far going into details on these more custom "encryption oracle" type since they are likely harder to generalize.
- Bobby: So is there a different test that might be better with the userspace component as well to take the new syscall interface out for a drive so to speak?
- Tyler: I'm in favor of sticking with the HOTP tutorial for now. I think that we can focus on exploring the kernel HIL and improving that. There is plenty of work there. We can follow up this work with an application that stretches the improved syscall interface next.
- Bobby: I can take on applying my proposed changes to the HIL and begin updating the oracle capsule with these changes.
- Tyler: One quick note when making the changes. There is a mess of nested virtualizers in the existing kernel crypto interfaces so you may run into some ugliness there.
- Tyler: The nested virtualizers are what initially motivated me to start working on the crypto interfaces in tock. We had mentioned originally expanding the tutorial to use AESGCM instead of HMAC in the encryption oracle capsule, but the AESGCM interface is currently broken. 
- Bobby: One drawback on AES GCM is that this doesn't cater to the lowest common denominator of hardware which could create some additional challenges.
- Tyler: For that exact reason, I think GCM could be a very interesting next step for us. It would allow us to explore SW crypto that uses simpler HW crypto primitives and exploring some of the userspace/kernel SW crypto stacks we've mentioned earlier. Definitely a more distant next step though for after we make these changes. 
- Tyler: Does opentitan provide support for GCM? 
- Kat: Yes, but with some caveats that result in the underlying operations being slow as it has to go block by block. 
