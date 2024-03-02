# Tock Core Working Group Meeting Notes 01/26/24
================================================

## Attendees
- Branden Ghena
- Hudson Ayers
- Leon Schuermann
- Tyler Potyondy
- Brad Campbell
- Andrew Imwalle
- Alyssa Haroldsen
- Pat Pannuto
- Alexandru Radovici


## Updates
### OpenThread Porting
* Tyler: From OpenThread porting, some good progress. Leon and I have been working on getting OpenThread in Tock. Leon is doing encapsulated functions, while I'm doing libtock-c. Both in parallel, to find something that works. We successfully fixed some faults for OpenThread as a libtock-c application. We still need to implement the abstraction layer for controlling the radio.
* Tyler: I was curious what people's thoughts were on doing a PR for the state we're at now. It runs and doesn't crash, but doesn't actually _do_ anything since it doesn't control the radio.
* Brad: PRs to libtock-c? Or Tock? I think big commits are fine.
* Branden: And if this is really self-contained, it matters even less. Plus, it would be good for a PR to actually work.
* Tyler: Okay, we'll stay in a branch for now.
### Kernel ELF Files
* Andrew: Tock's ELF file doesn't quite follow the correct ELF format for entry points. Neither Tockloader nor Tock-bootloader load the ELF file, they use the bin. So we're going to submit a PR to fix that. This is in the kernel.
* Hudson: Why does this matter for you?
* Andrew: We're using the ELF file for loading rather than the BIN file. So we want the entry point in the ELF file to meet the standard.
* Leon: Will this amount to changes in the linker script?
* Andrew: Not sure. I'm not actually doing it.
* Branden: We don't care about the BIN at all right now?
* Leon: I think one of Alex's boards might use it maybe?
* Andrew: So this PR will be coming soon.


## Exit syscall on process finish
* https://github.com/tock/libtock-c/pull/246
* Leon: This is a good change and something we should do. It does however break our API in a major way for downstream users.
* Leon: So I wanted to talk about what guarantees we provide for libtock-c and whether this should be a major release for libtock-c
* Brad: I'm not aware that we have any guarantees at all about libtock-c
* Hudson: So releases of libtock-c right now are related to the Tock version that they target. So it's sort of misleading that we have libtock-c on version 2
* Pat: The two releases have historically been simultaneous to ensure that it works in union
* Hudson: So we really haven't promised anything about libtock-c internal APIs
* Alyssa: Not really internal here. It's pretty visible
* Hudson: True. We should consider how this impacts people and give them advising about it
* Alyssa: Put it in the release notes at least
* Leon: That's good for me. I'll put this in the PR notes
* Hudson: And I think that's what we've done historically

## PMP redesign
* https://github.com/tock/tock/pull/3597
* Leon: I just don't know what the best step forward is. It's a pretty major change to what we should consider a very important subsystem. But it's a LOT of changes. Giant PR. So when do we consider this good to go, review-wise?
* Hudson: There are some reviews here. Chris, Alistair, Brad.
* Leon: Yes. And there have been OpenTitan tests on this PR.
* Hudson: I agree that you're unlikely to get other in-depth reviews on this large change. Given that we know there are problems and that this fixes them, I think we could move forward.
* Brad: I think this is a case where the most-qualified person is the person who wrote it. So we should go with it.
* Leon: It's always easy to say that your own code is good. I do think we have sufficient evidence that the current code is broken and this new design fixes those issues. There may still be issues, which we'll find in the future.
* Alyssa: My initial look indicates it's using references to IO memory, which sucks. Oh, it's actually references to statics and using those as pointers. But the statics have a size of u8.
* Leon: You're looking at the constructor for the board? What's happening there is that to set up memory protection properly, we need to know where the regions are from the linker script. We use those values purely as integers.
* Alyssa: I'd be more comfortable if we never made a reference to it. The only change I would make to fix that is "core pointer address of"
* Leon: This is copied from how we do process loading. Should probably fix everywhere, but it's in ALL boards, so we should make that a separate PR.
* Alyssa: Agreed. Rust's memory model doesn't work well with situations where the pointee is smaller than the actual size you're working with. Raw references are better for that.
* Leon: Okay, that makes sense.
* Hudson: Yeah, let's make an issue about that. But not hold up this PR on that.

## Cortex-M Hardfault Handler
* https://github.com/tock/tock/pull/3798
* Leon: A lot of Brad's PRs are blocked on this. And it feels like a significant issue for our threat model. Issue is that I can't really test any Cortex-M0 changes.
* Alex: We have pico debuggers and I can help you with that. Let's take it offline.
* Branden: So you just need to test on an M0?
* Leon: Right. I tested on an M4, but I don't have an M0 right now.
* Alex: I will do that. Just message me
* Brad: If we need to, we _could_ separate those two changes.
* Leon: The faster the better
* Pat: I will take a look ASAP

## Documentation Working Group
* https://github.com/tock/tock/pull/3815
* Brad: This would create a documentation-focused working group. The motivation is that I want the ability to merge a bunch of documentation changes on a much faster basis, rather than waiting on Core to look at them. This would oversee the Book and general repo documentation
* Hudson: Yeah, this seems good. From the PR, I also agree that it doesn't make sense for this to handle TRDs or Working Group call notes.
* Brad: Definitely agreed. I updated the PR to clarify that.
* Leon: It's still kind of vague here. Not spelled out clearly what the burden is to merge a documentation PR.
* Brad: That's a good point. If we could do that, it doesn't seem like we'd need to wait so long on current PRs. So it's kind of hard to say what "acceptable" documentation is
* Leon: Yeah. I think I'm onboard
* Leon: So what do we formally need to do to establish a new working group?
* Branden: I think there's a write-up on how to form working groups. We just did this for Networking group
* Brad: Seems like something the Documentation WG could answer!
* Branden: Call for concerns? (No concerns raised)
* Leon: I'll mark on the PR that we approved today

