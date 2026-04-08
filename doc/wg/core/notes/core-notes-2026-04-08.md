# Tock Meeting Notes 2026-03-18

## Attendees
 - Branden Ghena
 - Leon Schuermann
 - Johnathan Van Why
 - Amit Levy
 - Pat Pannuto


## Updates
### Tock Registers
 * Johnathan: Posted the PR for the revised registers system. https://github.com/tock/tock-registers/pull/11
 * Johnathan: The companion PR in the main Tock repo has some comments: https://github.com/tock/tock/pull/4772
 * Johnathan: The goal was to review the Registers design, then look at Tock, but everyone is doing the reverse
 * Johnathan: Also I want to merge https://github.com/tock/tock-registers/pull/10 to build on top of
 * Johnathan: I'll also be pulling stuff out of PR #11 and moving them into their own PRs for small non-controversial stuff
### STM32WLE5xx PR
 * https://github.com/tock/tock/pull/4695
 * Branden: On behalf of Tyler: this PR implements the STM32WLE5xx chip. It's self contained and I've reviewed it and I want to push it forward.
 * Branden: It has one weird hack/quirk to it. So the value of the STM32WLE5xx is that it has a LoRa radio literally shoved inside the chip die, connected over an internal SPI bus. But the interrupts from that radio go straight to the NVIC and stay high until you service them. Servicing takes time in Tock, as it goes all the way to userspace, but the interrupt staying high would trap us in a loop of pending interrupts. So this code automatically disables that interrupt when it occurs until userspace deals with it.
 * Amit: Merged!
### Supply Chain Security Work
 * Amit: On supply-chain security work as part of a grant, we're categorizing Tock code into tiers, with some kind of label about how sensitive it is and how much scrutiny and verification is necessary for changes to that code. That's in progress, and eventually we hope to start gathering feedback on that.

