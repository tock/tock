# Tock Cryptography WG Meeting Notes

**Date:** 7-14-26 

**Participants:**
  - Tyler Potyondy
  - Bobby Reynolds
  - Kat Fox
  - Amit Levy

## Updates
### Pending PRs
- Tyler: I saw Amit merged Brad's SHA capsule PR (4855). I looked through the AES HIL changes and those should be good to go as well. I'll approve and Amit should merge after the call.

### Proposed roadmap - oracle style encryption
- Amit: I would like to add to our roadmap looking into oracle style encryption, i.e., encryption you don't get to set the key for. It would be nice to look more into these when the time comes. 
- Bobby: For oracle encryption, pluton has a lot of this (keys essentially held in slots in hardware that aren't accessible to even kernel mode). We have a system of handles that allow you to use a key without accessing the key itself in plain text. This is a layer of complexity we haven't discussed here yet, but this is important to us and we have some prior art we can draw from.

### Matrix discussion
- Bobby: We had a lot of activity since the last meeting in the cryptography-wg channel. There was a proposed SHA digest HIL.
- Bobby: With some of the discussions, I have come to the conclusion that AES may have been the wrong first crypto mode for us to tackle.
- Bobby: I think some of the changes I tried to incorporate into the new AES HIL might be better suited to the other crypto modes.
- Tyler: Can you elaborate a bit more why it isn't a good fit?
- Bobby: An AES client will concretely know which cypher mode it wants to use, inputs outputs etc. The current design is "multimodal" when this probably isn't necessary and the current shape doesn't have enough compile time guarantees.
- Bobby: I think this is coming from the fact that we care more about the encryption modes themself rather than a generic AES in general.
- Amit: I agree with some of this, but I think this may be over fatalistic. 
- Amit: I think there are three categories of hardware and applications/users. The trick is designing around that. There is hardware that (1) is multimodal and supports most/all the options; the new AES HIL trait works well for this. (2) there is non-multimodal hardware where there is exactly one aes accelerator that operates in 1 bitlength/mode; for this hardware the new HIL isn't a great fit. (3) there is multimodal hardware that supports a small subset, this seems fairly prevalent (e.g., AES128 that supports ecb or cbc).
- Amit: On the application/usecase side, there is (1) I want some blackbox symmetric encryption (think libsodium users) where you just want some encryption. (2) there are protocol implementations where you need a specific cypher. (3) there is something like CA certificate verification or ssh etc where fundamentally different clients might use different things and you want to support as much as you can. Maybe your openssl isn't compiled with a certain set of cyphers. 
- Amit: In the end, we probably want to match all these things together. We want hardware that provides 0 or 1 encryption accelerators to operate on an ssl stream and we also want a multimodal hardware to also work on a specific impl (e.g., thread interface that requires specific implementation).
- Bobby: So how I'm interrpreting this is that the api as it is currently designed will have no recourse for the impl to return errorcodes no support for what is implemented by underlying hardware. The desire is to have fewer error codes that occur nominally. 
- Amit: Mostly yes, I do think some error codes are okay. I'm not sure if this is worse for AES or if this is the general shape of the problem for any cryptographic operation. Sometimes you care if it is sha256/hmac padded, sometimes you want to expose all these, sometimes you don't care.
- Bobby: The disconnect on my end is I don't see the case of say a thread dependency that this HIL doesn't work for.
- Amit: Ideally, I think we want to avoid an errorcode for say a massive thread app that can't work because the kernel doesn't support the encryption mode.
- Tyler: So to push back a bit on this, that could happen currently too if your kernel board file doesn't include the 15.4 driver. There always exists some responsibility to ensure your kernel you ship has the primitives you need. Granted the 15.4 case is mostly compile time enforced. 
- Amit: We want HILs to be as close to the hardware as reasonable because we don't want chip implementations that have most likely unsafe, unsound bits so ideally HIL is going to be very simple and have minimal logic.
- Tyler: I agree with this, I think in some ways, what we want is something like a "logical interface layer" that lives maybe as a capsule for each of these modes.
- Bobby: Just to clarify what we mean by close to hardware. One way is breadth, in terms of number of ops (e.g., delta between the number of ops HIL requires to implement/which the hardware supports). The other sense means is how much extra logic you need under the HIL for a given operation.
- Bobby: I think the proposed new HIL is about as close as we can be to the hardware om terms of the amount of logic in HIL.
- Amit: I agree.
- Bobby: I think that coordination/syncronization/virtualization should exist above the HIL.
- Bobby: In the spirit of reducing the amount of logic in the HIL, the current implementation in terms of breadth has a lot of code paths returning error codes and isn't close to the hardware.
- Amit: If the only kind of application was such that it wanted this interface exclusively, then it is okay for a chip to contain these errorcode paths. That is a reasonable case for saying this hardware just can't support this. An example of this would be that applications usually only care about an alarm (from a timing perspective) -- very broadly. You can't really handle a count down timer to virtualize it so you don't expose this to userspace. If you don't support alarms in hardware, Tock will usually have you implement the logic needed to make a timer act like an alarm.
- Bobby: This is illustrative of why I think aes/block cyphers aren't the best for this conversation (e.g., wanting generic encryption). AES block cypher modes want/need specific mode for the operations they are doing.
- Bobby: The crypto agility we are going after seems to be a better fit for other cryptography types (the case of I just want some encryption to work).
- Amit: From the perspective of the user what does this mean?
- Bobby: Generating key pair / signing with it. It is common that app probes and asks kernel which is strongest supported alg. I can use. In general the application logic doesn't change depending on which key strength you chose. The only thing that changes is your buffer sizes. You don't need to change where or how store private keys. This is what I mean by effectively interchangable. 
- Amit: I wonder if there are applications that never want to fail. Example: I'm signing some data that I'm going to store and want to verify that this is data I generated originally. Don't care which alg etc, just want sign and verify operation. Conversely, simple embedded system that assumes off device can verify/decrypt whatever I choose.
- Bobby: I'm not very motivated or look favorably upon a scenario that blindly uses crypto interface without caring which mode etc is using. Libsodium does this in a successful way, but I think for us this is sticking our necks out and makes me a bit nervous. 
- Amit: I'm not in favor of hiding the details fully either. 
- Amit: In tock process validation verifies the signature of process from the tbf header. For a particular board there was a family of signatures that you can handle, but you probably don't want to ship a library that has every signature verification scheme under the sun. There is a choice of if this fails dynamically or statically. 
- Bobby: So this process loader inspects and checks that signature matches cryptographically. Your desire here is to build a kernel for a board and say which signature encodings/public keys and their corresponding algs are going to be used. Desire is compile time guardrail that you have specified public keys for which the board contains the necessary logic to verify. 
- Amit: Yes, I suppose something like this. This is ergonomic and also resource constraint issue. 
- Bobby: So the question becomes within a family like RSA or something else how far into the type system do you go to express this vs runtime parameter. There is legitimate case for kernel needing to know which keys you need to be able to handle at compile time. On the flipside, there is another case where the ergonomics of using the trait/driver that provides asymmetric crypto, having trait for each mode that is type enforced also requires multiple implementations for callbacks vs the generic implementation that is multimodal and just has the one path.
- Amit: So each application wants a set of things, different hardware provides different things. To the extent those don't match up, the complexity of making them match should not be in the chip impl ideally. 
- Tyler: I agree don't want logic in chips
- Amit: Taking the case of AES, let's suppose that the current proposed trait is the interface some higher layers want. It would be trivial to have capsule that takes a bunch of discrete implementations and encodes specific modes at the capsule level.
- Tyler: So are you proposing keeping Bobby's multimodal hil with runtime error concerns we've discussed and hide this within the capsule?
- Amit: I'm saying we would have multimodal hil for hardware that is multimodal but also a hil for case where you just implement ECB.
- Bobby: I don't see the need for an aggregation layer in a capsule then for this. 
- Bobby: I would envision a single chip driver implements each of the single modal cypher traits that the hardware supports (you would need syncronization for this). Your thread library then would have your dependency on the ccm128 trait. Both drivers would then implement the multimodal and the specific mode then.
- Amit: I think this is where we diverge. In the case of a multimodal chip, I would just implement the AES HIL multimodal HIL directly and do not do specific implementations that need to coordinate. On the other hand, if you need specific traits to be implemented, the multiplexing / coordinating needs to happen but we do this not in the chips crate.
- Tyler: So you are proposing Amit that the nrf5x chip wouldn't use the new multimodal hil and would just implement aes ecb trait. Bobby's new hil would be used for opentitan/pluton type chips.
- Amit: Yes, but with the small caveat that the aes ecb trait would probably be used by Bobby's HIL which would call into the aes ecb trait. So Bobby's hil would be used for the nrf5x, just not specifically in the chips crate.
- Bobby: I'm still not convinced this is the right way to go about this.

## Next Steps
- Tyler: I will send over the nrf5x chip implementation for this driver. We can use this as concrete example to discuss more what we like/don't like since this is the "worst case" for this HIL.
- Amit: Worth considering if AES is worse than others and we picked a particularly bad one.
- Bobby: I think digest may be good to look into.
- Tyler: So should we shelf AES for now and switch efforts to digest?
- Amit: Digest has fewer distractions.
- Bobby: It will touch on single and multipart operations which isn't a concept for block ciphers.
- Amit: I will try to come up with a survey of different hardware has/provides for the accelerators and to do a characterization of what clients care about. 
- Bobby: Will put together syncronization proposal.
- Amit: Will take digest proposal.
