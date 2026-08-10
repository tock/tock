# Tock Cryptography WG Meeting Notes

**Date:** 7-28-26 

**Participants:**
  - Tyler Potyondy
  - Bobby Reynolds
  - Amit Levy
  - Hans Martin
  - Kat Fox

## Updates
- Amit: I did an analysis/survey of the hash hardware acceleration support in upstream tock. 
- Amit: The questions I was looking into were specifically if most of the acceleration support is multimodal/which mode they operate in (e.g., oneshot vs streaming). I have a big notes doc I can share later. 
- Amit: The high bit is that it is similarly complicated to the symmetric encryption we looked at with aes and things are multimodal. There are virtually no one popular uniform set of supported features. Most support 2 to 3.

## Revised HIL proposal 
### Spliting each cipher mode into a separate trait
- Bobby: From feedback last week, I split the CCM/ECB/etc cipher modes into separate traits. The client can choose which traits they wish to depend on. It also is a building block for modeling hardware vs the umbrella HIL we previously had been discussing.
- Bobby: For instance, the semantics of ECB itself are expressed in the trait. There are read/crypt functions, but no nonce since ecb doesn't support this. 
- Bobby: In the previous version, I didn't like that across each callback you needed to track in your head which mode you were in. This doesn't have that.
- Tyler: This looks good. One question, so if you don't support CCM and only support ECB in hardware, would/could/should your software implementation implement your HIL? 
- Bobby: Yes, trait should model the hardware as truthfully as possible, but this is possible. This is included later in the proposal so we can see this in more detail later.
- Amit: On the Cipher/[ADD] trait, do you implement this? 
- Bobby: No, chips don't implement this themselves. They would implement ECB or CCM etc. 
- Bobby: As a chip author, you would implement the trait for the cipher mode coupled with the key type that the hardware supports. If your hardware supports multiple key types, you would implement the trait multiple times.
- Bobby: So for example, on the nrf52840, it supports ccm and ecb in hardware. The board file would contain both to truthfully show this.

### Software Implementation of CCM Example
- Bobby: This is based on the ccm virtualizer upstream. It seems it was doing double duty to virtualize ccm and do some polyfilling.
- Bobby: The updated version mirrors the existing functionality, but uses the callback api of our new hil. 
- Bobby: One of the interesting things here is how virtualizers can use our syncronization primitive instead.
- Bobby: Instead of a virtualizer, the client would receive mutex that wraps the concrete thing you are trying to talk to. 
- Bobby: The client has to request access to the mutex, is placed in a queue, and then gets smart pointer to the thing they are wanting to use.
- Bobby: So for the ccm software impl, we have a mutex for the ecb hardware peripheral. 
- Bobby: At the entry point to the ccm impl, there is parameter validation, stage these parameters in the internal state machine, then request access to the mutex, which has an internal queue. We pass a handle to the mutex which is a callback to be notified when mutex is ready for use.
- Bobby: ECB was busy, it now is available, and our callback is invoked and we get a reference to the resource. There is some downcasting here, but skipping this for now, we get a reference to the ecb resource we can use/place in state machine of the ccm driver if we want. 
- Bobby: The CCM driver then performs the needed ECB operatin and eventually gets an ECB done callback. At this point we can continue in the CCM driver until we complete the CCM operation. At this point, we drop the reference to the ecb resource which frees the mutex. 
- Amit: On the mutex front, it seems generally good. I remember that using the mutex avoids some extra copying/buffers that would be required in a standard virtualizer?
- Bobby: Not necessarily. It primarily solves the problem of syncronization. I wouldn't represent it as more than this in all cases.
- Bobby: Because it is a reusable component, it is easier to maintain since it does 1 thing and it does it well and avoids shimming in buffering or other things virtualizers sometimes do.
- Amit: I remember vaguely from the tockworld presentation on this that this provides more generic multiplexing infrastructure and also changes the way we can think about how buffers flow through the kernel. Maybe those two things aren't intertwined. 
- Bobby: Not strictly. Buffer movement isn't tied to the mutex and you don't need the buffer semantics for the mutex.
- Amit: The hardware driver would be entirely unaware of this mutex, right?
- Bobby: Yes.
- Bobby: I have an example we can look at later of a syscall driver that would also be using ECB as well that is exposing ECB to an app. This demonstrates kernel coordinating access.
- Amit: Just to highlight this more, this really seems to not be a virtualizer but to be strictly a mutex. This requires a higher level of trust and a misbehaving client.
- Bobby: This is a good thing to draw out and is correct.
- Amit: I think one of the challenges is reasoning about when/how long something should be unlocked for via the mutex. SPI vs UART vs cryptography operation are different with how long the mutex should be held. 
- Bobby: So there is the issue of you drop your mutex guard and then what are the implications across operations.- Bobby: I think this can coexist with virtualizers and might work well for crypto domain, but maybe not others.
- Tyler: I also am concerned about how this would cause issues with a misbehaving/buggy client. I do think though that part of the issue/challenge in that case is related to Tock's usage of interior mutability. The case for bugs/starving other clients comes when you place the unlocked reference from the mutex into a cell type for that specific client, which is a bit orthogonal to the mutex primitive itself.
- Tyler: Overall, I like this and think its good. The existing cryptography virtualizers are not good and cryptography seems like a great place for us to trial this upstream. 
- Bobby: Next, we should talk about the downcast conversation.