## ProcessID Uniqueness
 * https://github.com/tock/tock/pull/4777
 * Leon: Opened RFC to bring to our attention to a ProcessID issue. IPC is interested in using ProcessID to link discovery and communication as a handle. It's a natural fit, but right now the implementation is a usize that could roll over with enough restarts and make it a non-unique handle. That made me think about other places we assume uniqueness, so I made this PR to circumvent these issues by changing ProcessID to a u64 which is sufficiently large to never roll over.
 * Leon: More generally, we want to decide if it's an issue that ProcessID values could be recycled. And what to do about it.
 * Amit: Leon and I went through a few cases that currently exist where ProcessIDs are used for security/safety. We determined that currently all the cases we looked at don't have actual issues, although we're very close to having an issue. For example, whether a grant is accessible or not, and whether a capsule can get a Rust reference to uninitialized memory relies partially on ProcessID but also on other stuff so it's safe.
 * Amit: We also looked at callbacks, which I think isn't safe? It's hard to fully determine and keep this in your head. Under certain setups you could issue a callback with sensitive information, and it would go to the wrong process if the original process has died and the ProcessID was recycled. And there's no way for the capsule to be aware or deal with that.
 * Amit: The summary is that recycling ProcessID over the lifetime of a boot of Tock is problematic. It's either actually vulnerable, or else the non-vulnerableness is dependent on other things that aren't guaranteed
 * Amit: So, what do we do about it? The simplest option might be to just not allow the ProcessID to wrap around. So it can only monotonically increase and if it reaches the max then you panic and restart. The u64 proposal is big enough that then you can't imagine a realistic scenario which would cause the panic.
 * Branden: I like panicking at max, but then you can come up with scenarios where a process could cause the kernel to panic, which feels very ungood.
 * Amit: Right. And a u64 pushes that hundreds of years away. The cost being an extra word of storage on 32-bit systems
 * Leon: Johnathan asked about the costs. I did look at benchmarks, and this change to add a u64 does bump code size by 500 bytes per board right now. That's surprising to me, so I'm going to need to look into that more
 * Amit: We could also parameterize this, so a board can choose the bit width of ProcessIDs defaulted to u64. Also, what we're gaining is somewhat significant in that we can trust ProcessID as being unique. So they could be used as runtime capabilities for purposes of allowing or denying some kind of behavior (send a message, access something). And that could save code size.
 * Leon: I'm not entirely sure if it's possible to offset cost in that way.
 * Leon: I also looked into how Linux solves this, as it recycles IDs. But it never uses process IDs as capabilities, and instead uses a dynamically created resource to refer to the process.
 * Johnathan: There are race conditions with killing processes in Linux
 * Leon: True, but not an issue within the kernel, only for userspace.
 * Leon: I also found an interesting statement Brad said, where he's worried about adding another variable in Tock which could have security implications. And I'm saying that ProcessID already has security implications
 * Leon: I'll try fixing up the u64 implementation and see if some simple stuff reduces that code size cost
 * Branden: Another cost is that if you want to expose ProcessID to userspace, like IPC does, you need two registers instead of one. Not the end of the world, but it is a cost
 * Amit: We could have a descriptor table in the capsule that maps a u32 to a ProcessID.
 * Amit: Is the use of a ProcessID a capability?
 * Leon: No. It's a handle. It's somewhat forgable. The kernel just tries to forward a message to some process based on the untrusted value. When you authenticate an application, this is the handle that we provide you for referring to that process instance.
 * Johnathan: If you have an app that wants to talk to a service, it knows the service's AppID, but not ProcessID. So it asks discovery to give it a ProcessID that corresponds to the AppID it knows. Then future calls only use the ProcessID. So if ProcessID is non-unique then you could have impersonation. Ultimately the goal is for a process to know the AppID that it's talking to, but AppIDs don't account for state management of apps crashing while you talk to it, so ProcessID is somewhat of a session handle.
 * Amit: Okay, so it's somewhat of a "bad" capability. We don't have a table of exactly which processes can be communicated with
 * Branden: Right. We discuss that in the docs, but don't go with it right now.
 * Amit: So this is a case where uniqueness is a valuable property. There is that additional unaccounted cost for userspace process to get/store ProcessIDs.
 * Amit: So we need to make them unique. We could leave them as u32 with no overflow, or we could make them a u64, or we could have a process-specific descriptor table.
 * Leon: Currently we assume that they're unique in places. So if we don't want to guarantee that, we need to rethink all the ways we use it.
 * Branden: I think the room agrees that uniqueness is valuable. So the question is what the cost is of doing it and what the method is.
 * Leon: We could make some ProcessID that's unique per Process Slot. So we could handle 2^32 restarts per slot in the process table. We'd just have to prevent that one process from overflowing. But reordering the process array at runtime would break that, with app loading. In fact, I think ProcessID totally breaks right now if we modify the process array.
 * Branden: We could also assign ProcessIDs differently.
 * Amit: They could be random. Which wouldn't guarantee uniqueness but would make collisions unlikely.
 * Leon: Xous does this, but with 128-bit UUIDs.
 * Amit: Or you could keep track of outstanding references to ProcessIDs and only assign one that's unused. That would make assigning expensive, but could make sure we only recycle actually recyclable ProcessIDs
 * Johnathan: What if whenever a process wants to use IPC, the first time it wants to talk to another process it does some special initialization. Then if the other process ever restarts then you restart stuff. And the kernel tracks session aliveness.
 * Amit: That only solves the problem for IPC in userspace, but not other uses of ProcessID.
 * Branden: That's pretty-much the current use of ProcessID. It represents aliveness of a session and we say "session broken" if it's no longer valid
 * Leon: Implementing some other session tracking mechanism could require more memory than this u64 change.
 * Branden: Here's my take. Use u64 for ProcessID. Leon spends an hour trying to make code size palatable and succeeds. Then IPC design will consider whether sending a u64 to userspace is too big a cost to bear, and if it is IPC figures out its own solution to it.


