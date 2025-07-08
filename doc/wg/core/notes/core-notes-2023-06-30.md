# Tock Core Notes 2023-06-30

Attendees:
- Branden Ghena
- Alexandru Radovici
- Phil Levis
- Brad Campbell
- Leon Schuermann
- Johnathan Van Why
- Hudson Ayers
- Alyssa Haroldson
- Amit Levy


## Updates
 * Leon: One of Alexandru's students has been spending a lot of time on an Ethernet driver for an STM chip we support. It's up as a PR. Pretty exciting to see a full-featured chip with working ethernet support. We can run a web server on that code in userspace. The PR is up on a branch, then after it gets merged we'll do a bigger PR from that branch to Tock Master.
 * Brad: Leon has also been making Elf2Tab changes
 * Leon: Not all of them correct! I spent the last week reading through ELF files, which are horrible.
 * Brad: But, there's a new flag for setting the protected region size, which is great.
 * Brad: Most of my work this week has been getting the Key-Value stack squared away.
 * Hudson: I also saw some PRs to libtock-rs from Leon
 * Leon: Those are mostly related to the Elf2Tab Stuff
 * Johnathan: FYI, I'll be out next week, in the week after, and then out for three weeks after that.


## Key-Value Stack
 * https://github.com/tock/tock/pull/3508
 * Brad: Story is that I thought it would be great to use key-value stuff. But the current implementation has some issues. Not virtualized in the kernel (I added that). Assumptions about size of buffers. When you retrieved a value you didn't get the length of the value, just a buffer and no indication of how much of it is valid. Various lower-level details had issues too.
 * Brad: Because it's so broken right now, it's really not usable. I'd like to just get it fixed. I'm not sure what the appetite is for fixing it. I'm looking for more comments from Alistair about whether he agrees with the direction or if I'm misunderstanding something major. I'm worried that the PRs will sit forever.
 * Hudson: Do the outstanding PRs fix everything? Or is there still stuff broken after them?
 * Brad: If all of my PRs were merged, we'd be in a good spot.
 * Hudson: So you just need PR review and testing.
 * Amit: Alistair has owned this and is presumably using it somewhere. It's unclear from his comments whether your changes are misguided.
 * Brad: There are two levels to this. The key-value stack in the Tock kernel, and the TicKV library. TicKV seems to mostly work. I added one change to the data size, which seems unambiguously good. There's a wrapper with an async API as TicKV is synchronous. TicKV seems to return errors in cases when there aren't, seems to be a disagreement about conventions. So we could consider changing that. MOST of these changes though are in the key-value store which is in the kernel.
 * Amit: I think if we were all to review, we'd probably say it's fine since we don't have any strong attachment to the implementation. But Alistair might disagree. Has he given any high-level issues with stuff? Essentially: should we just get eyes on stuff and merge it, or do we need a bigger discussion?
 * Brad: I don't know. I tried to bring it up, but haven't gotten a clear answer. I does definitely need reviewing.
 * Amit: Are there stake-holders other than Alistair?
 * Brad: There has been at least one issue posted to Tock about TicKV. But I can't say if other people are really using it.
 * Alexandru: We tried to use TicKV, but it was unusable for our purpose.
 * Hudson: I've been going through Brad's PRs. Looking at Alistair's comments, I don't see any high-level complaints. So I don't get the feeling that he'll be upset if this goes through or that the changes are a worse design.
 * Amit: Okay, I'll review and approve those. I'll also reach out to Alistair about it individually.
 * Hudson: I don't think there's any agreement to stability for the key-value interface. So updates seem great.
 * Amit: It's plausible that there are multiple use cases with different use cases. We could plausibly even end up with multiple key-value interface designs for different things. Rather than trying to maintain "one true interface", we can have multiple solutions.
 * Brad: And because we have HILs, we're pretty good on that front to start with
 * Hudson: Is my understanding correct that part of the urgency here is hoping to use the key-value stack for the tutorial?
 * Brad: Yes
 * Hudson: That seems cool and is a good motivator
 * Brad: I did open a tracking issue: https://github.com/tock/tock/issues/3524
 * Brad: It is hard to track all of the changes. I made separate PRs, some of which break if pulled on their own.
 * Hudson: Maybe we can start with the smaller stuff and then merge the others as one big PR?
 * Hudson: #3489 and #3490 build. #3491 is the first "broken" one.
 * Amit: #3508 is the big "all-inclusive" one. And I think in practice it's not _so_ big. We could do this as just one PR.
 * Hudson: So I think we look at #3489 and #3490 first, merge those if warranted, then move to #3508
 * Brad: That sounds okay. I just worried about one PR that changes HILs and Kernel and capsule, etc. But it sounds like we've got that all under control


