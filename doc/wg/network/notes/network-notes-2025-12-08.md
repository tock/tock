# Tock Network WG Meeting Notes

- **Date:** December 08, 2025
- **Participants:**
    - Alex Radovici
    - Tyler Potyondy
    - Branden Ghena
    - Leon Schuermann
- **Agenda:**
    1. Updates
    2. IPC Discussion
    3. Over-the-Air Updates
- **References:**
    - [Libtock-rs 15.4 Issue](https://github.com/tock/libtock-rs/issues/586)
    - [IPC RFC](https://github.com/tock/tock/pull/4680)


## Updates
### IPC Industry Use
- Alex: Talking with industry, pointed them at RFC and one-copy IPC might be acceptable. Also concerns its too slow, but zero-copy is hard especially to isolate for security.
### Libtock-rs 15.4 Failure
- Branden: Libtock-rs issue about 15.4 support: https://github.com/tock/libtock-rs/issues/586
- Tyler: My suspicion is that they're using the wrong kernel image, but I haven't tracked the libtock-rs implementation lately. Probably libtock-rs should be added to Treadmill tests
- Branden: Hard here as you might need to live on libtock-rs master, but that might fail for unrelated reasons
### Tensile Tests
- Tyler: Actually, Tensile failed last night, unrelated: https://github.com/tock/tock-hardware-ci/issues/44
- Leon: I think it was a transient error in Treadmill. I'm going to rerun manually and see if it's good. It would be nice to not have these false positives, but I don't want to mask true failures.
- Branden: This also seems like experience to build over time, what are the errors and are they common enough to worry about
### Environmental Sensing
- Tyler: Working on UC Santa Cruz project, sort of like signpost 2.0. Supposed to be easy platform for non-CS people. We're porting it to Tock. Minimum viable example is working. LoRa in libtock-c has only been tested on Apollo, but we're running on the STM32W which has LoRa functionality. We are able to connect to TheThingsNetwork for a long duration. There will be stuff upstreamed there soon.
### Typesafe Hardware Work
- Leon: Tyler's work presented at Tockworld is going to be in ASPLOS this year!
- Tyler: Will send around copy once camera-ready is done.
- Branden: Awesome to see Tock being useful to research work!


## IPC Discussion
 * Branden: RFC got posted: https://github.com/tock/tock/pull/4680
 * Branden: Two major points to discuss. 1) How service "names" are registered. 2) How important is server authentication?
 * Branden: For this first part, there are at least two mechanisms for discovery we've thought of. I thought of using string names provided by the application code at runtime. These have to be fixed-length to be stored in Grants. Upside is that application can pick whatever and it's clear in the source. Alternative Leon mentioned was using Package Names in the TBF header. This is what the old IPC used. It's nice because application _can't_ just change it, but I'm worried that it's a bit hidden in the build system which makes it hard to figure out.
 * Alex: I'd go for the TBF header so an app can't lie.
 * Branden: That can lie though. At compile time someone could set that to anything.
 * Leon: But you can verify it and with app-signing you can know that apps come from a signed place. Technically the name and the app code are both signed. But the header is MUCH easier to verify. Harder to check that the name that's allowed matches a promise.
 * Branden: So not fundamentally secure, just easier to trust.
 * Leon: They are verifiable where application code is not
 * Branden: One downside here is that the name is in the compilation system, not in the application. That's not an issue for Microsoft
 * Leon: We could have a hardcoded string in the application and have Elf2Tab grab it if we wanted to.
 * Alex: Why is that an issue?
 * Leon: It's in the build system, but not in the application source code.
 * Alex: They could find it in the build system.
 * Branden: How does elf2tab pick a name if you don't specify it?
 * Leon: I think the libtock-c build system uses the folder name by default. It _is_ possible to grab from the application source code.
 * Branden: I'm worried application source code is more magic. Have some magic identifier you have to use
 * Leon: Actually, right now for applications we use as IPC services, we override the name in the makefile.
 * Tyler: Yeah, that really tripped me up. https://github.com/tock/libtock-c/blob/master/examples/rot13_service/Makefile#L9 PACKAGE_NAME here
 * Leon: The other unfortunate bit is that the ROT13 service right now, you can't terminate because the package name is too long to parse as a string in the process console. It cuts you off. That's a separate issue though
 * Branden: We could also trivially make the name something different. Doesn't have to be org.tockos.*
  * Tyler: Are we planning an app-signing mechanism? If you're shipping an application as a service, you probably want it to be signed or know the origin. Are we thinking about that?
 * Branden: Tock already has support for that.
 * Tyler: Are we considering that in the IPC design at all?
 * Branden: Yes and no. Registration is asynchronous explicitly because we don't know what the registration mechanism might be, and we want to allow for a system that checks signatures and says "no". But for now I was thinking about not having any of that implemented. Given that we have app-signing in Tock already, it's not clear to me that IPC should be double-checking stuff.
 * Leon: Yeah, IPC shouldn't
 * Alex: Industry I've talked to really just want a signed binary with multiple applications and the kernel. They don't want to side-load apps.
 * Alex: They know for sure the ShortID. We could search by ShortID instead of by name.
 * Branden: How does ShortID work again?
 * Alex: ShortID uniquely identifies a running binary. You can't run two apps with the same ShortID. It's identifying an application, not a process. The problem here is that it gets derived from the credential, or it gets assigned uniquely locally which is hard to find.
 * Branden: Are ShortID and AppID the same?
 * Alex: No. AppID is much longer. ShortID is generated based on the credential, AppID, or whatever.
 * Branden: That sounds really hard for normal users to use. Fine for companies. Totally reasonable third mechanism.
 * Leon: I think there's no one correct discovery mechanism. Different threat and usability models. Vastly different. Trying to cover a lot of ground with one mechanism will break a mechanism. Doesn't seem possible to find one best mechanism.
 * Leon: So maybe something fine for users, but allowing downstream users to implement something else, seems fine.
 * Tyler: I agree with Leon that what Alex explained is very different from my own sensor network model. I don't think we should just do the simpler version though. We should future proof it. This is a pretty generalizable mechanism, the same across all use cases. Maybe we could start with the simple case, but we could expose a trait, like a pluggable scheduler, where we could design the operations exposed to change what goes on under the hood. Doing this with a trait could alleviate concerns.
 * Leon: I'm less convinced a trait would work, as the identifier provided is going to be pretty different. The protocol will be different too. I don't think a standardized trait will work for discovery. Seems overly restrictive.
 * Tyler: Is it though? You could make a less opinionated interface that accepts an array of bytes. Could be generalized.
 * Leon: Using something like ShortID probably requires a multi-step identification process. I'd guess that you have to do one lookup for the ShortID and another query for whether there's a valid signature behind it. I'm not convinced it's the same interface. Maybe a good first step is to try to compile as many scenarios as we can think of. Check with stakeholders about what they need. We could then look for whether there should/can be one shared interface.
 * Leon: Even if it isn't shared, there should be one clear resulting identifier that can be used in other IPC mechanisms.
 * Tyler: The only reason I'm pushing back on the multi-capsule design is that it feels like it could be brittle, with differing errors for instance. 15.4 has this problem where we're maintaining multiple interfaces. It would be good to have a generalized interface.
 * Branden: Proposal is two registry mechanisms. That should make it easy for someone to make a third version eventually. And it would let us look at the two and see where commonalities lie.
 * Branden: Also, given that we want to show that the IPC ecosystem is able to support replaceable capsules, it would be great to have an example of that initially to encourage people about it.
 * Leon: It's also true that no one here knows much about app-signing. Maybe I'll need to do some reading into how that works. We'll have to think about how that links into IPC and a secure discovery.
 * Branden: I'm still thinking we might not need much IPC support for app verification, given existing app-signing stuff. But I'll agree that I don't know for sure.


## Over-the-Air Updates
 * Branden: Sorry we didn't get to this today
 * Tyler: Nothing urgent here. We'll discuss more in the future when we have time. Interested in loading, deleting applications. Including over LoRa. We'll discuss more.

