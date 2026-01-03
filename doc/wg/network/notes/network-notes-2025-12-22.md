# Tock Network WG Meeting Notes

- **Date:** December 22, 2025
- **Participants:**
    - Branden Ghena
    - Leon Schuermann
    - Johnathan Van Why
- **Agenda:**
    1. Updates
    2. IPC and AppID
- **References:**
    - [IPC RFC](https://github.com/tock/tock/pull/4680)
    - [AppID TRD](https://github.com/tock/tock/blob/master/doc/reference/trd-appid.md)


## Updates
- Tyler: (from slack post) STM32WLE5xx support PR https://github.com/tock/tock/pull/4695 low-power SoC with an integrated SX126X radio for LoRa. It's a rather large PR that could use eyes.


## IPC and AppID
 * Branden: I think what you're getting is we could still use process IDs for IPC, and if we wanted to, we could also look up the process IDs to app ID map. Maybe a short ID
 * Johnathan: That's sort-of right. For Tock security, it's required to have that lookup option. It makes more sense to use AppID instead of short ID.
 * Branden: Put a pin in that, I'm not convinced there is an app ID we can hand out to userspace. I think it's a concept, not a concrete type.
 * Leon: Does that work the other way? Can applications get a process ID for a given application ID?
 * Johnathan: Maybe. That leaks whether the application ID is running. Which is probably fine to leak.
 * Johnathan: On, does AppID exist, I guess it seems like it doesn't. There should be a function to return this
 * Branden: The AppUniqueness verifier implementations all seem to: return true, use ShortID, or use package name.
 * Johnathan: It really should be implemented though. There should be a way to get an AppID.
 * Leon: Makes sense, could be tricky to implement.
 * Johnathan: Isn't the type an `&[u8]`?
 * Leon: Yes, but it could also be dynamically determined at runtime, so we'd have to allocate memory somewhere. An owned generic thing could be better
 * Johnathan: AppID probably sits in TBF headers anyways.
 * Leon: I think we didn't want to limit ourselves to that, although it might be the case in practice
 * Branden: Could we use ShortID?
 * Johnathan: Well, userspace can't use a ShortID meaningfully. It would have to convert a ShortID into AppID or AppID into ShortID and compare them. A ShortID is dynamic at runtime. An AppID is fixed and known.
 * Branden: But an AppID isn't a real thing?
 * Leon: I think for signed apps, the public key for the app is the AppID, well a hash of a public key. That's the long AppID.
 * Johnathan: 32 or 48 bytes of data
 * Leon: So that's too expensive to pass across the boundary a lot
 * Branden: You'd just use AppID once to confirm things
 * Leon: ProcessID refers to an instance of a process, changes when it restarts. Two instances of one binary have two ProcessIDs. But multiple binaries have the same ShortID.
 * Johnathan: Yes, they can't run concurrently though if they have the same ShortID.
 * Leon: I thought if there were two applications with unique AppIDs, but hashed to the same ShortID (hash collision), then you'd choose another ShortID. So I thought it would be totally fine to have multiple instances of applications have the same ShortID.
 * Johnathan: That's actually for IPC. If you wanted to refer to an application by ID, you need to know that that's only one running thing.
 * Leon: So for a theoretical version of Tock that can run multiple instances of a binary, those instances are supposed to have different AppIDs, for instance by adding some "instance" ID to it.
 * Johnathan: It's also worth distinguishing how AppID is supposed to work. Per the TRD, ditch ProcessID and use ShortID.
 * Branden: Issues. We need an identifier that changes when the process reboots. And we don't want to make a different mechanism for handling that if we don't have to.
 * Branden: Stepping back, the goal is that you'd hard code an AppID for a process in userspace, then you'd compare against it to validate things.
 * Johnathan: Yes
 * Branden: Can you trust an AppID more than a Package Name?
 * Johnathan: Yes. An AppID can be from a public key. So only the entity controlling the key can sign a ProcessBinary so that it has that AppID. Which prevents impersonation.
 * Leon: The additional guarantees stem from the fact that this AppID can only be produced after the kernel has checked the signature. So the AppID attests to the kernel's verification process. Meanwhile, a package name could still be verified, if the whole app is signed, but it depends on the kernel setup for whether applications can be loaded without signatures
 * Johnathan: I don't quite understand the last part
 * Leon: If we sign the ProcessBinary TBF Headers, that includes the PackageName header. So you naively assume that if you have an application that's signed, that PackageName is also a trusted thing.
 * Johnathan: Oh, that's naive though
 * Leon: Right. If the kernel were to only load applications that are signed by trusted public keys, then you can trust that. But that's not always the case. Instead applications could not be signed and use any PackageName.
 * Branden: I'll note this is neither extreme of checking. We're checking some things but not all things. If we check all things we don't care. If we check nothing we don't care.
 * Johnathan: This is something missing in the threat model. Use cases, threat model, and how to configure kernel and what security properties you get. This would be multiple companies signing apps, not trusting each other, and running on a system.
 * Branden: I agree that's a use case, although I'm not sure it's the most common
 * Leon: I was tracing down why package name can't be trusted. This hybrid setting is why.
 * Johnathan: I think there are cases where package name is untrusted that aren't this hybrid setting. I don't have a concise summary, but if you look at Android where apps are written by different developers, they may want to share data. Applications can't be trusted, but developers can sign them to keep trust.
 * Branden: That's still the hybrid model to me. Multiple signers that you don't all trust.
 * Leon: Okay, that clarifies to me when applications should use AppID over package name.
 * Branden: So lets say that you want to hardcode AppIDs for verifying. How would you get it? Run it in  Tock and copy-paste it?
 * Johnathan: The author could publish it.
 * Leon: You could run the hash yourself outside of Tock for those cases.
 * Branden: Is that enough for trust? If this ProccessID's 48 bytes match the expected 48 bytes, then you trust this server?
 * Johnathan: Yeah, you trust that the developer owns the public key. Given that the OS works right
 * Leon: Right now AppIDs are an untyped system of bytes. If multiple schemes in the kernel extract AppID, then there's presumably a chance that different schemes could collide. Then comparing this AppID wouldn't be sufficient. That would be solvable if AppIDs were typed/prefixed by an identifier that describes the in-kernel scheme that produced them in the first place
 * Johnathan: I agree with that
 * Branden: But a kernel will only run one AppID scheme, right?
 * Leon: I don't think that's necessarily true. Imagine you have two different types of public keys. RSA and Curves. There could be collisions.
 * Branden: There could always be collisions.
 * Leon: A different system could just use AppIDs in TBF Headers
 * Branden: But you'd do manual AppIDs or from keys, not both at once.
 * Leon: I'm just worried about being open to attacks that we didn't want
 * Johnathan: Opens us up to mistakes
 * Leon: Yes, exactly. It's a design that makes you skittish. Not being able to tell what produced the AppID. And also being able to compare two AppIDs and have a chance that they're treated as identical even if generated by different schemes. That's a bad practice
 * Branden: I do agree, but I will say this is a scenario that's weird. You care highly about AppID but also didn't bother ensuring that the kernel only uses one clear scheme
 * Branden: I see this as a third discovery mechanism for IPC. You could discover via AppID.
 * Branden: If you wanted to implement AppIDs, then you could use the pre-fixing based on scheme when doing so
 * Branden: Is this worth implementing though? It's a lot of work
 * Leon: If you expect downstream users to implement their own discovery mechanism, and we don't strongly care about demonstrating a secure version
 * Branden: The trustworthy version is where all apps are signed by one signer. Then you can trust all of those apps.
 * Leon: That doesn't sit right with me. AppIDs are one unified mechanism for trusting apps. But then you're saying "ignore that" and just expect that all apps are signed.
 * Branden: I'm focusing on the use cases. The company use cases right now all seem to have entirely signed applications / entire binary. So they don't need to validate individual applications like this
 * Leon: It is a chicken-and-egg problem, where we'd maybe use this if it existed, but have a hard time bothering to implement it just for our use
 * Leon: Maybe a good middle ground is to have a story and a document for how discovery should work with AppID. And then kick the implementation can down the road. I agree that we shouldn't waste time digging deep into this rabbit hole that's sort-of aside to IPC.
 * Branden: For documentation, should it be discovery or just post-facto validation?
 * Johnathan: For discovery it could be fine. Having an extra discovery mechanism could be nice if you have just a "thread server" and there are multiple instances that could be used for it.
 * Leon: Implementing verification/validation first doesn't prevent us from doing discovery later. For a package name or string-based system, we'd want to be able to do AppID verification. Then AppID discovery wouldn't need verification, but that's just a third mechanism.
 * Branden: Okay, we'll document a validation/verification mechanism too then.
 * Johnathan: I do feel bad to not have support for this, but I think the thread model is ambiguous about the need for it. Given that it's easy to retrofit, I think that's not the end of the world
 * Branden: We would want to be clear that we're just not supporting that threat model. If you care about that, you shouldn't use the mechanisms that we implement as-is.

