# Core Working Group Call Notes
## August 6th, 2021

## Attendees
 - Branden Ghena
 - Amit Levy
 - Leon Schuermann
 - Hudson Ayers
 - Brad Campbell
 - Jett Rink
 - Pat Pannuto
 - Gabe Marcano
 - Vadim Sukhomlinov
 - Anthony Quiroga
 - Alexandru Radovici
 - Philip Levis


## Updates

### Panic tool
 * Hudson: panic finding tool is still in progress. Expect a PR after Tock 2.0. Usually straightforward to find panics given the information it returns.
 * Amit: How does it find panics?
 * Hudson: Takes an ELF file with debug information. Walks upward from the panic format function, which is called by all functions. Looks for calls to the library functions that call panic format. The debug information on those calls can give file and line numbers.

### Tockloader
 * Brad: There's a little support in Tockloader now for manipulating a TBF header within a TAB file. That lets you test process loading stuff with fuzzing. This let me test that an unsupported app is actually rejected.


## Tock 2.0 Testing and Release
 * Amit: We have RC1 out and Brad has been doing testing so far
 * Brad: RC1 was tagged last Friday and since then I've been testing on SAM4L, nRF52, and a little on RISC-V. So far like ten little fixes for things I've found. Along with some stuff to Tockloader. Nothing major so far, just little stuff.
 * Brad: Most of those PRs are in, so it might be good time to tag an RC2 very soon here. I'd say once the PRs are in, Tock 2.0 passes sanity testing at least and it's time for everyone else to run test suites.
 * Amit: What should the rest of us do to accelerate this, since you've been doing the bulk of the work?
 * Brad: I've been tagging PRs with "release-blocker". I think there's just one for the nano33 USB sitting around. Libtock-C stuff has all been merged, I believe. PR #2731 (https://github.com/tock/tock/pull/2731/files)
 * Amit: Why is the USB delay from #2731 a thing?
 * Brad: Basically different OSes have different implementations of when they start receiving on the USB port versus when they send control messages. On Linux it's very fast. On MacOS it seems to be slower.
 * Amit: Oh, it's not that the USB stack isn't working. It's a delay on the host side that isn't being buffered.
 * Brad: Right. If you look at the USB traffic, everything is fine. It's just that the host program isn't getting it. Definitely problematic because missing the boot-up messages makes it look like nothing is working at all.
 * Amit: Okay, so RC2 is just waiting on #2731?
 * Brad: I am in favor of that
 * (general agreement from attendees)


