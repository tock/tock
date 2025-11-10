# Tock Meeting Notes 2025-11-06

## Attendees
 - Branden Ghena
 - Amit Levy
 - Leon Schuermann
 - Hudson Ayers
 - Brad Campbell
 - Johnathan Van Why

# Agenda
 - Updates
 - Forbid Unsafe in Chips

## Updates
### Network Working Group Update
 * Branden: Talked about Treadmill Thread tests, PR for that. https://github.com/tock/tock-hardware-ci/pull/39
 * Branden: WiFi PR has been updated and needs reviews: https://github.com/tock/tock/pull/4529
 * Branden: Discussed a path to move forward on IPC documentation
### Crypto Working Group Update
 * Amit: Met for first time. Going to be on Fridays probably every other week. Articulating problems. Figuring out path for redesign.
 * Amit: One noteworthy thing is that crypto might really want shared libraries in userspace. We'll put some thought into that.
### Libtock-C CI Broken
 * Leon: CI on libtock-c RISC-V is broken. It's doing something weird and I don't understand it. It would be great to have someone knowledgable take a look

## Forbid Unsafe in Chips
 * https://github.com/tock/tock/pull/4626
 * Amit: specifically, extracting unsafe parts of chips into a separate crate
 * Brad: Long-standing thought that while chips do need to use unsafe code for some valid reasons, most chip code is completely safe Rust. The MMIO mapping is one obvious unsafe use.
 * Brad: But we've never enforced not having unsafe. So we find in practice that people tend to sneak it in accidentally. So we actually have unsafe all throughout the chip crates and it's not trivial to remove.
 * Brad: One option would be to try to fix this moving forward. Fixing old stuff would be tough. But I made an example of nRF5x and pulled the unsafe into a separate crate. This PR is kind of big at this point, since it did that. So I want to figure out what people think and how to make this PR understandable by others.
 * Amit: Reviewing a PR with a lot of changes that more-or-less do things from a design document is easier than one that's equivalently correct but there is no already-established design.
 * Amit: One question, if we're going to encourage a pattern of which things are or aren't in unsafe crates, we need to think carefully about what it means to expose a safe interface. DMA registers is a classic example of this. Where it is mechanically trivial to expose a "safe" interface to MMIO registers that is actually quite unsafe.
 * Leon: I think overall these are nice changes we want. But I'm specifically worried about details like this where if we forbid unsafe entirely it can encourage "false safety" where APIs are actually still unsound underneath.
 * Leon: For the discussion on a DMACell, that would still require unsafe calls. So I think having chips actually forbid unsafe would be a really hard challenge. We could instead have a policy of strongly discouraging it, with some specific exception patterns.
 * Brad: First, DMA has always been a thorn in our side, but right now we're just ignoring the safety issue. That problem is there today so whether you allow unsafe in the crate doesn't change the issue.
 * Brad: Second, I can see how we could put the DMA stuff in the unsafe crate. But I don't think we can have code at-scale in an unsafe crate and manage that. That's how we got where we are now.
 * Leon: What I'm worried about is that the reason we can propose a PR that forbids unsafe in the nRF chip is that we have buffers in TakeCells that we use for DMA. That is unsound and we will need to replace this eventually, just no one has gotten to it. What I'm worried about is that there is no possible safe solution that we could replace unsound TakeCells with. So we could forbid unsafe now, but then to use the future design, we'll have to undo that.
 * Amit: On what Brad's saying: we have this issue already and aren't changing it in this PR. Right now, the way we're using a TakeCell to store a reference to the original cell is not canonical safe rust. I think it is sound because of how we use it. The whole crate that knows the semantics of the hardware is ensuring we use this in a sound manner. It only extracts something from a TakeCell when it knows the DMA operation is done, for instance. My point is that the boundary about what enforces soundness is murky and beyond uses of safe and unsafe. And eventually if we solve the DMA issue, that will break whatever safe/unsafe boundary we make right now
 * Leon: It gets even trickier because TakeCell is unsound for keeping a reference across DMA. We could make a change to prevent mis-compilation but not stop a chip from abusing the API. But the way we currently architect chips, it'll be virtually impossible to have a type that's entirely safe as used by the Chips crate.
 * Brad: At some point we end the chain of unsafe, right? Capsules are forbid unsafe. So at some point we have DMA but want to transition to safe code. So somewhere it's possible to make the leap to safety. Based on that hypothesis the line is somewhere between chips and capsules. But we could push that down into chips, right?
 * Leon: The problem is that it's the driver for a peripheral that has a semantic understanding of when hardware will access a buffer. So the only place to convert into a safe API is in a driver with that understanding. That happens to be a Chip driver right now. I'm having hard time thinking of a design that splits that out.
 * Amit: I think we could. The interface would need to be a wrapper around a DMA register as a whole, and when I take out the buffer the unsafe stuff, for example ensures that the DMA is terminated. Or returns an empty slice if DMA is ongoing.
 * Leon: Okay, that's on the path Tyler is proposing. That's certainly a departure for how we're writing things now, maybe more-so than Brad is proposing here.
 * Amit: I do think this proposed change is plausible and doable. Doing it naively based on how Chips crates are structured right now might be wrong, but that does motivate coming up with an aspirational design at least. Articulate what the separation ought to be, and then we try to do it and see if what problems we run into and update the design based on experience.
 * Leon: I also do think this change is nice. A good improvement. I'm just worried about coming up with the policy now and having a fallacy that the way we currently design things will be sufficient. If we can impose this policy for new chips, that's great. Even forbidding unsafe for now, knowing that will have to change later when DMACell lands. But we could have a policy for having DMA be an exception. And long-term we could pull out that unsafe too.
 * Brad: I like where you ended up. However I would propose we pursue what I have in the PR, but leave any drivers that use DMA in the unsafe crate entirely for now.
 * Amit: That seems broadly reasonable. I'm not sure it's a blanket "only DMA" statement.
 * Brad: Right. It's not only DMA.
 * Amit: We're interacting with hardware, which has side-effects not always represented in the language, and we have to think about how those side-effects leak
 * Leon: A remaining concern is what the cost of doing that would be. We'd split a number of chips from one crate into two crates. I don't know if there's a significant cost in having multiple crates. Maintenance, compile-time maybe, more files and more confusion about what goes where
 * Amit: Another proposal, it's unfortunate Rust doesn't give us module-level granularity of forbid unsafe. We could not separate the code into two crates right now, but instead we could have a special module in the chip where we say unsafe is okay. And things outside the module are not allowed to use unsafe. No mechanical enforcement unfortunately.
 * Brad: Mechanical enforcement is a goal. It's so tempting for chip developers to use unsafe. Things like no-ops that feel super benign. Anything we can offload to the compiler to help us review is great.
 * Brad: Okay. One of the reasons I wanted to try this on the nRF5x crate is to see what this looks like and how bad it is. Certainly the nRF and the SAM4L will be the worst chips due to how much they use DMA.
 * Brad: My takeaway is that we shouldn't make the DMA problem worse. And doing things to make it "look" safe if not helpful in the long run. So let's not make it worse until we can actually solve the problem.
 * Amit: So what are the next steps?
 * Brad: I want to look at the PR again and see what it would look like to move the DMA back into the unsafe crate. I think I could try to have a document explaining what's going on as a starting point. I'm not 100% sure what would go in the document.
 * Amit: Getting a handle on what a safe wrapper around DMA would look like is interesting.
 * Leon: That's still a bit away. In the short term if we exclude DMA drivers from the safe crate, the question is how many drivers do end up in the safe crate. If it's like 80% that's great. If it's like 5% maybe this isn't worth the effort.
 * Brad: I think it's like 90%. We'll see.
 * Amit: They need individual thought though. Just not using unsafe or DMA is perhaps insufficient.
 * Leon: I am slightly positive that most of the misbehavior that drivers can do if you use them incorrectly should hopefully not interact much with Rust's understanding of soundness. A clock peripheral which could make the system stuck isn't unsound in a Rust sort-of-way.
 * Amit: Agreed. I'm not saying don't do this. I'm just saying we need to reason per-driver about it.
 * Leon: This is also a huge diff. So one of Brad's questions is how to sustainably make these changes and review them. I do think this is pretty mechanical and could be waived through.
 * Amit: Thoughtful explanations would help me waive stuff through.
 * Leon: Making these things not use unsafe code isn't just a simple rename. The AES driver, for instance, does have substantial changes.
 * Brad: So two steps. Getting rid of unsafe, and separately organizing things.
 * Branden: Back on the document idea, something Brad could add are examples of how unsafe sneaks into chips today, and which of those uses are good and which of them aren't. That's something I think Brad has a feel for that others will not.
 * Brad: Another question that comes up here is how do we deal with cases where there's unsafe today. Some amount today is static buffers since it wasn't obvious where they should come from. Do we have a good idea about how to replace cases where we need static buffers?
 * Amit: Is the problem that we need to use unsafe, or that we're using them in an unsound way?
 * Brad: Just that we need to use unsafe.
 * Amit: Looking at the nRF stuff, the difference between say, UART and AES, both use DMA and rely on their being a buffer that the driver manages. AES uses a static buffer and UART doesn't. I think the main difference for why this is the case is that for AES the buffer needs to be a very specific size. That sounds like a type issue. Rather than having a TakeCell for a dynamically sized slice, a TakeCell for a fixed-length array. The reason they're static mut isn't a good one, it's just what we did.
 * Leon: Right. Convenience. If we had the ability to predict dynamic arrays, we would have also made them static muts at the time. Now Rust is telling us to make nothing a static mut. Our drivers are singletons, but Rust doesn't expect that. We should just give in and change all of these to be passed in from the Board top file.
 * Brad: I'm on-board with that Leon. My model is that some peripherals may need to insert a header byte, for example, and that's why they end up having a peripheral-owned buffer.
 * Amit: The peripheral could still own the buffer, it's just declared in the board and passed into the peripheral.
 * Brad: Is that okay, that feels weird for chips.
 * Branden: We do it for capsules now. Seems fine
 * Brad: Right now we can create chips without passing in any buffers.
 * Branden: We could change that for the future though.
 * Amit: What's an example of having a buffer that doesn't come from a capsule.
 * Brad: Multi-part transactions where we need to combine multiple things into one buffer.
 * Amit: For example, the encryption HIL could have an associated buffer type.
 * Brad: I can see where you're going, but that's a big hammer to avoid passing-in a 48-byte buffer.
 * Brad: The next place this goes is if we want a pattern of passing buffers into chips, then are we okay with boards having components for chips rather than copy-pasting buffer passing.
 * Amit: Yes. There's value to that regardless. Something like that helps with a better board instantiation system anyways
 * Brad: Okay, and we can do components without unsafe. So I think this is reasonable.
 * Amit: I do want to see cases where the chip really needs to hold the buffer internally. Many chips take one from a higher level. Maybe radios that add headers.
 * Branden: That's what PacketBuffer was supposed to solve. But we never finished the design on that one.
 * Brad: In nRF5x AES, line 48 in upstream, there are meaning to various bytes. That's one example.
 * Brad: In 802.15.4, the chip needs to send ACKs and needs buffers to hold those ACKs in.
 * Leon: Why does any of this change? Why can't we pass in the buffer from Boards with a known length?
 * Brad: Amit wants to not pass in a buffer at all. Have it come from capsules.
 * Leon: Okay. That's possible. We might have to use a pin to keep the struct from moving during DMA.
 * Brad: That's another great example of why we need to forbid unsafe
 * Amit: These things were written a while ago and we were just trying to make stuff compile.
 * Brad: For sure. There's an art to finding the right way to push back on the compiler. We found lots of things that worked, but sometimes there are better solutions which aren't intuitive.
 * Brad: To wrap up, looking at my commits in my PR, I think I can easily separate removing unsafe from reorganizing stuff.
 * Amit: Should figuring out DMA also go on the wish list for new Tock registers?
 * Leon: Interesting to explore, but doesn't have to be part of the core abstractions.
 * Johnathan: It might be outside the scope of Tock registers.
 * Amit: Right now we use a u32 as the type for a DMA base pointer, which isn't unsafe to write to. Should really be a pointer type. So maybe worth putting on the wish list.
 * Johnathan: Interesting provenance question: if you do a volatile write of a pointer to an address, what does that do?
 * Leon: I tried reasoning through memory barriers for DMACell and I don't think there are good stories for any of this.


