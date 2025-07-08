# Tock Network WG Meeting Notes

- **Date:** January 27, 2025
- **Participants:**
    - Tyler Potyondy
    - Branden Ghena
    - Leon Schuermann
- **Agenda:**
    1. Updates
    2. TAP Driver
    3. UDP
- **References:**
    - [15.4 Streaming Process Slice Issue](https://github.com/tock/tock/issues/4323)
    - [15.4 Ack Issue](https://github.com/tock/tock/issues/4313)
    - [Ethernet Adapter HIL PR](https://github.com/tock/tock/pull/4324)


## Updates
- Branden: I have two students (Anthony and Jason) working towards RP2040 WiFi. On the RPi Pico W board, there's a WiFi chip. It connects over SPI, but for some reason doesn't use the actual SPI pins on the RP2040, so you MUST control it over PIO. So, my students are working on a PIO driver that implements a SPI interface for the radio. They're part-way into getting that working now and are still actively moving on it.
- Tyler: I put up some issues that we talked about last time as relatively easy to-do items for new people to work on. That's the 15.4 ACK issue and the streaming process slice issue.
- https://github.com/tock/tock/issues/4323
- https://github.com/tock/tock/issues/4313
- Tyler: We also have a student working on 15.4 stuff. He's going to start on the ACK stuff. There's another student interested in BLE stuff, so he's going to start by testing the existing driver and expanding on advertising mechanisms. Then he could go onto more BLE from there
- Leon: I'm convinced that BLE could work with encapsulated functions

## TAP EthernetAdapter Driver
- Leon: Porting this to process streaming slice. This takes Ethernet frames from the MAC and puts them into a user buffer, and also does the opposite from userspace.
- Leon: It was written several years ago, and Ethernet is a fast bus compared to normal Tock speeds. So an end device might send frames in quick succession, and if we do the regular allow/unallow we'd end up dropping a bunch of frames. So my solution years ago was a ring buffer in the kernel, but my implementation was bad and used a ton of stack size.
- Leon: The new streaming process slice does this better. We might still have to drop frames at some point, but we can at least support smaller bursts without drops.
- Branden: It also gives a notification about dropped packets, right?
- Leon: Yes, but no higher level layer cares about that notification. The most important part is to be able to do a small burst of 2-5 packets, even if we couldn't sustain it over a long time
- Leon: It isn't trivial to do this change without breaking userspace too
- Leon: First change was to update the HIL for the TAP driver: how it connects to the other drivers
- https://github.com/tock/tock/pull/4324
- Leon: So, we'll start here. Then another PR will come soon with the driver.
- Leon: This also lets existing Ethernet drivers in the branch be merged into Tock mainline. They've previously been reviewed, but only live on the branch.
- Leon: Something we could do is look over this HIL for immediate questions (yes, doing it)
- Branden: The packet contents make sense from the comments
- Branden: It's weird to me that there's no `receive_enable()`
- Leon: It just starts at initialization right now, but that would be easy to add. We could add enable and disable
- Branden: Why is `len` a `u16`? Feels arbitrary
- Leon: I think there's a good reason for that. Ethernet is weird and has this type field in the header, which can be a proper type for Ethernet2 in which case there's no length. But IEEE 802.3, the type is used as length instead, which is 16 bits.
- Leon: The problem here is that if we follow Ethernet2
- Branden: Do others actually use Ethernet2?
- Leon: From Wikipedia - The max length is 1500 octets. If length is greater than that, then the frame has to be Ethernet2. So jumbo frames are Ethernet2
- Leon: So, you might have a good original point about maximum length though. I don't know it
- Branden: Do we need to support jumbo packets on embedded systems?
- Leon: There's no good reason to ignore them. The default MTU is 1500 bytes anyways. The maximum jumbo frame size seems to be (super jumbo frame) 65535 bytes. So 16 bits remains fine
- Branden: Great. Put a comment about that
- Tyler: Question about the ID. You used "opaque" about it. Could you explain that? Why do we want that?
- Leon: I'm saying that the Ethernet MAC doesn't inspect or interpret this value. The motivation here is that I was building an Ethernet HIL for time synchronization originally (1588 PTP), with timestamped frames. For this system, you want to correlate timestamps for sending with timestamps received. And I didn't want to have a guarantee that frames are transmitted in order. So this number lets you correlate a transmitted frame with a frame done event. The important part is that the ID is used in both the transmit function and the transmit_done callback
- Tyler: That makes sense to me
- Tyler: Also about the timestamp. Is this a certain time unit?
- Leon: No. We need this field sometimes, but there's no Ethernet standard for timestamping. Almost all MACs have a timestamping unit. And I want Tock time-sync, so this is valuable. But you always need some out-of-band communication/configuration with your network card: which clock, which byte of the frame to timestamp. Then the MACs give you some value that represents the timestamp with respect to some counter.
- Leon: So, arguably this has no relevance to this HIL and is impossible to use in a generic way. But it has to be on the data path because it needs to be applied to data packets.
- Leon: This is in case you know your exact hardware, configure it, and want timestamps.
- Branden: You could disconnect them. Have a EthernetAdapterTimestampClient which would receive `transmission_identifier` and `timestamp` only. But that seems like a lot of complexity without much gain
- Leon: Yeah, having multiple callbacks would add complexity about ordering in the client
- Branden: Okay, definitely not worth it
- Leon: Overall, I had a use case for it, which is why timestamps exist
- Branden: I think it's worth leaving. It's trivial to be "None" if no one cares
- Leon: I'm also convinced that its pretty-much the only non-packet value that needs to be on the datapath and that pretty much every MAC supports it. So timestamps are a common special case here
- Branden: Is it `transmit_frame_done` or `transmit_frame_complete`? Which one is used more in HILs?
- Branden: After looking, we switch between `done` and `complete` pretty interchangeably in SPI, I2C, Radio, etc. UART uses `transmitted` which is the worst, so as long as we don't do that, it's fine
- Leon: I'll make some notes and small updates
- Tyler: And then this is not controversial. We can approve it

## UDP
- Tyler: On TRD semantics and allow buffers. From userspace, if you allow a buffer say for configuration. I write an source address and destination address into it and share it with the kernel. Without resharing the buffer, am I allowed to update values in it?
- Leon: It is not
- Tyler: That's what I thought. That's currently what's happening in the UDP app. So that needs to be updated
- Tyler: (sharing screen) We have the Transmit syscall command here. It expects that we've previously bound to a port and it has a buffer that's been shared previously. Then we're going to check that the source address in the buffer matches the alread-bound address. This seems strange to me
- Branden: To parrot back, there's an address that you configure in some way, and it has to match something
- Tyler: Transmit expects binding to a port which specifies a source address. Then when we transmit a packet, we have to allow two buffers. One is the packet we're transmitting and one is a configuration buffer. The configuration buffer holds source and destination address
- Leon: Why is this a separate buffer? It feels like it should just be one
- Tyler: Okay, yeah this double check seemed weird to me.
- Leon: I would think that we'd want some clear separation of layers. The kernel is forwarding UDP frames, not raw frames. So I don't understand why we can't just accept UDP frames... I guess you need the IP as well. And we don't want proper IP headers because we're making a compressed version in the kernel
- Tyler: I want to find the minimum subset here to update. I could update the app to make the two buffers match. I'm considering how much needs to be changed
- Leon: Is it easy to have the app allow two buffers?
- Tyler: Yes. Previously the app updated the contents of a globally shared buffer arbitrarily. Now we can re-allow that buffer multiple times and match this.
- Branden: I would say this makes sense to me. You have data and metadata and you send both
- Tyler: All you send in C is the destination address. You already bound to a port, so you don't need that. But the configuration seems to include it anyways
- Branden: You could also ignore configuration fields that are irrelevant
- Tyler: In general, it feels weird to place things in a global buffer and assume it never changes. Feels like a foot-gun, so that's my discomfort. And this interface relies on that behavior. I think we can keep it though and fix it up
- Branden: I do think it's fine to just make it work right now and leave the interface pretty similar to how it exists now
- Leon: I don't feel confident understanding this right off-the-cuff. But I agree with not putting too much time into this. Some comments about what's weird in addition seem valuable
- Branden: Probably put those as a comment
- Tyler: I'll start with the app, because I think we can fix this all on the app side of things

