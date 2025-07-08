# Tock Core Notes 2021-07-09

Attending:
- Alexandru Radovici
- Brad Campbell
- Branden Ghena
- Hudson Ayers
- Leon Schuermann
- Johnathan Van Why
- Arjun Deopujari
- Vadim Sukhomlinov
- Anthony Quiroga

## Updates

* Hudson: Continued to work on code size reduction including grant stuff to save space. Working on other changes that are "free" rather than removing functionality


* Leon: Upcall swapping PR is merged. ProcessAliasing PR is on the way too, hopefully will be merged soon. Only remaining PR for Tock 2.0 after that is kernel reorganization
* Brad: That sounds right.


* Brad: Released a new version of tockloader on pypi. Hasn't changed in a while but there were new commits, so it seemed like time to push. It does more autodetection of boards that are not using the tock bootloader.
* Arjun: Tockloader fixes some bugs with flashing nano33ble


## Tock 2.0 Progress
* Hudson: Callback swapping PR is merged, which leaves very few things on the tracking issue. Aliasing of process-allowed memory is the big one, but it's on the brink of being ready. CRC stuff was merged (and Phil has some further stuff that has CRC fully working) so #2632 is unblocked. Just some final reviews would be good.
* Leon: I posted a comment with the remaining question. This is significant and commits to how we handle this, so we want to make sure it's right. We do want to double-check `transmute` calls with someone in the Rust community to ensure our unsafe code is actually correct. I don't have a contact there though.
* Leon: This looks good though and is close to being mergable. Just needs a thorough review since there have been some rebases.
* Brad: On getting someone from Rust, if we consider the hypothetical that some new understanding comes to light and it's not sound, my question is "what is the next best option"? Is there a fallback and how much would it require changing?
* Leon: I'm most concerned about `transmute` while slicing. We could replace the indexing operator with some other method. If our basic assumption of using cells is incorrect, although it's been confirmed by several people, we could change that without changing external interfaces, but only with a performance hit. I think the chance of that happening is slim.
* Brad: So it's low risk to move forward.
* Leon: Agreed.
* Johnathan: The chances of using cells being incorrect is zero and you can quote me
* Hudson: Okay, so we don't need to wait. We can merge as-is and look at later. It's clearly an improvement on the current unsound design.
* Hudson: Other PR is Brad's reorganized kernel create #2659. It mostly does some stuff we already agreed to in other PRs and issues to reorganize stuff. We've been wanting to do this, so we should really just do it now as part of the 2.0 change rather than making changes as a bunch of steps for people with external repos.
* Hudson: Also, tockloader install issue, which I don't know if anyone is working on. Also changelog and testing.
* Brad: Tockloader install does need to be fixed and I think we have an easy solution, which is to just switch to a 2.0 version. You'd then have to type a different name for the 1.0 version. That just needs a release so we have the released version of libtock-c to compile.
* Hudson: So this would be fixed immediately after a 2.0 release.
* Brad: yes
* Leon: Additional issue, we have a lot of unsoundness with device drivers and DMA. #2637. Multiple people agree that they are issues. It's straightforward but going to hit all chips. Probably something we should change before the release.
* Brad: I think there's pushback on blocking on more stuff. So, what does the change look like?
* Leon: We currently store buffers for DMA operations in takecells, but this isn't okay for Rust ownership rules. To fix, we can replace with a DMACell, which holds the parts of the slice and later reconstructs it when returning.
* Brad: So we could do it at any time.
* Leon: It's just dangerous since it could be broken with a compiler change.
* Hudson: I think we could include it in the release, but don't want to wait on it. We've gone long without a release anyways. We do want all the ABI changes at once, but this could be a 2.1 release.
* Branden: Does this change require updates from everyone with out-of-tree builds?
* Hudson: Only if they implemented their own chip with DMA and also did it unsoundly. So it's probably fine.

