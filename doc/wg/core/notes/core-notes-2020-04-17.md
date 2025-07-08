# Tock Core Notes 4/17/2020

Attending:
 * Branden Ghena
 * Amit Levy
 * Brad Campbell
 * Vadim Sukhomlinov
 * Johnathan Van Why
 * Pat Pannuto
 * Samuel Jero
 * Alistair
 * Garret Kelly
 * Philip Levis
 * Andrey Pronin
 * Hudson Ayers
 * Leon Schuermann

## Updates
 * Hudson: I have a PR for a scheduler trait, which people should look at.

## Tock 1.5
 * Amit: Freeze for 1.5 was today.
 
 ### Timer
 * Amit: First status was the timer bugs.
 * Phil: I only got to testing core logic for the timer, overwhelmed with administrative stuff temporarily.
 
 ### HiFive
 * Amit: Next is HiFive support. Kernel out of the box is running just fine. I used a binary from a while ago and showed that applications work, although I couldn't get C userland applications to properly do anything. No crashes, but no output. A lot of the work was just trying to compile it at all, haven't gotten into debugging yet.
 * Alistair: I think that's enough, because libtock-c is pretty much unusable for RISC-V right now.
 * Amit: The emulated QEMU board I'm using is the 1 rev B, right?
 * Alistair: Kind of. It's just a made-up board that kind of matches the rev B one. But not specifically.
 * Samuel: It has usermode write with rev B doesn't. It claims to have a lot more RAM.
 * Alistair: I think the opposite actually. It doesn't have any memory. You can't do debugging.
 * Amit: Are people comfortable with the progress towards this milestone?
 * Brad: Yes
 * Brad: I thought you were making a 1B board?
 * Amit: The board we've already got is 1B ready?
 * Brad: I don't think so. They are significantly different one has BLE.
 * Amit: This is all just in QEMU for now and we don't support BLE anyways.
 * Alistair: Without changes, the 1A board supports the 1B. So as long as we don't add changes, it should support both for 1.5.
 * Brad: Why make it difficult? We could just make two boards, or else deprecate the 1A.
 * Samuel: We don't have any features only on the 1B right now, except the PMP.
 * Alistair: So Tock 1.5 would support the 1A, then the future commits would only support the 1B.
 * Amit: I think in the long run, like the nRF51, there's no reason to support the 1A. No PMP or usermode stuff.
 * Brad: Okay. So 1.5 supports 1A, afterwards we only support 1B using PMP. So just add a note about what's supported right now and what the future plans are.
 * Amit: I am writing this documentation.
 
### Syscall Filter PR
 * Amit: Three PRs that are close to include in the window per Brad.
 * Brad: First is syscall filter which is merged. Adds a hook for someone to write a syscall filter, but doesn't actually change anything.

