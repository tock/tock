# Tock Network WG Meeting Notes

- **Date:** February 10, 2024
- **Participants:**
    - Alex Radovici
    - Branden Ghena
    - Leon Schuermann
    - Felix Mada
- **Agenda:**
    1. Updates
    2. Ethernet
    3. Libtock-rs
- **References:**
    - [4324](https://github.com/tock/tock/pull/4324)


## Updates
- Branden: Group of students working on WiFi for the RP2040. First step is a SPI driver over PIO to talk to the WiFi chip. They have a synchronous SPI working and are now working on an interrupt asynchronous version so it'll work with Tock.
- Leon: Actually, you could stick with the synchronous version in Tock for now if needed. Just do deferred calls after the synchronous action so you can get a client callback.
- Alex: We needed to do this for the 8080 bus since it does not provide interrupts at all
- Branden: I didn't even consider that. Good to know it's an option if interrupt stuff stalls
- Alex: On our side, it's Darius who's working on the RP2040 stuff. He made the PR for the Cypress stuff. And right now he's working on the RP2350.
- Alex: Interestingly the RP2350 has a huge bug. GPIO inputs lock up at 2.2 volts to logic 1 for pins 0-47. This affects SPI and I2C too. https://hackaday.com/2024/09/20/raspberry-pi-rp2350-e9-erratum-redefined-as-input-mode-leakage-current/
- Alex: So, we've got a renewed focus on the RP2040. I connected Darius with your Northwestern students via email


## Ethernet
- https://github.com/tock/tock/pull/4324
- Leon: This PR is updated.
- Leon: I spent some time thinking about naming: Raw or Data. We don't know what a more-fully-featured Ethernet HIL would look like. Even a more featured HIL with control over things will still probably have Raw Ethernet frames. So I settled on EthernetAdapterDatapath
- Leon: I think it's nice to have the Datapath be separate here, as you could have this same Ethernet-ish Data path for other communication mechanisms like process-to-process communication
- Leon: Apart from that, nothing hugely controversial
- Leon: The Tock Ethernet branch doesn't have the enable/disable functionality yet, but that'll be easy to add
- Alex: Very similar to CAN which has a Controlpath and Datapath
- Leon: Next up is moving the TAP driver to the StreamingProcessSlice. Does CAN have any progress towards a userspace abstraction for StreamingProcessSlice?
- Alex: In C it's fine. In Rust, we're not sure. We have a problem with sharing the slice. We can't swap a slice in safe Rust. I have an issue about this from a year and a half ago
- Leon: I thought we settled on StreamingProcessSlice over a static buffer which has unsafe operations
- Alex: We never really settled on anything
- Alex: If we merge the Ethernet stuff, we will need this
- Leon: What we have now is a TAP driver which sends everything to userspace. It really needs to be updated to StreamingProcessSlice which is going to be a big improvement over the current hack
- Leon: I originally wanted a libtock-rs implementation for StreamingProcessSlice before merging it
- Branden: Well, that could be waiting for years
- Leon: I'm wary that we could be migrating to StreamingProcessSlice with us never being able to produce a sound implementation for libtock-rs. That could increasingly preclude some subsystems from ever working with libtock-rs
- Leon: Right now we don't have any subsystem ported to this in userspace. Ethernet and CAN are the first two
- Alex: The penalty that libtock-rs puts us in with copying is a disaster
- Leon: Is the active strategy to ignore libtock-rs for CAN?
- Alex: We ignore it because we can't use it. I've been trying to port Embassy on top of it, but it's impossible to layer on top right now due to buffer sharing
- Alex: If we want to have libtock-rs useful for something, we have to change this
- Branden: One challenge in libtock-rs has been engineering effort. It might be good to bring this up to the Core working group in the context of Tock effort planning
- Leon: Well, one side is that libtock-c is a path towards advancing the kernel. And we can keep punting on libtock-rs stuff.
- Branden: That has been the status quo in Tock for some time
- Leon: For this new subsystem, there's a different concern. The semantics StreamingProcessSlice exposes are potentially hard to represent soundly in Rust userspace. That's different from prior cases where we expected it was just effort to make things work. Now we're not sure at all if it even can be soundly implemented in Rust userspace
- Leon: In my mind, we should make sure it's possible to soundly support this before adopting in Tock kernel
- Alex: Libtock-rs today is very sound, but not incredibly usable. Which is hurting adoption
- Leon: So, I'm going to spend time this week on porting Ethernet to StreamingProcessSlice. I could make a custom C driver, a generic C driver which would work for CAN, or start with libtock-rs
- Alex: I would concentrate on C
- Branden: Do you think it's not possible to represent StreamingProcessSlice in a Rust userspace, or libtock-rs specifically
- Leon: No. It's the requirement that we can swap buffers without an unallow that's different here.
- Branden: I am worried about the timesink trying to update libtock-rs which could derail Ethernet effort
- Leon: We should bring this up at a core call this week. My concern here is that we're moving from ignoring libtock-rs to instead adding features that actively hurt libtock-rs. I want to make sure this doesn't bite us in the future
- Branden: For now, I think it makes sense to continue pushing Ethernet forward in Tock, to use StreamingProcessSlice, and to focus on the C userland
- Leon: Okay, I'll focus on generic libtock-C implementation of this
- Branden: And it doesn't have to be perfect for now. Just best-effort at making something that seems like the generic implementation everyone could use. CAN will come around eventually and fine-tune
- Branden: This work will be another PR to tock-ethernet?
- Leon: Yes. And I'll make the enable/disable updates ad hoc as I go
- Leon: The next batch of PRs to the staging branch will be the Ethernet implementation adapted to the new HIL. Some VirtIO stuff
- Leon: Then after that is the TAP driver with LWIP in userspace. Probably a few days of effort with a rewrite of it
- Leon: After that is the remaining Ethernet implementations


## Libtock-rs Support
- Branden: An action-item here is to discuss libtock-rs status on the Core team call. We can consider what the key needs are for libtock-rs and think about priorities for them
- Branden: Specifically, we want to focus on the StreamingProcessSlice abstraction and updating kernel drivers to use it, which would harm libtock-rs
- Leon: I would love to hear about OxidOS's needs for a Rust userland. I think most teams using Rust userspace are either doing a single Flash image or fixed locations, so thinking about relocation needs would be useful there
- Alex: Most clients want to write C apps. They want Rust HAL APIs. So porting Embassy would be valuable, because that's what they use now.
- Alex: One thing we're looking into is RISC 64-bit support, which has an MMU and avoids relocation issues
- Leon: Tock's syscall API sort of looks like a classic async thing DMA over buffers. Could we just layer Embassy on top of the syscall API without libtock-rs? It sounds like libtock-rs is doing a bad job as a HAL for an async runtime. But maybe the Tock syscall interface would be better
- Alex: I agree. We're looking into this. Writing all the abstractions is another challenge though.
- Branden: Embassy support for Tock would be quite valuable

