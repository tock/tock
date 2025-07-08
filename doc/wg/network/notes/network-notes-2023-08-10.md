# Tock Network Notes 2023-08-10

## Attendees
 * Branden Ghena
 * Alexandru Radovici
 * Felix Mada
 * Leon Schuermann
 * Cristian Rusu
 * Tyler Potyondy

# Introduction
 * Alex: First meeting! Goal to define how networking is going to work in Tock. Round of introductions first!
 * Felix: Working at OxidOS, mainly on CAN bus. Started with Tock a couple of months ago. Worked on embedded for four-five years.
 * Leon: PhD student at Princeton working with Amit Levy. Working on Tock for five years. Funny enough my first real project was an Ethernet driver for Tock. Still in progress...
 * Tyler: Master's student at UC San Diego working with Pat Pannuto. Only involved in Tock for a few months. Working on adding support for Thread networking: 802.15.4, 6lowpan, UDP, etc.
 * Cristian: I teach at University of Bucharest. Background in wireless communication: 5G stuff. I'm here because I know about Tock and networking.
 * Alex: Professor at Polytechnic University in Bucharest. Started on Tock in 2020, interested in Rust OS for teaching. Still interested in network communication for teaching.

# Organization of meetings
 * Alex: Let's begin with organization. I say the same as Core WG model: someone takes notes and posts to github. Markdown.
 * Leon: One thing that bugs me from the Core call is that when you volunteer notes you end up having to multitask and do notes later. Maybe we could have a way to share the notes in real time, so someone else can take over
 * Alex: That sounds good to me. What platform?
 * Leon: Tool called HackMD
 * Branden: I vote for that for next meeting
 * Branden: How do we handle agenda? Should we send a notification in advance, or start the call with agenda. Or end with agenda?
 * Leon: We can play this loose. I do like having the call for items as a reminder
 * Branden: Is sending out the agenda reminder on Slack fine?
 * Alex: We'll need to make sure everyone is on it
 * Alex: What about drawings? Need some collaborative platform for that.
 * Tyler: Generally in Tock, the documentation in terms of diagrams is pretty lacking in Tock. I think a great goal would be to ensure that we create visual documentation as part of our working group.
 * Alex: I used draw.io in the past
 * Branden: Alex has an ACTION ITEM to figure out a drawing platform

# Working Group PR
 * Branden: https://github.com/tock/tock/pull/3578 needs two more small edits before merging
 * Alex: I will update that tonight
 * Branden: We can merge tomorrow on the core team call

# Overview of Network Stacks
 * Leon: Would be good to do a recap of what people on this call have worked on. Then we could have visualizations of those for next week.
 * Alex: CAN, Ethernet, Thread all seem in progress

## Ethernet
 * Leon: Trying to start with Ethernet. Been working on this for a long time. We have discussed at several past meetings of Tock groups. We specifically chose Ethernet as an example because it's one of the simplest transports to integrate with existing IP networks. Back in 2022 we didn't have any layer2 transport implementation. Over the last year we have written three proper implementations of Ethernet MAC layer. They differ in feature set, APIs, buffer management. So a key takeaway now is defining higher-level interfaces and going above layer2 in the OSI stack. IP layer, UDP transport, ultimately UDP/TCP based applications. We have a work-in-progress branch in Tock and Libtock-C where the Tock kernel has a Ethernet TAP driver (TAP/TUN from BSD Linux) which is a driver which forwards Ethernet layer2 to userspace and back. And we have userspace libraries to implement IP: LwIP and `smoltcp`. We're at the stage in the work-in-progress environment where we can share a small HTTP webpage. At Tockworld 2022 we had decided to start with Ethernet layer2. It's a great place to start, but definitely not what we want in the long run.
 * Branden: So we have three implementations?
 * Leon: We do have three entirely different Ethernet implementations. STM32F4 series by Alex's student Cristian. Lite-ETH for FPGA by Leon. VirtIO QEMU device by Leon. Fundamentally different.
 * Branden: Something I'm interested in is how they compare/contrast. Topic for a future meeting.
 * Alex: We have the same TAP driver implemented for each?
 * Leon: The drivers all use the same HIL to interface with the TAP driver.
 * Branden: Oh wow. So all three of these fit with the same interface.
 * Leon: Amit has been working on Ethernet stuff. He has some form of Ethernet-over-USB (USB CDC-EEM), which presents the Tock board as an Ethernet device when attached over USB. I believe he has some "rudimentary" IP and TCP stacks in the kernel, mostly for his own testing.
 * Alex: Is this for the nRF chip?
 * Leon: Should be anything that implements a USB stack
 * Alex: We have a USB stack, semi-working, for the RPI2040

