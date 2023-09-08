# Tock Network WG Meeting Notes


2023-09-07
===

- **Date:** September 7th, 2023
- **Participants:**
    - Alex Radovici
    - Tyler Potyondy
    - Branden Ghena
    - Leon Schuermann
    - Cristian Rusu
    - Felix Mada
    - Ioan-Cristian Cirstea
- **Agenda**
    1. Updates
    2. Buffer Management
    3. Discuss 14.5 Layer Security (maybe during updates?)
- **References:**
    - [14.5 Layer Security](https://github.com/tock/tock/pull/3652)
    - 


## Updates
### Drawings
- Alex: collaborative drawing with draw.io isn't really possible. Connection to github can commit automatically, but doesn't really share. Screensharing is the best option
- Leon: A bunch of commits seems terrible
- Alex: We could make a branch and commit there
- Branden: Screensharing worked great since we're all joining on computers
- Leon: Always make sure to export PDFs, which embed the draw.io source and can then be editable
### Notes PR
- Alex: PR for last week's notes should finally be good-to-go. Pulled Leon's fixes
### Clocking
- Alex: Clocking issue with CAN that we'll push PRs for
- Felix: Peripherals in Tock don't have a way of determining what their clock is. So some might run at 100 MHz others at 50 MHz depending on prescalers. I'm working on adding a trait so peripherals can determine that information and add some functionality to enable/change the PLL (but not on the fly). On STM32, a problem is that the PLL is an analog circuit and based on the target frequency, other registers also need to be changed. That needs to be done in a certain order or the PLL won't lock.
- Alex: To put this in context, we have created a more full-featured CAN driver, but ran into the clock issue
- Branden: Often we just configure all clocks at boot. Why is that not sufficient here.
- Felix: It is sufficient, there are some other registers to change the voltage regulators. Can't change those with peripherals enabled. Need to disable peripherals, change configuration, re-configure PLL, wait for PLL lock, re-enable. Changing prescalers also requires changing some overdrive-registers. Only required for some frequencies.
- Branden: Boards can still do that at startup right? The SAM4L does something similar
- Felix: Yes. But the interface for doing so isn't clear. I'll
- Alex: Main problem, different problems tackle these problems differently. The behavior _within_ the peripheral changes depending on the frequency.
- Branden: checking the clock makes sense to me in peripherals. nRF52 does the right thing, by only having one choice. SAM4L required some decent work. There was some work like "Power Clocks" out of Stanford which did such things, but it didn't ever reach the main branch
- Leon: Some of power clocks fine-grained control of the SAM4L clocks has made it into Tock in some fashion. Really arcane traits for the SAM4L exist. Probably not applicable to other chips. For other chips, the state of the art is that peripherals implement some non-standard way of interfacing with the clock manager. Some NXP chip has some traits for having peripherals be generic over a clock rate. There are tradeoffs like not being able to configure them at runtime.
- Alex: Networking needs high-speed clocks. We have the shift the clocks up to do stuff quickly. But saving power might want to shift down the clocks.
- Leon: A good point. Configuring clocks is complex. Not sure if we'll be able to find a generic way of configuring clocks.
 - Branden: Takeaway -- you could get away with a non-standardized version, if it is going to make your life easier. If you have something portable, we'd love to hear about it (e.g., on the core call).
- Alex: Need to be portable to other chips.
- Felix: Interfacing problem is not at clock initialization, but when peripherals are changing clocks. E.g., changing the UART baudrate when switching clocks. Ioan did some tests when changing clocks and had to catch panics to show that the UART could print anything
- Alex: Moving forward, Felix can prepare a PR and we can talk about it on the core call.
- Branden: The STM32 is great as it gives you so many options, but we've been taking the easy route with the chips we primarily support.
- Alex: We'll probably need some interfaces that have "full speed" or "power saving". Could just leave unimplemented for some chips. We don't really need to dynamically adjust, just two modes.
- Branden: Great insight
- Felix: If you want to go to sleep, you have to notify peripherals. So if something is buffered, it needs to finish the transaction first. Similarly capsules could have something buffered. This back-and-forth is going to need to happen with everything.
- Leon: One question is how other embedded OSes solve these problems. Sounds like an engineering nightmare even without Rust
- Alex: Matters more with networking, as high-powered clocks take lots of power and low-power mode matters even more


## Buffer Management
- [Presentation slides](./2023-09-07/leon_tock_buffers_presentation.pdf) [Web link](https://docs.google.com/presentation/d/1Yh2bvCnUM0obqiZIPPpip9j1CCBqhZPg1Ts2P_x9u3g/edit?usp=sharing)
- Leon: Put together some slides on stuff before calls. Looking for active discussion here about goals and constraints
- Leon: First is goals. Network packets can be large, so compute and memory is expensive. Really don't want to copy buffers. In Tock now, we basically always do copies for virtualization. But multi-KB buffers are too big for this. So zero-copy whenever possible.
- Leon: Want to support wide variety of hardware. Generic as possible.
- Leon: Generally avoid run-time overheads, including spurious memory allocations or over-allocated memory
- Leon: Other thoughts?
- Branden: Dynamic memory is one. We don't want to do runtime dynamic allocations in Tock as they could run out leading to difficult to predict bugs. They should either be compile-time or be allocated upon initialization.
- Tyler: In 15.4 you have to add keys, which are for encryption/decryption. The userspace driver actually stores them and there are some lookups where you ask the driver for the linked list of which key it is. This is because there's no dynamic allocation. You don't know how many keys it might need, so they go in grants. Makes for a complicated lookup
- Branden: Especially if you wanted a capsule that could send packets.
- Leon: Continuing, I thought about Ethernet implementations to start (didn't look into 6lowpan or CAN yet). Starting with simplest: LiteEth MAC. Uses something like DMA which exposes two SRAM ring buffers. So you have one location for TX one for RX. Packets MUST be contiguous allocations. You put data in these or read from these as arbitrary memory. Could even pass a reference to part of the ring buffer area up to the rest of the kernel, ring buffer can hold multiple packets.
- Leon: Next is VirtIO MAC. Hardware and Software share ring buffers via "descriptors", a struct with a pointer and length and flags. Can chain descriptors. DMA does handle linked lists of data. No constraints on granularity or length. Packets require a VirtIO-Net header appended to them with configurations.
- Leon: STM32 or NXP iMX.RT1060 MACs. These are representative of full-featured MACs in embedded chips. Similar in spirit to VirtIO but with more constraints. Minimum/maximum buffer sizes for contiguous allocations. Might need multiple descriptors for a really big buffer. Packets need to be a minimum size or there will be under-run when transmitting.
- Leon: Finally, external MACs. SPI bus to chip. Internal ring buffers on chip that can be read/write. So buffer copy operations can be asynchronous operations in Tock.
- Branden: Added asychnrony here makes things really complicated.
- Leon: I agree. It's really common though, especially for other networks (LoRa). Unavoidable
- Alex: WiFi is like 99% external chips too
- Leon: For other protocols, I haven't looked much yet. Amit and I talked about about USB EEM (Ethernet over USB). Feels a bit like an external chip Ethernet. Requires a header to be pre-pended to packet. Can't presume that we can split into multiple buffers.
- Leon: Can others fill in how other protocols differ?
- Felix: SPI to CAN converter with asynchronous buffer transfers with possibility of failure. Microcontrollers with internal peripherals get immediate feedback about failure or success of frames. Some have big queues of frames, some are smaller. Asynchronous operation adds a lot of complexity.
- Alex: Peripheral I had for CAN had a specific memory location for buffers with slots for each frame (frames are fixed-length). Memory space for FIFO that is hidden is common too. Chips that are automotive-grade have a big chunk of memory that can be configured for how to use for sending / receiving.
- Leon: Wild collection of different interfaces and expectations is somewhat worrying. CAN does seem to meet the variety from Ethernet MACs. So hopefully any solution meets both of these.
- Tyler: Memory is passed from layer to layer. 15.4 is the hardware layer and DMA is configured on the nRF52 with a pointer and length and the radio copies directly into that when it receives a packet. I think for packet receiving, you have to change the DMA pointer after reading the packet. Then that buffer gets passed up the layer stack with some copies occurring in places.
- Tyler: For the RF233, I'm not very familiar. External chip, likely with asynchronous buffer management.
- Leon: Summary - I had feared that Ethernet was going to be way more complex. But all of these have complications and I'm actually encouraged by that.
- Branden: A complexity is that some things can follow linked lists and some require contiguous allocations.
- Leon: This is going to take multiple meetings to talk about, but lets keep going a little.
- Leon: Looking at the current Ethernet HIL, the TX path takes a static u8 buffer, so hardware could send it. The RX path takes a slice `[u8]` (not static), as it could be a slice of something from lower memory
- Alex: CAN actually has the same interface https://github.com/tock/tock/blob/d702f33e9c8c0d01df0135d9e404ac68329a4ac0/kernel/src/hil/can.rs#L788
- Leon: USB has to add headers, which is easy because you can reslice to remove the headers. Adding headers is hard though. Could split packet over two USB transactions, but that's not the same as other Ethernet interfaces.
- Alex: Another use is framebuffers for drawing to screen. Need to pre-pend SPI header to each data payload so it does copies.
- Branden: SD Card does this too
- Leon: Prototyping some "PacketSlice" code in Rust that replicates the SKBuff structure from Linux. Basic idea is a set of Rust types that allow you to fall back to treating something as a contiguous buffer allocation. But with two markers for head-room and length of data (for tail). Could support pre-allocated buffers with excess memory beyond your packet size. With const generics, we could have a PacketBuffer with a given amount of headroom. Could change the type when writing to buffer. Interfaces could specify required headroom.
- Leon: Second idea is that we could still create a linked list of these SKBuffs. A flag in the generic type could control whether it is allowed to be a non-contiguous allocation or not.
- Leon: Been working on this. It's hard to get this working in Rust when all of this has to happen statically at compile time.
- Branden: This is great. I really appreciate the thoughts here. What I really like here is the compile-time type-based checking of whether you've got the right type of buffer and enough space available. I was thinking about this idea and grappling with the issue of "how do you know if there's enough headroom available" question.
- Branden: I think that the original high-level design was for SKBuffs to be a contiguous memory space while BSD mbufs had linked allocations of memory. But I suspect that over time they've both become complicated enough that they could replicate each other.
- Leon: It's quite possible to get SKBuffs with slight different semantics and just break upper layers. They've gotten very complex.
- Branden: Something to consider: Mbuf has an "m_pullup" function which can MAKE something contiguous from a non-contiguous buffer.
- Leon: I have something like that in my prototype
- Tyler: Where would this live? Any ideas?
- Leon: Not sure yet.
- Alex: Could be applied to many buffers in Tock, framebuffers and SD Card and stuff.

