# Tock Network WG Meeting Notes

- **Date:** August 04, 2025
- **Participants:**
    - Alex Radovici
    - Branden Ghena
    - Leon Schuermann
    - Tyler Potyondy
- **Agenda:**
    1. Updates
    2. WiFi Check-in
    4. STM32 Ethernet Check-in
    3. IPC Use Cases
    4. Potential IPC Mechanisms
- **References:**
    - [WiFi PR](https://github.com/tock/tock/pull/4529)
    - [STM32 Ethernet PR](https://github.com/tock/tock/pull/4524)
    - [IPC Use Cases](https://docs.google.com/document/d/1iL_DPMygbB4XAEZMSESP8Q-xA7Kq04C-B1lLUfB114E/edit?usp=sharing)
    - [IPC Mechanisms](https://docs.google.com/presentation/d/13mYERv0iKvBPOsu52jsd4jjZalbd_KXHbwDGdt8qgRw/edit?usp=sharing)


## Updates
### Future Meetings
* Cancel meetings on August 11 and August 18
* Next meeting planned for August 25
### QEMU Updates
* Leon: Brad and I have been hacking on a way to use QEMU to virtualize a board with a screen and keyboard. Might be cool for teaching purposes and wouldn't require a board for everyone. We're at the point where some basic stuff works!
* Branden: My OS class uses VirtIO for GPU drawing. Just the higher layer
* Leon: For RISC-V right now, but screen should be portable to other architectures
* Alex: We're doing VGA support too.
* Leon: But we could use ARM or RISC-V with VirtIO GPU.
* Alex: Can x86 use VirtIO GPU?
* Leon: VirtIO is a bus for para-virtualized platforms. Sets up ring buffers between host and target. A bunch of devices can use that common bus to talk to the OS. We have that implemented already in a crate. So QEMU adds some MMIO registers which you use for the bus. Peripherals then run on top of that.
* Alex: Can you share some examples of this?
* Leon: Yes. Some examples in Tock already and some other files I can share that make arbitrary queues. Most of the communication happens over shared memory. MMIO is just for handshaking.


## WiFi Check-in
* https://github.com/tock/tock/pull/4529
* Alex: Hold off on review for now, lots of changes in flux
* Alex: Firmware repo is in the works. Based on GeorgeRobotics firmware repo. PR in the works here: https://github.com/tock/firmware/pull/1 Conclusion from Wednesday was a separate repo with firmware crates, which we pull into Board files. Capsule will have init that Board will give the blob too
* Alex: Working on half-duplex SPI / SDIO transparency. Realized the SPI HIL doesn't fit this well right now. We're considering transmit/receive from UART as the generic interface. Underneath that we'll have the half-SPI and SDIO.
* Leon: It's not weird for SPI peripherals to send an interrupt when data is available. But typically the interrupts would go outside of the interface. So what's the issue there?
* Alex: I think the interrupt occurs within the PIO. Which makes it hard to disentangle things. Still a work-in-progress here. Essentially, you send commands and asynchronous get back something later.


## STM32 Ethernet Check-in
* https://github.com/tock/tock/pull/4524
* Leon: PR was merged. I inadvertently broke the STM32 ethernet support. Ionut fixed it
* Leon: Also had to fix baud rate. Another PR does that
* Leon: Once that's merged too, we can extract from tock-ethernet and make a PR into master with the STM32 stuff. That'll be well-tested and working. Code-quality is good and Ionut understands it well


## IPC Use Cases
* https://docs.google.com/document/d/1iL_DPMygbB4XAEZMSESP8Q-xA7Kq04C-B1lLUfB114E/edit?usp=sharing
* Alex: Use cases that my colleagues have gathered from automotive contexts.
    * Networking and CAN signals transferred into one app, but then a separate app would do logic
    * Periodic messages come over CAN buffer, with various sensor data. The only thing you care about is the latest message and timestamp. Never care about history, just most-recent
        * Branden: So no FIFO, just want most-recent. Shared memory maybe
        * Alex: Usually 8-byte or 16-byte. Numbers
        * Alex: These are commonly used and needed
        * Alex: These are usually broadcast to everyone
        * Branden: Could be "every capsule allows a value" then Service makes a command that copies into everyone's allowed buffer. Buffers would always hold the most-recent value
        * Alex: Could just be numbers though. Not a ton of memory basically ever
        * Alex: Clients could just do a command and get response with the value
        * Branden: So Server would post to capsule with a command, clients would read value from capsule with a command
        * Branden: Do I need different value IDs?
        * Alex: Yes. Server could have several value IDs available.
        * Branden: And clients are requesting via value ID
        * Alex: And capsule would have a template of number of values. You would know at board file creation time
        * Alex: Some messages would be value number and process ID (or some handle for them). You'd pass the handle and that would send the value to the handle.
        * Branden: That's just a ring buffer of length one, right?
        * Alex: I think it's the same
    * Notification messages: counts number of events that occurred. Number of interrupts per unit time gets you information
        * Alex: Counting the number of times an event fires
        * Branden: How is this IPC?
        * Alex: Shared-memory counter value that updates in real time. Doesn't have to be shared memory, could copy for sure.
        * Branden: Okay, so this could be he same as Periodic messages, where you just read and get the most-recent counter value?
        * Alex: Yes. Either read and it resets, or you read it and it continues incrementing from there
        * Leon: Sounds like Linux VDSO
        * Alex: Producer makes a system call and system distributes to everyone
    * Queued numeric messages: care about history of messages. FIFO queue. Someone needs to read it. Queue could be of size 1. They must arrive in order. Queue filling is possible.
        * Branden: Does this go to a single app?
        * Alex: Could be broadcast or could be single application. For example, networking
    * Queued buffer messages: Same idea, but arbitrary length of bytes. Packets.
    * Ring buffer: Same idea, but overwrite oldest if necessary. Could also be numerical values or buffers
        * Branden: Both ring and queue are necessary in some cases?
        * Alex: Networking with TCP needs a queue. UDP can do a ring buffer and lose packets. Depends.
        * Alex: We need these kinds of buffers for using existing C networking stacks. Then IPC to send/receive packets

* Tyler: With the use cases and different buffer types, is the purpose telling us how to create IPC? What's the context for them?
* Branden: I think the goal here is that you need these behaviors, and IPC mechanisms would have to be able to create these
* Alex: Yes. These are the kinds of things AutoSAR apps would need
* Tyler: Are there any pitfalls for implementing these in libtock-rs? Are these theoretically possible in libtock-rs?
* Alex: Anything with buffer exchange is hard in libtock-rs right now. Numbers are fine.
* Branden: We would need these behaviors to exist in a Rust userland. How they're implemented is an open question
* Branden: I found this super useful as guidance
* Alex: I will say that we don't know how this will hold moving forward. Automotive industry may change
* Leon: Interesting because it's a different set of concerns and design points


## IPC Mechanisms
 * https://docs.google.com/presentation/d/13mYERv0iKvBPOsu52jsd4jjZalbd_KXHbwDGdt8qgRw/edit?usp=sharing (go to slide titled "Mechanisms Overview")
 * Branden: Wrote up some ideas on possible IPC mechanisms and I was hoping to get ideas from all of you about it. Feel free to fight me on these
 * Branden: First, Manager capsule that brokers IPC - verifies and share AppIDs
   * Branden: each app has to register with this capsule and get an AppID
   * Branden: Services wouldn't discover clients this way. Whenever a client uses a service, the service would get the client ID.
       * Tyler: notification mechanism in the kernel? cleanup?
       * Branden: whatever ID this provides needs to take into account the current liveness of the application (e.g., Process instance ID)
       * Leon: How would the service know if the client is "authentic"
       * Branden: did not think of this yet.
       * Leon: I think both sides would like to know this
 * Branden: 4 mechanisms
     * Notifications - command to upcall (*server to client* - only one considered / client to server / client to client)
     * One-copy mailbox: allow-to-allow copies (or command-to-command copies)
        * Limit to one outstanding value at a time per client
        * Client A wants to send a value to a service -> allow value, values sits there
        * Server says I would like to read a value, which copies one client's value from their allowed buffer to its allowed buffer
        * Possibly multiple mechanisms for choosing which client to read from
        * Leon: like a DMA engine, client controls the buffer, service acts like the device
        * you can reply to the client
        * Leon: the client could abort and retry a transaction
        * Leon: does the service need to wait on the client
        * Branden: The server would issue a command that tries to copy data to its buffer - this may fail
        * Leon: The server would never need to wait on the client
     * Two-copy channel: allow-to-fifo-to-allow copies
        * Initialized with a maximum size (either from Grant region or in Board file)
        * Either dedicated to specific Service, or first-come-first-served for reservations
        * Leon: why does this exist? Why can't you just use the one-copy mailbox?
          * Branden: if your app wants to have several outstanding messages.
            * Leon: why can't we do queueing in the application?
              * Branden: I think we could. Difference between those is that it doesn't keep a FIFO ordering between application requests.
              * Leon: That could be good to not do that. That we're round-robining between clients. Could add something to Mailbox that requests next-from-application-id explicitly
     * Shared memory: share Client memory chunk with Service
        * Limited by MPU in practice
        * Service "activates" chunks of memory to connect them and keep them alive
        * Leon: Service could keep a process as a zombie forever (yes)
        * Leon: Could maybe automatically map on fault, but need a deactivate too
        * Branden: This allows Service to choose how to spend its MPU slots
        * Leon: This is a "fast-pass mechanism". It's hardware dependent and not as capable on all systems. So it could be a challenge to use this but fall back to something else
        * Branden: I think you don't do this. You use this for platform-specific code
        * Leon: Are there rules for who accesses the shared memory at what time?
        * Branden: No, just shared memory
        * Leon: You could not run client while Service is accessing shared memory
        * Leon: Want to use this for huge buffers of data which are too expensive to copy. But also want to validate that memory. If you don't have a guarantee that it won't change, you could have time-of-check, time-of-use problems. Needs to be either exclusive or read-only memory.
        * Branden: Other design there would be a shared memory capsule which own the memory, and dole it out to one application at a time. That would give exclusive access
    * Tyler: Cool to get some iteration on design ideas
    * Leon: I think it's nice that you could sort of pick some of the simple ones here and implement them, and get something for IPC, without having to implement everything including the complicated stuff
    * Branden: Something to think about moving forward - are there application architectures that you straight-up could not implement with these designs? Want to make sure we have stuff for many use cases

