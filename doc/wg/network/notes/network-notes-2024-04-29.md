# Tock Network WG Meeting Notes

- **Date:** April 29, 2024
- **Participants:**
    - Alex Radovici
    - Amalia Simion
    - Tyler Potyondy
    - Branden Ghena
    - Felix Mada
- **Agenda**
    1. Updates
    2. PacketBuffer Progress
    3. Thread/15.4 Check-in
- **References:**
    - [PacketBuffer Console Work-in-Progress](https://github.com/CAmi307/tock/tree/6a8822afdfe19f59e22ac84aaa60a4c6a7b60e61)
    - [15.4 LQI PR](https://github.com/tock/tock/pull/3972)
    - [Timer Update PR](https://github.com/tock/tock/pull/3973)
    - [Thread Tracking Issue](https://github.com/tock/tock/issues/3833)
    - [OpenThread Libtock-C PR](https://github.com/tock/libtock-c/pull/380)


## Updates
- None today


## PacketBuffer Progress
- https://github.com/CAmi307/tock/tree/6a8822afdfe19f59e22ac84aaa60a4c6a7b60e61
- https://github.com/tock/tock/compare/master...CAmi307:tock:dev/second-try-append-headers
- Amalia: Last time, we discussed adding constants for each layer in the struct for each layer. The ones above and below. We did that and it worked! We could append header and footer data in each layer as the packet moves.
- Amalia: In the last meeting with Leon, we decided to document things and do some testing for stability of the code.
- Amalia: After that's finished, the next big thing would be using the same mechanism for the receiver in Tock. Also some mechanism that would send the received message to the targeted process. You could imagine a user typing a message intended for one process, but not others. Some host-side application would append headers when sending that data, which Tock would decode.
- Branden: What data is being appended right now?
- Amalia: For the moment, the appended data is the process ID of the application. I'll probably need more at some point. As the kernel and applications need to be disambiguated.
- Branden: Something we had discussed when console change was first discussed. It would be nice if the data in the header/footer was semi-human-readable for people who aren't using the proper tool that can decipher it. For example, you might encode process IDs in ASCII instead of as raw numbers. Any thoughts on that?
- Amalia: No thoughts yet
- Branden: What's the plan for eventual upstreaming?
- Amalia: Definitely needs cleanup first. Probably moving some constants around, to board initialization and not in the components directly. Another thing that's missing at the moment is initialization for other boards, this is only on the Microbit so far.
- Amalia: The plan is definitely to upstream, but no particular timeline right now.


## Thread/15.4 Check-in
- Tyler: Two outstanding PRs. First is https://github.com/tock/tock/pull/3972
- Tyler: #3972 is the Link-Quality Indicator which Brad and Branden commented on
- Tyler: This is really the RSSI value, which is being passed through callbacks/upcalls so upper layers can use it. The upcall previously had Pan ID, source address, and destination address. I replaced Pan ID with LQI, since it can be parsed from the packet.
- Branden: My take from the discussion is that we should remove Pan ID, source address, and destination address from the upcall. If they are all trivially parsable from the packet data, we should provide helper functions that do that parsing and not send values which could possibly conflict. Then that leaves us with room to send LQI, which isn't in the packet data at all. Along the way, we should remove all the dead code that won't be necessary anymore after the update. So, I think once you make those changes, this PR should be good to go.
- Tyler: Other PR is the timer update: https://github.com/tock/tock/pull/3973
- Tyler: We don't need to discuss this, but I'll be updating the OpenThread logic based on this
- Tyler: This is necessary because we have a get-time-in-ms function, which wants increasing time since boot. But the ticks values wrap at 2^24 ticks for Nordic (about 7 minutes). And in the Timer interface now, we don't have a way of knowing when the wrap is going to occur. And OpenThread fails when the timer wraps back to zero.
- Tyler: The new driver and libtock-c updates should make this timer continually increasing instead.
- Branden: Reminder to update the tracking issue: https://github.com/tock/tock/issues/3833
- Tyler: We're also ready to start reviewing the OpenThread libtock-c PR: https://github.com/tock/libtock-c/pull/380 Brad's been giving lots of help throughout

