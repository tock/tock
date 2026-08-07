# Tock Network WG Meeting Notes

- **Date:** August 03, 2026
- **Participants:**
    - Branden Ghena
    - Tyler Potyondy
    - Leon Schuermann
- **Agenda:**
    1. Updates
    2. IPC PRs
    3. IPC Open Questions
    4. IPC Next Plans
- **References:**
    - [IPC capsule PR](https://github.com/tock/tock/pull/5015)
    - [IPC documentation PR](https://github.com/tock/tock/pull/5016)
    - [IPC libtock-c PR](https://github.com/tock/libtock-c/pull/580)


## Updates
- From slack from Amit: Use Claude to make BLE connections with a very simple GATT server running on nRF52840. Timing sensitive stuff can be handled in kernel. Just a proof-of-concept for now with a terrible design and architecture. Will keep iterating on this. Seems like most modern chips have a Host-Controller Interface split that would let us share high-level part of implementation between chips.
- From slack from Amit: Messing around with LoRa with a Seeed Studio board.


## IPC PRs
  * Tock PR with capsules: https://github.com/tock/tock/pull/5015
     * Overview of the capsules that were created
         * IPC Identifier
         * IPC Registry Package Name
         * IPC Registry String Name
         * IPC Relay Request

 * Tock PR with documentation: https://github.com/tock/tock/pull/5016
     * This is most a refactored version of the prior document moved from RFC to TRD

 * Libtock-c PR with drivers and apps: https://github.com/tock/libtock-c/pull/580
     * For each capsule I made a client and server app to test them
     * I also made a "round robin" test for Relay Request where A send to B sends to C back to A

 * Any questions from the group
     * Tyler: libtock-rs support. Are there any potential issues with this design?
     * Branden: Most of it is pretty straightforward, just normal allows and upcalls. One big issue is StreamingProcessSlice which is not in libtock-rs and also maybe difficult.
     * Leon: Libtock-rs can probably do static buffer StreamingProcessSlice.
     * Tyler: But sending data back and forth is more straightforward for inter-language communication too.
     * Branden: Yeah. Just need a way to realize Rust types out of arbitrary bytes.
     * Leon: Transmuting bytes is easy with the ZeroCopy library. We can use that in userspace fine.
     * Leon: Did you run out of space for u64 identifiers anywhere? In the syscalls maybe?
     * Branden: Once in upcalls. Normally when you receive a message you get the length and the IPC ID of who sent the message. But if the message is too big to fit, we partially copy it up to the buffer length, send an error, and then we only get 64 more bits, so we send the IPC ID. You have to implicitly determine, based on the ESIZE error, that you have a full buffer of data. I'd prefer that to be explicit, but it's fine.
     * Leon: Could return the size and the receiver could tell the server what the maximum size. That's a Linux-y idea. You could imagine a very complex Tock app to doing that. Or responding: I've got the first 50 bytes, now send me the next chunk of data
     * Branden: Yeah, both would work in this system.


## IPC Open Questions
 * Do you think these PRs need to be split into smaller chunks?
     * Branden: Was a little concerned with bigness, but didn't want to flood the zone with tiny PRs either.
     * Leon: With small PRs, you never see the big, overall structure. So you'd get approvals faster, but it would be harder for people to reason about.
     * Branden: That makes sense. It's also not a rush.
     * Tyler: It's also self-contained, and doesn't remove prior IPC structure.
 * Should there be a way for an app to get its own IPC Identifier?
     * Leon: Could be nice for debugging
     * Branden: Good point, although in practice if you list processes, their process number is also their IPC ID unless ShortIDs are actually assigned on the platform.
     * Leon: What about app A tells app B to send a message to app C? No, then A would know about C
     * Branden: I think I might avoid it for now. It's trivial to implement in Registry capsules if someone needs it. And I'm a little wary of leaking it to apps, although I don't have a reason why
     * Leon: Not worried about leakage. It's forgable, not authenticated. Pretending that it's not something you could guess is the wrong choice as it might trick a human.
     * Leon: On the other hand, exposing an API makes people think they should use it. Might confuse people, and that's a bigger concern.
 * Does the Thread Network Server use case make sense? Is it missing anything big?
     * Tyler: Makes sense to me


## IPC Next Plans
 * Round-robin iterator for Grant space
     * Already have a design that works for this
     * Should I add a generalized RotatableIterator to Tock? Or just focus on the grant iteration?
     * Leon: Any idea what the overhead of these are?
     * Branden: No. Not really. I tried to compare two alternative iterator approaches using cargo bench, but the two implementations were both essentially zero cycles always. I can't tell if that's actually true or if there was a bug with my test setup. I also tried to look at the assembly, but it's big and I can't make heads or tails of it.
     * Leon: Okay, usually they should be performant, but there are some edge cases to think about.
 * Add other libtock-c examples?
     * ROT13 service would be easy
     * Thread would be great here
         * Tyler: That would be doable. There isn't anything in libtock-c right now. Need to think through message interface
         * Branden: Could be a good undergraduate task
     * Don't really want to touch tutorials if I don't have to
 * IPC Relay Message capsule implementation
     * I think this is pretty much ready. I have some notes in the IPC slides about what the syscalls should generally look like. https://docs.google.com/presentation/d/13mYERv0iKvBPOsu52jsd4jjZalbd_KXHbwDGdt8qgRw/edit?slide=id.g381d6984e74_0_0#slide=id.g381d6984e74_0_0
 * Design of IPC Share system
     * Branden: Hoping to discuss this at next meeting. This is going to be a long time out from being implemented though.