## HILs and HIL Implementation
 * Phil: Something that came up with Alexandru looking at SPI, we wrote this document about how HILs should work. But older HILs don't really look this way, since they're old and before we had this much experience. So it would make sense to go through our HILs and make sure they meet our intended design for the state of HILs. Seems like this would be worthwhile to do.
 * https://github.com/tock/tock/blob/master/doc/reference/trd-hil-design.md
 * Phil: So, the question is how we should do this. It seems like we should have HILs in line with our guidelines.
 * Leon: I think this is a great idea. It would be great to get the HILs working similarly, but we can also standardize naming of certain methods, because they're all over the place. Not a technical concern, but you have to look up what the method is named any time you want to use something.
 * Phil: A trivial example is that getters and setters are different in different HILs.
 * Leon: Uniformity would make it easier when making new HILs, which will be copied from old HILs.
 * Hudson: Controversial statement. We have ~30 HILs, most of which are in service of one or two hardware platforms. Updating all of these is going to be a lot of effort. Analog Comparator HIL, for example, is basic and doesn't really extend to multiple chips. My guess is that the energy of the core team shouldn't be spent on all of these, and instead we should pick a few very important cases. I think fixing all 30 should be relatively lower priority than other things in Tock.
 * Phil: I agree with that. Only a few are ready for TRDs and stabilization.
 * Leon: I would agree. Even the "core" HILs have lots of discrepancies.
 * Amit: A proposal, we have a bunch of different types of capsules that are all intermixed in one crate. The console driver and the alarm driver are different from say, a specific sensor driver. So we could add a hierarchy of which capsules are more important and which are possibly better examples of how to make capsules. So the proposal is that there's some separation on capsules just like on HILs.
 * Leon: So on quality?
 * Amit: Let's say on "core-ness". There are HILs that every board in the main repo MUST have and therefore the chip MUST implement them. Maybe they aren't all up to the standard today, but they should be. Then there are HILs and drivers that are more specific. Some might be very high quality, but are more likely to not apply to every board or need modifications in the future. So I think how "core" they are might be a good divider.
 * Phil: We've talked before about what is the "core" of Tock versus contributions that aren't maintained at the same level. There are some sensor drivers where only one or two people have working hardware. We want them to be available, but it does make testing tough.
 * Leon: I'm concerned whether there should be a technical separation or if it's just a documentation issue. I think there are good reasons for both of them. We could think of importing something from a non-core HIL as having less stability.
 * Hudson: I think for capsules there's some reason to want to put them in a different crate. Out-of-tree boards might not want to rely on the non-core capsules. Especially because there's still an issue with Cargo and procedural macros where just compiling code could be a risk.
 * Amit: So, for example, all the stuff in the "net" module, the 6lowpan stuff, is a lot of code and it's nice, but if it were a separate crate, then someone without networking at all, various out-of-tree stuff can ignore it altogether.
 * Phil: How about kernel::hil and kernel::extensions
 * Leon: Well that's not about "core-ness" as much as "whether you want to include it". We could split into technically different crates. Buses, networking, sensors, etc. Just making the distinction on whether it is "core" feels arbitrary. And means drivers might move between crates based on that subjective criteria.
 * Amit: Functionality does make sense. I think that also happens to separate them based on "core-ness" or level of attention.
 * Phil: A thing to be careful about is that we're not talking about code quality. It's more degree of attention, maintenance, and use.
 * Alexandru: What about both? Functionality and maintenance. Core capsules split by functionality, then a separate crate with "universe" stuff made by other people, like ubuntu.
 * Leon: It could become a lot to manage, many crates. Ubuntu does this, but has the same apt command to install anything. Whereas, for Tock, someone implementing a sensor that goes in universe. Then moves to core because it's "important". Feels weird.
 * Amit: I think this would just work itself out. We would probably separate capsules like "temperature sensor" which are generic over multiple underlying sensors. And ones that are user-level drivers exposing a particular piece of hardware. I imagine that things that would normally be in "universe" would tend to be more one-off sensor drivers or quirky hardware peripherals. Whereas things in a "sensors" crate would be things exposed in a generic way that are general-purpose.
 * Alexandru: That's more-or-less what I was thinking. Specific sensors would be in "universe".
 * Leon: So splitting interfaces and virtualizers from actually hardware drivers. That does have the problem that importing one crate pulls in drivers from all sensors.
 * Phil: The difference between "universe" and not is the administrative domain of who maintains them. So sensors that the core group supports are fine. But we don't have a mechanism where someone can contribute a sensor driver and NOT be caught up with testing at each release.
 * Brad: I like the categories I added to the README https://github.com/tock/tock/blob/master/capsules/README.md
 * Alexandru: We do have to be careful that we never want the import path of things to change. Moving stuff has a high cost.
 * Amit: If there is not a board or chip in the main repo that uses a certain driver, does the driver belong in the main repository? We do have a system where boards or drivers can use things not directly in the tock repo.
 * Leon: If we had a crate with virtualizers and a crate with hardware, that might make it easier. Many cases right now have drivers, virtualizers, and userspace interfaces all in one file. A system for this might make it easier for people to keep things out-of-tree and use stuff. If we split up interfaces from hardware implementation, we emphasize the importance of the interfaces.
 * Amit: I agree
 * Amit: So to summarize, going back to Phil, I think we agree this is a good thing to do. How do we go about this? It's the next focus after 2.0 maybe?
 * Phil: The big question is who wants to do it. It's a lot of work to think about the APIs. We should first come up with a prioritized list and see if people are willing to volunteer.
 * Phil: I've found, for me, that writing the TRD first, at least a draft, makes it easier. Explaining how things work with exposition helps clarify things to me. I did a SPI draft. I'm starting to work on UART. We still have I2C, ADC, etc.
 * Leon: My takeaway from TRD 104 is that it doesn't make sense for one person to write it all down. It needs to be iterative with more than two eyes on it to make sure it's right. Just looking at the "final product" tends to lead to overlooking things.
 * Phil: Yeah, writing is always better with an editor.
 * Amit: Okay, so let's make sure to bring this back up in the agenda after the 2.0 release.
 * Phil: That sounds good. In line with the capsules discussion, we should probably have two categories of HILs. Very important, should be standardized. Or just smaller, less used, not ready stuff. GPIO versus Analog Comparator, for instance.
 * Amit: Yes. We still want both. But we don't have a perfect standardized version yet.
 * Phil: Yes. And if you look at what Alistair has been doing, we're missing HILs for some things like public key crypto.

