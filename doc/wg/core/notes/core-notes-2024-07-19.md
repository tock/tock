# Tock Meeting Notes 07/19/2024

## Attendees

- Amit Levy
- Phil Levis
- Hudson Ayers
- Leon Schuermann
- Branden Ghena
- Pat Pannuto
- Brad Campbell
- Alyssa Haroldsen



## Updates

Hudson: A 15.4 driver to libtock-rs that isn't sound, one would use dynamic allocation, so we need to add it to libtock-rs. I'll have a PR in a couple of days, so we can expose some external allocator for where we want the heap to start, for 802.15.4 and dynamic allocation. 

Amit: OpenSK patches libtock-rs, to be able to do dynamic allocation. Could be worth looking at what they did.

Hudson: I don't understand how the scoped API will work with dynamically allocated buffers. 

Leon: Seems like we will figure this out after we have dynamic allocation implemented?

Hudson: Yeah. I suspect that even your approaches for dynamic allocation is something we want Johnathan's review on.

Brad: I've looked at OpenSK and what it would mean to upstream it. Initial analysis is that USB CTAP is a relatively minor difference, just command numbers. However persistent, non-volatile storage API is very custom and not likely for us to upstream, so will have to be rethought. And I don't think we can take the OpenSK app and run it on an upstream kernel. We're going to have to figure out how to handle that.

Amit: Right now, OpenSK is carrying patches. As an intermediate, would it be reasonable for stuff that's not easy to upstream, to convert to not patches but just to separate capsules. Some of this might be straightforward. For example, they patch the kernel to add boards, rather than add crates. Certainly some stuff needs to be patches. But could the storage just be a separate stack as a capsule, that isn't necessarily upstreamed, but plays well as a module that they have.

Brad: No, because they modify memop.

Amit: Long-winded question, short answer!

Brad: Reading and writing storage isn't a problem. The issue is calling to know what storage is available is through memop.

Amit: But wouldn't that be possible to port to a system call driver? 

Brad: Yes, but that would require modifying OpenSK.

Phil: Why are we so interested in OpenSK?

Amit: We talked at TockWorld, there was agreement, we've long been in search of canonical, long-term applications that we can rally around and use as benchmarks. OpenSK seems like a good one, it's well maintained, they keep it up to date. It's representative of applications we might care about.

Amit: Update on porting Tock to CHERI. Lawrence Esswood, who has one of the ports of Tock to a CHERI architecture, is in the process of making it public. github.com/tock/tock-cheri. Basically versions of the internal port that are cleaned up from super-specific Googley stuff. I can't quite get the toolchain working. But another outcome from that breakout was to try to get rustc upstreamed to support some kind of CHERI. We met again between Microsoft and Google and others, we are planning on putting together an RFC for Rust (a "goal RFC").

Leon: Update on Tock foundation. We have hired our first employee! He should join this call. He's working on the testbed infrastructure, assigning him initial tasks. Lots of onboarding. I'll make sure he joins next week's call.


## Device Passthrough

Amit: Two major items: device passthrough and more support for building directly with cargo.

Amit: https://github.com/tock/tock/pull/4044

Amit: Alistair opened a PR a few weeks ago, proposing a mechanism to expose MMIO directly to certain processes. Says he has a bluetooth stack on the apollo3 working with this mechanism. General benefits here are clear. Why you might want to do this. This is a pretty big departure from what we say, on the tin, what the Tock threat model is. So do we want to support this upstream at all? If so, how?

Phil: Long ago, we wanted to do this. But we backed off, DMA.

Leon: Alistair asked that we don't have detailed design discussions without him, he would like synchronous interactions.

Amit: Yes, we don't want to design things.

Phil: Yes, we want to respect Alistair's involvement. But it seems like the big question is the threat model. A design has to not violate it? Is that a requirement?

Leon: Right, in the abstract, we can't. Because we can't ensure an arbitrary driver doesn't violate soundness.

Phil: Sure for abitrary devices. But could we say "to use this without violating soundness, a device must have these properties?"

Amit: The more likely scenario is that a particular implementation of a peripheral, plus a particular process, can be done soundly. Unfortunately, the main place you do this, is you have a big blob, who knows what it's doing. The statement has to be "I trust, or have validated, this big blob of C, is well behaved."

Phil: Right, but one question is whether you could separate out DMA. 

Leon: It is more nuanced. Many peripherals share DMA engines. Writing to a peripheral, disabling its clock domain, can mess up other peripherals.

Phil: It would be good to know what this space is like. Even if it's only 20% of peripherals could be "sound", that could be useful. But perhaps whole chips can't do it soundly.

Brad: We have chip crates. Userspace code is on the other side of the threshold. If the userpsace code just passed in a bunch of pointer manipulations (loads, stores), and kernel executed them, would that be OK? What amount of flexibility needs to be there, before we say "that violates the threat model"?

