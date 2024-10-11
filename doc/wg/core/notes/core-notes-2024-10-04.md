# Tock Meeting Notes 2024-10-04

## Attendees
- Branden Ghena
- Lawrence Esswood
- Hudson Ayers
- Leon Schuermann
- Chris Frantz
- Tyler Potyondy
- Johnathan Van Why
- Kat Watson
- Brad Campbell


## Updates
### Treadmill
 * Leon: Treadmill update! We now have a first step of automated testing of Tock merged. This runs on our own infrastructure on a VM that has access over USB to a real hardware board. We've been fixing some bugs we're seeing as we run. We're also working on scripts for particular tests we want to run. Ben should have a PR for this in the next week or two.
 * Leon: One great thing: most people shouldn't have to care about Treadmill, it _should_ just work. What tests we want to run is actually something people will care about though, so I look forward to much discussion on that PR.
### Build scripts PR
 * Hudson: I had a chance to look back at the build scripts PR issues, and I think I have the build fixed there
 * Brad: Hooray! This includes the linker stuff?
 * Hudson: Yup. CI is running now, but I think it's good
### RAM Applications
 * Brad: There's been a bunch of discussion lately on running applications directly out of RAM. We should be able to have apps that relocate themselves to RAM, with no modifications to the kernel on today's platforms. Amit is really the one working on this
 * Lawrence: What if you're loading from somewhere that's not executable?
 * Brad: That's a future platform rather than today's
 * Lawrence: A patch I had was to do this once. There was a kernel loader that handled it.
 * Leon: It would be good to connect people who are interested and/or working on this. It's also good to spread awareness


