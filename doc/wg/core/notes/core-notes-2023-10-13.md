# Tock Core Notes 2023-10-13

## Attendees
- Branden Ghena
- Alyssa Haroldsen
- Philip Levis
- Pat Pannuto
- Brad Campbell
- Johnathan Van Why
- Alexandru Radovici


## Updates
- Alyssa: Attended "Open Source Firmware Conference" and gave a talk on five tips and tricks for embedded rust. https://docs.google.com/presentation/d/1WaeCaDqaJjYKmaIOEiYUlXzdFhc62127b03xm4nqxi0/edit?usp=sharing


## Yield WaitFor
- https://github.com/tock/tock/pull/3577
- Alyssa: It would be good to come back to this. When working on an async runtime, there was a way to structure yield that would save a lot of code size. It's been a few months since I was playing with it though.
- Alyssa: Refreshing memory, I wanted "Yeild-WaitFor-NoCallback" but it would also let you know whether a callback was run. So you know whether to run the futures or poll it.
- Branden: Wait, how do you want a callback?
- Alyssa: Hmm. Confused things. Does it wait forever on the callback? (yes) I wanted to wait on everything
- Phil: So a regular yield. Maybe you wanted to yield but to know _which_ callback executed?
- Branden: Isn't that solvable with a global flag variable or something?
- Alyssa: The issue was saving data everywhere instead of just having it returned in a register
- Phil: A code size thing
- Alyssa: Especially for the async runtime, which has a code size issue for sure
- Phil: Back to the PR, I think "Yield-WaitFor-NoCallback" has a big downside. You can block indefinitely waiting on a specific callback, which could delay all the other outstanding callbacks in the system. My concern is that this could create a new set of problems, which are maybe less significant, but we're not sure yet. Example, after printing for a while, we realized the usefulness of yield-for. So until we really write code, we can't see how big the wrinkle is.
- Phil: Example, two libraries. One assumes that it's got some latency requirement for its callbacks. The other uses "Yield-WaitFor-NoCallback" and stops the first from working with the added latency.
- Pat: This idea pushes me more towards the "Select" style of thing, where you can select what you do or do not wish to run. It gets messy once you have multiple owners/interests in a single app, which is what libraries sort of do.
- Alyssa: I would caution library users that aren't aware of the user's needs to not use the new yield. But the big addition here is not code size but safety. Guaranteeing no reentrancy is really helpful.
- Brad: We do have latency today as it stands
- Phil: Today if you do a yield, callbacks still arrive for other things. Other things still get the processor.
- Brad: That's true, if you do all the work in your callback.
- Phil: Generally, the libraries are expecting asynchronous callbacks.
- Branden: The library was always waiting on the user to eventually call yield. Here, we're just leaning towards longer wait times.
- Phil: Indefinite wait times
- Alyssa: I think this is a documentation issue. I see this as an academic concern that we could avoid in practice
- Phil: So a write works well, but what about a read?
- Alyssa: Don't do that. Make sure you're using things for the real library need
- Pat: Thinking about how other libraries and runtimes do this. In C, the default is that everything blocks and nothing is async except for signals.
- Phil: There's async posix and there's posix
- Pat: So expect everything to block unless you explicitly sign up for async everything.
- Phil: So for me, the blocking commands seem a bit safer in that developers can explicitly decide what's going to block.
- Alyssa: On the other hand, we've discussed that it's not great to have a split ecosystem of drivers where some do and some don't block. They should match the userspace goal
- Pat: Like the Rust ecosystem, throw things at the compiler or typesystem. So maybe some kind of "blocking" capability gets passed if the library wants to privilege to block. Capabilities are for handling things that aren't memory safety, but could be or not allowed by the system.
- Phil: What bothers me with "Yield-WaitFor-NoCallback" is that you get a mix of libraries which do or don't use it and do or don't expect it. It becomes unclear from the userspace about which libraries you can compose together. Knowing which libraries could malfunction could be a documentation or naming thing
- Alyssa: What would make something "Yield-WaitFor-NoCallback" safe?
- Phil: A library that doesn't expect timely delivery of async callbacks?
- Branden: My example would be a radio driver partially pushed into userspace. It's got timing guarantees on responding to packets, or else an acknowledgement gets missed altogether. So it can't function with arbitrary long delays
- Phil: And a safe example would be a printing library. It doesn't expect asynchronous delivery of callbacks.
- Alyssa: So something that would malfunction if callbacks are delayed versus something that doesn't have a timing requirement
- Phil: Another example. So lets say I write a dumb library which responds to the button interrupt. When it gets a callback it toggles the LED. So it waits for async callbacks and does a toggle. This is not safe for "Yield-WaitFor-NoCallback". Because if the user presses the button, there could be a noticeable delay before the LED changes.
- Phil: And a print driver doesn't relinquish control, it does a yield waiting in a loop. The library doesn't return until it's done. So it's safe.
- Phil: So if the print library does a loop and some other callback runs, then that other callback could do a "Yield-WaitFor-NoCallback". So there's a delay there
- Branden: Is that an issue? Print doesn't care about the delay
- Branden: This is all a timing/latency issue. So things that are timing sensitive should go in separate apps.
- Alyssa: This is up to the discretion of the library author
- Phil: And we could mark them in some way so we can tell them apart
- Phil: I do remember Oxide starting with Tock, spending a week tracking down a bug because they missed some semantic about callbacks. So I worry about someone just composing libraries and things not working for subtle reasons.
- Alyssa: I do get that concern
- Phil: The reason this came up for me is that it's an implementation decision whether to use "Yield-WaitFor-NoCallback". So this internal detail bubbles up in a way that users need to be able to see and understand.
- Alyssa: I think it's a specialty syscall for specialty circumstances and should be communicated as such
- Pat: I agree with the complication and am not sure about the answer. Mixing async and sync is hard and maybe you just need to choose which world you live in. It's nice to have this sync capability but mixing it will always be hard and awkward. So being explicit with it seems like it could be important. Compiler-enforcement would be nice here instead of documentation-enforcement.
- Phil: Should we have a libtock sync and a separate libtock async?
- Johnathan: Libtock-RS is kind of in a halfway point. It's kind of more sync, but there are often operations with a timeout or some way to abort it. That's done in Tock by running two things in parallel with async mechanisms. And that's where you do a couple things at a time, but it's still a synchronous structure for a program. Definitely a hard problem in general. Much easier to write synchronous APIs, but cancellation becomes a real concern.
- Alyssa: There was an Open Source Firmware Conference about using Rust in embedded. The example of running an operation with a timeout was so elegant with futures.
- Johnathan: Libtock-rs is like that now, but with uglier syntax. So it might be achievable to support futures in Libtock-RS if everything is together. The compiler could figure it out. Where doing everything async everywhere would lead to no hope that the compiler could take away all the fluff.
- Johnathan: So far my experience is just larger code sizes. Still a work in progress
- Alyssa: I agree that an async runtime will always have some problems. The way libtock-rs is doing it now is fine enough, just not super elegant. Getting rid of closures for unowned memory seems impossible to get rid of.
- Johnathan: Maybe with pinning. Whoever owns it will need to pin it.
- Alyssa: If you have your allow API instead of accepting a mutable reference, accepting pins, that could work maybe. Hard though.
- Johnathan: Taking a step back, if you want to write a driver with an async implementation, you need a handle that says what lifetime you can use data for. The lifetimes look different between pin and closures though. One change we could do is instead of having a closure-based API, you app could have every driver expose operations which take closures for what to do while waiting for the operation to finish.
- Alyssa: I think it should be possible for closures, but it should generalized to a larger zero-copy trait.
- Johnathan: No time to look at it anytime soon though
- Phil: So my conclusion is that there will be issues, but they're probably minor and we'll figure out a way to manage them. My concerns were somewhat reduced. So I think we can move forward when we have time


