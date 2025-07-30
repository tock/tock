# Tock Network WG Meeting Notes

- **Date:** July 14, 2025
- **Participants:**
    - Tyler Potyondy
    - Branden Ghena
    - Leon Schuermann
    - Vishwajith Govinda Rajan
    - Gongqi Huang
    - Alexandru Radovici
    - Ionut Cirstea
    - Irina Nita
- **Agenda:**
    1. Updates
    2. WiFi Updates
    3. IPC in Multi-Tock Scenario
    4. IPC Use Cases (did not cover)
- **References:**
    - [WiFi PR #4529](https://github.com/tock/tock/pull/4529)
    - [Muti-kernel IPC](./2025-07-28/ipc.pdf)
    - [IPC Use Cases](https://docs.google.com/document/d/1iL_DPMygbB4XAEZMSESP8Q-xA7Kq04C-B1lLUfB114E/edit?tab=t.0#heading=h.5hhktnom69b3)


## Updates
### Treadmill Networking Tests
 - Leon: Ben has been working on getting the 802.15.4 tests and OpenThread tests in Treadmill. I think that's working at this point. Not merged just yet, but soon we'll have a nightly CI for it.
 - Tyler: Awesome! Any assistance needed there?
 - Leon: All good. Mostly questions about the testing framework itself, the tests are good
### Libtock-rs
 - Alex: Ionut has been working on integrating SmolTCP into libtock-rs. He sent an issue with blocking problems. https://github.com/tock/libtock-rs/issues/578
 - Leon: Quick response without a detailed review yet, I think the raw system call version is the way to go for now.
 - Alex: My question is whether we go with libtock-rs, or if we need a separate Rust repo with less safety guarantees? A similar issue comes when porting Embassy on top of libtock-rs.
 - Leon: Good question for the Core working group call
## Ethernet Status
 - Leon: We had tock-ethernet branch that we merged into Tock master. It took the first batch of essential infrastructure and merged into master. What hasn't been merged there was the STM32 ethernet driver and Amit's USB EEM driver. Those should also be ready, but kind-of got forgotten. They can be separate PRs
 - Ionut: I can merge the STM32 ethernet stuff
 - Leon: Now that we have the fixed STM32 ethernet PR, let's merge that into tock-ethernet first. Then we can make a PR from that branch into master.


## WiFi Updates
 * https://github.com/tock/tock/pull/4529
 * Branden: Context is getting WiFi working on the RP2040. It's a really cheap little board, very exciting.
 * Alex: The WiFi chip is made by Infineon. RP2040 and RP2050 has the same chip. We also have a very similar chip on the PSOC that we're going to get working (slightly different, but doable)
 * Irina: Due to the Infineon being mostly closed-source, the work had to be inspired from public C driver from Infineon and Embassy implementation. The code on the RP2040 chip has a lot of different commands and IOCTL commands, which go over a half-duplex SPI over the PIO.
 * Irina: There's also an SDIO interface which Darius is working on. The same commands can go over that.
 * Irina: This PR is a proof-of-concept. We intend to move it to an abstract capsule for talking to the chip
 * Irina: We also made a very minimal WiFi HIL based on the Infineon chip. Eventually that could be a more general WiFi HIL, but for now it's cypress-specific
 * Irina: We also made a libtock-c example with an HTTP server that works on top of it
 * Alex: Two issues with licensing. Licensing of the code blob comes from Infineon and has its own license. Can be used on RP2040 and RP2050, but not allowed for anything else. I suggested a `firmware/` directory in the root that might hold the blob and license and note that it's not under our normal Tock licenses
 * Alex: Also, the packet struct definitions come straight from Embassy. Those are MIT licensed right now, but we're hoping to re-license as MIT + Apache.
 * Alex: Our hunch for WiFi is that the actual chips are SDRs. If they released the source code, you wouldn't be allowed to use them per FCC licensing. So you really need the certified blob to make them work. This is a hunch though
 * Leon: Awesome effort. I have a list of things to talk about: specific or generic HIL, firmware blob and licensing, capsule or chip driver
 * Leon: For the HIL, there's a tension here as we don't have any hardware-specific HILs in the tree. That's something we've talked about in the past, but we didn't need a HIL because we just moved whatever was implemented there into the capsules crate and had them create their own bespoke interface. The reason we have HILs are 1) to make things hardware-independent (a bunch of timers with one shared interface). That's moot in hardware-specific HILs. 2) HILs have an interface contract between two crates that don't depend on each other, but only on the kernel. That's the reason I think this HIL is here.
 * Leon: So what should we do? We could have the kernel hold an interface but make it not be a HIL. Or we can solve this another way by moving this to capsules and not in chips.
 * Branden: I think we move it to capsules, not chip. Then the interface can live with the capsule for now, and eventually it could become a HIL if we have several and want to standardize
 * Leon: Good question if there's a need to have the driver in chip
 * Alex: Another, slightly-different, WiFi with a different driver underneath (SDIO) is coming soon. So we want some way to make this work
 * Alex: Both devices are a little different, but should be interchangeable for applications. We should have one capsule for exporting to userspace, but needs a HIL.
 * Alex: This chip also works on a half-duplex SPI, the other on SDIO, but they don't care and just need read/write commands. So the capsule will be on top of a generic bus interface
 * Leon: Those are two different things. One is a layer above WiFi which exposes the WiFi interface. That's what I'm questioning whether it should be a kernel HIL immediately. There's also the layer underneath the WiFi chip, which to-me should be easy to support if we move this entire code into the capsules crate. You can have two drivers that share a common interface. You can also define the abstraction for the underlying interface in the capsule. The only reason that Branden and I are skeptical now, is that having something in kernel/HIL is a pretty high bar historically, that they need to be general. And I don't want those discussions holding us back.
 * Alex: So the suggestion is: WiFi capsule anyone can use. Underlying interface that adds the actual drivers.
 * Leon: Yeah. So create a new crate in capsules, CYW* for the family of CYW chips. That would contain the drivers for those chips and the interfaces for above and below them.
 * Alex: We actually need an SDIO HIL eventually. We should also add a half-duplex SPI HIL.
 * Leon: Those sound reasonable. I just don't think they're technically necessary. The capsules crate could define an internal interface for a specific chip crate.
 * Branden: We definitely want those HILs long-term. I question whether the half-duplex SPI could just use the SPI HIL. That might make things easier
 * Irina: I'll have to look into that. Not sure. There might be some blocker there?
 * Alex: Let us know on the PR whether we can do that.
 * Branden: That makes sense. So HILs for buses like SDIO and SPI. But then other interfaces go in the capsule crate itself
 * Leon: Avoiding kernel HILs will make the PR go faster. Plus we don't know how other boards/chips will interact with this yet.
 * Alex: Is there a way to add an experimental HIL to the kernel?
 * Leon: I am traditionally in favor of these. It creates churn when you want to stabilize it, but it's possible. Bring up with Core group, I think
 * Branden: Historically the answer answer is to just make the interface in capsules when it's experimental
 * Leon: Okay, so that's moving to capsules and interfaces
 * Leon: For the licensing issue. The conclusion was that generally we're very hesitant to include binaries in the repo. Bad for git, bad for licensing issues. What we did in the past in Libtock-C was to keep files on a mirror and the build system downloads the binary and keeps it locally on the users computer. That side-steps the issues. Since these files are just in the board crate, we could put these in the Makefile
 * Alex: The firmware is tied to the WiFi chip, not the hardware board. It doesn't change depending on RP2040 or PSOC.
 * Branden: Logically it should be tied to the capsule. Practically, each board could just reference it
 * Leon: Or we could make a new crate on crates.io which contains this binary blob. Then we could have the capsule rely on it. So an external crate for the firmware blob specifically
 * Alex: Embassy already has that crate, I think
 * Leon: Okay, we could just reuse that! Pin a specific revision
 * Branden: I think people in Core would accept an external crate for firmware blobs
 * Alex: here's the crate: https://crates.io/crates/cyw43-firmware
 * Leon: Seems like exactly what we want, except that it's non-commercial
 * Branden: They go through this random person's repo, instead of directly through Infineon


## IPC in Multi-Tock Scenario
 * [Muti-kernel IPC Slides by Gongqi](./2025-07-28/ipc.pdf)
 * Gongqi: Sharing my experience with IPC for multi-core Tock. Has some interesting constraints when redesiging the new IPC.
 * Gongqi: Multicore tock is a multi-kernel design. Each core has a whole Tock kernel in a separate address space. Communication is through message passing between kernel instances. Only shared memory is the messaging channel. IPC goes through this message passing channel.
 * Gongqi: IPC Design constraints. Asynchrony is important. Inter-kernel communication is async. Relies on inter-core interrupt. The current Tock`ipc_discover` is synchronous which won't work
   * Service/client identifier needs to be kernel-instance aware. Need to know if it's a core-local process or from another core.
   * No memory sharing between kernel instances, except for inter-kernel message channel.
   * The other kernel instance may fail, avoid poisoning and fate-sharing. So message-passing is needed.
 * Leon: So can we move objects between kernels? They're technically in the same address space, right?
 * Gongqi: No. The only shared address space is the IPC channel.
 * Leon: What if we had a chip where the cores shared physical addresses?
 * Gongqi: It depends. In my case, each kernel instance only accesses its own memory region. They have a single shared region for IPC. No direct access to objects.
 * Branden: And having direct access would require some fate-sharing?
 * Gongqi: Not necessarily. As long as you move the exclusive ownership to the other kernel
 * Leon: Both of those designs sound possible for us. We could imagine a more efficient design moving Rust objects. But the assumption that they don't share memory could have multi-kernels over distinct chips and a serial interface.
 * Branden: Yeah, we've been interested in multi-chip for a while now
 * Gongqi: If sharing the same address space, you could move reference of objects. But that would be limited by ability of MPU.
 * Gongqi: Our IPC interface looks like this
    * (See slides for list of interfaces)
    * Needs an ID that's a global process ID, accounting for process and kernel
    * Register callback, either as a service or client
    * Explicit send message with timeout constraint
 * Gongqi: IPC implementation uses message passing
    * Require dedicated tx and rx buffers
    * 1 copy for intra-kernel IPC (between two processes on the same kernel)
    * 2 copies for inter-kernel IPC (two processes on different kernels)
 * Gongqi: Alternatively: Message passing by reference
    * Limited by MPUs. If you don't use the MPU, you can increase efficiency
    * No RX buffers, zero-copy, no poisoning or fate-sharing
 * Branden: How do you avoid fate-sharing? If you have a reference and one kernel crashes, you're still referencing its memory.
 * Leon: If the kernel never restarts, you're safe. Or you could wrap them in a type that checks that the kernel is still alive before allowing the reference
 * Gongqi: The reason for fate-sharing in my mind, is to avoid poisoning of shared memory. If one process crashes, then the shared memory could be invalid. Might require both processes to be rebooted. For example, a shared mutex could remain locked forever
 * Branden: I'm not convinced of the no poisoning or fate-sharing point at all. Even if you have a wrapper to check if the other process crashed, the memory will still be invalid.
 * Gongqi: The internal share buffer is atomic and locked, then it's possible to avoid fate-sharing. The only way to touch the memory is through atomic operations. So it'll be valid during your operation. Could be deadlock, but no invalid memory
 * Leon: And message-passing is more interesting to us then shared memory right now
 * Gongqi: To summarize, asynchronous, message passing, with timeouts for responses.
 * Gongqi: And we need handles that have additional information than process ID. Something about generation of the process if it restarts, although we want to avoid the covert channel. The handle needs to be more customizable.
 * Branden: What about two channels at once? This is something Microsoft had mentioned that they wanted, multiple communication channels per app. Is that something you considered?
 * Gongqi: Might be better to have to processes there. Or if the user wants that, maybe a library abstraction that virtualizes the single IPC. Alternatively, if you wanted true multiple IPC per process, then it would be tricky because you're gonna modify how discover works and modify the process handle. Discover would need to be dynamic.
 * Branden: Other question, how many messages in flight at one time?
 * Gongqi: Just one for now. The internal channel uses streaming process slice plus copy from a fixed-size buffer. The interface 