## Tock 2.0 Testing Plan
* Hudson: How do we plan to test? Will it be the normal release testing or something more comprehensive?
* Brad: How would it be different?
* Hudson: We don't always run all tests on all boards. For example, like half the STM boards don't get tested sometimes. Maybe we want to raise the bar here given that there is more space for errors.
* Brad: Seems reasonable. But the underlying challenge is still there that we need people signed up to test things.
* Alexandru: I have a bunch of STMs and can test things on them if we have a specified test suite.
* Hudson: I'll post a separate issue for 2.0 testing. We do have the release 2.0 issue and maybe I'll put it there.
* Branden: There were a few drivers we promised to test later too, like SD card.
* Hudson: I think SD Card and maybe HMAC. It didn't have a libtock-c driver at the time but maybe does now.
* Johnathan: I think CTAP is like that too.
* Brad: So what I'm hearing is that we should make a bigger effort initially to get people to sign up. But we'll probably fall back to our normal "can't wait forever to test everything". Which seems reasonable. So the TODO item is to write down the things that we deferred testing or don't normally test or that might explicitly be a problem.
* Hudson: So I should post an issue with all the boards and the testing we want and assign people to them. We could put teeth in it and say we'll remove untested boards, or not.
* Brad: We do have a notion of support levels.
* Branden: We can probably have a decision piecemeal for things that fall behind.
* Brad: My thought would be to have two classes of boards. Tested for 2.0 and not. And to make it clear in the readme.
* Leon: That makes sense. On the tock website there is a, very outdated, notion of stable versus unstable boards. So moving boards we can't test to experimental seems reasonable.
* Hudson: I think we haven't maintained a hierarchy like that, but it seems reasonable.
* Brad: So we could have some written classification. That would also help us keep track of this and eventually to phase out boards that aren't being tested anymore.
* Hudson: Last comment on this is that we don't want to block 2.0 on stuff that hasn't been submitted or won't be all that quick if it's not ABI blocking. But, maybe we should look at the list of outstanding PRs and look for things to force through before release. My opinion is that we shouldn't do that this time, but maybe others disagree.
* Branden: I agree with that strategy. 2.1 can be all the great updates to Tock, while 2.0 is the ABI changes and soundness issues.
* Brad: I didn't see anything in particular I want to include.
* Hudson: We can still merge things that are ready, of course.

## Appslice swapping in capsules
* Brad: Related question, I'm still confused about what we're promising to userspace around appslices and swapping. TRD says if you call allow you'll get the old buffer back. We're not enforcing that and leaving it as a TODO, which I think makes sense. But are we violating the TRD104 promises? Are we going to handle it soon? I think we need a record of where we stand.
* Leon: I think this is a valid point and it's confusing because of how it developed. Hudson and I planned on enforcing restrictions before Tock 2.0, which is why it's written in the TRD104 this way. However, we've since decided we don't have to enforce for soundness and it's okay for now. But it would still be a good TODO. The TRD currently has expectations we don't meet in our codebase, so we should edit the TRD to say our expectation and our implementation.
* Hudson: I thought we had said capsules were allowed to refuse to return a buffer. For example if we were passing it into a DMA request
* Leon: Well, we can't use userspace buffers for DMA anyways. Capsules can refuse operations, for example if there is a complex piece of hardware. So if the buffer only goes away if the process dies, that makes things easier to handle. Otherwise _every_ operation would have to be abortable.
* Hudson: I thought there was also the idea that a capsule can store info in an allowed buffer and could rely on it still being there. So we didn't want to let processes revoke buffers at any time.
* Leon: Yes, but it can't be trusted, since the process could change data in the buffer whenever it wants.
* Hudson: The reason I'm asking is that maybe the text in TRD104 is incorrect if we permit capsules to refuse an allow call. Which I thought was originally the plan.
* Leon: I'm confused.
* Hudson: Currently the documentation of TRD104, I believe, is that if userspace requests a buffer back, it must get it back. But that doesn't sync with my memory that allows can be refused.
* Leon: I think the wording is that _if_ it gets a buffer back, it must be the right one. And the "must" restriction there is what we don't enforce.
* Hudson: Okay. So we should submit a PR to TRD104 to modify the language a little and open an issue to enforce non swapping buffers.
* Leon: I will do this
* Brad: Does that resolve those TODOs?
* Leon: From a correctness and compliance with the TRD point of view, yes.
* Hudson: It makes the TODOs a feature request on github.
* Brad: Sounds good to me

