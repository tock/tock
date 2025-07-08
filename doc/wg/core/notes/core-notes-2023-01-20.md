# Tock Core Notes 2023-01-20

Attendees:
 - Branden Ghena
 - Phil Levis
 - Leon Schuermann
 - Hudson Ayers
 - Jett Rink
 - Alyssa Haroldsen
 - Vadim Sukhomlinov
 - Brad Campbell

## Updates
 * Hudson: Amit went through and approved and merged the two license PRs: the checker tool and policy doc. He also submitted some updates to the PR that adds license headers to files. Johnathan found a few small issues with that PR, but we're very close to being done with it.

## Significant PRs
 * License PRs!
 * CAN HIL was merged thanks to Alex & team for the code and patience


## Deferred Calls
 * Hudson: Leon and I have been working on this
 * Hudson: As a refresher, today in Tock there are two types of deferred call.
 * Hudson: The normal "deferred call" that's only used in chip peripheral crates. For example, the SAM4L uses it for the USART and Flash controller. The implementation supports only up to 32 deferred calls, but it's a very clean interface since it relies on global variables in each chip peripheral and in the kernel. Not a lot of boilerplate, just make a handler. Then the chip.rs file maps deferred calls to peripherals.
 * Hudson: The other type is "dynamic deferred call" which was just in capsules, but is now also in the kernel app-checking infrastructure. Twelve total places, but capsules that are used by many boards like virtual UART. It uses dynamic dispatch from the scheduler to select the appropriate handler, which has vtables everywhere so some size overhead and less good API.
 * Hudson: I have a few very incomplete slides on this that may improve in the future: https://docs.google.com/presentation/d/1Y9bT053LlU0097AmE4MmVwDxmF09VgUXJ1wHRVb3ndE/edit#slide=id.p
 * Hudson: One downside of the "deferred call" implementation is that there needs to be an enum listing all tasks. So downstream changes would require forking the entire chip crate, instead of depending on it. Some other downsides are that it uses our own implementation of AtomicUSize, which keeps us on nightly rust for core_intrinsics.
 * Hudson: Downsides for the "dynamic deferred call". It has about 100 bytes per client extra code size. Plus 20 byte vtable per use of it in RAM. Plus some runtime overhead since it can't be inlined. Also it's more annoying to initialize and use; much less clean. Capsules that use it have to accept an object in their new functions and store a handle in an option. Finally, it panics at runtime if there are not enough slots allocated. People have to remember to add this in main.rs. AND the panic happens before the board setup finishes, so the panic doesn't print anything nice out, and just looks like a silent hang. Terrible to debug.
 * Hudson: So here's what Leon and I have been working on to fix things. https://github.com/tock/tock/compare/master...hudson-ayers:tock:defcall4
 * Phil: Which of these problems does this solution solve? Or all of them?
 * Hudson: This is a lighter weight dynamic deferred call with a simpler interface. It solves the problem where you get a panic before the board setup finishes. Now it's after the kernel loop starts. Not registering is also a panic now. This approach has lower code size and RAM overhead from "dynamic deferred call". It also replaces both types, so there's just one type to think about and use. Easier to explain hopefully. It is more expensive than the old "deferred call", but it's less expensive than "dynamic deferred call", so I expect the total change for most boards to be a net reduction.
 * Leon: It's also worth talking about tradeoffs. We thought about a lot of approaches, and the only major downside of this approach is that it makes it less obvious how deferred calls work under the hood and requires an understanding of how stuff is routed in Tock. We're just registering stuff in capsules now, but it can be hard to find where routing happens since there are some globals. It's a little less obvious.
 * Leon: Still, I think it's our best solution. We did think about other possibilities.
 * Phil: I think that's not too bad. Documentation can solve that issue.
 * Phil: It's also worth saying that this is a long-standing problem in systems like this.
 * Hudson: One other key limitation is that since this still uses a bitmask to track state, it has a maximum of 32 deferred calls. It's straightforward to increase that to 64 or 128, or could even use an array.
 * Phil: Why use a bitmask?
 * Hudson: More efficient lookups instead of iterating
 * Phil: Why would you need to iterate?
 * Hudson: The bitmask makes it really easy to set a bit when it wants to schedule itself
 * Leon: A ring buffer would mean we'd have to check the existing bits to see if a call is already scheduled
 * Alyssa: A ring buffer doesn't have to have a check and a branch for when it would overflow, you can mod the index
 * Phil: I think they're saying that knowing if a particular deferred call is already in the ring buffer is iteration. You can't have any more than once, or else it could overflow since it only reserves space for what exists
 * Alyssa: This is still a work in progress?
 * Hudson: Yes. Any particular issues?
 * Alyssa: I see some usize instead of u32. And more doc comments would be great.
 * Hudson: Definitely. I'm using the bones of the code you suggested for lower-cost dynamic deferred calls, although I made some slight changes when storing the callback.
 * Alyssa: You're avoiding doing an unsafe cast of the function pointer.
 * Hudson: Leon was concerned that there wasn't a guarantee that the ABI isn't the same, but the closure should go away at runtime.
 * Alyssa: That should be a guarantee
 * Hudson: We workshopped this with Rust people on the subreddit (main Rust subreddit), and they proposed this. It compiles to the same assembly
 * Alyssa: Great. With no cost, I have no concern.
 * Hudson: Moving to the 15.4 driver, which used "dynamic deferred calls".
 * Hudson: We used to store a handle and a call. Now it's just a call. You also don't have to pass in from main and can just make it yourself. There is a register method, which is a part of the client trait along with handling calls. Finally, you can just "set()" a call now, no unwrapping necessary. So I think this is a lot cleaner. Less in main and less in each component, although components MUST still remember to call register. But if it doesn't, you'll get a panic in the kernel loop, so it's easy to spot.
 * Hudson: That works by the kernel loop verifying all deferred calls the first time it starts (both count and registration)
 * Hudson: So this is where we're at for now. I've been trying capsules to start.
 * Hudson: One interesting one. The LPM screen capsule. It had a single call, but multiple handles for different types of callbacks. To implement that, it did a match on the handle type that was returned from the deferred call. So this code would check which of the handles it owned matched the returned handle. With our new mechanism you can't do that anymore. Deferred calls don't return anything to you. Every call could take in and return an ID, but I'm not sure it's worth it. No other capsule does anything like this. Each call would be larger.
 * Leon: Yeah, I'm against adding an ID. It's easy enough for capsules to have their own state that tracks which deferred call is active. I thought based on the design that everyone would use this idea, but we only added it to the docs recently. So for the one or two places that use it, I think we can work around it.
 * Hudson: Yeah, so that's an overview of what we've worked on and where it's headed. We're going to finish porting this for the capsules, then I need to port over app verification in the kernel. Then we'll take some measurements once we finished removing "dynamic deferred call". Then we'll port over the chips, add docs, make a PR.
 * Phil: I have a question about macros mostly. This challenge was one of the major challenges that TinyOS encountered, and handled with language extensions in TinyOS 2. The way we eventually solved it is that every deferred caller is given a unique number starting at zero, and somewhere else can get that count to initialize the structure that holds them. So every caller has a fixed index into the array. Makes most operations constant.
 * Phil: I'm wondering if we could use that same idea with Rust macros. We can make it so each trait gets a number, but we also need to get at compile time a sum of all those (the total).
 * Leon: We have been thinking about this approach. There is a mechanism in rust call TypeID. Which can give unique numbers to any given type. It doesn't have the property that they are necessarily sequential. We could use the numbers to allocate an array and track which capsule. But we have no way to count them. A fixed number of deferred calls and using a counter, we still can't really with the current state of Rust be generic over that number. The type system isn't there. We did think about a deferred call scheduler which is generic over the number of calls it have. I tried implementing it, but the generics just aren't there with reasonable code quality.
 * Alyssa: Rustc is allergic to global analysis.
 * Phil: So we can separate into two things. 1) can I assign a unique identifier, and the answer seems to be yes. If they can be dense and start at zero, they can also be used for an array-based queue. And 2) can you know how many there are so you can make the queue large enough. Number 2 there is less important, as we could just allocate RAM space for it, as the N here is quite small.
 * Phil: Two links: https://stackoverflow.com/questions/51577597/how-to-automatically-generate-incrementing-number-identifiers-for-each-implement and https://github.com/tinyos/tinyos-main/blob/master/tos/system/SchedulerBasicP.nc
 * Hudson: How did you store a pointer to the actual function that needed to be called?
 * Phil: In TinyOS you didn't have to. In Tock it would have to be stored somewhere.
 * Hudson: Let's say that call 3 was set, how did TinyOS change to that call?
 * Phil: What TinyOS would do, and we can't, is parameterize a function by a constant. So there's not just one but N of them. And you can say "execute version 47 of this function". So the compiler makes the switch statement. In Tock, when you register your deferred call, you'd stick the handler in the array of function pointers. And the ID thing would just be for maintaining the queue.
 * Phil: I have long thought we CAN'T do this in Rust. I just wanted to bring it up again as some things are getting better.
 * Alyssa: I'm not sure why it would require constant folding of const generic, instead of inlining regular arguments.
 * Hudson: I think the challenge Leon was describing was that you would want some const generic parameter use to parameterize the global array by, if we had some way to count the number. You'd also parameterize the calls by that constant.
 * Alyssa: I think you can hack this with linker scripts, but I really think it's best to avoid global analysis.
 * Hudson: This would possibly cause havoc for incremental compilation.
 * Alyssa: It would also be static and not dynamic.
 * Hudson: We actually are fine with static. We didn't want dynamic, it was just an implementation choice. It wasn't like you could change the target.
 * Leon: We did try having a switch in the board state, but that didn't work well
 * Alyssa: I think calling a function pointer is more elegant than a switch statement anyways. I like the idea of automatically setting up the magical switch statement, but I suspect that runtime function pointers is more elegant.
 * Phil: Yeah. I wasn't exactly suggesting that. I just wanted to explain how TinyOS did it. In Tock, there shouldn't be a switch statement. The good thing would be the automatic maintenance of the structure such that it will never overflow, will easily maintain the ordering, and will allocate only exactly as much memory as you need.
 * Alyssa: What are the blockers for that kind of scheme? That's what I didn't understand. Did it require full knowledge of the number of tasks?
 * Hudson: Yes.
 * Alyssa: What about a maximum number of tasks?
 * Hudson: That's this now.
 * Alyssa: We could use a ring buffer instead though
 * Phil: Ring buffer would keep ordering, which would be nice
 * Hudson: That's true. Existing code, I don't think has FIFO ordering.
 * Leon: Neither "deferred call" for sure, or "dynamic deferred call" which we maybe just didn't implement. It did technically allow chips to assign priority.
 * Phil: Yes. Interrupts are done in the same way where we scan the bitmasks. Starvation is problematic.
 * Phil: So if you can do the counting, then you can maintain the ordering. And we could just oversize it, since it's really not big.
 * Hudson: Confirmed that "dynamic deferred call" isn't FIFO today.
 * Alyssa: Okay, so if FIFO wasn't expected, it could remain unexpected.
 * Hudson: It wouldn't be a huge lift to go from bitmask to fifo.
 * Leon: Wouldn't take more space. Seems possible.
 * Hudson: Alright. So no final decisions yet or anything. But thanks for the thoughts and feedback.
 * Alyssa: Could we make it so that the trait for deferred calls can't be made into a trait object by giving it `Sized`? Make it clear the intention is to NOT use it as a trait object. It would stop you from making a `dyn` object out of it.
 * Leon: So anything that is not sized, can no longer implement the trait.
 * Alyssa: It also stops implementing it on a slice, but I don't think that matters.


