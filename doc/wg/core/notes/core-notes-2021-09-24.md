# Tock Core Notes 2021-09-24

## Attendees

 - Gabe Marcano
 - Amit Levy
 - Arjun Deopujari
 - Alexandru Radovici
 - Leon Schuermann
 - Hudson Ayers
 - Jett Rink
 - Vadim Sukhomlinov
 - Pat Pannuto
 - Johnathan Van Why
 - Brad Campbell
 - Philip Levis

## Updates

 * Philip: Chatting about UART HIL, thanks Pat for the comments, making some progress there.
 * Leon: Hudson updated PR, have seen some failures. Tock registers, problem with `try_from` trait from latest Rust.
 * Brad: Anything we can see?
 * Leon: Just the CI failures.
 * Brad: I'm having a hard time following what's going on with Tock register.
 * Leon: It's nothing specific to register, this is just where it fails in the CI. On the latest Rust edition, our trait colliding with new trait from Rust.
 * Jett: Can we just rename our trait.

Seems to be a consensus that renaming our trait would be a good fix.

## Hudson's presentation on Tock size optimization

Hudson presented on work on reducing code size at Google. Has asked notes not to be included.

## App ID discussion

 * Phil: Based on previous discussion, I made some updates to the TRD. Major changes relate to application identifier that could encompass multiple processes, backed away from that. Wanted to decouple grouping from concept of App ID. What the text now says is that the Tock kernel will never have 2 processes running that have the same application ID. Implication is that this might require the kernel to walk through all processes to check this. It also talks about App identifiers will persist. If an application runs, we powercycle the device, will it get the same identifier again? Current text says this is true for global identifiers, but not for local. These are kind of the major changes. Thoughts?
 * Amit: Is this OK for the kernel, when it can trust TBF headers, to use these without checking process?
 * Phil: No. Because someone might install two--
 * Amit: Sorry, if there's an external application that statically checks all of this, like with OpenTitan?
 * Phil: Well, if they make a mistake, but I guess you could do that.
 * Jett: I would think in that case have the verifier approve everything. We have everything concatenated together and that image is signed and we have a stage in the bootrom that does the verification of the image.
 * Leon: I would assume this check runs as part of the old processes function and iterate over the processes loaded and presumably one could write their own processes function that would not call out and do this check.
 * Jett: So this check happens before Tock is running. We wouldn't use Tock's verification because the verification happened before.

Discussion on will there be code in the kernel to do this check?

 * Amit: I'm asking if TRD is specific in this regard. Basically, if our upstream implementation is that our kernel always does this verification, and Titan wants to remove it because they can check this statically. Not ideally for them to be carrying their own patches to do this, but does this violate the TRD, or would it just be a code change?
 * Phil: I think this would violate the TRD as it is written, but it's a draft, we can change it. We can change it so that the kernel does it unless there is an out-of-band mechanism to do this.
 * Jett: I was imagining that the two new traits being defined were going to be on something on kernel resources where board/project could give their own implementation. There's a standard one, or could override with your own.
 * Philip: Right, but these traits don't do the check, they're ways to look things up and map. The thing that would do the check would be in the kernel.
 * Jett: It would be nice if we could push this checking logic to somewhere we could override.
 * Brad: This is a very different promise, if we make it optional, the kernel can't provide this guarantee if it doesn't do the checks.
 * Phil: We can say that the verifier has to do this, that either it does this check or rely on an out of band mechanism to ensure this is true.
 * Amit: There's various ways we can interpret this. It can make a promise because it made the check itself, or because it is relying on an assertion that it believes it to be true, sort of like relying on the type checker for safety.
 * Leon: I think currently the TRD is saying if the verify policy assigns the same application identifier, then the Tock kernel must not run more than one of them at any given time. I think this statement would still work if we use static assertion. Just saying that this must not happen is sufficient to say that in the upstream policy we want to verify it because we can't otherwise reasonably assert this check, but downstream folks may do their own thing, and if they're in violation of this statement, then any resulting undefined behavior is going to be their problem.
 * Amit: Right, this seems good to me. I don't know if we need to specify how this implemented by Titan or whatever. To the extend that we can, avoid dictating a particular way to do this that might be redundant with Titan's checks.
 * Leon: We should not weaken the requirement as currently written. We should require downstream developers to respect this, so we should leave it as must.
 * Jett: The check should be in the kernel and not overridable.
 * Phil: Wait, this means the kernel must enforce this and must always do a dynamic check.
 * Jett: There's a lot more than just checking that there's more instances of the same local IDs. If you're using short IDs should be relatively cheap to check. If we can have this check in the kernel, but the actual loading of if this app is safe or trusted or something like that, this is what is customizable and what can be overridable.
 * Phil: I think the check will be really simple, especially if it's done with short IDs.
 * Amit: Oh, mapping global identifiers to short IDs, and because, if you're doing that, if the short IDs are unique, so are the global ones.
 * Phil: Yeah, but actually it's the other way around, because short IDs must be unique, then global ones need to as well, otherwise compressed traits get really complicated or non-deterministic.
 * Jett: Something I was thinking, compressed trait, depending on how easy or hard it is to pull it out, we can override it.
 * Phil: Absolutely intended that a verifier policy is defining compressed.
 * Amit: Right, so the check is literally just a loop, and checking for a 32 bit number and not a huge string.
 * Phil: Yeah, so need to be careful about the short IDs and how they're assigned.
 * Amit: OK, I buy this now, I'm convinced.
 * Jett: Yeah, keep it in the kernel.
 * Phil: I agree that if we were checking at application identifier level, which are long cryptographic things, I don't want it scanning 4Kbit keys.
 * Johnathan: Haven't had the time to read through the last set of changes. My only overarching concern is this doesn't do anything for storage. Implicitly says that we'll provide security for apps known to the kernel. Apps that don't have a short ID don't get security for storage or really anything else, and I'm not sure it's something we want.
 * Phil: I think the thought was to get this part right, then let's tackle the storage question. I think that short IDs and storage IDs need to differ is right, but it could be that we provide some trades about storage IDs and what are the access permissions for short IDs to storage IDs.
 * Phil: This is not big, but it's going to have a lot of implications to the system on security. I'd encourage everyone to read over it, and think about it.