## Checking app ABI at load time
* Alexandru: Is there a way to flag apps with the version of the kernel that they need? My students keep loading tock 1.0 apps into tock 2.0 and they fail.
* Brad: There is support in elf2tab with support for that. The code's there, so we should do that.
* Alexandru: Is the kernel checking this?
* Brad: No. It's not even merged. The information is only in the TAB, not the TBF. So it wouldn't get flashed into the board.
* Alexandru: I wanted it in the TBF so the kernel wouldn't try to load it and hopefully would display a debug message. I was thinking about using some of the flag bits to write the kernel version. Or maybe add another header. I wanted opinions.
* Brad: The reason support isn't there today is that we thought of the loader as doing the check, not the kernel.
* Alexandru: Agreed! However, in practice we keep running into this error again and again.
* Brad: I'm *SO* sympathetic to this. I didn't think of what you described. And it's hard to know what version of the kernel you have.
* Alexandru: At least we could have the ABI listed. Doesn't have to match kernel exactly, just the ABI, which the kernel should know.
* Brad: Yes. The kernel also knows the release version it is. The issue is letting tockloader know. That's why there hasn't been any progress previously.
* Alexandru: And tock bootloader knows, but openocd doesn't know. RPi pico writes a version string to a well-known memory address
* Leon: I think it's tricky because there are many stakeholders and we don't necessarily want to write things like that to shipped app version. Maybe it's best to collect opinions from all over.
* Branden: Does everyone agree to check ABI at app load time in the kernel? Seems reasonable to me. The mechanism might be tricky, but the action seems valuable.
* Hudson: Yes.
* Brad: Yes. I think it's good if we can do the check in the loading. In the load_processes function. Then it would be easy because load_processes is just a helper that boards are free to not use.
* Leon: Then out-of-tree stuff can make other choices about loading as other people can write their own version of the function
* Alexandru: So you're saying it should be optional information. And if it's not there, then skip the app.
* Hudson: However, for most 1.x compiled apps, the information isn't there.
* Alexandru: But those shouldn't load. Okay, I'll send a PR that can make this work.
* Branden: And this would be a great addition if it's not too much work. You're not the only one who's going to run into this issue.
* Brad: Hudson's point is a good one though. If you compile a 1.0 app, you have to make sure this header gets into the TBF.
* Hudson: The idea is that this check would only be in the 2.0 kernel release. So a 1.x app works fine without this check on a 1.x kernel. For a 2.x app and kernel, then nothing being there means don't load the app.
* Brad: But that's not true if the user doesn't update elf2tab.
* Hudson: That's bad. So if people don't update tockloader/elf2tab then 2.0 won't work.
* Brad: They all have to work together to ensure the flag is there
* Alexandru: I'm missing something. Tockloader extracts the TBF and writes to flash. Does it do changes?
* Brad: It does a _lot_ of stuff
* Alexandru: Reorganize apps sure. I haven't seen it change headers.
* Brad: It doesn't just leave the binary alone, it does a lot.
* Alexandru: Okay, I'll look into the three and see how they need to be updated
* Brad: I think the other issue is that load_processes doesn't really parse the TBF header
* Alexandru: At least the main header is parsed since it needs the addresses.
* Brad: Correct
* Leon: So, I think what Alexandru is saying is that if he puts it into flags, then this will work.
* Brad: So I don't see how version is a flag
* Alexandru: We would reduce the flags and take a few bits of that to be ABI number. They're already set to zero for 1.0 apps.
* Leon: It is _only_ a should. They are reserved undefined. It's just a little dangerous to assume that they're going to be zero.
* Branden: That's super hypothetical though. Everything just uses elf2tab.
* Brad: Why not add a new TLV instead?
* Alexandru: That would be fine too. We could even have it specify the full kernel release in the future. Or a min and a max. It would be more difficult to implement and need to be parsed, but more flexibility.
* Leon: So it would use three additional words.
* Hudson: Tockloader does parse TLVs though. Does it ignore unknown TLVs? Would this break tockloader?
* Brad: In theory, Tockloader should give you a warning but pass on through the TLV. But that code is hard to test and I didn't rigorously ensure that it works. So it's a little hard to promise.
* Alexandru: Technically it should ignore extra TLVs since the docs say it passes on custom TLVs.
* Brad: That's almost certainly not true on very old versions of Tockloader though. Let's assume that it does the right thing and the TLV ends up on the board.
* Brad: So we do need to talk about elf2tab. How do we ensure that the user has the newest elf2tab?
* Alexandru: Does the toolchain ensure this?
* Leon: No. I used an ancient version for years without noticing.
* Hudson: We could add logic to 2.0 libtock-c which requires updating elf2tab. Could be an issue for some people though. We could add stuff to auto-update to the Makefiles
* Leon: We always might end up with some users who have out-of-date stuff
* Alexandru: We could just have a debug message and load the app anyways.
* Leon: Yeah, but we still have a ton of output that will confuse new users.
* Alexandru: We could have another flag to decide whether to load the app or not...
* Brad: I don't think we should have a config flag for this transition case. In 6 months no one will know 1.0 existed.
* Alexandru: Agreed
* Brad: In general, I don't think there's any issue with adding a TLV that says here's the kernel support an app needs. That seems very reasonable.
* Leon: Okay, so we'll wait for Alexandru's PR then.
* Hudson: I do think the general approach sounds good.
* Leon: Additional question, on a previous call we said it would be helpful to have metadata in the kernel binary to allow tockloader to recognize the version on the board. Would we still do this in parallel?
* Brad: We could do this in parallel
* Alexandru: Can you explain more?
* Leon: We agreed previously that various metadata should be in the kernel to allow other tools to determine things about the kernel, like version or other stuff. But it's not a 2.0 blocker and we'll figure it out long-term
* Alexandru: Okay, I'll look at the TLV path and work on a PR then


