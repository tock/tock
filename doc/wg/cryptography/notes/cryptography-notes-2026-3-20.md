# Tock Cryptography WG Meeting Notes

**Date:** 3-20-26 

**Participants:**
  - Tyler Potyondy
  - Bobby Reynolds
  - Kat Fox

### Crypto Syscall Interface Changes:
- Bobby: One critique of the old syscall interface is statefulness. There is a pattern of "set_algorithm" followed by crypt command, followed by finish command. Given application has to call set of syscalls in sequence. This makes it hard to reason about multiple apps interacting.
- Bobby: Additionally, at the system call layer, the current interface seems to have commands carved out for specific hardware/crypto modes.
- Bobby: What I propose changing to is allows for iv/buffers/keys and then a single command that specifies which crypto mode.
- Bobby: This syscall would then be abstracted via a public api of the form `libtock_aes_new_cbc_encrypt(..)`. This seems better to me since this looks closer to what a normal cryptography library and also would be nice for if we want to shim this (such as with sw crypto).
- Kat: I like this a lot. My one critique/concern is maintaining an async interface. 
- Kat: The async model is challenging to reason about the ownership model. I have some ideas for how we can do this with a mutex of some sort, but I think we don't need to worry about this in the initial design.
- Bobby: At the syscall layer, it is async to the app.
- Tyler: This seems like it would work and map well to async/sync versions of this.
- Bobby: Aside, for what our team works on we have more complex command loops. We have brainstormed something neat in Tock (usermode or kernel?) of having event handles instead of an upcall/callback model. This would allow different async ops in the kernel that have unique event handle that a usermode app could query.
- Bobby: This might go a bit beyond the charter of the crypto working group, but seems like it would be interesting.
- Kat: I like this idea. My primary concern with hw cryptography is after the operation completes, we need a finalize call (this does cleanup like shredding memory and gathering results). You can do this with some wrapper logic to perform the finalize, but this makes for a bulky ISR. Ideally, it would be really nice for usecases with a coprocessor to have an event fire or a thin upcall to flag that an operation is finished. 
- Kat: It definitely isn't the end of the world if the finalize op/interrupt indicating it is done are bundled together, but it could be nice to have these separated.
- Bobby: One other usecase for us is that RSA keys are expensive to generate. We're talking minutes potentially (dependent on hardware). This isn't ideal. We have schemes in place to allow background gen of RSA key while foreground remains responsive (long running keygen in background). 
- Bobby: The upcall when the RSA finishes handling goes into our event loop.
- Bobby: Async scenarios are also forefront for us as well.
- Bobby: Do we feel it is ready time to go towards a proof of concept?
- Tyler: I'm in favor.
- Tyler: My question is what do we need to do for this? Is this just the HIL redesign? HIL redesign + syscall redesign as well?
- Bobby: I think we should ground this in either a unit test  going after a preexisting tock application/workload
- Bobby: Would openthread be a good usecase to motivate this?
- Tyler: I don't think so. OpenThread is actually pretty simple and boring in that it just needs AESCCM which we do in hardware. This wouldn't get at the more interesting HW/SW crypto divide we've discussed or the cases in which a platform lacks support for a certain hardware crypto, but uses say an ECB block as input for GCM. 
- Tyler: I think the hotp key tutorial or the root of trust app might be a better motivating case for now. The hotp tutorial is what kicked off a lot of these conversations in the first place.
- Bobby: This sounds good to me and fits with our usecases.
- Bobby: I like working towards something that has "crypto-agility" 
- Bobby: It sounds like shaking of some bit rot and getting it functional, then starting to try adding these changes to this example.
- Tyler: This would be for nrf52840. Any concerns with that?
- Bobby: Picking a board in the public domain seems best. I can steer us away from design choices that would be at odds with Pluton.
- Bobby: From what I've heard about crypto interfaces for the devices you care about, this blend of hw/sw crypto seems applicable. Is the HIL design we are steering towards usable for you? Is there anything we can proactively do to avoid that?
- Kat: The only things I'm seeing that might prevent issues are retaining some "finalize" interface through the sysyscall interface. The other is potentially needing some bulk write interface to avoid having to perform many allows.
- Bobby: I think adding finalize changes the api from a one shot to a streaming interface. The state needs to in some way be maintained by the client. It almost starts to look like an object oriented interface. 
- Bobby: One of the tensions we've encountered is some devices only support incremental streaming, other devices support oneshot operation (more nuance depending on key material, security properties, etc). 
- Bobby: In my mind, we start by designing all possible algs and operations and return an errorcode. Adding sw shim seems like a later add on.
- Tyler: It seems we could hide/encapsulate these differences with oneshot vs streaming potentially?
- Bobby: Oneshot interface can always be presented as begin, update, finalize. You can always go incremental to oneshot. If we have for starters a one shot api, the kernel can handle the state machine of finalize. We lose in this case though the ability to stream from userspace with this though.
- Tyler: I am in favor of having 2 syscall interfaces, one streaming syscall interface and one oneshot interface.
- Tyler: This seems like it might simplify the implementation and these usecases are distinct enough in my mind that it should be separate syscalls. The downside obviously is runtime failures, which we are trying to avoid.
- Bobby: One concern / issue we ran into related to this was for a oneshot hmac hardware. We wanted to map this onto a streaming interface. Because of the way the hmac algorithm works, the intermediate steps can leak state so it is not possible to have streaming operation. 
- Kat: Hmac seems like a great example. One example I've seen of something that wouldn't be easy to represent as a oneshot is a software streaming hash implementation (firmware measurements).
- Bobby: To summarize, it seems we are in agreement we want one shot and streaming apis and it seems we have a feasible path towards supporting this. We decided to use the RoT or fido device as a motivating example on an nrf. 
- Bobby: One other idea is to do this in qemu too potentially.
- Bobby: For me, I'll take in the feedback from today and expand it to support the different modes of operation.
- Tyler: I can tackle confirming the current tutorial we are going to use as our example is working.
