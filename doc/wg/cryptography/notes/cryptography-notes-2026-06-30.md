# Tock Cryptography WG Meeting Notes

**Date:** 6-30-26 

**Participants:**
  - Tyler Potyondy
  - Bobby Reynolds
  - Kat Fox
  - Amit Levy
  - Hans Martin

## SHA Capsule - https://github.com/tock/tock/pull/4855
- Bobby: Are we in agreement that SHA is in the purvey of the wg?
- Amit: Yes, I think so.
- Bobby: Are we happy with the shape of the SHA HIL? If we aren't, do we want to wait on some of these changes?
- Amit: The current state is that they are a complete mess. Brad observed that the existing capsule is copied from the hmac capsule and doesn't work with sha.
- Amit: Changing the SHA capsule to make it decent is blocking people using it. I have some things I'd like to see changed, but at the same time, this seems to be an improvement over what is currently there.
- Amit: The tension is that if getting it right takes a while, we probably don't want to block and we label this more experimental.
- Bobby: I agree that we don't want perfect to be the enemy of good. At the same time, we should add this to our horizon for deciding what the end state should look like. Specifically, what does Tock's governance look like moving forward for code touching crypto related functionality.
- Tyler: Amit, can you articulate some of your concerns a bit more.
- Amit: The interface in particular. The interface is exposing a hash/verify interface that seems non standard and would be challenging/memory intensive to implement in hardware or in applications. 
- Amit: It seems tuned for the software implementation of what we provide already.
- Bobby: I'm confused, why is there a verify function here?
- Bobby: Verify here seems like it should not be in the kernel/capsule. Typically we use the hash to verify, but verify is more "business logic".
- Amit: I agree with this. I see some reasons (e.g., memory savings) we'd want this here, but broadly, I agree.
- Amit: Internally, there are some weird holdovers that makes the driver non generic over the bit size. This is good in some ways since the hardware/software libraries are 256, but there are relics still of the more general interface. 
- Bobby: I agree with these criticisms, but at the same time I'm not sure we want to force all these fixes.
- Amit: Yes, and this is not a clear final state.
- Bobby: The only grey area I would call out is if this PR opens the door for applications to build a dependency on capsule functionality we want to eventually change. 
- Bobby: Most of our future changes likely wouldn't break this functionality (besides the verify).
- Tyler: So do we want to merge this with the caveat it is experimental?
- Amit: I propose we relabel driver number to experimental range, push back on adding the verify method. 
- Bobby: I am anti having the verify method that is more of a creature comfort.
- Tyler: Keeping the verify method in userspace is very much on theme with some of our more recent discussions having the kernel do less and pushing this functionality to userspace.

## Process for Merging PR
- Amit: Logistically, one approved maintainer needs to hit approve to merge. What do we want our policy for the wg to be for merging?
- Bobby: Long term tactics (e.g., crypto governance) or short term plan for these PRs?
- Amit: We shouldn't let these PRs sit for too long. 
- Bobby: My opinion here for these PRs is that silence implies approval in the meetings. If someone from crypto-wg approves and no feedback, then we can merge.
- Tyler: I would like to discuss this the notifications mechanisms for crypto-wg PRS. We have missed a lot of these PRs just because notifications weren't working. 
- Amit: We can look further into this (solving offline). 
- Consensus: Mechanism moving forward is to tag @mention the crypto-wg team when crypto-wg label is added.
- Bobby: Do we want approval/merging to be async or wait for meetings?
- Amit: Preference is for async.
- Bobby: So async, with silence implying approval for PRs and discussing further if needed in biweekly meetings.

## AES HIL Changes PR - https://github.com/tock/tock/pull/4861
- Consensus: Mostly constant naming changes, uncontroversial and should be approved soon. 

## Crypto HIL Redesign - https://github.com/tock/tock/compare/master...reynoldsbd:tock:aes-hil
- Tyler: I've worked briefly with the new HIL implementing for the nrf5x chip and have so far liked it.
- Bobby: I'll give a quick overview and then we can talk about next steps.
- Bobby: Pattern for HIL is pass in parameters for the encryption. Otherwise, uses async/callback driven pattern for retrieving key material/nonces/buffers etc.
- Bobby: Depending on the cipher mode you select, different sets of callbacks will be invoked. Data moves through buffers that are not static. This gives flexibility for working with this.
- Tyler: Yes, this was nice and makes it easier for the impl with them not being static.
- Bobby: We have an enum for modes and the enum contains mode specific info as needed. 
- Bobby: I would like to extend the flavor of this HIL to other crypto interfaces. High level feedback wanted and specifically what are the next steps/things we want implemented around this HIL.
- Amit: What does it look like implementing this on chips that don't have support for all the combinations of all the modes and key lengths. 
- Bobby: From the intention of the API, this is going to be an errorcode from the crypt method. In this case, the recent PR for user services would be compatible with this (e.g., sw crypto in userspace).
- Amit: Right, but the concern is in practice, there is then going to be a lot of runtime checks and branching (e.g., which mode is supported, which key size etc).
- Bobby: So there is always going to be a return code, so I don't see a world where client code is not going to need an error check. So I would argue that the impact is negligible vs the cognitive complexity for encoding this into the typesystem. 
- Tyler: It seems a lot of this complexity could be encapsulated in a single "cryptography controller" capsule that routes syscall commands to hardware if supported, or back to userspace software crypto library via something like userspace services.
- Amit: So if we moved key length and mode to be generics could we mitigate some of this?
- Bobby: For keylength being a generic, would we have marker traits?
- Amit: Or a const generic.
- Bobby: What about if hw supports multiple key sizes? 
- Amit: We could have marker traits or multiple structs, but I see the larger point. 
- Amit: I think keylength and mode should ideally be specified in the type-system. If kernel clients (e.g., syscall driver) I know at compile time which keylength and mode I'm going to use. If the hardware/system doesn't support that, then getting an errorcode of unsupported is an unrecoverable error. So it seems this should be in the type system. So what is the downside for the type system version? 
- Amit: One answer from earlier is that it isn't obvious for doing this if impl supports multiple key lengths. 
- Bobby: If we go down the rabbit hole for representing this in the type system, I would break apart the cipher mode/block cipher algorithm. Then you would have AES or other block ciphers so you can mix and match these as traits. Then as a client, I could take something that implements the underlying primitive like ECB or CTR. 
- Bobby: My suggestion (but also question) is can we keep something like this that has runtime checking, at least initially, and try to lift more into the type system over time. 
- Tyler: I had some similar feedback for encoding more into the type-system previously. Bobby convinced me why this is challenging though. It is still worth Amit taking a look at though to see if he can think of a nice way to encode this.
- Bobby: So next steps now that we are reasonably happy with the interface is to start implementing for specific chips, update the syscall interface/capsule that uses this and then update the respective libtock-c/rs bindings. Once we do this, we can see if we are happy with this revised interface and then from there move onto a new crypto mode hil we want to improve.
