# Tock Core Notes 2020-11-20

## Attending
 * Branden Ghena
 * Amit Levy
 * Leon Schuermann
 * Arjun Deopujari
 * Johnathan Van Why
 * Vadim Sukhomlinov
 * Pat Pannuto
 * Brad Campbell
 * Phil Levis
 * Garret Kelly
 
 ## Updates
 ### OpenTitan
 * Brad: We decided to rename OpenTitan board to reduce confusion with the overall project. This is one kernel that runs on the hardware, not necessarily the only or final version.
 * Garret: We'll also have another top-level board pretty soon. So this clarifies which one this is.
 ### 
 * Johnathan: I sent a PR, the Platform Design Story, to Libtock-rs. And it's being discussed in depth. https://github.com/tock/libtock-rs/pull/256

## Dynamic App Loading
 * https://github.com/tock/tock/pull/2190
 * Brad: We want to give a quick overview to check people's thoughts. Right now it's just the traits, but an implementation is in progress.
 * Arjun: (Shares screen. Diagram shows interactions of modules as a cross. AppLoader in the center talks to four separate capsules: WirelessProtocol, DecompressionModule, VerificationModule, and ValidationModule)
 * Arjun: DALS - Dynamic App Loading System. You should be able to load data from a remote source into flash at runtime. Then should be able to run an app from that data. Latest PR has most recent updates. There is a capsule AppLoader than handles everything and connects to other capsules. WirelessProtocol gets data. DecompressionModule decompresses the data. VerificationModule for crypto checks to verify signature. ValidationModule to decide whether to run that app right now.
 * Pat: Is validation different from some of the AppId stuff we've been talking about?
 * Arjun: Not necessarily. I wanted the system to be very modular and configurable.
 * Johnathan: If we have verified AppIDs, I expect the VerifiationModule to run the same checks as the app-startup-signature-check. So the decision of _whether_ to run it is separate. Right now we don't really have a decision of whether to run apps, maybe a disabled bit or something. But it happens before verification at startup. Where your model has it the other way around. Which isn't bad, just interesting.
 * Arjun: Do you think the verify/validate model should be flipped?
 * Johnathan: I think it makes sense in that order for the dynamic system. I'm just thinking back on my proposal for startup loading, which does validation before verification. I don't see an issue either way right now, but it's something to think about for sure. In DALS you want to see if it's corrupted _before_ you decide to run it.
 * Amit: The central component (AppLoader) receives potentially compressed blocks from the WirelessProtocol and then decompresses it. Why that instead of having the AppLoader instance receive already-decompressed chunks? i.e. having the wireless protocol decide on that decompression.
 * Arjun: So you're saying that whatever receives data might immediately decide to decompress it?
 * Amit: Yeah, you could decide on different schemes. Plus maybe USB doesn't have compression, but 6lowpan does.
 * Arjun: That definitely makes sense to me.
 * Brad: When we were thinking about this, we wanted to separate handling of data versus transmitting bits. So transport implementations don't need to implement decompression. So it made sense to have it be an optional thing called from AppLoader. Then we could reuse decompression scheme rather than building it into the protocol. Maybe instead WirelessProtocol should configure whether decompression is used.
 * Brad: I would say right now, I think it should stay as is. We can always have it be a null operation that's implemented in the future.
 * Amit: Definitely makes sense for decompression to be a separate module. But maybe it should be pipelined between protocol and AppLoader. Or maybe the AppLoader instance is the thing aware of the app protocol which can make decisions.
 * Phil: I think you want to separate layers. In the presentation layer, we add headers that decide whether it gets decompressed or whatever.
 * Leon: How does loading apps interact with a currently running app? An HTTP server might be a good usecase. There's a lot of complexity that we might want to keep in applications rather than the kernel. But is there an interface for applications to start an application update?
 * Brad: The current implementation is that the WirelessProtocol could be in the kernel or through a syscall interface. So the short answer is absolutely yes. In the short term, I don't know if it'll be implemented. But it's definitely part of the design for the long-term.
 * Amit: One thing that IS embedded in the design is that the chunks given to AppLoader from WirelessProtocol are written to flash by the AppLoader. Which seems like a reasonable choice, but it doesn't support the app already existing in flash and pointing the AppLoader at it. That would, of course, have MPU alignment issues, so maybe it's necessary.
 * Leon: I'm not concerned about the flash issue. I just wanted to check about syscalls.
 * Brad: There are a ton of interfaces here that are part of this that we're not talking about because it gets overwhelming. So you're talking about Validation, you've got an app and want to decide whether to use it. So maybe a single interface with the kernel could support both use cases.
 * Amit: So this high-level thing would also then hopefully handle loading from an SD card, where the "Wireless Protocol" is just loading chunks.
 * Arjun: For writing to flash, one big change that we made is that the AppLoader has a reference to a non-volatile storage driver. So you'd abstract that writing part away.
 * Leon: I think we're not concerned so much about the flash abstraction, but rather whether apps can be replaced in-place or whether they're written to another location in flash and then replaced.
 * Arjun: I think replacing in-place would be pretty complicated. I think it would be a lot easier to just make a new instance and delete the first one.
 * Brad: And it would be safer, in case something goes wrong. You are spot-on that the interface for which flash and where do things get loaded is not specified in this design.
 * Amit: At a high level this is something we wanted from the beginning. And we've been unsure about how much internal interfaces would need to change. How much of this do you think could go entirely in capsules versus changes to the core kernel.
 * Arjun: A lot of it could go in capsules. On the PR feedback, we original had AppLoader as a "trusted" capsule, but we got some feedback suggesting parts will need to go in the kernel. Anything touching flash is really where there are a lot of significant security concerns.
 * Brad: Everything in that design can be a capsule, but we do still need to verify the size of the TBF header before anything goes in the app linked list. So that will have to go one layer "below" this design. There are of course going to be many other questions too that come up.
 * Johnathan: In the security model, we want confidentiality of apps (don't share them), integrity of apps (verify signatures and TBF headers), and availability. But we already have to trust capsules for availability.
 * Amit: We might need a more subtle threat model to handle this.
 * Johnathan: I don't want to take something that could be cleanly implemented as a capsule and make it go in the kernel just to satisfy the thread model.
 * Brad: I think this fits in the current threat model. The core kernel has to provide a "safe" region of flash that the AppLoader can access. Then the AppLoader itself doesn't have to be trusted.
 * Amit: One concrete use case would be a region of flash that's "don't touch this" and then a region given to this AppLoader which only it can touch. And maybe it could mess up that flash and those apps, but it can't mess up apps in the trusted region.
 * Leon: That would work, but as soon as we give the AppLoader access to all the apps on the board, the ability to replace one app with another would be a security issue.
 * Brad: I'm thinking of this in the dynamic sense. So the core kernel is choosing whether to give empty flash or to allow an app to be overwritten.
 * Leon: I'm excited to see how that would look.
 * Amit: I think as a general notion, if the AppLoader is allowed to delete certain apps, then fine.
 * Branden: We're also talking about two things here. Which regions of flash can be accessed by the AppLoader, which the kernel can hand it and can happen entirely in a capsule afterwards. The other is which apps are currently running on a board, which the kernel will need to decide.

## Potential Tutorial
 * Brad: I just wanted to bring this idea up as a thought. Since everything is online right now, doing a virtual tutorial would be pretty natural. Post 2.0 would be a pretty natural point. And we support enough common dev platforms that people can reasonably purchase them before hand.
 * Amit: Especially with the NanoBLE, which is easy to get and has a ton of features,
 * Brad: The main issue with that board is that it doesn't work with Tockloader. Once we have a good way to add Tockloader support, I totally agree.

## Tock 2.0
 * Phil: https://github.com/tock/tock/blob/5da4e80ab43e2aad869948e83c4c3383a0c8cf26/kernel/src/driver.rs#L198
 * Phil: We've hammered out the new Driver trait. One of the approaches that Leon was arguing for and I think is a good one is that we've changed all instances of Driver to be LegacyDriver and now there's a new Driver trait. There's some inefficiency now, but that'll go away once we transition. We plan to write a few capsules based on the new Driver and then write down some guidelines about the best practices. We'll follow up with a list of updates needed and hand out work to everyone.
 * Phil: There's a Result return type now, and we discussed whether low-level pointers or high-level kernel types should be passed around. We decided to use the high-level types.
 * Phil: It might be that we have Results that the Driver has and then turn them into primitive types so we can do a copy. For instance, we can't copy an AppSlice.
 * Leon: We have these higher-level types that we want Capsules to work with, rather than low-level primitives. Especially because in the current design the capsules cannot access, for example, the pointer in a callback. But we might need to have Copy or Debug implemented, and might not want the larger types to be part of the syscall return. But the Driver point of view should remain constant.
 * Leon: The next steps are definitely still to implement the 2.0 Syscall in the scheduler so the kernel calls these methods on the Drivers. It will be compatible with 1.0 for now as we transition one step at a time. We'll port IPC and yield and memop which are independent of capsules. Then in one or two weeks when we're more sure nothing will change, we'll ask others to create PRs against this branch to change capsules. In parallel we might want branches on libtock repos to change them to new system calls too.
 * Phil: Just so everyone knows, Amit agreed to do IPC. None of us want to touch it.
 * Phil: So after thanksgiving we should be ready to go.
 * Leon: We do need some time to be sure that there aren't any necessary changes and that the buffer overlap solution is actually compatible. Because we'll have to do all the work if we tell people to port and then something changes.
 * Johnathan: This is the tock 2.0 dev branch? (Yes)
 * Leon: Should we do wrappers for raw system call interface in libtock-rs?
 * Johnathan: I'm already re-writing libtock-rs in place. Take a single crate and breaking it into multiple crates with different names. We might end up with two tock modules in libtock-rs one on 1.0 and one on 2.0. I'm not too concerned since there's a lot of driver rewriting going on anyways. We're making lots of efficiency changes.
 * Leon: So we only have to care about libtock-c
 * Phil: I'm hoping that once Leon and I do a couple, most of it should be mechanical.
 * Johnathan: Question. If I const allow a buffer, I have to retrieve it with const allow, right? (Yes)
 * Leon: And anti-aliasing will apply across the two allows too.
 * Phil: The identifier spaces across allow and const-allow will be unique.
 * Jonathan: One thing that's nice about 2.0 is that in 1.0 I had to force streams of data for Hello World to go into RAM, which was unfortunate.
 * Phil: Yeah, with test routines all the buffers have to go into RAM too.
