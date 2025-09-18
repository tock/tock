# Tock Meeting Notes 2025-09-17

## Attendees
 - Branden Ghena
 - Alexandru Radovicii
 - Johnathan Van Why
 - Pat Pannuto
 - Leon Schuermann
 - Tyler Potyondy
 - Amit Levy

## Agenda
 - Updates
 - Cryptography Working Group
 - Tock at Eurorust
 - Plan for using SingleThreadValue

## Updates
### Lockfiles and External Dependencies
 - Leon: There's a discussion about adding lockfiles to repo. Still need fresh comments to break deadlock. https://github.com/tock/tock/pull/4589
 - Pat: One resolution was a PR to add lockfiles, which can go in parallel with external dependency policy
 - Leon: I don't believe we finished the discussion on lockfiles yet. I can move it to a new PR
 - Pat: I think we support lockfiles. We're disassisfied with them, but they seem better than nothing
 - Leon: Okay, I'll open a PR and see what people think
### Network Working Group Update
 - Branden: Network WG update: We're back to every-other-week meetings this fall. We discussed two things.
 - Branden: First was WiFi PR. That's waiting on authors right now, and doesn't need help from us. They plan to split out the WiFi capsule from the underlying transport (SPI/SDIO) and update the PR in the coming weeks.
 - Branden: Second was IPC. We discussed feedback from Tockworld and possible solutions. We'll update the group with a new draft proposal of IPC mechanisms in the coming weeks.
### Keyboard and VGA PRs
 - Alex: Fully functional keyboard and VGA display running process console
 - Leon: I briefly looked at the VGA PR. You're doing text mode right? Not rendering characters in a framebuffer?
 - Alex: Yes, text mode. The same VGA driver will support video modes as well, and figuring out how to make this generic. There will be types for text-mode and for graphics
 - Leon: Interesting. Some overlap with VirtIO since that VirtIO-gpu driver can do graphics
 - Alex: Great. Good to have both.
 - Alex: Comments on PS/2 PR would be helpful. A mouse PR will follow in the next bit

## Cryptography Working Group
 - https://github.com/tock/tock/pull/4604
 - Amit: We discussed this previously and now have a PR to start it. Initial team Amit, Hussain (Microsoft), Kat (zeroRISC), and Tyler (UCSD).
 - Amit: PR has proposed charter with code purview. Happy to take feedback and make adjustments
 - Branden: There should be capsules in the purview of the Cryptography working group, right?
 - Amit: I phrased it as HILs and System Call drivers.
 - Branden: I think it's worth calling out specific capsules, just like specific HILs right now.
 - Amit: Sounds good.
 - Branden: We should also update Github labeler based on those files
 - Amit: Okay, I'll update the PR. 
 - Branden: We could also address the chair. Doesn't have to be Amit, and he's got a lot on his plate
 - Tyler: I could lead it (with some help from Amit)
 - Tyler: Another suggestion for the charter, we could add a discussion of rust-crypto libraries or other external dependencies. There could be some merit in thinking about that
 - Amit: I basically agree with that, but I'm considering how to phrase it for the charter. For example, we should own the ghash capsule with the external dependency
 - Tyler: I think there's a second capsule with a rust-crypto external dependency. And there will be more in the future
 - Branden: Saying that the WG is responsible for owning/maintaining external dependencies with crypto
 - Branden: We could use approval on github as a vote to accept the WG
 - Amit: Just wait to approve until I update though
 - Amit: Looks like this just needs consensus of core team

## Tock at Eurorust
 * Alex: OxideOS and Wylodrin have a big booth at Eurorust and we could showcase Tock over there. That's in two weeks in Paris.
 * Alex: Question is if we should demo and what we should demo
 * Alex: Secondly, they have an "impl" unconference day, if you're a maintainer you can propose your project and people can come talk about it and you can help them submit PRs. I would be willing to submit Tock to this too
 * Branden: On the second part, not clear that people could make a meaningful PR on the spot?
 * Alex: We could just show them what Tock is and how it works
 * Pat: Could be a QEMU demo of Tock. That doesn't even require hardware
 * Alex: And likely a hardware demo of Tock at our booth
 * Branden: I'm supportive of this, for sure. We might have demo/tutorial stuff from Mobisys this past spring right?
 * Pat: I don't know that it's turnkey, I think the documentation is in the Tock Book
 * Alex: What is the goal of the tutorial?
 * Tyler: Root-of-trust theoretical stuff mostly, with a small Tock example. Bigger parts were dynamic app loading and Thread network, didn't have as much to do with root-of-trust. There wasn't a turnkey root-of-trust on the board. But it's somewhat there
 * Alex: There's also the HOTP application, right?
 * Pat: Yes. From Tockworld 6. That's definitely possible to demo
 * Alex: We'll also have an ARM64 demo. We can also show off QEMU. If anyone has other ideas please send them my way
 * Branden: You could also spin up boards with a Thread network
 * Alex: I'm a little worried whether that will just work
 * Pat: It should be pretty turnkey! I do agree though that if it doesn't "just work" it could be hard to debug
 * Tyler: Yes. It would just be a couple of hours of work to set up, and I'd be willing to test it if Alex is interested in it
 * Alex: What would I use as a border router?
 * Tyler: You can do mesh of nRF52s without a border router. The example app right now sends temperature sensor data back and forth. We built a custom screen that plugs into the dev board for our tutorial to display it, but you could print over console
 * Alex: Interesting. I'll think about it. I have a couple of DK boards
 * Alex: And generally, I'm super happy to hear about interesting demo ideas. In addition to QEMU and ARM64.

## Using SingleThreadValue
 * Branden: We want to remove uses of `static mut` in the kernel, and we now have the SingleThreadValue type, so we need to start using it. Brad has a design example in #4519 of how to handle this for panics. However, it's some manual work to adapt each board to it. So we need a strategy for handling that work.
 * Branden: Reminder that this is a release blocker for Tock 2.3
 * Pat: Is removing all static muts a release blocker, or just panics?
 * Leon: No. We can remove pretty much all of them though, apart from some tests. The vast majority are panics
 * Branden: Brad's suggestions of how to do this. 1) keep static mut 2) assign people to boards 3) someone does the work for all boards 4) we try to offload to AI to do the porting given we have an example
 * Leon: It is relatively easy to do, BUT how static printing works for different boards is rather board-specific. I suspect AI is unlikely to succeed for that reason.
 * Alex: I have some students who could work on it if there's a good example.
 * Pat: Example is here: https://github.com/tock/tock/pull/4519
 * Pat: But I think that still needs some work. So the action item would be making sure that is complete and works, then we could share it with everyone
 * Leon: I thought #4519 was premature. We're not totally sure of when we could access these values in practice. My suggestion is to port one popular ARM board and one popular RISC-V board, then run those boards for a bit and panic them a few times to make sure it all works. I think we should do that before porting all the rest, to make sure they won't have to change
 * Pat: Okay, so proof-of-concept with testing for two boards as a first PR (or maybe two PRs). Which is fixing up this draft and doing another. Then a later PR.
 * Pat: I can try to do ARM and Leon can try to do RISC-V
 * Leon: Should be do-able.
 * Branden: Is there still an action item to fix up the draft in #4519
 * Leon: No. We can just do that as we do our update. This draft PR will close once they are ready