## Thread
 * Tyler: Hudson worked on a 15.4/6lowpan stack historically. As an aside, for the Nordic dev boards the chip doesn't have hardware support for automatically sending acknowledgements and it turns out that we have to re-write 15.4 for the Nordic boards to support OpenThread. The board was being turned on/off after each packet, which ruined timing for acknowledgements. Imix did have hardware support for ACKs. Moving on from here, we've been thinking about routing in IPv6 layer which will be shared with Ethernet for example and Thread uses UDP as well.
 * Tyler: Current capabilities of the Thread stack. It is limited so far. It can allow a child to join a router as a sleepy end-device, most simple device operation. The userspace boundary there is not very clearly defined, just working on the capsule side of things. My application connecting to userspace will have some API for joining and using a network.
 * Alex: So you'll have a capsule that does all of the work. The capsule would interact with the stack instead of userspace. Interesting to see a capsule as the "user" rather than userspace.
 * Tyler: Currently none of this is upstream though. It's on my own branch. Should be pushing it soon, after rewriting the Nordic 15.4 driver.
 * Branden: Any thought about Thread on top of Imix? Or just Nordic boards for now.
 * Tyler: Just Nordic for now since it's what I have. Reaching out to Hudson about other boards. No reason it _shouldn't_ work on other 15.4 boards, but it's untested.

## CAN
 * Felix: CAN is a weird protocol. If your buffers are full you actually have to prioritize which frames should stay. A lot of the stack is about prioritizing frames. I can transmit frames if I have a high delay between them. If I overload it with frames per second right now, it goes crazy. CAN has pretty limited hardware IP, the same implementation in multiple MCUs. The difference is often which version of the IP with some small additional features. The STMF4 has just a couple of hardware queues. The bigger NXPs have hundreds of queues. Also they have a lot of outputs. Can use an MCU as a gateway to connect one CAN bus to another. I have a small stack that's all in the kernel for the receiving part. Sometimes it misses frames, but it can detect that it misses frames. On the transmission part, if I send a ton of frames then it goes crazy.
 * Felix: Most of the stack is in software, but I have two hardware abstraction layers. One for the universal way of working with CAN hardware, one for very specific hardware where the manufacturer changed a lot of things. That latter HAL just has a simple "send" option.
 * Branden: What does your userspace interface look like?
 * Felix: I have a couple of buffers where userspace can put frames. Interface just sends. On receiving, most frames just arrive into buffers.
 * Branden: What does the interface to userspace look like?
 * Felix: There is a special interface that userspace adds which has a header that the capsule decodes. The capsule then encapsulates the data from the userspace and figures out how to send it.
 * Leon: Do you have any thoughts on interoperability between Ethernet and CAN?
 * Felix: CAN is really just a way to send 64 bits. You have to decide what the bits mean. There is a CAN transfer protocol for sending huge frames. It still doesn't define meaning of bits though. So for Ethernet to CAN, you just stuff the ethernet packet bits in CAN.
 * Leon: So for bridging, I would make my own encapsulation protocol with headers, and then make my own translation component which translates the frames.
 * Felix: Yes, sort of. In AUTOSAR there is some logic. Certain signals correspond to certain collections of bits.
 * Leon: So a question from Tockworld I had was how to make an interface to bridge protocols. So I'm interested here
 * Alex: I think in cars they use mostly UDP. So they take the frame from CAN and drop it in a UDP frame. Then they just need to add retransmission, for some types of packets, others they don't care if they lose.
 * Branden: Do you have support for large packet types?
 * Felix: Not at this time, although we can fragment across small packets

## LoRa
 * Branden: The existing LoRa support in Tock is entirely Alistair's effort, but I'll say some things on it. LoRa is long-range wireless (kilometer range) while still at low energy and at low bitrate (kbps). There only exists really one family of chips for LoRa communication, which are usually communicated with over SPI. So Alistair's efforts connect a raw SPI interface to userland, and there are C libraries to drive the LoRa radios over SPI. This would usually have problems with timing, but LoRa is very slow, with a full second between reception and an ACK, for instance. So this works.
 * Alex: A LoRa gateway could be an interesting connection of networks here: LoRa + Ethernet
 * Branden: Yes, although a LoRa gateway is complicated and we don't even come close to having support for it. Gateways need to service multiple incoming packets simultaneously, which does lead to timing requirements.
 * Leon: Do you think LoRa will still benefit from a buffer management interface?
 * Branden: Yes. It still has buffers to pass around like anything else. Since it's not so timing sensitive, copies aren't an issue, but it could still use them.

# Wrap Up
 * Leon: Think about things to prepare for next week's meeting.
 * Alex: Definitely action item for drawing
 * Tyler: As I've been working on 15.4 driver, there doesn't seem to be a lot of considerations for being low power. Is that something we want to talk about?
 * Branden: I don't think so. Tock has historically been low-power-ish, just like it's real-time-ish. Not actually real time, but fast enough. Not actually low power, but lower power enough. 
 * Leon: Amit once described Tock as a platform for your OS. You can configure the kernel and set options to either target high-powered gateway devices or lower powered devices with less resources. I think the lesson, in my view, is that it doesn't hurt to make things more efficient. Including functionality which uses more power wouldn't kill us though.
 * Tyler: Example, in Thread the radio is probably just always on
 * Branden: We eventually want an interface to turn it off, but that seems okay
 * Leon: One action item for me is digging up the half-finished buffer management proposal (based on Linux SK-buf). Not finished in any way, but good to present on.