## Renaming LeasableBuffer
 * https://github.com/tock/tock/issues/3504
 * Brad: We should be using the idea of a LeasableBuffer more. Really solves passing slices and lengths around
 * Brad: I think the name, while descriptive, hinders use because it sounds scarier than it is. I think a rename would help us get it off the ground. I thought of "splice", but I'm not married to it
 * Hudson: Why "splice"?
 * Brad: Riffing off of slice
 * Phil: I would argue against "splice" because it means something for Vector already
 * Phil: I would split this into two parts. 1) should we use it everywhere, and I think it sounds like maybe yes. 2) does changing the name have an impact, and I'd like to test that.
 * Amit: I am in favor of using LeasableBuffer everywhere. I intended to
 * Leon: We do still have this pending issue of unsoundness when dealing with DMA peripherals. We can't retain a Rust slice in memory when doing that. So if we did use LeasableBuffer, and made some small change underneath, we could solve soundness there too. As of now, it doesn't do this. But I think if we changed everything to use it, we could make the modification once inside it and fix stuff everywhere.
 * Brad: I agree with Amit that it seems so intuitively an improvement. If we had done it two or three years earlier it would have been better. But my other rationale right now is that it's just hard to use due to naming. Typing "leasable" is rough to start with and LeasableMutableBuffer is a big mouthful.
 * Phil: I have used LeasableBuffer several times. When I didn't use it was when I wanted to be very explicit about memory and where I am in it. And if LeasableBuffer changed later, that could affect things. I didn't want the automagic
 * Amit: Yeah. Hopefully it shouldn't appear as magical, since it's not doing much. It's just allowing you to pass a sub-slice without requiring the code to figure it out and length checks and stuff. So you can treat it just a like a regular slice.
 * Phil: Brad's point about the length field is well-said. It's this weird thing. C allocators keep track of how long stuff is: you just have a pointer. But in those cases you're not usually moving the head pointer, just the end. I could buy that whenever a system call passes something in, we should use a LeasableBuffer for continuity. And you likely wouldn't often bother to pull stuff out of it
 * Amit: For the network stack, we did this thing where we had a buffer to fill up a network packet, and libraries fill in different header parts in the buffer. So there are multiple slices to be passed around. And passing offsets/lengths could be error-prone.
 * Brad: If you _really_ want to manage your memory, you could just use a pointer.
 * Phil: Definitely don't want that
 * Brad: I agree that LeasableBuffer sounds like it adds uncertainty. Where slice is built in and sounds trustable
 * Amit: I think the name implies that there is weird behavior under the hood. Where as &[] sounds like something which will never change and can be trusted. It also affect us implementing it. If it were called "slice", then we'd never consider taking on features.
 * Phil: It wasn't the name for me. I had motivation of just wanted to know exactly what my code was gonna do.
 * Amit: It is okay not to use it everywhere. And it is easy to interchange between the two.
 * Phil: Let's try changing an API: UART or SPI or something, and see it
 * Brad: An example exists. I just switched the key-value stack to use LeasableBuffer
 * Phil: I'll look at that and see how it feels. I just think the name isn't the most important thing.
 * Alyssa: Docs and examples would increase usability too
 * Alyssa: I do disagree with the name "splice". It's actively confusing as it's about joining things. What about subslice or something like that?
 * Brad: Also, if you look at the PR, there are doc additions too. https://github.com/tock/tock/pull/3519


## Digest Trait
 * https://github.com/tock/tock/pull/3479
 * Brad: There's an "update the digest HIL with set_client()" PR. That relates to adding the HMAC. We have to decide on the HIL first.
 * Brad: Phil, can you look at this?
 * Phil: Looks fine. When we can do proper upcasting, it would be okay. But I approve this
 * Brad: This just needs to be merged


## Protected Region Size
 * https://github.com/tock/tock/pull/3515
 * Brad: Everyone who looks at this part of Tock wonders the same thing. TBF headers have a portion at the beginning of application space that they can't change. So there's a protected_size field that was meant to mean the number of bytes at the start that they couldn't touch. But that got implemented in Elf2Tab as meaning the space between the end of the header and the start of where the app can access. The whole region is already protected. So I think we can just change meaning to meet Elf2Tab. That's what Leon's PR above does.
 * Branden: There's something I'm not understanding. These are in flash? Can't they not be modified anyways?
 * Leon: This is an important contract about what parts are mutable. So if we put apps into RAM, this would matter. Or some microcontrollers which allow ROM to be modified in some way.
 * Amit: It's plausible that an application could write to things if the region isn't write-protected
 * Leon: I had issues with applications and noticed that the spec and practice. So given that this field exists, we need to bring it in sync with the documentation
 * Branden: Okay, this seems good. Is there any disagreement? (sounds like no). Then we should all take a look at the PR and approve and merge it
 * Leon: One more thing about Elf2Tab changes
 * https://github.com/tock/elf2tab/pull/70
 * Leon: Part of the changes I pushed was to have the protected region size specified in the ELF file. This feature is often about padding the start of apps for non-position-independent implementations. So we can now embed it in the ELF binary to allow the linker to put the header plus some padding to align the start of an app to a specified address. So in libtock-c we have application start addresses pushed back by a certain number of bytes to try to reserve room for the header, but we rely on Elf2Tab aligning things to 256-byte boundaries. So all of these changes are to make it more sane to have start addresses specified and to have the linker understand them
 * Leon: My overall goal is to make non-position-independent code (which we're stuck with right now), more intuitive to use and understand
 * Brad: There's a new API between a userspace linker and it's compilation, and Elf2Tab, via a flag


## TockWorld planning
 * Branden: I will send an email about TockWorld planning to some subset so we can have an extra meeting about it