## Pointer Returns from Syscalls
 * Initial email: https://lists.tockos.org/hyperkitty/list/devel@lists.tockos.org/thread/ACZBGBDXYF6UMB64BWXM63MFGKZYCD76/
 * Lawrence: Right now syscalls don't just return values, they also have an enum of what type they send back. Some things are sent back as u32s when they're actually pointers. So something needs to be done for CHERI.
 * Lawrence: Some options, we could add new enums, or have a variant on CHERI. We are worried about breaking userland-kernel interface stability. I'm curious what approaches we ought to take. How important is backwards compatibility?
 * Lawrence: One option is a Tock 3.0 where we change the success result from Allow, which is currently specified as a u32, but that's not always our pointer size
 * Leon: We definitely don't want to break our ABI across minor releases. So, we could have a variant that changes for your new platform but is the same as your old platform
 * Lawrence: Right now, I've got a system that's doing that. So return usize would exist, that would then transform into a u32 on existing systems and u64 on CHERI
 * Leon: I also think there's a question about why we chose the types that we chose. There was frustration by Phil and I about the lack of knowledge about sign and pointerness for things sent in Tock 1.0 to userspace. So we really wanted to stabilize things.
 * Lawrence: Were people using the ABI directly in Tock 1.0 instead of a wrapper like libtock-c?
 * Leon: People were using libtock-c, but drivers there were varied in what they did
 * Leon: Overall, I do think more semantic information is useful in this interface. So a pointer versus number return result would be valuable
 * Lawrence: We'd actually end up with a pointer plus usize for allow returns. I'm a little worried that it would explode into a bunch of new return options
 * Johnathan: For a Tock 3.0, we could even have a bitfield that describes the types returned
 * Leon: I am not so worried about the explosion, because we can add them lazily as needed
 * Lawrence: If they're Rust enums, is there a danger that the kernel translates incorrectly? Does it just transmute, or does it check?
 * Leon: It checks.
 * Lawrence: And there's an assumption that the user trusts the kernel to return a valid enum
 * Johnathan: Is this about the different types the kernel can return? That seems to be expansible in the future?
 * Lawrence: Yes. I was just worried that the types of enums in userland and kernel could be out-of-sync
 * Johnathan: `libtock-rs` certainly assumes there may be new syscall return variants in the future
 * Leon: Is there any dissenting opinion about these options?
 * Brad: Not quite a dissent, but I struggle with how we're going to resolve this if we assume we can have pointers anywhere. If a command argument can take a pointer, for instance
 * Lawrence: I think the plan would be to not allow that. Those must go through allow
 * Lawrence: Two more places the size pops up is command and upcalls. We were originally focused on return types, but all are worth discussion
 * Brad: I do think we're all in broad agreement that this should be addressed. But it feels like something more structural here than just choosing types. Maybe I'm not quite right about that or others have an understanding I don't. I think there's a general concept, above and beyond Tock 2.0, and we need to record that wisdom as direction for this change
 * Lawrence: To clarify, I think the majority thinks that minor changes can't add new type information to the enum. In terms of how this should progress, when should patches land that introduce new types?
 * Leon: For 2.0, we had a branch. We did one last 1.x release slightly before merging 2.0 into master. I am a little worried about leaving this change in a branch where it bit-rots for a while
 * Lawrence: Mostly this could be backwards compatible to start, and CHERI could just live in a branch until ready
 * Leon: I do also think there are a lot of things going on in your PR. Metaptr and raw pointer and things
 * Lawrence: I think I'll be reducing those
 * Leon: I don't mean reducing variants, I mean understanding the point of each type for drivers
 * Lawrence: For CHERI I can be specific. We do want to be more generic though. For CHERI, at any time, userspace might need to be able to access an object at a location, rather than just sending an address. So an authenticated pointer is bigger than u32, but just an address, like a buffer they already own, is usize.
 * Leon: Do you really ever want addresses without access rights?
 * Lawrence: Maybe the free interface. Or locations of buffers that applications already own
 * Johnathan: About metaptr, does it represent capabilities that are pointers only? Aren't there other types of capabilities?
 * Lawrence: I'm also using it for the register file, so any capabilities.
 * Johnathan: So it's more than just pointers. Okay. The other thing that confused me, in non-CHERI you're relying on the cast to copy provenance information, correct?
 * Lawrence: I wonder if I could rewrite that wrapper layer with strict provenance in mind.
 * Johnathan: I used `*mut ()` in libtock-rs for similar things. Maybe I'm thinking too strictly about it.
 * Lawrence: Maybe I should change to strict provenance APIs. Would that cause issues across the kernel?
 * Johnathan: It's problematic. There is a library that would help, but the authors are not responsive about how we could license it.
 * Lawrence: Okay, maybe I'll make a partial step that direction but not a full one
 * Lawrence: Right now I'm either making it a usize or a CHERI type, which is wider on CHERI platforms. It should probably be `*const ()` in my Rust code
 * Lawrence: A cast to usize is probably still required. This is used for all the different register types you might possibly have, and I'm assuming that the one that holds a pointer is the largest. But just because those conversions exist, doesn't mean it should be usize.
 * Johnathan: I think metaptr is what I call register in libtock-rs
 * Lawrence: I do think they _could_ differ. Some CHERI systems could have separate register banks, for instance.
 * Lawrence: Okay, I think I'm happy with what to do about return types from system calls right now
 * Lawrence: We also have to discuss the arguments to command, and the upcalls
 * Lawrence: For command, should the command number be u32 or usize? Probably u32 since it's just a number
 * Leon: I'm reasonably certain that the TRD104 already says that those numbers are u32s. And if it doesn't, we should
 * Lawrence: Should the other two arguments be usize?
 * Leon: I thought they should be u32s. But there was really no difference and changing just felt like churn
 * Johnathan: Libtock-rs already has them as u32s.
 * Leon: I think the same motivation applies for command arguments and return types. Wherever it's not explicitly necessary, the interface of drivers shouldn't change based on the platform you're running on.
 * Lawrence: But some things always change, like pointers
 * Leon: Yes. And we weren't trying to limit to 32 bits, but have stability across platforms
 * Johnathan: Addresses shouldn't go into command anyways, but lengths and offsets can
 * Leon: The terminology and semantics are a bit tricky here
 * Lawrence: Commands might provide lengths and offsets, which are usize
 * Leon: If we only had 32-bit values, we couldn't do that, but we'd at least be consistent
 * Johnathan: Do we have a 64-bit platform we care about?
 * Lawrence: I do!
 * Johnathan: Libtock-rs testing environment on host is also 64-bit
 * Leon: My solution here is that we should retain the ability for sub-commands to accept usize arguments. But for a subcommand, we should have the same amount of specificity. We should figure out how to encode this in the command
 * Leon: Another PR, the Brad and Amit one on auto-generating system call boilerplate https://github.com/tock/tock/pull/4112 could provide syntax that would have a type associated with them. Then the command wrapper would cast usize down into those types or leave it, depending on the types.
 * Lawrence: What that would mean right now, is that those arguments should be usize. And a wrapper would cast correctly.
 * Leon: I think we should have documentation about types of sub-driver-numbers. But I agree that our interface should be usize
 * Lawrence: The last one is upcalls. Currently it's u32 in the kernel userland. The three generic integer arguments. I propose that they become usize. It's easy for userlands to truncate these to u32 where necessary. I've got a patch for libtock-rs that does it.
 * Lawrence: I do really think these upcalls could be sizes. And so they should be in a usize
 * Leon: I think things break if they're not usize
 * Lawrence: Right. On 32-bit systems it mostly doesn't matter. In Rust it matters. In C it doesn't. In the kernel, people are using upcalls all over the place, and changing it might cause issues all over the kernel.
 * Lawrence: I'd like to upstream having the tock kernel use usize for the upcalls. And libtock-rs could update. Libtock-c can stay as-is until it wants 64-bit support
 * Leon: Are you planning on using the logic to switch types based on the system?
 * Lawrence: The prototype for the upcall would go to usize in kernel and userland
 * Leon: I think that seems fine to me. We should probably have a typed enum again, which interprets. Fundamentally conveying usize makes sense
 * Leon: For me, the more interesting part is that you're proposing a lot of good and necessary changes. But just implementing and throwing them into a PR is problematic. It makes it harder to think about them more generally and it's hard to think about in terms of 2.0 or 3.0 releases
 * Lawrence: I've been talking to Amit personally about it, but we could have more public discussions in Slack or Email
 * Johnathan: This is really a refinement of TRD104. So this should be a document first, either updating or superseding TRD104
 * Leon: That would be great. It would be a comprehensible diff of what we're planning.
 * Lawrence: One backwards compatible solution, and one not would make sense
 * Leon: And if we had the documents, that would be easier for us to review and think about in isolation. Many less lines of code change to think about
 * Johnathan: If we're talking about 3.0, there are more changes I'm interested in
 * Branden: I will warn everyone how long Tock 2.0 took. Particularly because of those "many changes"
 * Leon: Yeah, Tock 3.0 might be years of timeline
 * Lawrence: Oh, I'm hoping much faster (end of the year). So maybe avoiding 3.0 would be best
 * Leon: I do think the mailing list posts are good for discussion and figuring this out. Then an update to the TRD would be great
 * Johnathan: Adding a new return value is not a breaking change, so maybe we're okay there
 * Lawrence: It is if I update some old things to use the new interface... But we could leave old stuff alone and only update CHERI to the new method
 * Lawrence: We could even have a backwards-compatibility shim in the code to make it clear
 * Leon: We can definitely create the perfect new interface, but only apply it to 64-bit platforms. As long as we keep the 32-bit ABI unchanged
 * Johnathan: I'm saying slight stronger, I think 32-bit CHERI could be a different ABI from 32-bit non-CHERI. As CHERI is brand new, so we're not breaking anything
 * Leon: I think multiple ABIs is acceptable. The only issue is code that starts touching drivers