### GPIO Syscall Stabilization
 * Brad: Next is GPIO syscall. The stabilization from quite a while ago. The consensus is that GPIO is low-level hardware that isn't portable or virtualized, we should treat it like low-level I2C or SPI and not stabilize it. PR is ready, just needs more approvals.
 * Amit: I am supportive of this. I just want to bring this up for people on the call. Our working hypothesis is that we could not come up with reasonable scenarios where what you want to do in a deployed system is expose raw GPIO to applications, rather than exposing as LED, Button, CS for SPI, etc.
 * Vadim: I think there should always be an abstraction instead of the raw GPIO. By not exposing GPIO, you'd be more likely to expose virtual capsule interfaces.
 * Amit: Still makes sense to have the GPIO driver in the repo as a dev tool for prototyping things in userspace. But for that usecase, stabilizing isn't important. On a natural system with actual applications, you'd remove that driver or filter it out.
 * Samuel: I also strongly agree. In principle everything through GPIO is really some other device. On the other hand, what I'm actually doing with Tock is exploring how architectures and OS can be redesigned, so it's convenient to have low-level toggling of GPIO for debugging.
 * Amit: Okay, so agreement that this is good. No planned sweeping changes, but it's never going to be part of the promised interface.
 * Garret: Sounds good
 * Alistair: Yes
 
 ### Virtual UART Bugfix
 * Brad: Last is bug from virtual UART capsule. The UART aborts stuff. Some logic was just wrong. Tested on Hail and the PR makes things better. Things still work. Makes sense to pull it in as a bugfix. #1757
 * Amit: Background is that someone porting Tock to stm32 chips noticed this. They were writing the uart chips driver, and noticed all the aborts. This fix isn't a functionality bug for correctly written boards.
 * Phil: My concern is the case this was trying to cover and whether it's still covered.
 * Brad: I think the comment notes that the case should be covered in the normal send/receive path. But definitely check.
 * Phil: Yeah, there was a non-obvious failure path this affects. I'm checking.
 * Amit: Pushback might be that we want this to be the release that say, openSK, pins to. And this bug doesn't exhibit problematic behavior on mature platforms. And immature platforms don't care as much about pinning to a release. So maybe it's not worth the risk to include in 1.5.
 * Brad: I'd be more receptive if I knew why it worked on the SAM4L. It seems like it shouldn't... But maybe everything is fine if I trace it enough. This seems like a straightforward bug to me.
 * Phil: The logic that's being cut out is for the case when you're in the middle of a read and another read comes in. I can't wait for the previous read to complete for the new one to start. So, it cancels the current read and then restarts so that you can get the new read. This matters if you do a read of 100 bytes, read in 17, then someone else does a read of 10 bytes. You want one read to get 100 bytes and the other to get 10 bytes from 17-26. So, we should test this fix with the overlapping UART test cases, collecting a subset of what another read does.
 * Hudson: I think they're in a kernel test on imix.
 * Phil: Should work on any board, not imix specific.
 * Brad: Problem is that this causes issues in the case where that's not happening.
 * Phil: Let me read through this PR. I agree that there's a bug. This PR may or may not correctly fix it.
 * Hudson: I just ran the virtual UART test, and it does seem to work for PR branch.

### Tock 1.5
 * Amit: Deferring on timer stuff for now, merge the RISC-V changes (documentation only), merge GPIO system call non-stabilization, and merge syscall filter.
 * Amit: I think we're otherwise done with what's going into 1.5 and should start testing.
 * Brad: I think I'm on the side of not risking a regression and not to merge virtual UART. Unless Phil can take a look very soon and is all for it.
 * Amit: Agreed. So testing should start early next week! This means that we don't merge additional PRs until release is ready (bugfix only).
 * Brad: I'll create a release candidate. In that issue is where testing documentation goes. We have a final signoff where we release 1.5.
 * Amit: Cool. So just need to merge or not-merge everything remaining in the next few days. Should be straightforward.

## Tock Communication
 * Leon: Is there a central location to share information and see what's going on? I was working on Ethernet stuff, and wanted to make sure I wasn't duplicating work with others.
 * Amit: This has come up with multiple of us working on simulators too.
 * Amit: I propose we pick whatever platform a try it for a bit.
 * Leon: I think a Trello might work. Something unbound from github repo permissions. Github integrated would be fine, but someone with permission would have to update it.
 * Amit: I do want to not use external tools. But blocking on people with permissions is annoying. I'd rather have us out of the loop.
 * Leon: Okay, so let's just try some tools. I can create an issue.
 * Brad: Context is that this is a recurring problem, but we haven't yet found a good solution. So I guess we should try something new.
 * Alistair: Isn't trello like github projects?
 * Leon: Yes, but we want people who don't have github permissions to be able to post.
 * Alistair: Problem with secondary thing is that people won't sign up and it will be out of sync anyways.
 * Amit: So starting an issue and trying some things seems like a fine start.
 * Leon: Worst case we can hit the delete button. We will need to advertise it.
 * Amit: We should have it linked to the README and blog, etc.
