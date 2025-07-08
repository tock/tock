# Tock Network WG Meeting Notes

- **Date:** November 02, 2023
- **Participants:**
    - Tyler Potyondy
    - Branden Ghena
    - Leon Schuermann
    - Felix Mada
- **Agenda**
    1. Updates
    2. Buffer Management
    3. OpenThread Support
- **References:**
    - [OpenThread Platforms](https://openthread.io/platforms)
    - [OpenThread Porting](https://openthread.io/guides/porting)


## Updates
- Leon: Merged STM32 Ethernet PR into Tock-Ethernet! Really high quality code. Next is merging the branch itself
- Branden: The more you can limit the changes to just "ethernet" stuff, the easier that PR will be to merge
- Leon: Also needed to increase the stack size for boards. Hoping to fix that. The only change outside of ethernet stuff will be adding this stack to some demo boards
- Leon: Boards are: STM board now, LiteX simulation, LiteX ARTY board, QEMU with VirtIO, nRF series with ethernet over USB. So we can polish our initial HIL, ensure that things work, then make one coherent PR


## Buffer Management
- Leon: Update is no updates yet! Spent a few hours after the call last time, and almost all of the code compiles, but there's still some that doesn't which may require some architecture redesigns. Might be losing some type information when passing things down that we still need when passing buffers back up.
- Leon: Even from this current work-in-progress design, I think we can extract a simpler design that will support what we need for some initial tests.


## OpenThread Support
- Tyler: https://openthread.io/guides/porting and https://openthread.io/platforms
- Tyler: Amit's feedback has been pretty strongly anti-capsule implementation. I think it's made sense to start there. Especially since we've exposed some bugs in 15.4 implementation.
- Tyler: Longer term though, it needs to be robust and bullet-proof for adoption. So for Thread support, we have a really good opportunity to keep the library as a process in userland.
- Tyler: There's a guide for OpenThread already to porting and what hardware you need. Requirements are: 15.4 which we have, alarms are good, True RNG is fine on Nordic although other boards are unclear and I don't know if we have an RNG driver, non-volatile storage I'm unsure
- Leon: The insight I'll give is that there are a few non-volatile storage options, some of which work for users and some don't as well. There is a Key-Value storage, there's also a region of flash that you can just write to. Both have some issues though, and aren't bullet-proof implementation right now. For example, the region of flash doesn't support atomic operations at all, so it could break across power cycles.
- Tyler: Two thoughts on that. The strategy I had related to Tock: so far when working on Thread I found several issues across other capsules. The use case showed that the bugs exist, then we could fix them. So I don't mind pushing on stuff.
- Tyler: My guess for non-volatile storage is that it's only for preserving across power cycles, so that's probably fine.
- Branden: For RNG support, I'm not sure, but the nRF has a peripheral for it that works great.
- Branden: But it's limited if there's no peripheral available. There is a paper by Phil and company about building your own RNG too...
- Leon: Support is actually pretty good. There is a HIL and several non-nRF boards have RNGs
- Tyler: Impression was that there is existing work on it.
- Tyler: Support for OpenThread should be relatively easy, been done for RIOT and FreeRTOS.
- Branden: Turning the OpenThread C library into an application in userspace? (Yes)
- Tyler: Priority scheduler -- maybe more of a concern with this?
- Branden: We have one: https://github.com/tock/tock/tree/master/kernel/src/scheduler
- Leon: And I did an analysis of it, looking at a precision time protocol in userland: https://leon.schuermann.io/publications/2021_Schuermann_ptp-time-sync-embedded-systems.pdf
- Branden: I'm not sure if anyone's actually using them though
- Leon: There are a few upstream boards playing with them. So I suspect they at least basically work, although there are some well-known long-standing issues. For example, the priority scheduler has quadratic time complexity when scheduling, so it's very expensive. Fixing it requires redesigning the list infrastructure though, so it's a big lift. Overall, you have to be prepared to open a can of worms. But coming at it with a motivating use case will help things to move forward
- Tyler: This is definitely something to circle back to. Going to be a long-term effort, with little progress in the short-term.
- Tyler: I am really interested in the idea of having OpenThread as an application rather than reimplementing it ourselves. It seems really useful long-term. There are possible parallels in BLE for example, if such libraries exist
- Branden: More generally, I think just having a userland communication library pushes on a lot of other Tock infrastructure and will show us what needs improvements. For example: inter-process communication
- Leon: One thing Brad got excited about last time was C implementations for network libraries isolated in the kernel (https://dl.acm.org/doi/10.1145/3625275.3625397)
- Leon: So far, I think we did try some BLE libraries in userland, but the latency was just too high for them.
- Tyler: One more thing, I've been working on a GRFP application, and it was useful to get feedback about my research proposal. What Pat and I came to was developing over the course of a PhD a software version of secure IoT, similar to what AzureSphere does with hardware. So the first step for me is thinking about networking. So that's continuing to push me in this direction and I'm very excited about it
- Branden: Back to OpenThread porting. I think the next really hard challenge is just thinking about what the real latency requirements are. I'm hopeful that it's not too tight, but I'm not sure
- Tyler: Agreed and hopeful. The really tight part is auto-acknowledgements which are happening in the capsule still.
- Leon: And if it turns out it can't work in a regular C app, we could keep pushing on my isolated C kernel stuff, which will essentially run it as a capsule. The biggest issue right now is that it's on RISC-V right now, and we'd need to port to ARM for Thread stuff, as I don't think there are any RISC-V boards that support Thread right now

