# Tock Network WG Meeting Notes

- **Date:** November 24, 2025
- **Participants:**
    - Alex Radovici
    - Tyler Potyondy
    - Branden Ghena
    - Leon Schuermann
- **Agenda:**
    1. Updates
    2. WiFi PR
    3. IPC RFC
- **References:**
    - [WiFi PR](https://github.com/tock/tock/pull/4529)
    - [IPC RFC Branch](https://github.com/tock/tock/tree/dev/ipc-rfc-docs)


## Updates
- Leon: Thread CI workflow failed first night after merge :( I think it works now though.
- Tyler: Seems to have been a network issue, either testbed or receiving server. Hasn't shown up again.
- Tyler: I'm confident now that it's actually succeeding, reading serial output after packets are received. It would probably make sense to have an intentionally failing test at some point, but this is good for now. These are the same scripts I use for release testing
- Branden: Where do we see failures? The main Tock repo or somewhere else?
- Leon: This is in the tock/tock-hardware-ci repo. If it fails it should open an issue.
- Tyler: For now, if it fails and it's a real failure, I would manually end up escalating that to an issue in the main Tock repo.
- Tyler: The tests now send packets with Thread and 15.4 and attach to a Thread server.


## WiFi PR
- https://github.com/tock/tock/pull/4529
- Branden: Overview PR for WiFi is still looking for more comments. Three big parts. 1) chips additions for RP2040 which adds a new PIO SPI option and importantly DMA support. 2) two big capsules folders, one for WiFi generally and one for the CYW43439 wifi chip. 3) Board file for the RPi Pico W separate from the RPi Pico, but mostly linked back to the original chip.
- Branden: This is pretty close to ready. Just some documentation comments from me last round. It would be useful to get additional feedback soon, as the author has time now and will have less time for changes later.
- Leon: Looked through everything except chips crate changes. In general it looks very good.
- Leon: I will say that I don't really understand much of the bus-related infrastructure. It's a bit all over the place and uses a bunch of jargon like "backplane" that means something, but I'm not sure what. I suspect it would make sense given a deep dive into specs sheets. It's in capsules though, so it won't break other things.
- Leon: A couple of things to address. It says "wifi" everywhere which I think is problematic as it's a very particular interface that's just a control plane.
- Leon: The other problem is that wifi is a protected trademark. The more generic, non-trademarked term is wireless LAN or 802.11. We should be careful about this.
- Tyler: For Thread, we can't say we're a Thread-certified thing. We probably should be part of some Thread group for using their stuff. We definitely can't use the Thread logo.
- Leon: We could always rename later though. I don't want to fight about naming.
- Leon: Other issue is a brand new way to parse structs with buffers. I think this is a third way in Tock to parse buffers and we should really have one shared solution. Zerocopy is a Rust crate to do this.
- Leon: The cyw4343 sdpcm.rs file handles parsing rust structs into buffers and vice-versa. What has been added here is a fairly clever macro to parse and encode buffers.
- Leon: So this is the third way in Tock. They're all slightly different. We're also reinventing the wheel a lot here. So, it's not actionable for this PR, but we should have common solution.
- Leon: In my opinion, we should also use https://crates.io/crates/zerocopy
- Leon: I would only consider that as a follow-up PR. Wouldn't want to hold this PR on that.
- Alex: Rough, this has proc macros and a bunch of dependencies of its own
- Alex: Question: if the structure is C aligned and packed, can't we just cast it?
- Leon: No guarantee that it's aligned or has valid types. Zerocopy checks this stuff at runtime.
- Leon: C is more forgiving, but also is _still_ unsound. Zerocopy hooks into the rust compiler, and then inserts runtime checks whenever casting between structs and buffers occurs.
- Leon: An absurd amount of effort has gone into Zerocopy and making it correct. Any amount of re-engineering Tock would do won't be as good. We don't have to use it, but we've got to compromise
- Alex: Users of Tock will need certification at some point. Zerocopy would be a big challenge there. It could be really valuable to use, but we need an alternative for certification too. Compiler vendors should be certifying this
- Leon: Didn't want to derail here, but I am sad about reinventing the wheel in Tock. Maybe I should look into whether there's a single scheme we could use throughout Tock at least. Maybe even something that optionally uses Zerocopy
- Alex: This wifi stuff is certifiable at some point. Zerocopy is not
- Leon: Apart from those two high-level comments, I think the rest is just easy-to-handle nits
- Branden: Any concerns about chips looking ahead?
- Leon: Mostly about soundness. If it's the same type of DMA unsoundness we have everywhere else, we can fix this in one go. If it's new kinds of unsoundness, we should iterate on it.
- Leon: Last note, this is really close to being merged. Hopefully this week (general agreement)


## IPC RFC
- https://github.com/tock/tock/blob/dev/ipc-rfc-docs/doc/rfcs/2025-11-22--InterProcessCommunication.md
- Leon: How do you want us to leave feedback? I made some changes directly for nits.
- Branden: I think ephemeral messages is fine for now. Eventually we'll have a PR with real comments and I'm happy to bring up longer discussions there. Small comments I'll just address.
- Branden: Mostly looking at goals and non-goals. As I've been writing the initial use-case example, I'm coming up with more things we need to handle. For example, when you wait on an asynchronous mailbox and a service process dies, you'd never get a notification.
- Branden: Kernel changes seem pretty straightforward for implementing the proposed mechanisms. ProcessID fits our needs well. Also, we can re-use the "callback on context switch" infrastructure for process state change callbacks.
- Leon: One larger comment, there's a paragraph that unnecessarily constrains us as a client-server model. That seems unnecessary, as that can happen in descriptions for mechanisms. And also its insufficient for some services. Asynchronous Mailbox would be out-of-scope given that paragraph. So let's re-write or delete it entirely.
- Branden: Sounds reasonable. May delete or rewrite that
- Leon: No other complaints yet. I'll try to send or commit comments
- Branden: Shared memory is the next thing to work on. I could draft that to get comments from you all in a future meeting.
- Leon: I'm worried about building a perfect system on paper before implementing and seeing what the issues are. The more we architect without implementation experience, the bigger those issues are when we find them.
- Leon: So instead we could do implementation work first. We want to not get stuck in design forever
- Branden: I agree with all that. We could have a system that's usable and useful without shared memory.
- Branden: When does the document get a PR? We could have a PR with TBD still.
- Leon: I don't see an argument either way here.
- Branden: Okay, I think a draft PR with this, after Network WG comments, with TBDs still in place would be fine. That can go in parallel to actual implementation work.
- Leon: Yes. Shared memory is complex enough that it's going to need thoughts from other and could get stuck in design for a while.
- Tyler: Useful to have this actually written down. And I'm excited to see actual implementations.
- Tyler: I think we should freeze the RFC and have new updates as changes take place. See the progression over time. Useful for working group output so others can refer to it.
