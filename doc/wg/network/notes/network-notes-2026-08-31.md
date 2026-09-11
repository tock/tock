# Tock Network WG Meeting Notes

- **Date:** August 31, 2026
- **Participants:**
    - Branden Ghena
    - Leon Schuermann
    - Marshall Clyburn
- **Agenda:**
    1. Updates
    2. Userspace Services
    3. IPC
- **References:**
    - [Userspace Services PR](https://github.com/tock/tock/pull/4869)
    - [IPC Shared Memory Design Doc](https://docs.google.com/presentation/d/10t5i1Glh1Ty86Eqo-VoCDSikgMftE_Y-NXnAaWRS2cM/edit?slide=id.g3f792985ddc_0_0#slide=id.g3f792985ddc_0_0)


## Updates
- None


## Userspace Services
* https://github.com/tock/tock/pull/4869
* Marshall: Going through wording changes in docs, and maybe some missing info in places. Wanted to discuss some on this call.
* Leon: Diagrams are very helpful
* Marshall: I'm just going to walk through the comments on the README from top to bottom and discuss the non-trivial ones.
* Marshall: In the README docs, when I wrote "available for the entire platform" I was trying to suggest that it's usable in a variety of ways. My use case is userspace app calling into the kernel, which then calls into the userspace service.
* Branden: Yeah, I was trying to clarify that "anything that can use a HIL" can use this
* Leon: What are the implications of this for our threat model and kernel-level guarantees? Diverges from previous distinctions about what things we trust in the system. It's a complex thing to think through.
* Branden: Just having the capability doesn't mean people have to use it.
* Leon: Yes. But we do want to think through what the guarantees of the system are. Potential havoc that userspace service could cause for your system. I think that should be in the README. We want to make clear that it's not a highly trusted thing. Putting it in capsules/system also seems to imply trust to me. I'll comment those on the PR too
* Marshall: Back to README, I'll document sending messages to service before it started. It'll return an error, but I should check that the error is meaningful. There's no promise that it's distinct from other errors the HIL might return.
* Branden: It's hard because apps don't necessarily know they're using this, so they don't know to try again and that communication will work later. Definitely should have some thought
* Marshall: For buffer passing, the userspace app has a separate rw allow for each buffer passing through the HIL. I'll document that
* Marshall: For error reporting, it's only async, not sync errors from userspace.
* Branden: That's a problem though, right?
* Leon: So there are two error paths for any operation. If you start the operation, it might synchronous return an error. Or it might start the operation, find something is wrong, and then return the error asynchronously later. It's possible some HILs assume certain checks can be synchronous but with a userspace service it must be asynchronous. For example, some HILs might expect returning synchronously if hardware is disabled, but userspace services won't know that until the logic occurs in the app. So all errors have to be returned in the async callback. It's possible this breaks some HILs, but really the HIL should be updated to allow that action. The contract for the HIL should allow that.
* Marshall: Some synch error handling logic, like wrong buffer sizes, goes in the Service Interface, like digest.rs.
* Branden: GPIO Async had a similar issue, where the original GPIO HIL had a lot of synchronous assumptions and had to be revised.
* Leon: I think this is a clear limitation of userspace services that should be documented, although not a fatal problem at all. More broadly in the Tock project we should revisit the rules by which HILs are written to make them fully asynchronous. Which kind of sucks, but is important for more complicated use cases.
* Marshall: So the answer to this Github comment is that the Service Interface can return some trivial synchronous errors, but the userspace service only returns asynchronous errors.
* Branden: Oof, here's another serializer/deserializer implementation
* Leon: Another use case for Zerocopy. We have several of these throughout Tock now: network interface, RISC-V, some others
* Branden: Is there anything you want us to direct our attention to in this PR that isn't the README?
* Marshall: No particular part that needs special attention right now.
* Leon: We do want to get the architecture right, but the actual implementation isn't the most important thing. We should make it clear it's not something that we promote as a core promised-working part of tock, but just as another capsule you can use. And it's a cool thing to add


## IPC
* Branden: Want PR reviews IPC from Leon, but that's paused until release so it's not a rush.
* Branden: For a brief "awareness" discussion, I want to point to some open questions that I'm chewing on for Shared Memory in the doc here: https://docs.google.com/presentation/d/10t5i1Glh1Ty86Eqo-VoCDSikgMftE_Y-NXnAaWRS2cM/edit?slide=id.g3f792985ddc_0_0#slide=id.g3f792985ddc_0_0
* Branden: When it comes to allowed memory, we'll need to ensure that no allows are active in a memory chunk that changes accessibility. I made a proof of concept to iterate all allows for all drivers for a process to check that they don't overlap.
* Leon: I'd consider not permitting memory allows within shared memory, or at least within memory that's not originally your own.
* Branden: I'd really like to if possible. Particularly, I considered that losing access to a chunk of your original memory is pretty symmetric to losing access to. A chunk of memory that's not your own.
* Leon: Why would we lose access?
* Branden: That's one implementation of exclusive access to shared memory. Disable from original app and enable in new app. Of course it adds a further issue of having non-contiguous memory spaces within an app, which I suspect breaks assumptions in Tock
* Leon: We could instead disable apps that have shared memory until it returns to them.
* Branden: Yes, that's another implementation, although I'd really like to avoid it if possible.
* Leon: We'll need to think about how much is platform-specific versus not platform-specific. Concerned that many MPU/PMP/MMU implementations differ enough that we're not going to be able to generalize at all. Most of this will need to be platform specific is my guess
* Branden: I'm hoping that the capability scales, but that the interface for sharing is the same across platforms. We'll see
* Branden: Overall, I just want people aware of these problems in this space and thinking about them. I'd say once a week I suddenly realize a new issue with shared memory and add it to this doc. Let me know if you think of one