Amit: I think in today's threat model, without device passthrough, the layer at which we extend absolute trust is very very thin. Chip crates can use unsafe. Their role is, or should be, to basically expose hardware manipiulation in a way that is safe. For example, DMA registers, responsibility of the chip create to expose a usize (pointer) as a sort of type-safe buffer, with the right lifetime, such that, as much as possible, as much of the logic, occurs in safe Rust, ideally in untrusted capsules.

Brad: So it's about that thin layer providing that safe interface. That's where we place our trust. Could imagine doing something similar in userspace.

Amit: The issue here is we have a legacy library.

Leon: We can't reason about the safety about this in an unconstrained sense. But we can reason about it with a specific peripheral, and a specific driver implementation. My issue is that the interfaces and such as too general, to allow it to be used for general peripherals. But we want someone to codify that they've thought through this for a specific pairing, so it's safe. How general should this be?

Phil: Seems like a great conversation to have with Alistair.

Brad: I think we want this as narrow as possible.

Pat: I think Bluetooth is unique in this opaque binary blob problem. I think this speaks to it being a hyper-specific to Bluetooth problem again.

Amit: Other examples I can think of, ARM's CryptoCell. I think a bunch -- they need direct hardware access? Chips with integrated acceleromters, step-counting, the guts are in some proprietary algorithm. Some WiFi stuff.

Leon: Another perspective: this is not meaningfully different from not linking in a blob into the kernel. The reason why I think this is niche is BT needs a particular heap. Why can't we just link the blob into the kernel.

Amit: From a threat model, malicious code with access to DMA has arbitrary access. But that's different than bug containment. I think our takeaway is that our bar is relatively high -- we don't just want to include and encourage this. But we are open to it, especially if it solves a particular problem, and would like to avoid generalizing if there are not a bunch of cases.

Brad: This feels more like AppID. Requires a long TRD. It's going to require a lot of eyes on it. It's something we want to maintain, we'll want lots of people to use it.

Leon: Let's follow up with Alistair.

## Makefiles

Amit: 15 minutes left, let's do build system. Brad, please kick us off.

Brad: It does seem like after years of cargo development, it's included the barest of minimum of features that there's enough support for the type of things we want in Tock that off-the-shelf cargo commands just work with Tock. It's a nice milestone. It still requires nightly cargo, but it's at least upstreamed.

Brad: https://github.com/tock/tock/pull/4075

Brad: This moves most of what we have been doing with the Make system into cargo. Make is just a wrapper aroudn this. It's taken a while to figure out how to get cargo to be something sustainable, and manageable, we have something like 30 crates with different architecture targets.

Phil: Using cargo is just better than Make, right? It's the standard Rust thing.

Brad: Big difference for Make to cargo. If you set the RUSTFLAGS variable, it deletes any other configuration you have, and just uses RUSTFLAGS. It's not an append, it's a replace. 

Amit: It's worse than that. 2 variables. Two variables. One overrides the other. Also, cargo configs have a priority among them, they don't compose, unless you do something specific and know what you are doing.

Brad: So Pat and I are cargo skeptics, but we can set a flag, which our build.rs flag checks for, if that's there, you're using the Tock build system. If not, then you are not going to build the standard way. If you know what you are doing, you can set this flag. One version of this question: should we do this, should we not, should we do something else?

Amit: 3 options. 1: not do it at all. 2: do it, do it in build.rs for boards, which would apply both to make orchestration and directly with cargo. 3: do something similar, but only in the make system, so it's less intrusive. In any case, this is something external boards don't have to use.

Pat: This is an ergonomic thing for people not comfortable with cargo and Rust. We would still expect people to just use cargo commands. Silently trampling configurations is a bad idea, we have to work around it, but people who know what they are doing can do it.

Pat: This is a real failure case in Rust.

Amit: Sure, but this is a different population use case from our standard users. We've never encountered this issue, that Make completely ignores the flags.

Hudson: I have attempted to set flags at the command line, it would cause my builds to fail. Oh, all the toher flags must be dropped.

Leon: Something I don't hate, it's set in a cargo environment file, which is not an integral part of the codebase. So this is something only upstream boards have to adopt.

Amit: My suggestion is put this in make, check if the flag is empty.

Brad: I would rather move build.rs into a proper crate, so downstream boards can just use it. Then basically provide build.rs support that does both. Upstream use it, downstream can call whatever functions they want.

Amit: Compromise. Ok, if that's the theoretical plan. I'm OK with that. I'm comfortable with keeping the warning in build.rs for now, if and when it's abstracted into a crate, downstream boards can use it we can then make it optional, we can figure out how when we make it.

Brad: How do we call cargo from stable? 

Amit: I looked into it, I don't know. Maybe run a command that doesn't do anything, then parse the flags, but everything does do something. So it's not cheap.

Brad: It would just be better to not have it print unstable.

Amit: That should be straightforward.





