# Tock Core Notes 2022-10-07

Attendees:
- Adithya Anand
- Alexandru Radovici
- Alyssa Haroldsen
- Brad Campbell
- Chris Frantz
- Jett Rink
- Johnathan Van Why
- Leon Schuermann
- Pat Pannuto
- Phil Levis
- Vadim Sukhomlinov

## Updates
 * Phil: My one update is that this is my first week at Google -- I'm on leave
   at Stanford -- so I need to figure out what that means for my participation
   in decision-making processes.
 * Pat: Looking at PRs merged in the past week, there was an OpenTitan one, and
   a lot from Brad in a big clean-up effort. I think we'll talk a bit more about
   that here. We have a pretty deep backlog of stuff that is ready to go.

## App ID
 * Phil: I think app ID is in the final stretch and has an approval from Brad. I
   want to go through documentation one more time and clean up. There is one
   change since last time, a structural change. There is "app uniqueness", which
   the way that a particular identifier policy can tell the kernel "are the app
   identifiers for this policy different". It allows you to make it pluggable
   what exactly an app ID is. It used to be that the trait also had a method
   `has_different_identifiers`, which given a process and an array of processes,
   will tell you whether they're unique. There was a default implementation of
   this in the trait and the intention was that nobody would ever reimplement
   it, in part because it checks the short IDs, and Hudson suggested that it
   should be moved into the kernel crate so nobody can accidentally violate
   kernel guarantees. That method has been pulled out of the trait and made a
   kernel function.
 * Pat: I guess the biggest point of action is to ask "what stands between us
   and merging this in"? Should we all review over the next week?
 * Brad: I mean, it's been open for a year and a half, so we're had time to
   look. In my mind, we're just waiting on Phil. If you want to look at it, you
   have until Phil gets these last nits worked out.
 * Pat: Okay, I'm not going to fight that. I've reviewed most of it and am happy
   with it.
 * Phil: After the merge we will have time with it in the tree to iron out
   kinks.
 * Brad: Since you're looking at documentation, would you mind making a note in
   the Imix board about how to get an application that will work.
 * Phil: Okay.
 * Pat: I think it's basically ready to go, lets see what happens and how it
   shakes out.
 * Phil: I'll go over the documentation today to make sure everything is
   consistent.

