# Tock Network WG Meeting Notes

- **Date:** April 01, 2024
- **Participants:**
    - Alex Radovici
    - Amalia Simion
    - Branden Ghena
    - Leon Schuermann
    - Tyler Potyondy
- **Agenda**
    1. Updates
    2. PacketBuffer Proof-of-Concept
    3. OpenThread Update
- **References:**
    - [PacketBuffer Integration Work](https://github.com/CAmi307/tock/tree/dev/pb-integration)
    - [PacketBuffer Implementation](https://github.com/CAmi307/tock/blob/dev/pb-integration/kernel/src/utilities/packet_buffer.rs)
    - [#3933](https://github.com/tock/tock/pull/3933)
    - [#3940](https://github.com/tock/tock/pull/3940)
    - [Libtock-C #380](https://github.com/tock/libtock-c/pull/380)


## Updates
### Amalia Introduction
- Alex: Introduction, Amalia is doing the packet buffer implementation with Leon. Working on project as part of a graduation project.
### Encapsulated Functions
- Leon: Working on encapsulated functions a bunch. It seems to be pretty-much working at this point. One example we may want to tackle is porting some C network stack using that framework.
- Branden: Could do OpenThread so you get a comparison
- Leon: OpenThread is VERY complex. Probably too big of a thing to start with
- Alex: I'm very interested in porting the CAN stack with this. If you could give us access.
- Leon: There have been a lot of changes to it previously, so it wasn't made public. But it's pretty decently stable at this point.
- Alex: Well we could get started on CAN stuff on our own. Very interested. We could do the effort and send you questions if needed. No tutorial required or anything
### OpenThread
- Tyler: Some outstanding PRs in libtock-c and the kernel itself. Would be good to add to agenda today.


## PacketBuffer Proof-of-Concept
- Leon: Amalia and I have been meeting for last several weeks. Actually porting the UART subsystem and console driver over to a PacketBuffer implementation.
- Leon: Porting started pretty smoothly, but we realized there were some conceptual issues. One thing that was hard in Tock was the idea of splitting a buffer over multiple separate buffers, effectively a linked-list of buffers. So we removed that for now. We still believe it could be supported with the current design, but removing it simplified things. So we have a simple linear buffer that can be annotated with headroom or tailroom so things can be pre-pended or appended respectively. The sizes of head/tail room are part of the types, so that there is enough space should be guaranteed at compile-time.
- Leon: Amalia has ported most of the UART HIL transmit path and callback path, changing types from taking a static mut buffer to taking a PacketBuffer. Right now we always annotated zero bytes for headroom or tailroom.
- Leon: One additional weakness in the original design. The way we composed buffers was with a generic type wrapper that encoded the head/tail room. To be compatible with different implementations, we might need to reserve more head/tail room than is required for a given device. Those annotations are actually the MINIMUM amount available, and the underlying type could have more space. So both the wrapper and the underlying buffers store head/tail room numbers, which could differ, but the underlying buffer will always be greater than or equal to the wrapper.
- Leon: So the problem: we store this in a struct and the struct has a reference to the actual buffer. The issue was that the buffer was one memory allocation and the struct was a second memory allocation. This changed the premise of how we construct the types in Tock. So you needed a TakeCell for the buffer and a second TakeCell for the struct.
- Branden: The struct holds a reference and not the actual buffer?
- Leon: Yes, because the size of the actual buffer is unknown. We're using traits
- Leon: So, what we do is store the struct values in a fixed offset in the beginning of the slice, making that part inaccessible to users. These are the actual head/tail room, in addition to the wrapper. So that means we have one allocation of space, just with a little extra space.
- Leon: The most important bit here is that what we have now actually works.
- Branden: This is similar to the C idea of having a struct with a zero-length array at the end. And then just increase the allocation size for array data. But rust isn't okay with that.
- Leon: Yeah, that's right.
- Leon: There is a runtime penalty for using this design. We're constructing this type from a Rust byte slice, which has no alignment rules. So when we store a usize at offset zero, that might not be 4-byte aligned. So our code has to manually combine the bytes back together into a 4-byte type. This is effectively an unaligned read, which slows down runtime a little, but seems probably okay. We could probably do some things to force alignment if it was required.
- Leon: The nice thing about the current design: the slice-based approach is just one implementation, and we could have multiple implementations that all worked simultaneously and wouldn't affect downstream users. The interface remains stable
- Tyler: A few questions. Is the code for this in a branch somewhere?
- Leon: Yes, a branch in Amalia's code.  https://github.com/CAmi307/tock/tree/dev/pb-integration
- Tyler: Are you just trying to iron out the type system details now, or is there a mockup you're working with?
- Leon: Amalia has been working with a proper partial-integration of this into Tock. This is going to be an iterative design, where we should go back to the interface and add convenience methods wherever useful.
- Branden: If the head/tail room was non-zero, are there methods to work with that?
- Leon: Yes? I believe they all exist, although they likely aren't tested at all. Here's a link to the actual packet-buffer file, with the interface right at the top: https://github.com/CAmi307/tock/blob/dev/pb-integration/kernel/src/utilities/packet_buffer.rs
- Branden: Okay, I think this is a "complete" type system interface, but likely not complete functionality interface
- Leon: Yes. That will need work, but isn't overly concerning.
- Leon: We also need to reason about safety. I'm pretty confident it's sound, but its worth an overview
- Leon: Finally, I believe Amalia has this _actually working_ on the UART code-path now, where printing still works. It's not giving any advantages yet, but it's not breaking anything.
- Branden: So hypothetically we're at a point where we could get metrics, like size increase or latency perhaps
- Amalia: I finished the implementation with zero head room and tailroom and it seems to be working now. I didn't find any issues yet. No overly large latency. No runtime errors. Right now I'm trying to get the point where we would prepend something in the buffer, and I'm running into weird compilation issues. So that's the current status.
- Leon: I'm very excited to continue work on this
- Branden: How hard was the effort of changing to this? Was it mechanical or a big redesign, or somewhere in the middle?
- Amalia: For me it was huge, because I was also understanding Tock at the same time. It's not quite mechanical, but there's a pretty clear flow for adding this.
- Leon: I haven't been doing the integration work myself. From looking though, there is a large buy-in cost to making the changes at all. But the changes themselves are rather mechanical. The hard part was figuring out how to transition the Tock code to this, but the diff isn't actually big. You're replacing parameter/callback types, and you have some magical incantations you need to know about for converting types.
- Branden: And having a working example would definitely help future efforts to understand what the transition would look like
- Branden: Future plans on UART side -- move forward into work on prepending things to implement console virtualization?
- Amalia: Yes, that's correct.
- Branden: To some degree, diverges from this work. Your goals will become somewhat different from Leon's goals.
- Amalia: At the same time, I'm working on an host-side application that performs filtering based on headers introduced with the packet buffer.
- Branden: This is awesome, long standing issue.
- Branden: To Leon, what's the next step as far as we're concerned for the PacketBuffer stuff?
- Leon: I do agree that Amalia and I will diverge at some point, but I don't think we're there yet. The interface we currently have is limited, and as Amalia needs things I can add them. The experience overall will guide what is needed in an interface. Thinking about the APIs for prepending and appending. So definitely still a lot of collaboration.


## OpenThread Update
- Tyler: Biggest thing I want to ask for is some eyes on some PRs
### PDSU Offset Fix
- https://github.com/tock/tock/pull/3933
- Tyler: Pretty minor change to update the offset used for some metadata.
- Tyler: Amit had asked why we were even looking at the offset at this layer and why it wasn't lower in the stack. For further context: the PSDU is an artifact of the RF233 on the IMIX. It needed an extra two bytes for the SPI header when sending to the radio module. To keep the HIL consistent, we needed two extra bytes in the offset. We are taking this as an opportunity to avoid having this bubble all the way up the stack. We don't have enough expertise to definitively say whether this is right.
- Branden: Ready for some eyes?
- Tyler: Yes, ready to go! Thread network works with these 
- Branden: Skimmed this PR already, was not sure about some things, but will take a look!
- Tyler: Indeed this requires some niche knowledge.
- Tyler: There's an extra two bytes at the start of the nRF and RF233 radios. It's a byte that's needed by the radio but not part of the packet. We definitely shouldn't pass that one byte up to MAC layer when receiving.
- Branden: So, whenever you send and receive a packet, there is an additional offset of two bytes?
- Tyler: Yes, on all boards. It's a part of the radio HIL.
- Branden: Does the nRF need those two bytes at the front, or does it throw them away?
- Branden: Probably what Amit and Brad are pushing back on -- why was this exposed to userspace at all?
- Tyler: Tricky -- it's there on the send side, not on the receive side.
- Tyler: There's also two bytes versus three bytes. We already have this idea that we have metadata on packets passed to userland. It was the offset to the payload and the payload length. And we just put those bytes into the PDSU section, since we had two bytes of space. So I added a third byte of metadata, which holds the MIC length for crypto stuff.
- Branden: So, this PR is only about having three bytes of metadata on receive packets?
- Tyler: Originally yes, until Amit's comment about the presence of the PDSU on the MAC layer. This seems like a mistake, because the PSDU offset is entirely radio-specific, not at all relevant for the MAC layer. As far as I can tell, it's not used for anything.
- Tyler: Comments on the PR can help understand this. Not much code that changes. If Branden can make some comments, this might unblock the discussion.
### Receive Encrypted Packets
- https://github.com/tock/tock/pull/3940
- Tyler: There's a trait called RXClient which is used throughout the 15.4 stack. You set a receive client, which is then passed packets. We have this 6LoWPAN bug that was an indexing error: one of the PRs that did not drop packets that we couldn't decrypt, when reading the offset on those packets that we couldn't decrypt, we read garbage from the offset field. This adds two traits suggested by Brad, that split this up into a "Raw receive".
- Tyler: Brad did say this is reasonable, but did not yet approve it.
### Thread Userspace
- Branden: Is there stuff in userspace to discuss too?
- Tyler: No. The one PR that exists https://github.com/tock/libtock-c/pull/380 has had a lot of discussion on it, so everyone here (modulo Leon, who should do testing), should not focus on that right now.
## Thread and EUI64
- Tyler: Design question there's this EUI64 thing, that's an identifier for a device. Nordic has this from the factory. It's common across radios for thread networks and Thread uses them to generate mac addresses and IP addresses. We need a way to pass the EUI64 to userspace. Currently, we grab it in main.rs and pass it as the address in the radio drivers. The question is where should we add a member to the struct that holds the full EUI64? It seems to me that the radio hil struct, the virtualmac, the userspace-mac, or userspace radio driver. To me, it seems that because this is radio-specific it makes the most sense to put it there. Was looking for thoughts.
- Leon: For the EUI64, is the cell type still important?
- Tyler: Yeah, I need an immutable way to store those 8 bytes.
- Leon: Immutable things can just go in the struct directly. Only mutable things need to be in cells.
- Branden: Where is the MAC address stored now?
- Tyler: A lot of places. The virtual mac plus some other places. Each client has a MAC address.
- Branden: Is our goal to present a radio that has multiple MAC addresses?
- Tyler: No for thread. It's a 15.4 6lowpan idea. And users can change MAC addresses right now, which we don't want for EUI64
- Branden: That makes sense. For Thread, we can just present one radio, but then use higher-level abstractions like UDP ports to separate data for different applications. Each application doesn't need to pretend to be an entire radio.
- Branden: I'd make a new syscall driver to present the EUI64 information. It seems like it's outside of the radio driver's purview.
- Tyler: For imix, it grabs the serial number and calculates the address based on that.

