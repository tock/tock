# Tock Network WG Meeting Notes

- **Date:** March 04, 2024
- **Participants:**
    - Alex Radovici
    - Tyler Potyondy
    - Branden Ghena
    - Leon Schuermann
    - Felix Mada
- **Agenda**
    1. Updates
    2. OpenThread
    3. Buffer Management
    4. Tock Ethernet
- **References:**
    - [2625](https://github.com/tock/tock/pull/2625)


## Updates
### WiFi HIL PR
- https://github.com/tock/tock/pull/2625
- Alex: I'm looking for someone who wants to implement WiFi for the RPi PicoW, which is the Pico plus a infineon chip that does WiFi. There's a Rust driver for that WiFi already that's asynchronous!! Our goal is to port that over to Tock at some point.
- Alex: One downside: we do need to load binary firmware onto the infineon chip too. So the binary will be data in Tock
- Leon: This is okay actually. OpenTitan has a firmware blob for the big number accelerator, and we even have the infrastructure to parse binaries from the TBF
- Leon: I also use a similar method for encapsulated functions
- Alex: Motivation, we have a class that uses this board but can't use Tock because we need to connect to WiFi
### PR Assignment
- Leon: Should PRs with the network WG tag be automatically assigned to someone on the Network Working Group instead of Core?
- Branden: I don't think it's necessary right now. Few PRs and we've been attentive to them


## OpenThread
- Tyler: Good progress! OpenThread stack works right now: fully joins a Thread network and remains attached with child update requests. The wireshark trace for OpenThread on Tock matches OpenThread on baremetal Nordic boards.
- Tyler: As part of that, a major fix and weird bug: in 6lowpan with fragmentation packets. With 15.4 the max packet size is 127 bytes. 6lowpan provides a way to fragment across packets and recombine. As part of that, the packets send in quick succession. The current design for receiving: the user process provides a buffer with 129 bytes of space. The kernel when receive is called transfers the buffer into the user buffer and schedules a callback. But the kernel maintains control of the buffer until the upcall has been handled. So what was happening is that the second packet arrived and overwrote the first packet. So the first one wasn't received and all of the packets were dropped. The solution is a larger buffer for queuing packets. So I made a way for the user process to provide a ring buffer instead of just one packet. That's not too much of a change in all honesty, but expect the PR today.
- Leon: This is a very familiar problem to me. The TAP driver on tock-ethernet runs into this problem. We can't be sure when userspace will be scheduled and for how long and whether it processed everything. The design of Tock's upcalls right now and capsules not knowing if they've been received is intentional, but there's a broader need for a more efficient ring buffer data structure. Way back when we redesigned the userspace, either the kernel or userspace should have sole ownership of the data when allowed. So a "shared" ring buffer is likely non-compliant. It needs to be unallowed before modifying it.
- Tyler: I very carefully handle that actually, and think I'm in compliance
- Leon: That's great. I think in the long run we'll want to have an explicit system call for transferring packets through a lock-read data structure.
- Tyler: Right now it's a pretty rudimentary data structure. The userspace unallows the buffer before making changes and allows the buffer back after copying data over to a separate userspace location. Long term, do you think we should implement a bigger fix now?
- Leon: Wouldn't want to hold anything up on this. Your PR is probably a good first step. What I want is a general solution that's got stability guarantees. So I wouldn't want to blindly promote a solution for general use
- Branden: What's left for OpenThread development then?
- Tyler: "Works". Only on channel 26 right now. Getting channel switching implemented keeps falling on the priority list, getting sending / receiving to work has been challenging. Need to be able to switch channels, clean up PR and submit it. There is some hand-waving around signal-strength indication (currently just hard-code RSSI to -50 and link-quality indicator). This information doesn't make it up to the 15.4 part of the stack. Thread works by having a child issues a parent request, and chooses best parent based on the RSSI (so important for router selection). In my opinion, the most challenging part is getting the radio packets correctly parsed. It made me really happy to see that Thread works generally.

  Current architecture: in a loop: call "do thread work", then yield. When a packet arrives or alarm fires, there's a delay before yielding again and I didn't know if that would be okay.
  
  Next steps: let device running, see whether it crashes or falls off the network.
### Remaining OpenThread Work
- Branden: What's left for OT? Channel switching, RSSI, flash implementation, ...?
- Tyler: Progress underway for the flash stuff by two students.
- Branden: Of those three, probably least important. The first two are probably useful for the tutorial.
- Tyler: Tentative timeline: have most of this upstream in the next two weeks.
### Tutorial Planning
- Tyler: One more thing on the demo front, Brad sent in the CPS-IoTWeek channel a message demoing a board with screen. However, we didn't find a board that doesn't have a JTAG built in. We looked for an Arudino shield with a screen, but there wasn't anything good. I may quickly try to make a screen shield myself to support the tutorial. Would be neat.
### 15.4 Design
- Tyler: 15.4 related design question. We have a send raw syscall now. We also need a raw receive at some point. We want OpenThread to handle the decryption. So we should let the user process do this instead of the kernel. So we need a way to handle a non-decrypted packet to the user process. Right now we're just delivering all packets so OpenThread works.
- Branden: Is it some packets are decrypted by the kernel and some by userspace or all either userspace or kernel? All in either direction seems easy. Some is harder.
- Tyler: Hmm. I think I just need to make a PR with more context. The change itself is minor but there's a design decision.
- Tyler: I will make a PR for this soon.



## Buffer Management
- Leon: Some interesting developments. Chatted with Alex's student Amalia twice. Talked about motivations and then went into the Tock UART stack where this would be deployed first. She's been refactoring some of the HILs to use to opaque packet-buffer type. I've been working on transforming the playground example into something we can really use throughout the kernel. Plans to meet again and keep pushing on the UART subsystem.
- Leon: For this first iteration while working on it, I realized we're likely better off removing the complexity of non-contiguous buffers to start with. We are confident that they could work in conjunction with everything else, so we could reincorporate them later. But for now, it's really adding a lot of complexity when working on the basic implementation.
- Leon: The current implementation has headroom and tailroom implemented for networking features. And documentation!
- Leon: I'll push my stuff to a branch relatively soon
- Branden: We're doing this for UART? I though the first target was the screen?
- Leon: We thought a better goal was the UART multiplexer. It's a sweet spot because we have a lot of existing infrastructure that could use it, and what we do is pretty simple: prepend four bytes to the buffer that indicate if it's a new message, an application ID, and 16-bits of length of the current stream.
- Alex: The console also needs headers too. The console multiplexes applications. The UART multiplexes data. There are three virtual UARTs: one for console, one for debug, one for processConsole. This is messy, but is good because there are multiple headers to append, which makes it a good buffer management example
- Leon: The good thing is that UART stuff isn't actually changing. So we can just work on stuff above it.


## Tock Ethernet
- Branden: Just a check-in about whether anything needs to be done here.
- Leon: Should really open a PR at some point. It does need a ring buffer to userspace solution, which Tyler's stuff could actually be useful for. The one single reason not to merge it today is that we'd have to increase the memory use for the stack frame on every board by a large amount.
- Leon: No one's using it right now, so it's low priority. If someone needed Ethernet support, it would increase in priority.
- Alex: Would be great to have it, but not waiting on it right now.


