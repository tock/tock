# Tock Core Notes 2023-03-31

Attendees
 - Branden Ghena
 - Amit Levy
 - Phil Levis
 - Vish
 - Johnathan Van Why
 - Vlad R
 - Pat Pannuto
 - Alexandru Radovici
 - Brad Campbell
 - Vadim Sukhomlinov
 - Alyssa Haroldson


## Updates
 * None


## Ordered userspace printlog
 * Phil: Ordered userspace printlog stuff. It's working such that all writes to the console are temporally ordered. But if there's not enough space it'll be chunked out and delayed, so kernel writes could be in between. User writes are lossless. Kernel writes are not and could fail. I had to fix some bugs: if you ask how much space there is left, it doesn't consider the warning message that it printed too much, so I could print less.
 * Phil: Unfortunately the version in there now requires changes to libtock-c so it can handle partial writes and loop. I'm going to try to remove that requirement so it can stay the same. It will mean that long userspace writes could take a long time to complete.
 * Alyssa: How does this handle multiple apps?
 * Phil: It is temporally ordered, but if there are many long writes, it might be a delay before the next occurs. But it will be ordered with concern to actions the app takes. So if you see a message, you know where the app was.
 * Alyssa: Does the application wait for the print to complete before issuing the new syscall?
 * Phil: Is the application print synchronous or asynchronous?
 * Alyssa: It's using blocking commands
 * Phil: Then it will be ordered. If it was an async print, then it could be theoretically possible that a kernel print will jump in before. In practice it would be pretty rare. If the userspace async prints and there isn't enough space, then the console driver starts a retry timer. If it starts that timer then the next system call leads to a tiny debug statement, there could be space for that and it might jump in. We'd otherwise need to block kernel prints, which we can't. But synchronous stuff is in order.
 * Alyssa: Our Tock fork has a blocking command syscall. That's not upstream yet, right?
 * Phil: No, not yet. Jett wanted to make sure it's good and stable before sending it up.
 * Alyssa: I'm excited about it, and I'll do a good review.
 * Phil: I'll do the fix for userspace soon.
 * Phil: Last time I mentioned the weird behavior of not printing things sometimes, and Brad was right in guessing that it was a malloc issue.


