# Tock Meeting Notes 2025-07-09

## Attendees
 - Branden Ghena
 - Alexandru Radovici
 - Brad Campbell
 - Johnathan Van Why
 - Pat Pannuto


## Updates
 * Pat: Leon and Amit are at OSDI where Omniglot just won a best paper award!!
 * Brad: We brought back naked functions into Tock. Rust has helped us out by stabilizing those. They've been in nightly for a while but are now in stable
 * Johnathan: I did look at the embassy async stuff and I left a comment there about soundness. https://github.com/tock/tock/issues/4497
 * Alex: Great. We'd love feedback on that


## Yield-WaitFor
 * https://github.com/tock/libtock-c/pull/521
 * Brad: Yield-WaitFor has been around for a while. This functionality is our compromise to help synchronous applications. You can specify one specific upcall to wait for, rather than just any upcall.
 * Brad: It's been implemented for a while, but we didn't have userland support anywhere. It seemed like we should actually start using it. The hangup has been that it's a little tricky to build a library around different types of yields. Mixing sync and async code is pretty difficult, as yield calls could have any upcall, even unexpected ones. Now, for the same driver, if you have a yield and a yield-waitfor in the same driver library, you could end up getting a return because your async operation finished and not your sync operation. That feels like acceptable risk because, don't do that. What we are going to do is some work on library headers to make it obvious if you're including both the sync and async from the same library.
 * Brad: Generally, yield-waitfor is more intuitive. It does what you expect with only one upcall.
 * Brad: Question is how do we move forward. Are we okay with libtock-sync using yield-waitfor everywhere and we just watch out for using sync and async for the same driver?
 * Brad: Also, question of whether we need to change every driver before merging any driver changed.
 * Branden: Sync and async mixing for different drivers is fine, right?
 * Brad: Yes. Your async operation won't happen until your sync finished. That's probably what you expected though. Things won't break returning because something else finished.
 * Alex: Observation here. The while-yield pattern should be an anti-pattern. You can have stack overflows. Imagine a packet reception wait loop which collects packets, and then calls printf in that upcall. That could receive another packet, which would printf again. You can stack overflow there, and it's really hard to understand. So I say we should _never_ while-yield.
 * Brad: Agreed, big gotcha. This would fix that. The only thing that would happen is that printf would finish.
 * Brad: Should we remove the yield-for function in libtock-c?
 * Alex: Yes. The programs are incorrect and it's not possible to stop the overflow bug.
 * Brad: Well, if you just do async there's no issue with a yield-for loop. But async usually isn't waiting for a specific thing, just anything.
 * Alex: People expect printf to just work, and it doesn't. It's hidden what's going on within it.
 * Branden: The yield-for function is only for libtock-sync, right?
 * Brad: I think that's it.
 * Alex: My developers used it and messed up. My vote is for moving all the library to yield-waitfor. The yield-for function can be removed entirely as it's misleading how it behaves
 * Brad: I did a git-grep and the yield-for function is largely used in libtock-sync. There's also some old stuff in libtock that never got updated. Some example apps also use it directly. We'd have to look into those
 * Branden: The apps are the thing. Assuming they can be implemented differently, we would get rid of the yield-for function
 * Brad: I can't think of a legitimate reason to use it in an async context off the top of my head
 * Alex: You shouldn't. Async programming in Rust clearly states that you may never do blocking system calls.
 * Brad: Okay, that's good feedback. At the worst we should deprecate it and note that yield-waitfor exists. But probably we should remove it
 * Alex: How would you call the wrapper function for the system call?
 * Brad: We have a function yield_waitfor which does it.
 * Alex: Okay, I suggest a deprecation message and pointing people to the new one. I don't know if C has a compile-time deprecation message
 * Brad: The syntax is different. Yield-WaitFor is a low-level syscall, like command or subscribe. If you were using yield-for at a higher level switching would be confusing. You should be switching to the synchronous API probably. I'd have to look into the apps that use it
 * Alex: I think yield-for is a weird mix of low level and high level. Unintuitive
 * Johnathan: I don't touch libtock-c, but it sounds like we're a little uninformed on yield-waitfor right now since we don't use it a lot.
 * Brad: The other issue is around timers. Timers are probably the most obvious API where you would potentially want sync and async combined. An every-one-minute timer, but then also block for 10ms somewhere in the middle. Mixing the two is dangerous. That's another concern, but I think maybe we just try to avoid it in the short term. I do think the drivers could be written so that this can be handled correctly and fix it behind-the-scenes. But that's the exception, not the rule. The benefits are still strong
 * Pat: I think the way to mix in userspace would be the ability for yield-waitfor to be a select interface where it could wait for multiple descriptors and the library could dispatch things based on which occurs
 * Brad: I think if you have multiple descriptors it's not an issue at all. You could wait for one specific upcall and ignore the other
 * Pat: The probably is when your library has synchronous stuff internally and you don't know that's going to happen
 * Brad: That's a stylistic question. Whether the synchronous function truly blocks everything. You can just handle the async thing late sometimes
 * Brad: So select could just select on a single descriptor. But it would need to mark things as pending
 * Alex: You definitely shouldn't call yield in a callback function. Things go bananas and stack overflow there. That's the anti-pattern
 * Brad: And yield-waitfor would make an additional anti-pattern of mixing sync and async for one driver
 * Brad: Okay, my todo is to update the document to note that we haven't switched everything and point to a tracking issue, which I'll make. I'll also look at removing/deprecating the yield-for function
 * Pat: I'll also note that a PR just came into libtock-rs which implements yield-waitfor https://github.com/tock/libtock-rs/pull/575
 * Alex: That's my student. We're going to only use yield-waitfor in libtock-rs since it's always synchronous. That would be the correct behavior
 * Brad: My only thought for libtock-rs is mixing sync and async. Detecting it. There's no sync and async split to my knowledge
 * Branden: Does libtock-rs support async at all?
 * Johnathan: The core was designed to be async, but the APIs implemented are all sync right now because the async APIs are too complicated and less documented
 * Brad: Buttons have listeners you can register, for example. So there's certainly an API for doing it.
 * Branden: So we do need some way to guide users away from mixing sync and async in the same driver. Maybe our experience in libtock-c can inform that after this is merged in
 * Alex: Could we detect yield calls in a callback and warn/crash so users know not to do it?
 * Branden: I don't know how we'd detect that
 * Alex: Maybe something with a trampoline and detect stack addresses before calling yield? Would people be interested in that? The kernel would somehow figure out that the yield was within a callback that it scheduled?
 * Brad: In the kernel?
 * Alex: Maybe? If it could figure that out
 * Brad: Maybe. Applications should be able to do what they want, I think? And it sounds really hard to do across architectures, with multiple architectures having to do it
 * Alex: Maybe the library then? It was extremely hard to debug this stack overflow when we ran into it. So hard. I want to avoid it
 * Brad: I think that being more conscientious of this in the API design will go a long way, but that remains to be seen
 * Branden: I'd say I'm not against a check, but it sounds hard, especially in the kernel. Maybe userspace but we'd have to handle multiple architectures which sounds hard still.
 * Alex: Okay, I'll think on this for a bit

## Debug Panic Capabilities
 * https://github.com/tock/tock/pull/4479
 * Brad: Debug inside the kernel uses unsafe quite a bit. It seems like this isn't Rust safety, so capabilities could handle ti and keep code from calling debug unless it's allowed. I wondered if people had thoughts
 * Pat: I think I left them unsafe because of fear and ignorance of not understanding the state of the world when we're panicking. Most of those methods can get called from the panic handler so I don't know if there's things to be reasoned about.
 * Pat: I don't think just leaving unsafe without documentation or thought is good either. But that's why
 * Brad: That makes sense. But why can't a panic context just call any function? By that logic, it seems like every function would need to be unsafe because it could be called by panic handling.
 * Pat: I think the answer is to read up around the rules on panic contexts in Rust, if there are any
 * Brad: There is one function that sits in a forever loop. That's bad. But most of them don't seem to need unsafe from a Rust library point-of-view. Just the ones that need static access to the debug printer.
 * Branden: Might need to grab Leon and Amit sometime to ask those questions

