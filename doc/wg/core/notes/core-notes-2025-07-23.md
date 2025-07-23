# Tock Meeting Notes 2025-07-23

## Attendees
 - Branden Ghena
 - Brad Campbell
 - Vishwajith Govinda Rajan
 - Alexandru Radovici
 - Amit Levy
 - Hudson Ayers
 - Johnathan Van Why
 - Leon Schuermann
 - Pat Pannuto


## Updates
### WiFi Support
 * Alex: We have WiFi on the Pico that connects to networks!
 * Amit: How hard was it?
 * Alex: Only a few days to get it working. We ended up using the PIO blob from embassy. Communication with chip is a half-duplex SPI
### x86 Features
 * Alex: We have a text console working on x86, with keyboard, and mouse is on the way. PS/2 implementations
 * Branden: What do you do with a mouse?
 * Alex: Graphical interfaces with input! Or just as a simple touch controller


## Static Mut
 * Amit: Addressing the long-standing static mut issue
 * Brad: This became an issue because we couldn't update nightly anymore, which forced our hands. We came up with the `addr_of` macro which placates the compiler. But we still can't update to the newest edition because of this.
 * Leon: Right. Warning is becoming a hard error. Maybe it just introduces a warning? Something like that.
 * Brad: So, this is a problem. What we kind-of converged on was having a wrapper type with clear expectations. For the few cases where having shared static mut is the best option from a code structure point-of-view we can use that wrapper. Then we just have to update things.
 * Brad: I was looking through PRs and realized that the CoreLocal attempt at this was over a year old with no progress. Not sure why it was lingering so long: fundamental issue or engineering effort. Also that PR is out-of-date anyways as things like the Processes array changed https://github.com/tock/tock/pull/3945
 * Brad: So, I made a new PR attempting to implement and apply it https://github.com/tock/tock/pull/4517
 * Brad: What are the thoughts here? What's blocking us?
 * Amit: I don't think there was anything blocking it except drudge-work once we solidify the interface? And that's less work since processes-array changed.
 * Amit: There was also some discussion of soundness. And determining what was or wasn't sound. Might have been a "perfect being the enemy of good" issue
 * Amit: Is your new type the same as our CoreLocal but with a different name?
 * Brad: No, no unsafecell
 * Amit: That's needed
 * Johnathan: I don't know why it would be needed
 * Amit: I wonder if Johnathan and Leon have thoughts on this as the experts here. The invariant for threadlocal in std is for like the linux pthread system, but here we only have one thread. But we might want a type that could evolve for multiple threads or cores.
 * Leon: I think the key difference between LocalKey and CoreLocal is that we're moving the unsafe to the constructor and we're always giving you the same instance no matter which context (normal, panic, interrupt) you're in. LocalKey gives different instances per thread. I question this design, as I think it makes it possible to access a value from within another context.
 * Johnathan: We're operating in an odd space. In general a Rust library assumes it could be called from multiple threads. Even for embedded code, there's the interrupt handler issue. But Tock has a unique stance of being truly single threaded, which is different from the rest of Rust. So this type is totally unsound, unless you're using it in Tock code. But I think they solve the problem for Tock. I think whatever type we make is _only_ going to work inside Tock.
 * Amit: If I use LocalKey in the Rust std from a context that's not a pthread runtime, that can also be unsound, right?
 * Leon: Yes. Registering a signal handler in a std Rust is an unsafe operation.
 * Amit: So it relies on an invariant about the runtime. Specific semantics of the threading library
 * Leon: So following that logic for Tock, considering the whole Tock kernel to be the runtime, we should by the same argument be able to get rid of the unsafe in the constructor. I'm uncomfortable from a gut-feeling perspective, but I think I agree with you on paper
 * Amit: We could call the constructor, new-for-single-threaded-tock-only, and keep it unsafe. And make whoever calls it assert things about the runtime. If and when we have multi-core support upstream, then we'd change things. The question is, are the interfaces reasonable for multi-thread scenario?
 * Leon: I think the interface is reasonable for multi-thread. It was designed for that.
 * Amit: We could imagine adding a check inside the `with` function before calling the closure, to check if you're in an interrupt context and panic.
 * Leon: We could. My concern is that we might need to revisit how our interrupt handlers work. They violate the safety and soundness assumptions this infrastructure makes. So, I think this might be sound, but once we start using this (and also the current static mut implementation) it's unsound for interrupt handler implementations
 * Brad: What about our interrupt handlers specifically is problematic?
 * Leon: Fault handlers transitively inspect process state through a static mut
 * Brad: The way we save state when a context switch happens?
 * Amit: No. The Hardware Fault handler when called in a double-fault and prints out process array for instance.
 * Leon: Yes. I agree with that.
 * Brad: That happens when a panic happens.
 * Johnathan: Panic is same-thread though.
 * Brad: Also, who cares about the panic context.
 * Leon: Panics could get stuck in infinite loops due to unsoundness
 * Amit: I think there are some real things to hammer-out here. I think the panic handler issue is a semi-non-issue because its more of a debugging thing. For decisions though, this still feels like the right design, but we'd want to avoid more churn on this. I would propose that we assemble a group of us, a task force, we meet regularly for a couple weeks to resolve this, and we make it happen. This is important and can be resolved, but I don't know that it'll get resolved in a Core WG call. There's also some amount of porting to do. I'd suggest: Amit, Leon, Brad, Johnathan for the taskforce
 * Johnathan: Yes. I just read the Rust atomics book, which is good timing
 * Leon: I also think we should move forward with this. I just want to not lose track of our interrupt handlers having unsoundness.
 * Amit: _some_ interrupt handler cases
 * Leon: Yes.
 * Amit: For example, adding a panic if we're in an interrupt handler context would solve that soundness issue. Although then the interrupt handlers wouldn't work...
 * Johnathan: Ultimately I think we need a new mechanism for communicating data with interrupt contexts.
 * Amit: The normal primitive you would reach for would be a mutex, but then you're worried about deadlock in an interrupt handler.
 * Johnathan: Right. But if your API looks like a cell, then it's locked within a write function, and as long as the write function doesn't fault the panic handler should never see that
 * Leon: You could fault inside the write though
 * Johnathan: Unlikely, but yeah, possible. Could fault with debug data locked, and just print "oops". The fail-safe is just to throw out the type and write out raw bytes.
 * Leon: Serializing everything is safe.
 * Johnathan: Just dump the hex. It's a rare edge case.
 * Amit: Okay, so a few paths forward. One is to merge #4517 as-is, which only adds the type but doesn't use it. That's a place to start. Then also, we can do these task force meetings to figure out how to get rid of all the static muts by using this, including any interrupt handler issues.
 * Amit: The other option is to not merge #4517 just yet. But go into the task force to consider it more
 * Alex: How would capsules use this?
 * Brad: This should only be in main.rs and deferred calls. A few places in the kernel
 * Amit: Capsules can't reference static muts directly because it's unsafe anyways.
 * Alex: What about our static mut buffers that we pass into capsules? We static init?
 * Leon: Static init is safe. That's a static lifetime, but not a static global variable. The type is &'static mut and you have a reference. The unsafe thing is making a reference in the first place, and static init does that unsafe thing internally. Then using it is safe
 * Alex: Technically those buffers are global variables though?
 * Leon: Even if we do have to fix it, it would just be internal to the `static_init` implementation.
 * Alex: So we're most concerned about things where the symbol is accessed in multiple places
 * Leon: Right
 * Amit: And `static_init` has an internal check about dynamic usage which would be an issue
 * Alex: Okay, makes sense
 * Amit: So how do we feel about merging #4517 as-is? I'm in favor
 * Leon: Can we change the constructor name to indicate the runtime assertions? new-for-single-threaded-tock-only (maybe less convoluted)
 * Brad: What's confusing to me is that it implies there's some other way to create it
 * Leon: I like that you'd see in the source what the assumptions are. But I won't die on this hill
 * Amit: Since this PR doesn't use this anywhere, we could still bike-shed stuff like that. This would just set the default
 * Leon: Is the SingleThreadValue name okay? I've come around to it. If we ever went to multi-core or multi-thread, the name would need to change.
 * Johnathan: I do like the name. If we go to multi-core or multi-thread, we'd need to change it considerably anyways. Using generics maybe
 * Leon: Generics was the original design.
 * Alex: This doesn't seem to be mutable in any way? Am I missing something?
 * Johnathan: You have interior mutability
 * Amit: The reason for static mut, is that all global things must be static mut or implement sync. So ultimately at the bottom of this, we implement Sync, which lets us put it in a global.
 * Alex: So it's a wrapper for a value which would not otherwise be shareable between threads
 * Leon: And using the wrapper asserts that you don't share it between threads
 * Branden: So, if you think it should be merged approve it on Github. And Brad change it to a PR instead of a draft
 * Amit: And we do the taskforce
 * Brad: I just care about the usage
 * Amit: That's why I want you there, so we don't sit in pedantic implementation issues forever
 * Alex: Add me as well. I'll attend if I can. I'm interested in running code in interrupts which matters here
 * Hudson: I won't join, but I would be happy to help port if needed.

