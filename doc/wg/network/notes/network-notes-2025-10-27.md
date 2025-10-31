# Tock Network WG Meeting Notes

- **Date:** October 27, 2025
- **Participants:**
    - Alex Radovici
    - Tyler Potyondy
    - Branden Ghena
    - Leon Schuermann
- **Agenda:**
    1. Updates
    2. WiFi
    3. IPC
- **References:**
    - [WiFi PR](https://github.com/tock/tock/pull/4529)


## Updates
### Treadmill
 * Tyler: A few MS students who are interested in Tock and I gave them a project integrating OpenThread tests from certification. These are python scripts that interact with an nRF board attached. They check if a device can attach and send it some messages. It would be more robust than our existing tests. Goal would be to integrate with Treadmill.
 * Tyler: There are also OpenThread tests for Treadmill now, there's a PR but I don't think those were merged.
 * Leon: I've been checked out and I'm not sure of the status there. I'll check with Amit about it.
 * Tyler: Okay, I'll push on that too. It would be great to have Treadmill test OpenThread
### CI
 * Leon: We have a nightly workflow for Tock that's been failing for months now. It also tries to post an issue, but that _also_ fails. So we should 1) fix that and 2) add support for Treadmill to avoid silent failures
 * Leon: Not sure of solution right now
 * Branden: It's weird that nightly fails but merge PRs don't
 * Leon: Nightly is the only thing that runs MacOS tests. So those are _really_ failing

## WiFi
 * https://github.com/tock/tock/pull/4529
 * Alex: We have abstraction layer. There's a bug in the controller we're trying to iron out. Still using PIO to talk to it with a deferred call to fake the async part. The interrupt for the controller arrives before the deferred call is handled.
 * Alex: We're doing synchronous writes, but still getting an interrupt, not sure why.
 * Alex: So, we need to change the code and fix that. This problem will also go away once we get DMA working.
 * Alex: So this will be a hack for now, but we think we can get it working as-is.
 * Alex: We're also fighting the SDIO interface. That's a work in progress. It's not working like we expected. That would give us an additional WiFi board.
 * Branden: Awesome. It would still be helpful to have a push to the PR so we can start reviewing structure and design, even before we get to whether it actually works or not.

## IPC
 * Branden: Overview of recent changes: after our discussions I broken the Mailbox into two capsules, a sync mailbox and an async mailbox. Sync mailbox would be client request -> server response. There would be a guaranteed buffer sitting around for the server response, so the server will always be able to respond and will never have to wait on a client.
 * Branden: The Async would probably be Server->Client (but that's not required). It would use StreamingProcessSlice to let Server(s) add a message for a client. That can fill up, but would be an immediate failure and the Server could just drop the message it had for the client (and the StreamingProcessSlice can note that something was dropped). That also has an access-control-list for processes to choose which application(s) can send them async messages, so you could disable messages that are spammy or less important.
 * Leon: This is what we've been moving towards. Seems to solve use cases
 * Leon: Two thoughts: I talked about the general journey and struggle on IPC with a couple people at SOSP. They were sympathetic to the general approach of dedicated mechanisms for use cases with coordinated discovery mechanism. The feedback was that no one will ever implement their own mechanism. Which is maybe wrong given Alex's "share a number" needs.
 * Branden: I do disagree as well. I think there will be like 3 trivial mechanisms people add.
 * Leon: They generally agree that it's hard to have a single interface that covers all bases.
 * Leon: Item 2 is what's the next step. We should translate into a more detailed specification or a prototype implementation
 * Branden: Next step: specification first. Want to have people be able to poke holes into it. Building prototype for everything except shared memory will be easy. Just copying, and discovery. Updates to shared memory will require MPU updates which will be miserable. So that will be last.
 * Leon: Discovery might be messy because we may need to tweak kernel interfaces.
 * Branden: Not a good feel for discovery.
 * Leon: Could mock it out initially.
 * Branden: Going to push for next time we meet (Nov 10th): written documentation that we can look at for making a PR describing the mechanisms apart from shared memory.