## Obsolete ProcessLoad Variants
* https://github.com/tock/tock/pull/3805
* Brad: This really just removes two lines of code. It removes two process loading enum variants that we aren't actually using. So it does raise the question of _why_ we aren't using them. It feels like you should want to know _why_ it isn't working. But I think because process loading is asynchronous, we don't have an obvious way to return them.
* Brad: So, it kind of makes sense to remove dead code. But also this feels like a different issue of completeness/correctness. So I want to look a little deeper at the boot process to see if there's a better way to express where things are failing. So my impression is that this is an in-progress thing.
* Hudson: We should tag Phil on this
* Alex: Where this is coming from: we're trying to write specifications for Tock code. But we found this example that cannot be specified.
* Brad: Because you don't know where they come from?
* Alex: Yes. Every line of code has to have a requirement. And there's no clear requirement for these lines of code. So we need to make it more clear

## Handling PR Reviews
* Brad: Generally, my question is what do we do with PRs that no one is looking at?
* Leon: For this, I don't think many people feel comfortable judging whether a change is correct or proper for this subsystem. I have no idea how credential checking works, for example.
* Pat: So does that just put us in a position where this code is not robust? Maybe we need to have a default person from Core who needs to review each PR, and needs to go learn it if they don't know it.
* Alyssa: A review rotation could be useful. If you're not sure, you could pull someone else in or learn it. But at least pointing out _someone_ responsible would be nice. So PRs don't just sit around with no ownership.
* Hudson: It would be nice to have the review rotation apply after N days have passed without anyone reviewing. So the appropriate people review in many cases, but then _someone_ is dragged in if no one volunteers. Not sure how to automate this
* Pat: We could manually do this on Core calls for now.
* Alyssa: How does Rust do review rotation? Something like GWSQ (from google). You assign it as the reviewer and it assigns someone else. It might round-robin but there might be other strategies too
* Leon: I just found a github action that does this. I could play with it and modify it to assign people
* Hudson: I think that would be a reasonable approach
* Branden: I think what we're doing now is not working. So trying something at least would be beneficial
* Alyssa: `triagebot` is the tool


## Signature Credential Checking
* https://github.com/tock/tock/pull/3772
* Brad: This is a step in the right direction, but not the end yet. It's certainly easier to reason about as 100 lines of changes instead of a lot more
* https://github.com/tock/tock/pull/3793
* Brad: This is the same thing. A minor issue for credential checking
* Branden: This actually has reviews. So this is just a situation where we need to click the button
* https://github.com/tock/tock/pull/3807
* Brad: #3807 is also in the exact same situation
* Leon: Merged that one

## AppID
* https://github.com/tock/tock/issues/3813
* Brad: I sent this last week, but realized it would be helpful to write things down more first, which is this issue. So what I'm proposing is that credential checking, determining if a process is valid to run, should be separate from assigning an identifier for the process. This are sort of tied together because of use cases. So there's some general ambiguity here. But when you actually try to use this, credentials are great for checking hashes and signatures, but they're not great for identifying what app this is. Because if you change the code the hash changes, but the application ID changes the same. And when you check these, you end up copying around a bunch of code, because you might change the hash method, but the naming method is often the same.
* Brad: So in code, I think we should remove this trait and decouple the two operations. That's my proposal.
* Branden: But we'd still have a way to swap out either?
* Brad: Yes, each board could chose either independently
* Pat: I vaguely remember wanting to protect some AppIDs, so you can't do bad things?
* Brad: I definitely want identifiers to happen AFTER credential checking. I do think this is an important question though. Is there a reason that a non-credentialed app can't have an identifier?
* Pat: I think we want non-credentialed apps to have an identifier for debugging and development purposes. Blink and whatnot.
* Brad: Today you can implement that sort-of. The logic requires that you have _some_ credential. But that could be padding and you could write your checker to approve padding as a valid credential, then give it an ID. I'm not sure any of that was an intentional design decision.
* Pat: Part of this is probably okay, because if you have a board where you're worried about protecting AppIDs, then you could. It's just a think that board authors would need to know about...
* Brad: Okay. Good point. We could make that more explicit. If implementing the ID assignment aspect had an indication of whether it requires valid credentials, that could fix that. We could make it explicit in the way that the trait is designed.
* Pat: That seems reasonable
* Branden: This makes a lot of sense to me. Decouple the two of these but have an explicit ordering for them.
* Brad: You definitely don't want people to think that they can put the AppID in the footer, because you can't check it. We realized that's not going to work

## TockWorld
* https://github.com/tock/tock/pull/3806
* Brad: I have a PR that would put the blurb on the README. Any thoughts?
* Pat: Sounds good. We should advertise aggressively.
* Leon: As long as you feel we're ready to advertise, I'm good about this
* Pat: The ticketing part is still a to-do for me. Hoping to have that done very soon.
* https://github.com/tock/world.tockos.org/pull/2
* Brad: Also planning the days. Day 1 would be core group, so people on this call. Day 2 would be broader tock discussions. Day 3 would be a tutorial. So I want it to be clear that it's not a secret or invitation-only, but I want it to be clear that only people who really _want_ to be there should. So I was trying to clear up that intent.
* Alyssa: This text looks good to me
* Andrew: To me as well
* Alyssa: So the first day is targeted at contributors to Tock
* Branden: And the second day is users, or interested parties. Anyone.


