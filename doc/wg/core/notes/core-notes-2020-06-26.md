# Tock Core Notes 06/26/2020

Attending
 - Branden Ghena
 - Oskar Senft
 - Samuel Jero
 - Phil Levis
 - Brad Campbell
 - Amit Levy
 - Johnathan Van Why
 - Vadim Sukhomlinov
 - Alistair
 - Andrey Pronin
 - Pat Pannuto
 - Hudson Ayers
 
## Updates
 * Johnathan: Working on libtock-rs system call rewrite. Ran tests under miri which threw an error. Now I'm looking into how to resolve. Miri is the symbolic interpreter to rust that is good at pointing out undefined behavior. But it's also its own implementation of the language, so it's unclear if we are really at fault or not. Investigating.
 
## Tock 2.0
 * Amit: Not much update this week. We should get our stuff together and start making progress. Phil has been hammering away at timer and alarm interface (will discuss later) as one reason things haven't moved.
 
## Scheduler Interface
 * Hudson: Wanted to bring up what's changed. I still have an outstanding PR to make it possible to select different schedulers from a board main.rs file rather than a static kernel choice. Schedulers can be initialized using components as generic across processes, which is ideally just a single line of code in main. I iterated over a few different designs at this point. Last time I talked about this on a call I talked about how the limits of trait objects, `dyn` keyword, prevents a design where the kernel has a reference to the scheduler. The issue is that schedulers require references to chip and platform generics and as a result we can't have the scheduler be a `dyn` object. But if you use it as a generic, then it would bubble up to everything that holds a reference to the kernel. I moved to a design where kernel loop is called on the scheduler. Each scheduler implemented their own version, which then call kernel functions for syscall handling etc. I had every scheduler implement `do_process`, which execute processes, handles syscalls, and decides when to swap back to kernel. One concern many people had was that a copy of `do_process` for each scheduler was going to lead to bugs, particularly because it's code that can affect kernel correctness. The current design relies on a single `do_process` that's a function on the kernel struct. But it's not the same as the `do_process` we have now, since schedulers need to be able to select things that happen in `do_process` like delaying bottom-half interrupts for real-time scheduling, have different timeslice lengths. It's also important that schedulers can determine the systick implementation. It should be possible to use a cooperative scheduler with no systick for example. I made all of those things settable in the calls to `do_process`. They are variables today, but we could imagine them as const generics (but those aren't in stable rust today). Unfortunately, schedulers can only measure system state before and after calls to `do_process`. This means that time spent handling syscalls is attributed to the process that issued that syscall. That's how it works now and how linux works, so it's probably fine across all schedulers unless someone has a specific use case counter example. This applies to other things too. An energy-aware scheduler would only be able to take measurements before and after returning from `do_process` so energy consumed in the kernel would be attributed to the process. Again, I think this makes sense so it's probably fine. So that's the basic design. PR is https://github.com/tock/tock/pull/1767. I want this back on people's radar and for people to review and put forth any thoughts on it. The systick stuff that's discussed there now is going to be split off into a separate PR.
 * Phil: Hudson and I have had a few conversations about this.
 * Hudson: Specific action item is for people to review the PR.
 * Brad: Are there schedulers that need an initialization or setup step before they can do the main loop? If so, how does that work?
 * Hudson: Yes. A few I implemented need set up. That's done in the component right now. The custom would be that you initialize your component immediately before calling kernel loop and after initializing the rest of the platform.
 * Brad: So it happens when the component tells it to, but the logic is in the scheduler in the core kernel?
 * Hudson: The logic for setup for any particular scheduler would be in the new function for that scheduler. Although some additional logic could be in the component for that scheduler, such as initializing a linked list which needs `static_init`.
 * Brad: This is coming from the component discussion we had about how much structure is specified in the trait versus letting it be flexible. The trait only has the one function right now. Are there other common operations that could be clearly specified? Or should there just be the one function?
 * Hudson: I think the interface could be changed to have `get_next_process` and `process_finished`. But I think it ends up being less clean. The scheduler needs to tell the board when to sleep and how to handle interrupts. Pushing those things into a bunch of functions which would end up as no-ops for most schedulers seems like a less efficient design, which is particularly important for the scheduler. So I ended up on the side of less structure for that reason.
 
 ## Timer/Alarm Interface
 * Phil: I think the key thing is that we started with a reference doc to write down the APIs and semantics. A couple things are changing when implementing. The current status is that new alarm is implemented for SAM4L. New alarm virtualizer too. We've updated the device number since the API is different. We used to have this timer trait too, which wasn't implemented, but now is. The key thing about a timer is that, with an alarm you say here's when I'm setting the alarm and here's a delta-time for when it should go off. For the timer you just provide a delta-time, and "now" is fuzzy, but they can be repeating. So "now" is fuzzy as is the first firing, but after that it's a very precise interval for repeat firings.
 * Phil: Two things that would be nice to talk about from the implementation. Here's the branch we've been working in: https://github.com/phil-levis/tock/blob/time-redesign-v3/kernel/src/hil/time.rs
 * Phil: For this specific line: https://github.com/phil-levis/tock/blob/7fb8f4716bcc496baa2c546e60716089f168a4b1/kernel/src/hil/time.rs#L41 Guillaume found this important in prior works. You have a time value, a time stamp, and then can check if it exists between these two other timestamps. If an alarm has a t0 and a dt, because t0 _must_ be in the past, that means your alarm has expired if "now" isn't between d0 and dt. So suddenly a bunch of complex logic gets stripped away and this is all you have to do to see if expiration occurred.
 * Phil: The other question: https://github.com/tock/tock/blob/3b06ee36e312a0a57190fc5e7d919b436bc59f37/capsules/src/virtual_alarm.rs#L47 is what it means to set a client. When you set a client it takes that virtualizer, sticks it in the queue on the mux, and resets it. This means you can't ever change what the callback is for a given alarm, because it's somewhere in that queue already and the first call would leak everything that is after it in the list and then reinsert it. What this means is that set_client is not where you are inserted in the list, and instead it should be when you set a timer or alarm if you haven't been inserted already. And the important aspect of that is that some of the set_alarm calls have to take a lifetime since you're modifying data structures.
 * Amit: I'm going to have to play around with this, but you expect that taking the lifetime thing is going to work?
 * Phil: Yes. I did it with timers already and it worked. But it does mean that the signatures change. The one thing that is messier here is that because everything that implements time has a frequency and a tick, and lots of things call those, you end up with type signatures for generics that's slightly more complicated. Hopefully that's hidden in components so you won't really see it.
 * Amit: If it gets out of hand, I think it's solvable by wrapping multiple generic types in a single generic type. So you'd have one type that includes both frequency and tick. More important if there were more generics than two.
 * Vadim: Why not use 64-bit integers and everyone can look at the range and if it fits in 32-bits then use that, but if not use 64-bits?
 * Phil: There's a platform with a 24-bit one. There's basically a hardware bug where settings above 24-bits on an nRF platform don't work. One thing we discussed a lot earlier was that these generic traits will work for many different things. We may also say, "the Tock kernel expects X", which is possible. But the discussion was "no" for now. A board can just take it's 64-bit timer, for risc-v, and then can just run that everywhere. But necessitating that for a chip with only a 24-bit timer didn't seem like the right path forward.
* Phil: I've been running lots of tests on edge cases and random inputs. I've been running three alarms in parallel for like 5 days now and everything's working so far.
 
 ## Code Reivew for libtock-rs
 * Johnathan: Was considering writing up a code review policy for libtock-rs. We are currently in an ambiguous state where it's unclear how long it should hang out before being merged or when it's okay to merge it. One side is who can approve. Who needs to review a PR for it to be merged? The other side is how long we have to wait for code review. For who, I'm hoping to get some buy-in. Probably Core working group, Alistair, Woyten, and Torfmaster. I was wondering if anyone else had comments.
 * Amit: I have no objections to that.
 * Phil: I think that sounds fine. The one challenge is, certainly with libtock-rs history, is that there are a lot of people with many different perspectives working together. So there are sometimes very diverging opinions on what the principles of the project are.
 * Brad: I think having a policy makes a ton of sense. Maybe there should also be a coupled vision/goals document to make it easier to review a PR as towards or against libtock-rs goals.
 * Johnathan: I think that's a reasonable idea.
 * Amit: How does this mesh with plan to split parts of repo?
 * Johnathan: It's already split into libtock-core which has syscalls, and libtock-rs which has larger things like futures. Some CI tools like the test runner that checks QEMU also has lots of dependencies, which is fine since it's a host-side tool. I'm going to contribute primarily to libtock-core because that's what's most useful for my work. And in the PR that I'm planning with the new system call layer, I'll be introducing another crate nested in the core crate. Core will be a re-export crate and all of its functionality will be smaller crates. My goal is to completely replace the functionality of libtock-rs and hopefully we can then reverse the split and replace libtock-rs implementation with libtock-core. But that's a ways down the road.
 * Amit: It seems like one slight concern with a list of approvers is that it seems like the set of contributors for the sub parts of core are different and may have diverging concerns. In particular, Woyten and Torfmaster seem to be very excited about futures, which is great, but maybe they aren't so involved in libtock-core. And vice versa maybe Johnathan shouldn't be in charge of approving futures stuff. Potentially a solution would be to split core and futures into different repos with different merge policies.
 * Johnathan: Potentially. Or different owners for parts of the repo could be part of the policy.
 * Phil: Another solution would be to try to come together on a shared vision.
 * Johnathan: My vision so far has been hard to document because there are so many moving parts and it's been hard to explain.
 * Hudson: My perspective is that there may be a fundamental difference of opinion in what the acceptable overhead and acceptable usability is for libtock-rs.
 * Phil: Yes, but it's easier to have disagreements when they're hypothetical rather than concrete. For example, if we say 1 kB for a stack is okay and 2 kB for a stack is not, then we have a clear decision point. But if you just say, there is too much overhead for X, you need to bring it into concrete numbers. If you can first reach agreement on a quantitative value, then you can reach decisions, rather than having the value just be whatever your opinion is.
 * Hudson: I sort of thought that the design-explorations thing Johnathan was looking into did give a concrete minimum overhead that was too much in his opinion, but not too much in the opinion of others. Maybe I am mischaracterizing that.
 * Phil: My other comment would be that you have this complicated artifact that you want to communicate about succinctly, that happens to be a skill set that several people who write systems papers are good at. So maybe collaborating with Amit or I would be helpful.
 * Johnathan: Yeah, I'm still not sure we can document everything before we implement things.
 * Amit: I think one problem is that no one in Core has a plan to use futures. I personally don't like the interface and wouldn't use it even without overhead. That doesn't mean other people shouldn't be able to use it. But I think there's a difference in focus here, where I think we really do want a core libtock-rs library that doesn't rely on futures. Ideally, it should be reasonable to enable people to use futures on top of that core. But I wouldn't personally want to be maintaining a futures library since I'm not going to be using it.
 * Johnathan: I'm trying to write traits for async operations. And I think futures can be layered on top with adapters that is a future that works for any async object in libtock-rs.
 * Amit: And then futures could be a separate libtock-rs-futures library.
 * Phil: I think that might be a good path. We do not want to preclude people from using futures. And we do want to make sure futures can still be layered on top of what we do, but we don't want to use it. So splitting the repos seems good.
 * Johnathan: I think that just argues for separate crates, not necessarily separate repos.
 * Amit: Why not?
 * Johnathan: If futures is just a single adapter crate, it is probably small enough that it can just live in a separate crate in the same repo. Applications could live on top of it. In my ZST pointers doc, I had a few traits that could represent async operations. Implementing the libtock platform trait, I added improved versions of those traits. So everything in libtock-rs should be async using those traits and you can build adapter traits that takes in any object implementing those async traits and one would be a synchronous API and the other would be a futures API.
 * Amit: My sense is that Woyten and Torfmaster would want a bunch of those wrappers be in a single library. But maybe I'm wrong about that? I think maybe we don't want to maintain a bunch of futures-based wrappers in the same repo.
 * Johnathan: I think it will just be one thing in practice. Or maybe two since timers have an extra complication.
 * Brad: I think we have put a fair bit of effort in keeping kernel and userspace separate. So I think having N different rust userlands would totally be fine. The thing I'm most invested in is the user experience. Right now, it's pretty challenging to get started in libtock-rs. Pegging to a kernel release isn't great. Things that would make it easier to include things like libtock-c has to support various architectures, then I would be all for that.
 * Johnathan: I think the build system is an orthogonal issue. I think we could do build system changes now, there's just no immediate vision for doing so. I do agree that more documentation is needed.
 * Amit: I've started drafting an issue for libtock-rs where I'll post my vision based on what we've discussed. I think the code review policy at the core of this is good. I'm slightly concerned that because there are differing aesthetics, we might unnecessarily block on or burden reviewers, but we can deal with that when it happens. Setting out a shared vision, which I'll jump start, will hopefully help.
 
## Bluetooth
 * Alistair: I've been working on Bluetooth for the Apollo3 in my spare time. But something came up internally and now WD is interested in Bluetooth on Tock. So there's some effort on that now. So first, can I invite those people to this call?
 * Amit: Yes. Although this call is starting to get really big, so we should be careful.
 * Alistair: The plan is to use Rubble, the rust bluetooth stack. It's a lot of work and reimplemnting it doesn't make sense. But maybe rubble can be a capsule with a shim layer to make it work. But then tock would have rubble as a dependency? Which could be bad. But also it would be great to run that stack in the kernel. But the disadvantage is that running a bluetooth stack in the kernel is a security concern, because it's a big attack surface.
 * Alistair: We could alternatively run rubble in userspace, and have it make system calls to handle things.
 * Amit: I agree with the premise. Bluetooth is a complicated, multi-layer protocol and testing it is hard, so relying on a working implementation sounds good. From what you said, I don't think either choice would be unacceptable. I certainly don't see a reason that a large library that was hand-written for tock would be more trustworthy or less dangerous than one not originally written for tock. But we do need to assert that there's no unsafe there.
 * Alistair: We can of course tie it to a known, trusted release of rubble.
 * Amit: I think we would want the disallow unsafe pragma for it's library. Somehow. I think it would be better to have it in the kernel than a process. Because architecturally that's where it fits better, like 6lowpan. And process communication is tricky at best in Tock. So if multiple processes should us it, it's best in the kernel.
 * Brad: I agree that kernel would be nice. Because I want to use it from libtock-c.
 * Alistair: Well, you could have IPC or kernel calls to make any app language work.
 * Brad: That's hard in practice though. It has to be compiled for the right addresses and MPU ordering and tockloader and whatnot.
 * Amit: Unless doing it in userspace is way easier, then kernel might be good.
 * Brad: If it is in the kernel, what would the syscall interface be? Would that need to be created from scratch?
 * Alistair: Unclear so far. It's still a pretty new thought.
 * Amit: GATT could be a pretty clean interface in Bluetooth, actually.
 * Alistair: And then in the kernel using things like AES might be easier.
 * Amit: I think a big chunk would have to be in the kernel anyways. In the nRF for example, there are hardware-specific shortcuts to get timing right.
 * Alistair: And you think it would be acceptable to someday include rubble in the kernel?
 * Brad: I'm still not sure about having dependencies. I think it's pretty hard to track unsafe stuff.
 * Amit: Yes. If we added a directive removing unsafe, that would remove that concern.
 * Alistair: We might even be able to merge that upstream.
 * Amit: I suspect that some of rubble has unsafe to handle hardware, but we wouldn't use those parts.
 * Alistair: Yeah, we just want the stack.
 * Amit: Yeah, looking through unsafe is only in the nRF stuff and the demos.
 * Alistair: So seems plausible.
 * Branden: Moreover, I think it's something that putting some time into figuring out would be good. This isn't going to be the only time we run into there being Rust projects that would be nice to have in the kernel. There are now rust filesystems and things like that which we are interested in bringing into the kernel. And having a path forward for that kind of stuff would be really valuable, although also pretty hard.
 * Alistair: Okay, we'll write something up when we have some more progress.