## Design problems on Sync Cell
 * Alyssa: We've discussed a data structure that's marked sync on embedded platforms but locks behind a mutex on host platforms. It would have operations like Cell. The question is whether this design choice is reasonable for applications. That we can assume we'll be single-threaded with no preemption while things are running. There are yield points, but you won't have unsynchronized writes to memory.
 * Alyssa: So a question is what about host platforms. Should it be a mutex? Or a single shared global static or thread-local? Even behind a mutex it could be messed up if multiple tests are running concurrently.
 * Alyssa: Problem number 2 is that in the kernel we have to worry about preemption from interrupts writing to a sync cell. If that happens, you could have a corrupted type. You could be halfway through writing a bit pattern when things go wrong. It's undefined behavior too. So, what level of safety should be required?
 * Alyssa: I think that using another crate called zero-copy that defines key marker traits like "from bytes" that defines that all bit patterns are valid would be useful. So we have a bound similar to that? It would remove the undefined behavior, although not corruption. We could alternatively make all operations normal and safe and mark that it can't be used from interrupts. That's similar to solutions other systems use. Or we could mark most main operations unsafe.
 * Amit: When we are on the kernel side, you mean interrupt service routines, right? (yes)
 * Alyssa: Yes, in the interrupt context.
 * Amit: So, in general the rule in Tock kind of has to be, maybe some special case exceptions, but thou shalt not touch shared data from interrupt contexts. Things that capsules and the rest of the kernel touch must be accessed in a single threaded way.
 * Phil: When we designed stuff, we said if there was something super-performance critical, you could do some things in interrupt contexts, but all bets are off. In practice, we have never seen that. Does TI50 need it?
 * Alyssa: Not necessarily. We're just worried about the footgun
 * Amit: And there's nothing _preventing_ someone from putting some access within an interrupt service routine without at least being unsafe.
 * Alyssa: Is a big warning message enough?
 * Phil: For interrupt handlers, yes. They're generally considered stuff you don't touch unless you know something crazy is going on.
 * Alyssa: So it could have the same bounds as normal cell in user and kernel space.
 * Alyssa: There are other questions, like how to know what platform we're on. Should there be some kind of a flag that tells you if you're guaranteed to be single-threaded or not? Because SyncCell would need to swap out internal implementations.
 * Branden: Isn't there already a global flag if you're in testing mode?
 * Alyssa: 1) No, it's only for the crate under test, not other stuff, and 2) you might do host emulation where you're not in test mode.
 * Amit: Maybe we could have an implementation of thread-local storage for Tock.
 * Alyssa: I do want SyncCell to be zero cost like a static mut. For host stuff it could have costs, maybe.
 * Alyssa: Also what about mutex versus thread-local?
 * Amit: Does thread-local guarantee the right things? Is there guaranteed to be no other kind of concurrency that would break this in testing?
 * Alyssa: Not sure what you mean
 * Amit: If for some reason if someone tested with tokio and tests were async and concurrent within a thread or something.
 * Alyssa: It would still be sound and there wouldn't be data corruption. But there could be something writing and something reading and a state you don't expect.
 * Phil: I guess, and maybe I'm confused, that if you can represent it as thread local then there wouldn't be any waiting on a mutex, if it's recursive, so you wouldn't have to wait on the mutex anyways. Why bother implementing that if it's never going to matter. Plus implementing recursive locks is tricky to get right.
 * Alyssa: There's a warning now that reentrancy deadlocks. So it isn't recursive right now.
 * Alyssa: Also, disadvantage of thread-local is that it requires it to be constructed with a macro. Which is non-trivial.
 * Phil: So this sounds like a software engineering versus performance issue.
 * Alyssa: If OnceCell has been stabilized, that resolves a lot of my worries.
 * Amit: It has been. What are those worries?
 * Alyssa: Mutex required a constructor, so I needed to use OnceCell for it.
 * Amit: Mutex new() is const as of Rust 1.63
 * Alyssa: There does exist a thread-local struct library. But it requires stuff I don't want to use.
 * Amit: So back to my earlier suggestion, we could in Tock have a thread-local construct, which looks like the macro from the standard library maybe, which is essentially a no-op but is not a no-op in host emulation. Or maybe it does let us have some restrictions, and enforces no access to shared states in interrupt handlers. So this could address the software engineering need of having to declare variables in different ways on host platforms.
 * Alyssa: Not sure that would work. I see a lot of possible engineering issues. Very tricky to do correctly.
 * Alyssa: How do you write a thread-local without a thread ID?
 * Amit: One way of doing it, a platform-specific implementation, you know that the implementation is just a global. There's nothing. But the implementation on host-emulation platforms with threads would use the standard library one maybe.
 * Alyssa: How is the swap out done?
 * Amit: I don't know. Not sure if "whether std is available" is something we can check?
 * Alyssa: That would be neat
 * Amit: I've seen crates do this explicitly by having the std feature, but that's a little more poisonous. Probably have to pass it around everywhere.
 * Alyssa: I like the idea of exposing whether you want something thread-local or not. And giving that a shared interface. I still don't see how we'd be able to construct a thread-local without a macro. Seems necessary for how Rust does it.
 * Alyssa: So if SyncCell new used a macro for a constructor, is that okay?
 * Amit: I'm not sure I'm following.
 * Alyssa: If you made sync cells with a macro, is that fine?
 * Phil: I'd need to see the code. No immediate red flag, but I'd have to see some examples to know.
 * Alyssa: Do you have any better ideas for determining whether the host platform can support threading? Is there a global Tock flag we could create or that already exists that guarantees single-threading?
 * Phil: Having always lived in a single-threaded world, I don't know.
 * Alyssa: So we'd need some way to detect and add something to Tock to determine it.
 * Alyssa: Because we have a fixed set of platforms, we check the platform now. But that's not scalable to Tock in the same way.
 * Phil: In terms of C, there are compiler flags for this. Whether you're freestanding or not. I don't know of Rust equivalents. I'd guess that it doesn't because of how fundamental multithreading is.
 * Alyssa: There's something in the target for the toolchain. But I don't think it's available to compiled programs.
 * Johnathan: We're actually the only Rust project I know of that's single-threaded. Even the embedded stuff assumes threading in most cases.
 * Alyssa: Maybe something in build.rs with a config.
 * Phil: What's weird is that no-threading is the common case. It's only under host stuff that there ever is threading.
 * Alyssa: Okay, I'll review possibilities and come back.


## Licenses
 * https://github.com/tock/tock/pull/3317
 * Amit: We should do a last call on the license notice in all Rust files. Most of the files in the repo. In accordance with Johnathan's checking tool. There are some notes to fix, but otherwise I think that we're ready to go on this. It would be nice to do it quickly, because changes in main that conflict require fixes.
 * Johnathan: It massively sucked to review this. The only way I can review changes is by diffing against the version I just reviewed. I'm going to end up reviewing unrelated PRs.
 * Johnathan: Here's the problem. We need Leon because there are seven files he wrote where I'm unsure of the copyright year. So there are only 12 files that would change in total, I think.
 * Amit: I'll find Leon today and chat with him. Some are easy since we can see the year it was committed in.
 * Johnathan: It was on the edge of 2022 or 2023
 * Phil: You can move copyright forwards, but not backwards.
 * Phil: I'd like to propose we commit them as 2022 and we can fix it in a separate PR if needed.
 * Johnathan: I say switch them to 2023.
 * Phil: Doesn't really matter. That's fine.
 * Amit: I can fix the duplicate headers Johnathan commented on too. We could also piecemeal fix them later if we notice it.
 * Johnathan: SHA256 stuff was copied in and shouldn't have a Tock header. The rest is easily fixable.
 * Amit: I will fix right now.