### Notes from Bobby/links from discussion:
#### cipher.rs 
- https://github.com/reynoldsbd/tock/blob/3a6e3776de534feba8112b8d9636e39764b4662f/kernel/src/hil/crypto/cipher.rs 
- refactored into separate hil traits per cipher mode starting with ecb and ccm, the two supported by nrf52840

#### ecb.rs
- https://github.com/reynoldsbd/tock/blob/79e27537c03514f99bf2d3720ec8401d71fad022/chips/nrf5x/src/ecb.rs
- rewrite of nrf aes driver into narrow focused ecb impl

### ccm.rs
- https://github.com/reynoldsbd/tock/blob/79e27537c03514f99bf2d3720ec8401d71fad022/chips/nrf52840/src/ccm.rs 
- new nrf driver narrowly focused on ccm impl

#### interrupt_service.rs
- https://github.com/reynoldsbd/tock/commit/79e27537c03514f99bf2d3720ec8401d71fad022#diff-69786f7d247cd7c0740e47357f75744577a9e40a7c741580f1b5831c16b829d5 
- plumbing new ccm driver for nrf52840 chip only

#### sw_ccm.rs
- https://github.com/reynoldsbd/tock/blob/460e6b92248711e6c12d67ef0131dadd5961467f/capsules/crypto/src/sw/sw_ccm.rs 
- software impl of full ccm interface, backed by ecb driver
- 46: ecb_mutex reference held by struct
- 539: ecb_mutex.request() after validating ccm request
- 473: downcast ref and advance ccm state machine
- 553: EcbClient impl
     - data movement
     - crypt_done() handle result, advance ccm state machine
- 467: drop ecb ref at end of ccm state machine, before notifying client

#### framer.rs
- https://github.com/reynoldsbd/tock/commit/74b65430286921b0b364416f815a7ffd986ef9f5#diff-85e0d54f818678f55fd9b21778b576635e2fdfb6e2fe633fd3e1fdc312dcbaa3
- 347: ccm_mutex added to struct
- 500: ccm_mutex.request() after validating/staging ccm parameters
- 1139: downcast ref and call ccm.crypt()
- 1164: CcmClient impl
      - data movement
      - crypt_done() handle result
- 1049: drop mutex ref

#### board wiring
- https://github.com/reynoldsbd/tock/commit/b78aa65070c0f6f920214c1e1c6bf2f70502c49e#diff-7d6e642ac0f88ecf4a307b57525d2780f236bbfda0ce28f449cc4291d72abeca
nrf52840dk/lib.rs
- 314: mutex instantiation (buried in sw component, don't like this)
- 318: allocate # of mutex clients

#### components/software_ccm.rs
- 91: mutex instantiation example (don't like, should be standalone component)

#### aes.rs
- https://github.com/reynoldsbd/tock/blob/f4e0830ec43d857dc1a6f008d276d230348a19bb/capsules/crypto/src/aes.rs


#### driver_mutex.rs
- https://github.com/reynoldsbd/tock/blob/48c054e54ff8762374d0a668b5affb35224d8532/capsules/core/src/driver_mutex.rs
- example SyscallDriver exposing ciphers to apps included for completeness, needs lots of massaging before production ready; illustrates need for downcasting
- internal impl details less interesting than usage

## Next Steps
- Update to weekly cadence for meetings.
- OxidOS folks join next week.