## #3252 and #3258 (Allow notifications)
 * Alexandru: Recap of last time: the problem we faced is it is impossible for a
   capsule to determine if a buffer was swapped beneath it, which prevents
   streaming use cases. After a process swaps the buffer, the capsule doesn't
   know, so it will write to the incorrect index. We could have the process
   remove the buffer, send a command, then return the buffer, which risks losing
   packets. I submitted two PRs. One lets capsules be notified of swaps, which
   has the disadvantage of allowing capsules to take action on Allow, and adds 2
   kB to the kernel. The second used the MSB of the buffer's length field to
   indicate whether the buffer was already accessed by the capsule. Downside is
   it limits the length of the buffer to half of the possible RAM, which is
   probably not an issue for Tock. While looking at the size increase, I think I
   changed how the kernel searches for drivers, which decreased the kernel size.
   I sent that as a separate PR (#3276), which reduces the size by about 800
   bytes. Compared to #3276, the notification PR adds about 260 bytes.
 * Pat: All of that length manipulation is in the kernel, none of it is in
   userspace, correct?
 * Alexandru: Userspace shares a buffer. The kernel, before allowing the Allow,
   is checking whether the first bit is set. If the first bit is set, it rejects
   the Allow due to size. Instead of using the length in the kernel, I used a
   local register. The length is not the length anymore, it has 31 bits for the
   length and 1 to indicate if it has been accessed. In the capsule, it can
   access a public property of the structure that is set to true or false.
 * Pat: This *is* technically a change to our syscall API because you can't send
   a length with the top bit set. The reason I was thinking about this is, if we
   are going to make a change like that, should we reserve the top X bits for
   future use?
 * Alexandru: That works. In preparation for this I reserved only 1 bit, but the
   structure that reserves the attribute is a non-exhaustive one.
 * Jett: Can the capsule prevent an un-Allow?
 * Alexandru: No, the capsule can know whether the buffer has been swapped since
   its last access. If the application does it right you would know.
 * Leon: It is important to that when we say "new buffer", we don't refer to the
   contents, but to the location and size.
 * Phil: On one hand, I like it because it's a different way of doing it and
   it's nice to have multiple. However, a capsule needs to check whether the
   buffer was changed on each access.
 * Alexandru: No, capsules don't need to check if they don't care.
 * Phil: Yes, but it puts the onus on the capsule to insert these checks
   whenever they care.
 * Alexandru: Yes. The second approach prevents the capsule from taking action
   in response to a buffer change.
 * Brad: The first pull request, all that did in response to the notification is
   set a flag, right?
 * Alexandru: It allows the capsule to do something.
 * Brad: What does the actual implementation do? Was it not representative of a
   real implementation?
 * Alexandru: It just calls a function in the `SyscallDriver` trait.
 * Brad: It, in this case, is the kernel. I'm asking about the implementation of
   that function in the touch capsule.
 * Alexandru: It was just setting a flag.
 * Brad: If we were to add that notification function, is that they way it would
   generally be done?
 * Alexandru: Before that, the application had to acknowledge it read the buffer
   and has a new one.
 * Brad: What's confusing me is it seems like that implementation required the
   capsule to check if the buffer was acknowledged. The same check as in the
   other implementation.
 * Alexandru: Yes. There's no difference there. There's a bool in each case. In
   the case of say a network driver, there would probably be an index that would
   be reset to zero.
 * Brad: That I think is the key. Phil, when you're describing the difference,
   you're imagining that the notification function would reset the variable, so
   there wouldn't be a check.
 * Phil: Exactly, in that particular use case. To me, the tradeoff here is that
   from an ergonomics standpoint the notification is much better, but it has the
   drawbacks of allowing people to use it for things other than what's intended.
   Can prevent that in the mainline repository, but not elsewhere. I imagine the
   notification will result in far fewer bugs.
 * Leon: If an application issues an Allow call with the same buffer, should
   that be counted as a new buffer?
 * Alexandru: If the application does one single Allow and shares the same
   pointer, it would be set. I could add a check to not reset it for the same
   address. If the application would replace it with a null buffer and reset it
   that would count as new.
 * Leon: I recall in the early 2.0 syscall discussions, we discussed the fact
   that the new buffer types still expose the address of the buffer as well as
   the length of that buffer. Currently, it is technically possible for an
   capsule to distinguish the cases where the buffer remained the same or
   changed, but it is not sufficient for the capsule to know whether any Allow
   call was issued in the meantime. Is that an issue for you?
 * Alexandru: Probably, yes. I mean, if the application was super fast, and I
   never got a buffer before. Imagine this: I filled up a buffer, set it to the
   application, didn't receive any new packets, the application did an Allow,
   didn't swap it -- just read it, then swapped it back, I would never reset the
   state.
 * Leon: That makes sense for that kind of application. I was trying to figure
   out if we could try to reduce the number of applications of this mechanism to
   just care about actual changes of buffers in memory. But yeah, if that's the
   kind of use case we want to support then I realize that this is not
   sufficient.
 * Alexandru: The pointer is a private field.
 * Johnathan: Maybe I missed something, but in the case you are trying to stream
   data into an application and the network, couldn't the application swap
   between two buffers atomically and the capsule can check the address to see
   if they've been swapped?
 * Alexandru: That would work, but do I get the address in the capsule? I might
   be missing something, but that was a private field in `ProcessBuffer`.
 * Johnathan: I think you get the address in the capsule, yes.
 * Leon: It's a private field, but you can dereference and get its address. We
   cannot prevent that. I think what you said about the fact you can re-Allow an
   identical buffer whose contents have been changed is a compelling argument
   that this is not sufficient.
 * Alexandru: The application could swap it with a new one, do some processing,
   then swap it back, exactly the same. The next packet I get, I will get the
   same address. So I won't know the application went through a cycle of Allow
   Null, re-Allow the buffer. We will submit a CAN example with double buffering
   which uses this. Indeed we have two buffers. But for instance for the touch
   driver it could have only one if it expects a lot of touches.
 * Johnathan: It seems like the double buffering mechanism works for every use
   case unless performance is an issue, and if performance is an issue we can't
   decide between these approaches without benchmarking anyway. It seems like
   we're adding a controversial mechanism to make a rare use case a lot more
   elegant. I'm not really sure what that means for the PRs, though.
 * Phil: I feel a little differently. The idea that after you change a buffer
   you may have to issue a command, is leaking information about the capsule's
   implementation. In that way, that's not a great approach, which pushes us
   towards something that happens automatically in the kernel. Which mechanism
   is a separate question. Even outside high-performance implementations, just
   knowing that it is a different buffer is important.
 * Alyssa: I guess I'm trying to understand why this couldn't be a data
   structure that capsules keep alongside their other data, store the pointers
   with the last range.
 * Alexandru: Imagine the following scenario. You have a buffer, and fill it up
   to some point, and the application doesn't have another buffer. It swaps it
   with null, no events come in, the application reads the buffer, then the
   application swaps it back in. Then the capsule cannot detect that the buffer
   was read to reset.
 * Pat: This is only a problem because there's no coordination between the
   capsule and the userland about the state of the buffer. The two could
   coordinate, yes.
 * Alexandru: Through Command, yes.
 * Pat: If there are head/tail pointers in the buffer, then userland could
   indicate what was read.
 * Alexandru: That would require much more code than simply having a
   notification.
 * Pat: That goes back to Johnathan's question of "how common is this?". Is this
   an idiom the kernel needs to support, or a specialized case? I don't have a
   good feel for it.
 * Alexandru: For instance, if I want to do aligned reads I need to lose four
   bytes from the buffer.
 * Alyssa: 0-3 bytes.
 * Pat: I guess the other thing is that the way this is designed right now,
   everyone who doesn't care is paying the overhead of maintaining the extra
   state. I'm probing at the design space here.
 * Leon: I think Pat's comment is great as to the impact of this on capsules
   which don't care. The design challenge we're facing is the fact that on one
   hand we have this generic grant type w/ capsule-specific functionality, but
   on the other we don't have that information in the syscall handler. Would
   need to encode that information in the grant structure. Can't make it
   zero-cost.
 * Pat: Could maybe set it as a property of the process.
 * Leon: Yeah.
 * *[33 second silence]*
 * Pat: Alright, so there's a lot of kind of dead air of uncertainty here. Boils
   down to a non-trivial ergonomics improvement for some use cases with some
   overhead for all use cases. Where do we sit on that tradeoff.
 * Alexandru: For #3252 I counted something like 260 bytes extra, after kernel
   refactorings. I'm sure it's super correct, and I sent a PR with only that
   change. For the second, I haven't sampled the code size yet, but I'll do
   that. The penalty should be minimal -- checking and resetting one bit. If the
   capsule doesn't care the capsule code shouldn't change.
 * Jett: I hadn't seen the PR yet. Can we add this to the `KernelResources`
   trait or some other mechanism to make it customizable, or is it too
   integrated for that?
 * Alexandru: At least for the attribute it would be strange, as it wouldn't be
   clear what the longest length you could have for the buffer.
 * Pat: I suspect for the attribute case the impact will be no more than 100
   bytes.
 * Jett: That amount of space doesn't seem worth trying to customize out. The 2k
   number is quite a lot to pay.
 * Phil: Yeah 2k is a non-starter.
 * Alexandru: If you compile it now it's less than `master`.
 * Phil: You're talking about #3258
 * Alexandru: I'm talking about notifications. If you submit a driver number to
   an Allow/Subscribe that is invalid, you get invalid rather than nodevice. The
   only one that returns correctly is Allow Userspace-Readable. Every function
   was searching for the driver, which costs 260 bytes. I moved the driver
   search before the syscall dispatch. Reduces by 800 bytes on ARM and 400 bytes
   on RISC-V compared to `master`.
 * Leon: Regardless of which approach we take we could take advantage of that
   optimization, right?
 * Alexandru: Yes. That is why I sent the third PR.
 * Johnathan: Does this change the error returned by kernel if userspace tries
   to call a capsule that doesn't exist?
 * Alexandru: It returns `NODEVICE` as it would before automatizing these ones.
   It should return `NODEVICE` if the driver does not exist.
 * Phil: That's right.
 * Johnathan: This might've been a spot where I coded `libtock-rs` against the
   implementation of the kernel. I do recall finding something -- although I
   thought it was a Tock 2.0 thing not a 2.1 thing -- but I found something
   where TRD 104 did not match the implementation. I forget if I updated TRD 104
   to match but it was after stabilization so I didn't think it could be
   changed, I'll have to see.
 * [Ed. note: see the below "chat conversation on the side" transcript, as
   Johnathan's point above was continued in chat alongside the call]
 * Alexandru: I have the error there, I can put `INVAL`. Technically the TRD
   would not be correct -- Command would return `NODEVICE` but Subscribe would
   return `INVAL` for the same incorrect driver number.
 * Phil: Yeah it should return `NODEVICE`.
 * Alexandru: If my understanding of the kernel was correct it was not returning
   `NODEVICE`. The kernel would fail to allocate the grant which would result in
   `INVAL` or `NOMEM`.
 * Phil: I think we're not looking at a 2kB code increase. I really do lean
   towards the notification approach with the caveat that Brad's point is right
   -- we want to make sure this is narrowly crafted so it is used for what it is
   meant for.
 * Pat: So is that a way to move forward from here. One, lets look over #3276
   relatively quickly -- it looks like a good change. Get that in, then
   Alexandru can revamp the notification one to get size numbers, and explore
   how to change that interface to limit the scope of what it could do.
 * Phil: To Brad, because you're the one who raised the concerns about what this
   interface can do. What are your expectations and thoughts?
 * Brad: Where I am at is Rust always seems to have these magic ways to do
   things that I would never have been able to come up with on my own, and
   sometimes it doesn't. I think the intent is that this callback should only
   modify state in the grant region, that covers all use cases. Ideally, you
   would get some sort of closure-like thing where all you have access to is the
   grant region. I don't know how to do that. Make it not be a generic
   notification like Subscribe used to be, make it more narrow. If there's not a
   realistic way to do that, we need to look at documentation approaches to make
   it clear. Easy to check during code review if a function only modifies the
   grant region.
 * Leon: I would like to think that we could have it execute something that is
   only passed a mutable reference to the grant region. That seems conceivable,
   but maybe not -- perhaps talk offline.
 * Alexandru: The problem is if you enter the grant it will pay a higher penalty
   than calling an empty function. I'm not sure, need to think about it.
 * Johnathan: [Brings up the error code mismatch from chat]
 * Pat: We should probably just put this in an issue, because this is an issue.
 * Johnathan: Yeah
 * Phil: Definitely that looks like a bug.
 * Johnathan: This gets back to the conversation of "how do we handle
   implementation/doc mismatches". At the time, I didn't even think it was a
   question of possibly changing the implementation to match the documentation,
   but we've had more discussion since.
 * Alexandru: From the [missed] point of view, I'm most probably sure the user
   will expect a `NODEVICE` if there's not a device. At least for debugging,
   this will be a nightmare if somebody messes up a driver number.
 * Pat: This is probably a longer discussion we should discuss on the issue, but
   from the perspective of semantic versioning this is a bug and a bugfix would
   be a minor point fix, even though it is changing what the ABI does. Gets a
   little messy. A longer conversation than we're prepared to have now.
 * Pat: With that in mind, I think we have a few action items moving forward.
   Want more precision on scoping for what notification does, and Alexandru can
   explore some code and see what happens there, as well as trying to isolate
   the performance and size impacts. This issue we've surfaced in chat,
   hopefully Johnathan you can open an issue and we can discuss it offline then
   it will be a chunk of next week's issue.
 * Alexandru: That's okay for me.
 * Johnathan: Okay, I'll post it.

### Chat conversation on the side
This conversation happened in chat alongside the above conversation:

 * Johnathan: This is what I was referring to in libtock-rs:
   https://github.com/tock/libtock-rs/blob/1d785a043a95d83b410f6a099a6121fc101ca3b7/unittest/src/fake/syscalls/subscribe_impl.rs#L76.
 * Johnathan: Subscribe returns NOMEM if it is called with a non-existent driver
   number
 * Pat:
   https://github.com/tock/tock/blob/master/doc/reference/trd104-syscalls.md#42-subscribe-class-id-1.
   Reading very quickly, it looks like the TRD specifies what is done about
   invalid upcall, but not invalid driver number?
 * Pat: though maybe that's earlier..
 * Johnathan: IDK what TRD 104 means, but that's what was stabilized in Tock 2.0
 * Pat: If userspace invokes a system call on a peripheral driver that is not
   installed in the kernel, the kernel MUST return a Failure result with an
   error of NODEVICE. If userspace invokes an unrecognized system call on a
   peripheral driver, the peripheral driver MUST return a Failure result with an
   error of NOSUPPORT.
 * Pat: That's in §4.0 of TRD104
 * Pat: seems like we do have an implementation / documentation mismatch? :/
 * Johnathan: Yes

## Migrate `ProcessSlices` towards raw pointers #2977
 * Leon: I've been looking at an ancient PR of mine, #2977, which tries to
   migrate `ProcessSlices` towards raw pointers underneath. I think it's an
   important issue to solve but tons of things have come up and I haven't had
   time to get to it. `ProcessSlices` are currently built using `[Cell<>]` which
   is unsound in terms of Rust's aliasing rules, and the only sound solution
   seems to be to change the types to use raw pointers and use Rust's methods
   for working on raw pointers. I think there is general agreement this is
   useful and the API seems to be good. As part of the PR, we wanted to
   transition as many capsules away from panicing on userspace buffer accesses
   and perform more sensible error handling. I've spent a day rebasing it, and
   reworked tons of capsules to refactor them. The lesson from that is we should
   punt on converting capsules away from panicing accesses to a separate PR,
   because it is hard to do that without changing their behavior on edge cases.
   E.g. when a process asks a capsule to perform an operation that goes beyond
   the bounds of the buffer it has shared. That's kind of the state of that, I
   think the only thing missing is Miri tests, which I'm currently writing, to
   have some confidence that what we're doing is sound. Does that make sense to
   everybody?
 * Pat: I think so
 * Phil: It makes sense to me
 * Alexandru: Yes it does
 * Leon: The takeaway is I'm working on it again, I didn't forget it, it's just
   a lot of work and as soon as we get the tests I propose to try to get it in.
   Then work through capsules one-by-one to change them to sensible error
   handling.
 * Pat: It's a longstanding pain point so I'm onboard with getting this to
   happen.

## Component update PRs
 * Pat: On the subject of large mechanical changes, Brad has a small army of
   component PRs. I reviewed a bunch, tagged them last night. We'll probably
   merge them now, unless someone has a reason not to, we should probably get
   those in.
 * Phil: I `bors r+`-ed the basic one.

## Re-entrant interrupts
 * Jett: Does Tock OS ever have plans for or against having re-entrant
   interrupts? Like re-entrant M mode interrupts?
 * Leon: You should talk to Amit about that, he has some plans there.
 * Jett: Alright
 * Phil: It's useful to note that Tock's interrupt handlers are minimal -- they
   wake the system and let the bottom half in the kernel loop do most of the
   work. That's mostly because interrupts and Rust's safety do not play well
   with each other. Are you referring to reentrant interrupts directly being
   handled by kernel code, or do you mean in the current interrupt handler?
 * Jett: I do mean the top half ones. We have two scenarios where we have to use
   top half handlers for performance reasons. We have a requirement where we
   have to act within 10 microseconds, and one of the two takes longer, so we
   would like to give the other a higher priority, it would interrupt it and
   preform its small task.
 * Phil: If we were to wind back to the beginnings when we were first sketching
   this out, I think the statement would be that all of that is outside Rust and
   the core kernel, and it was always intended that you could do stuff like
   that. You would be in assembly land, and all bets are off -- you have to do
   it right.
 * Jett: Sure, and I definitely agree with you that you have to be careful with
   Rust and all that stuff in a fast interrupt handler. We're taking care of
   that, we just want the reentrant part. It looks close, but there are some
   things like `mret` that we don't save. I tried a few things, and couldn't get
   it to work, wanted to see what the general feel ways. Sounds like I could
   talk to Amit about it.
 * Phil: Sounds right
