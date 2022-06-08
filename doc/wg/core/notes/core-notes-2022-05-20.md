# Tock Core Notes 2022-05-20

Attendees:
- Branden Ghena
- Johnathan Van Why
- Jett Rink
- Pat Pannuto
- Philip Levis
- Hudson Ayers
- Leon Schuermann
- Vadim Sukhomlinov
- Alyssa Haroldsen

## Updates
 * None


## Tockworld Planning
 * Hudson: I sent out email requesting additional invite ideas for Tockworld. People should reply with other people they want invited.
 * Johnathan: I'm speaking with people and will have an answer soon.
 * Phil: I'm speaking with Dom too. So we should coordinate
 * Alyssa: Who's already invited?
 * Hudson: Everyone on the email chain. I added five people to the list, including Jett and some of Alex's students. Feel free to send other people though.
 * Alyssa: I have a friend in OpenTitan who might want to visit.
 * Leon: I'm at a conference and meeting with Dorota. They can't attend Tockworld, but I'm taking notes of the issues they ran into so we discuss in more depth.
 * Hudson: So the reminder is to invite additional people to Tockworld and send out to the email list.

## Mut & Unmut trait problem
 * https://github.com/tock/tock/pull/3041
 * Hudson: We talked about this some last week, and Alyssa among others had concerns. I spent some time thinking about this.
 * Hudson: Leon and I spent some time thinking about digest traits and mut/unmut problems and combined it with the DMA problems to see if we could solve both.
 * Leon: I'm working on a prototype implementation of the proposal. So the basic concept: with this new model we wouldn't pass down the actual buffer to the driver and have it passed back through a callback. Instead we'd pass a reference to a container which holds the buffer. Then peripherals would lock this container when using it, allowing synchronous or DMA operations. Afterwards the peripheral will unlock the buffer, which allows the capsule to access it again. I'm trying it in the core kernel and console capsule first all the way down to the UART for some chip.
 * Phil: As a bit of framing, the issues Alyssa raised last meeting, everyone agrees with. Multiple traits felt not great. We're in a "what is the least evil" situation. I think no solution will be perfect, there will just be tradeoffs: duplication, locking, etc.
 * Leon: I think this solution too will indeed add complexity, although I hope it won't be too much.
 * Leon: In the bigger picture, we currently have unsoundness in mutable DMA operations. And all of our HILs are built without using LeasableBuffer, so we can't pass a window into a slice. This approach does attempt to solve all of those at once. Which is pretty elegant.
 * Phil: Going back to something Brad brought up. When I was thinking about this and prototyping, I didn't consider something like this. I did consider something where we would have a type where a mutable or immutable buffer goes in, mut is transformed into immutable, and we can later reconstitute the mutable buffer out of it. So it would transform and then later transform back. I couldn't think of a safe way to do that without a lot of machinery.
 * Phil: So, my concern from Brad is how can we do this without using unsafe? Think particularly about virtualization. There are multiple handles that are passed around. The virtualizer returns a handle and also needs to call to the lower layer that would itself return a handle. We would have two handles that would both have a reference to the buffer.
 * Leon: Naive approach would be dynamic dispatch to convert these concrete types into their dynamic trait representation as a reference, and then pass that down to the hardware. This would be the simple solution to make virtualization work, because we would just for the peripheral lose the information as to whether it's a mutable or immutable buffer. But the capsule would still have that information. (Signal issues, hard to hear)
 * Phil: (repeating the question) A handle structure holds a reference to a buffer. And we have a runtime lock that lets us release that. If I'm a virtualizer, and someone calls a method, I need to return a buffer. But I also need to hold sufficient state so I can reconstitute that type to pass it down to get a handle back. So there are two references to the buffer.
 * Hudson: This is like TakeCell, I think. Multiple things can hold it, but only one can use it at a time.
 * Phil: But we have two separate handles here. Because the virtualizer needed to return a handle and the lower layer had to return a handle.
 * Hudson: It can't be. We have to just pass references to a single handle around. So there will be multiple references to a single handle. Whoever owns the actual buffer holds on to the container. But they can pass down immutable static references.
 * Leon: And interior mutability will allow us to represent the locking method. That's why we can change these to trait objects. So the lower layers never know what's inside, they just know it's a buffer that is readable.
 * Phil: Hmm. Hard to conceive how the call paths work here. Maybe I just need to see it.
 * Leon: A quick pointer is to forget about the upcalls. Nothing gets passed back anymore. It's just passing down an immutable reference, of which you can create infinite copies.
 * Leon: Playground example (warning, pretty dirty still): https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=376c9d2d22a477402d054354206207f8
 * Hudson: So the idea is that if you're a capsule that would normally be handed a big static mutable buffer, you would be handed instead a statically allocated ImmutableDMABuffer reference. It's constructed by static init in the component. This wrapper type gets passed.
 * Phil: Okay, so the part that's different is that you pass static handles around.
 * Hudson: Yeah, that's a key difference from my original post.
 * Hudson: Still an open question about how much complexity this adds. It could make virtualization ugly. But the two traits approach could be tricky too.
 * Phil: I have tried it.
 * Leon: I'm going to make a PR soon with a small example. That will let us discuss and explore whether this is a better or worse approach in terms of complexity.
 * Hudson: One thing you mentioned is that we could have LeasableBuffer implement these traits too. So we could have many types abstracted this way. Then presumably unlock at the end of the operation could also restore the buffer back to its full length from a slice.
 * Leon: Yes. Although just a thought experiment. I haven't verified that it works yet.
 * Phil: I admit that I get skittish about this. We have a narrow problem that is digest, and this is a very sophisticated solution. In terms of Rust elegance, that's great. But it could be a challenging learning curve. So I'm worried that innovative ways to use the language keep raising the bar to understand things.
 * Leon: I do fundamentally agree. One note: this would have to be an unsafe trait. So that means that every further implementation of this would have to go through a lot of review and be implemented in the kernel. So I'm hopeful that the complexity doesn't spread out through the OS. So we could limit the use to the narrow use cases.
 * Phil: I'm worried less about the lines of code and more about the complexity of HILs.
 * Alyssa: I'm also worried about when to use this, whether to copy the pattern into my own drivers.
 * Phil: To be clear, I think we should see this through and compare. Side-by-side is the best. Just that my intuition is to be wary.
 * Hudson: I am interested in whether Alyssa thinks this approach seems closer to what you imagined.
 * Alyssa: Closer, yes. Part of my concern is that static mut references are hard to work with. You can reborrow them, but you can't treat them as static immutable reference while still being totally sound. So I like the asref + static, but I want to think if there's a better lifetime to put here. Needing static seems to be the crux of the problem.
 * Phil: I'm kind of the opinion that DMA is a special case. It's a common thing on all hardware platforms, but maybe DMA drivers should have a bit of unsafe to turn a slice into pointer and length and later just reconstitute the slice.
 * Alyssa: You still have a lot of the same pointer provenance rules, just the language isn't checking it.
 * Hudson: It still doesn't solve the problem of immut/mut and which to reconstitute as.
 * Phil: I'm just talking about DMA, not immut/mut
 * Hudson: Oh, yes. We are still thinking that DMA drivers will need unsafe.
 * Leon: And I do have some basic code doing what Phil says, which we could investigate further if we want to keep the two-traits approach Phil has.
 * Phil: On copies versus generalization. If it's just two, copies are fine. But once the set is unbounded, or at least more than two, generalization is needed.
 * Alyssa: We could put it under a field trait for byte slices, but then we wouldn't be able to transfer other info.
 * Phil: I meant that for a long time we passed around slices rather than AsRef. Transitioning would clarify some things. But I think we should think more broadly about whether we want to.
 * Alyssa: If we don't generalize though, we require future drivers to either duplicate the API or not use both immut and mut.
 * Phil: Anything we choose will add complexity, for sure.
 * Alyssa: I'd say we definitely shouldn't require mutable buffers. But we don't want to require a copy either. Pretty big cost for read-only operations.
 * Phil: But if you hit DMA, your memory has to be static. Unless we can reason about lifetime of the reference and the physical time of DMA, it's tricky.
 * Hudson: For most drivers, I think it's fine for us to just have copies. Digest is kind of unique.
 * Alyssa: But if it's a static mut that you pass in, then you have to allocate it.
 * Hudson: Well, the capsule can just pass a short-lived buffer down, and in the kernel there's a copy into the kernel-owned buffer. That works for many HILs. But digest works with really big buffers where that becomes too expensive.
 * Phil: That is another approach. Here's we're hashing entire process images. We could maybe chunk it, 1 KB at a time, which is not a _huge_ amount of RAM. But a challenge there is that the memory copy will be a significant overhead compared to the operation.
 * Alyssa: Yeah, I got rid of a double endianness swap and it drastically increased the hash operation. I do understand the problem here. I would _like_ if we could take advantage of Rust reasoning about memory to avoid copies. I do like Hudson's design, although I don't think I totally understand the difference yet.
 * Hudson: Hopefully we'll have a better example for next week.
 * Alyssa: I do like the generic approach.
 * Hudson: I like unifying over LeasableBuffer.
 * Leon: Dorota mentioned that LeasableBuffer was important for them, but they had few examples of it. Really important for what they were doing.
 * Hudson: Okay, I think we have a clear path forward.
 * Phil: My last comment is that we should also keep in mind Butler Lampson's paper on principles of system design. Generalization is good, but not _always_ good. It's very easy to forget the tradeoffs there. (SOSP 1983, https://dl.acm.org/doi/10.1145/773379.806614)
 * Alyssa: Anyone can learn to use a complex API with enough guidance. But when you throw people at it, it gets hard.


