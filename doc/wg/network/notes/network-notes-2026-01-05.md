# Tock Network WG Meeting Notes

- **Date:** January 05, 2026
- **Participants:**
    - Branden Ghena
    - Leon Schuermann
- **Agenda:**
    1. Updates
    2. IPC Updates
- **References:**
    - [IPC RFC](https://github.com/tock/tock/tree/dev/ipc-rfc-docs)
    - [IPC Registry Work](https://github.com/tock/tock/tree/dev/ipc-registry)
    - [IPC Libtock-C Tests](https://github.com/tock/libtock-c/tree/dev/ipc-support)


## Updates
- Leon: DMASlice PR. https://github.com/tock/tock/pull/4702
- Leon: Somewhat relevant to networking. Open for eyes. Something helpful would be judging whether the documentation and design of interfaces makes sense. I think they're good, allowing things to change under the hood as necessary.


## IPC Updates
### IPC Registry Capsules
 * Branden: Implemented two IPC Registries: one based on a provided string name and one based on TBF header package name. Pretty small: each is around 200 lines of code
 * Branden: Tested with libtock-c applications and everything seems to work. Succeeds and fails as expected. Gets a ProcessID that matches the correct service ProcessID. In practice, ProcessIDs start at 0 and are sequential from there. It's annoying that there isn't an "invalid" process ID.
 * Leon: I think we had discussions about this in the past. We might make them strictly non-zero, which could help Rust do niche-filling in the kernel.
 * Branden: Question for group: Added callback for "new service registration". Triggers whenever a registration succeeds. Upcalls to _all_ applications, but the upcalls are dropped by any application that didn't subscribe. Doesn't provide any information about _which_ service registered. Johnathan proposed this.
 * Branden: Thoughts on this new callback? Any concern about leaking information? Should apps need to use a command to sign up for callbacks?
 * Leon: Seems fine and useful. Great that Johnathan thought about it. Just slightly worried about what our threat model is about information leaking, but I think this is fine. They'll know something else is running on the system.
 * Leon: Scheduling upcalls for everyone should be fine.
 * Branden: It _does_ put something in the queue for each application temporarily. Until the kernel goes to service the tasks and drops it.
 * Leon: General Tock problem that you can schedule multiple upcalls per capsule without limit. Ideally we should solve that for everyone and make it impossible for a capsule to exhaust the upcall queue. Tock 3.0 problem, as it would be a pretty decent change to semantics.
 * Branden: Okay, that's a concern for others then, not here in IPC.
 * Leon: Even if we ignore queue overflow, still enqueues work for the receiver, which is either all-or-nothing.
 * Branden: Yes, but they can just disable new registration callbacks. Discovery will still work and could be attempted periodically.
 * Leon: Choice between getting a direct notification and potentially having denial-of-service, or polling. Seems good. Lets applications choose their threat model. Losing direct notifications just loses efficiency.
 * Leon: Interesting problem, as we haven't seen application directly affect each other in this way before. Much worse that it's about resource consumption, and not just timing.
 * Leon: Something we NEED to fix, is ensuring that this doesn't enqueue a callback if the application has never subscribed.
 * Branden: There's something there because of wait-for. We put something in the queue in case a wait-for happens before the task disappears.
 * Leon: It's pretty bad design. Either clogs queue or is a race condition. Should be fixed, but I'm not sure how invasive it would be.
 * Branden: Rough because you always command and then wait-for, so you always need to get callbacks from the past.
 * Leon: This is rough. Maybe we need a 3.0 sooner rather than later.
 * Branden: Upcalls do need a redesign. They return results now and we just ignore them everywhere
 * Branden: Any concerns about duplication for two implementations? They're pretty small, and I'm not sure there's a great way to de-duplicate.
 * Leon: Eh. Not really a concern for 200 lines.
### Next Steps
 * Leon: Would it be good to assign tasks for IPC? I’d be interested in building an IPC transport. I'm most interested in Async with allowlist.
 * Branden: I think you're welcome to build the transport. The registries exist so you can test it (or just hardcode a process ID), and the documentation has the broad strokes of what it should look like.
 * Branden: I think the minimum bar for merging into master is the registry + synchronous mailbox. Async mailbox could come later. Memory sharing still needs to be fleshed out before implementation and could definitely come later. But they all have to be implemented so pick whatever.
 * Branden: Mostly self-contained. Challenge is interaction with process state-change callback which needs to be added to kernel, but I think that's straightforward to do.
 * Leon: No promise on timeline, but I'll play around with it.

