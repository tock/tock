# Tock Network WG Meeting Notes

- **Date:** February 24, 2025
- **Participants:**
    - Branden Ghena
    - Leon Schuermann
    - Alex Radovici
    - Tyler Potyondy
- **Agenda:**
    1. Updates
    2. Ethernet
- **References:**
    - [LwIP Ethernet Libtock-C App](https://github.com/tock/libtock-c/pull/494)


## Updates
- Branden: RP2040 PIO SPI efforts are ongoing. My students got interrupts working, which is very exciting! They plan to clean that up and make a PR to Tock.
- Branden: Next up is to work on the WiFi. One weird thing I hadn't realized earlier is that the RP2040 uses one wire to communicate with the Infineon WiFi chip, sort of a half-duplex SPI thing. It's pretty weird. Going to have to look into that more...


## Ethernet
- https://github.com/tock/libtock-c/pull/494
- Leon: (Live demo of LWIP app stack over QEMU with Ethernet which works! Serves a basic landing page over HTTP.)
- Branden: Remind me what hardware we have Ethernet support for?
- Leon: QEMU, LiteX, some STM32F4 with Ethernet
- Leon: We also have USB-CDC ECM which any board with USB _could_ use. That should work with any desktop OS
- Leon: Next steps are get the existing PRs merged into the staging branch, then PRs for more 
- Branden: Do other things need to go into staging branch first?
- Leon: I think we're good enough now to actually advertise that we have Ethernet support in Tock
- Branden: I agree. So we'd be good after this EthernetTap PR
- Leon: We need to update the LiteX driver up-to-date with the HIL as well. But that could happen in master later too
- Branden: Want to make sure that we document which things work and which don't. Some table somewhere of which chips support things
- Leon: Might be faster to just update LiteX
- Branden: Then remaining things on Tock-Ethernet branch are just more hardware support
- Leon: Then later would be updating Ionut's SmolTCP efforts for the kernel
- Leon: But first would be libtock-c userspace stuff
- Branden: Two PRs - streaming process slice API, and LwIP stack + application
- Leon: Those are both pretty close. The streaming process slice API should be first
- Branden: For the streaming process slice, we moved over to two buffers. Is that okay?
- Leon: Yes. Having the API accept two buffers seems fine. Users could still have one contiguous buffer if desired, which feels like a good interface
- Branden: Sounds good. I was surprised at how clean the interface was, by the way. Very nice
- Leon: Questions, I can't figure out how to stop Uncrustify from attempting to format the LWIP submodule
- Tyler: I ran into this issue in OpenThread. I have a dirty fix for turning off warnings for a library in a makefile, which OpenThread uses
- Branden: The "better" answer here would be figuring out how to remove the location from Uncrustify's invocation
- Leon: Yeah, I really don't know where to start. There are terrible Makefiles here that I don't want to fight
- Leon: Brad also commented that I should add a Makefile.setup to initialize and update the submodule. Our existing Makefile system handles this "automatically" if it finds it
- Leon: The problem, is that we include a Makefile within LwIP to grab some variables with filenames. But that occurs at initialization of the Makefiles, rather than when running rules File here: https://github.com/lwip-tcpip/lwip/blob/master/src/Filelists.mk
- Leon: The solution I see here is a two-phase system which realizes there's an issue with setup, then re-invokes the same make command afterwards. Which is messy
- Branden: So, the issue here is that git submodules are pretty terrible. A user just finds an empty folder with no code in it, and has to intuit that means they forgot to run a magical submodule update command. So Tock has taken the route of automagically updating these submodules for you. But that means you can't rely on any files in that directory existing when Make first runs
- Branden: I think the solution here is to just vendor the file list ourselves
- Tyler: OpenThread is a submodule, pinned to some release. The submodule is updated when you run `make` which initializes it and pulls stuff in.
- Leon: Right. But I think you hardcode all the C source files
- Tyler: Brad figured out hardcoding all the C source files here
- Branden: So we could vendor the Filelists.mk file from LwIP?
- Leon: But a user might want to update the submodule, and they'll have an issue with undefined symbols
- Branden: There's a cycle here where we can either make users in charge of submodules or we can hardcode the files.
- Tyler: For OpenThread, we used to compile the library separately
- Leon: So, we could have a different make step that just compiles the library, ignoring Tock, then pulls stuff in when building Tock stuff
- Branden: I think vendoring the file is still our easiest option here. It will break things when people want to update the submodule, but that's maybe a cost we need to have
- Tyler: For people adding, say MQTT examples, ideally your stack should be extensible but it's possible that adding another sub-example could break things in your port. I.e., adding new features will always be work at multiple levels, and rarely "just work".
- Branden: At least putting something in a README about how FileLists was gotten and how to update versions would be good
- Branden: So in summary, the next step is to merge the EthernetTap PR, then to make a PR from staging to master
- Leon: Yes. That PR from staging will have the HIL, a hardware implementation of the HIL, and a user of the HIL

