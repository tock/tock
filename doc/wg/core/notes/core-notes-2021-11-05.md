# Tock Core Notes 2021-11-05

## Attendees

 - Alexandru Radovici
 - Alyssa Haroldsen
 - Amit Levy
 - Brad Campbell
 - Brian Granaghan
 - Hudson Ayers
 - Jett Rink
 - Johnathan Van Why
 - Leon Schuermann
 - Philip Levis
 - Vadim Sukhomlinov

## Updates

- Amit: Since I won't be able to talk much, probably, let me quickly update on
  the USB stack process. I still have not written up what I'm doing,
  unfortunately, but I have been making progress as of this morning. I'm pretty
  close to having the CDC so the console is nearly functional , which I think
  will be one half of the drivers we have support for for USB at the moment in
  upstream. That should be a good platform to explain the benefits and drawbacks
  of the new approach and some of the challenges. That's it, keep an eye out for
  that.
- Hudson: Yeah, it's really cool.
- Phil: I have a bunch of updates on the credential stuff but that can be in the
  agenda item.
- Brad: This isn't really my update but I want to pass on something from the
  OpenTitan call. Alistair has been working on the OpenTitan big number
  accelerator and using it. With that perpheral, it basically runs user-provided
  code inside the peripheral and Tock or an application would pass in data to
  it. You have to provide the code. He was able to use the existing linked list
  structure of apps we have to include the OTBN binaries right there. It doesn't
  execute them as Tock apps, because they won't work on the main
  microcontroller, but they will work on the accelerator. That was a nice use of
  the existing infrastructure we have. It raised the question of if this is
  really valuable and useful, maybe we should have an explicit setting for
  binaries in the TBF array that are not actually applications. Thought that was
  kind of a neat update.
- Amit: That is pretty neat.
- Phil: I think one issue with those kinds of big number accelerators and OTBN
  in particular is "who determines the code that runs" and the trust boundary
  around OTBN. For example, when I ported to H1B, I basically copied what was
  used in previous code and basically passed a buffer with instructions and data
  and that kind of stuff. I think my sense is that this is something you want to
  encapsule in the kernel, have particular assembly images that provide certain
  cryptographic primitives, rather than running arbitrary code. It has to do
  with the particular security profile you have.

## Issue #2882
- Hudson: We have an item that Jett put on which is talking about issue 2882 and
  some of the associated PRs around that as well. We're talking about solutions
  for a Miri issue from casting a reference to a `u8` to a reference to a
  `Cell<u8>`.
- Jett: I can kind of summarize like what happened and how we found that. We're
  running Miri locally and found this issue. It is exposed by something I added
  in terms of having immutable references to `u8` slices and we wanted to single
  source code and have everything use `ReadOnlyProcessBuffer`. I added a casting
  method to do it, which is good, but running that through Miri we are seeing
  that is what is triggering this Miri issue. It's logically representing
  something that was already existing with applications and the kernel today --
  it's just hard to test that in a unit test that Miri can see because it's
  across the kernel boundary. But the conversion function is representing what
  we do in an application. What it boils down to, what it doesn't like, is if
  you have an immutable reference to a `u8`, it is undefined behavior to cast
  that to a `Cell<u8>`, even if you don't use the mutability properties of
  `Cell`. So like we do this, we have a read-only byte wrapper which is used by
  the read-only byte slice, which is provided by the read-only process buffer.
  At the end of the chain, the read-only process byte contains a `Cell<u8>`. We
  transmute our references and our pointers to this, and Miri is complaining
  that we are exposing a vector to try to mutate immutable memory. This could be
  a pointer into immutable memory, like flash memory, and we could theoretically
  write safe code (without the `unsafe` keyword) to do `Cell.set` and that would
  be undefined behavior. That is it in a nutshell, and we've been going through
  a few different solutions to try to solve it. The constraint is that we're
  using `Cell` because we want to tell `Rust` that this memory can change
  without Rust knowing about it -- just because we have an immutable reference
  to it doesn't mean it can't change. It could change in the application memory
  because the application could share a mutable buffer it could change. We do
  need that property of a `Cell` because we don't want to have an immutable
  reference to memory that actually can change, and the Miri issue is we don't
  want to cast an actual immutable reference to a cell. Those are the two
  conflicting constraints and we're going back and forth. It's looking like the
  way of solving both the Miri issue and to maintain soundness in terms of
  having an immutable reference with that memory changing under the hood is to
  use raw pointers. Alyssa, who is @kupiakos on GitHub suggested an exotic
  solution with zero-sized types and slices and pointers. It is exotic, and so
  we want people's opinion on it. Is it worth solving this Miri issue? I feel
  like yes, but we're not using incorrectly so we're not invoking the undefined
  behavior, and if the pointer solution will all be contained in that file
  `processbuffer.rs`. If it's really hard to understand is it worth doing
  because it solves both issues issues, do we just comment a bunch? So what is
  the way we want to solve this or do we want to solve this.
- Amit: Lean, I know you understand it quite well. I tried to brush up on the
  issue last night, and I'm still a bit ambiguous about what the actual
  unsoundness issue is.
- Jett: The two issues, one of them is unreported unsoundness like we're trying
  to follow internal rules carefully, and the other is the reported unsoundness
  from Miri. The reported unsoundness from Miri is casting an immutable
  reference to a `u8` to a reference to a `Cell<u8>`. That is what Miri sees and
  complains about. We shouldn't ever do that because that allows safe code to
  try to mutate that immutable reference, which is not allowed and would cause
  undefined behavior. Even wrapping the reference to a `u8` in a reference to a
  `Cell<u8>` is what is triggering Miri to say that is undefined behavior,
  because you are allowing safe code to mutate it which is UB. Does that help?
- Amit: Yeah, that part seems like it would go away if we create a read-only
  cell that does not expose a `set` method, only a `get` method.
- Phil: What stops that, as that was my first thought too.
- Jett: `Cell` is based off `UnsafeCell`. `UnsafeCell` is the language built-in
  feature that marks this kind of interior mutability that lets Rust know the
  memory I'm representing can change under the hood even with an immutable
  reference. There is no other thing beside `UnsafeCell` or something based off
  `UnsafeCell`. Even wrapping a mutable reference to a `u8` in an `UnsafeCell`
  will trigger Miri's complaining about undefined behavior because through an
  `UnsafeCell` you can then mutate it.
- Leon: The issue I want to challenge is, I'm not even sure whether during
  runtime we are causing unsound behavior in what we're doing. That is a very
  valid point we have currently wrapped it in a `Cell` but the way we're
  treating the `Cell` and the way we're limiting its API surface is not
  something that should -- and I say should because I'm not too deep in the
  compiler internals of Rust and how it translates to opitimizations and
  assembly -- but we are limiting the API surface we have on our cell type to be
  only immutable accesses to the underlying memory. Because the Miri issue
  persists even if we are accessing it through an `UnsafeCell` using explicit
  `unsafe` methods -- not unsound but `unsafe` -- we have to be aware of what
  we're doing with the returned reference, as far as I understand this issue
  it's more a warning issued by Miri. It says if we were to strictly follow the
  borrow checker rules with the underlying borrow stack which is implemented in
  the Rust compiler, it could lead to unsound behavior if we were to use the
  interfaces correctly.
- Johnathan: Except if I remember from the error message, it's indicating that a
  retag (re-borrow? re-tag? I forget what it's called) operation occurred during
  that cast which caused immediate undefined behavior. Miri doesn't really warn
  you about something that's risky, it tells you once you've already made the
  error.
- Phil: Going back to Amit's question, we have this piece of memory that in C
  parlance is `volatile`. This thing can change underneath us, we have read
  access to it, so why don't we write an API for that? Not use `Cell` or
  `UnsafeCell`.
- Leon: That's a good question. I think the proposed solution by Alyssa is to
  use regular pointer read and writes, which are perfectly safe and sound as far
  as Rust is concerned. I think the issue with that solution is it abuses
  zero-sized types and slices of zero-sized types in a way which I'd call mildly
  crazy. It took me a bunch of time to really understand what is going on --
  it's really unintuitive and doesn't follow the normal rules of how a slice of
  zero-sized types normally behaves in Rust. I wouldn't necessarily say this is
  unsound but I would at least run it past a few more experienced Rust
  developers to see what they think of that. That is, I think, one definitely
  valid solution to this issue that causes all of these issues to go away.
- Jett: To elaborate, it very much is using raw pointers, but instead of using
  raw pointers in terms of `*const u8`, it uses a zero-sized type and the
  address of that zero-sized type is a proxy for the slice you are going to use.
  So it's a zero-sized type that has an address in memory and that is what is
  represented in the buffers. When you slice it, you have to walk the zero-sized
  type forward and make a new zero-sized type when you're trying to slice it and
  then dereference it. It is effectively using pointers under the hood.
- Leon: And the reason why we are looking at this solution is because it is the
  only one we could come up with which matches the constraints we are given by
  the `Index` trait. The `Index` trait is the one executed on your thing when
  you use the `[]` operators in Rust. It requires you to return a reference to
  something, so you cannot return an integer or pointer or safe wrapper around a
  pointer because that would need to be allocated on the caller's stack, which
  doesn't work when you return a reference.
- Phil: I have one other question. This is really an implementation question,
  right, in the sense that it is how do we implement this without invoking
  undefined behavior, but the external APIs to the rest of the kernel are not
  changing, right?
- Jett: Correct, we wouldn't want to do that.
- Phil: I want to read this stuff more closely to understand how it works, but
  my intuition is that this implementation may not be perfect because it is
  exotic and exotic is bad but it is better. So okay, lets use this solution
  which fixes this problem and in a couple months someone could come up with a
  better approach that is less exotic.
- Leon: I'm not sure that we wouldn't ever need to change internal interfaces
  within the kernel, because if it turns out the solution proposed by Alyssa is
  indeed not sound -- it's at least tricky to follow -- then to support both
  in-kernel and in-process buffers we would need to change the interface. We
  wouldn't be able to use the indexing operators any more. So the change would
  be pretty straightforward which we would have to do in the kernel for us to
  change to a solution that user raw pointer writes and doesn't rely on
  zero-sized types, but we wouldn't be able to use `[]` anymore and would have
  to change to a method like `.index()`. It would only be a syntactical change,
  but it would be a change which propagates through the entirety of the kernel.
- Phil: I'm saying something different. If we can change the implementation
  without changing any of the interfaces such that we solve this problem, but
  maybe we want to improve the implementation down the line that's okay. If we
  then decide the new implementation requires that we change the interfaces then
  we should go back, unless we're sure the interfaces have to change period.
  Interface first, implementation second.
- Jett: I'm confident that Alyssa's solution will work and we won't have to
  change the API to the rest of the kernel. To give credibility to Alyssa, she's
  been doing Rust for a really long time and is really good with Rust, and
  `unsafe` is one of her specialties she likes to look into. I have confidence
  her solution will be good and will be sound. We should absolutely still vet it
  and get more opinions, but I have confidence that it will actually solve the
  issue and we won't have to change the APIs.
- Leon: I didn't want to challenge Alyssa's knowledge of Rust, I'm just voicing
  concerns that it looks crazy and we should get others to look over it. It's
  not easy to write, and easy to make mistakes.
- Jett: I agree. I wasn't trying to imply you were challenging her, I wanted to
  give you background on Alyssa. I had to walk through it with her too, because
  I was very interested and it makes sense but I don't know enough to find the
  holes in it. I can understand how it is supposed to work but I am not
  experienced enough to find the holes.
- Phil: These are exactly the kinds of cases where comments are helpful,
  detailed to really explain the issue that's going on and why this approach is
  there. These things are complicated, some things require really technically
  subtle solutions and that's okay as long as you document it for people down
  the line so they know you're not crazy.
- Leon: My stance is because we aren't aware of any real-world impact that this
  currently has, what Miri does complain about is simply that we do have some
  violations as far as Rust's borrow stack is concerned. We should probably
  think about this and try to come up with a solution and run it by Rust folks
  and not prematurely merge something we'd maybe need to revert again.
- Amit: A related question is, and maybe Alyssa would know this better, is there
  a chance this is undefined behavior that could be fixed upstream in Rust? In
  other words, maybe this is a use case that is not well-defined now and there
  might be upstream fixes that we could lobby for which would allow us to go
  back to something closer to the cleaner solution we have now. In some future
  version of Rust that may be better defined.
- Leon: Our problem is pretty unique, in that we are trying to unify memory that
  is managed externally. We are creating references into process memory that
  Rust cannot reason about, with references that are outside of the memory
  structures that Rust knows about at compile time and can reason about. Because
  we are gluing these things together there is a lot of weirdness which Rust is
  not eager to solve. It's a very constrained and special problem we have.
- Jett: I've invited Alyssa, she'll make it to the meeting in a few minutes. I
  don't think it's an upstream issue in Rust, I think we have two conflicting
  requirements and the only way to handle them with soundness is to drop down to
  raw pointers.
- Leon: There may be a chance for the things we are doing to be sound if we
  weren't also using process slices to handle kernel memory. We still have an
  `UnsafeCell` around process memory, particularly process memory in flash which
  is immutable, but the `UnsafeCell` would potentially -- as far as Rust is
  concerned -- allow mutation using `unsafe` code. That is not something which
  Rust can reason about so we're touching on this gray area where the Rust
  concepts do not map cleanly onto what the machine exposes. It is the purpose
  of `unsafe` code to define how those things interact.
- Amit: This seems to me like it might be a general problem. In our case it is a
  system call boundary -- what about Rust in the Linux kernel that may be
  interacting with C code? What about Rust not as a consumer of C libraries but
  as a library itself that is used by C code? In general, the problem of wanting
  to ensure that Rust code is only using memory in a read-only way but that
  other code might in fact mutate seems like a problem that is beyond the very
  specific use case we have in Tock.
- Jett: One of Rust's answers to this is you drop down to raw pointers and use
  `unsafe` at those times you need to access memory that others outside Rust's
  control can mutate.
- Vadim: I have an idea here. It looks like multithreaded communication. We have
  an object that can be modified by both threads -- I'm thinking about the
  kernel as another thread even though they run on the same CPU it's concurrent
  execution with side effect same as in multithreaded environments. I think we
  need to think about implementing `Sync` and `Send` constraints for the buffers
  which we will transfer across the boundary and make sure the kernel and user
  applications respect this. Some would like an implementation of like `Rc` or
  `Arc` in a way that wouldn't have such burden as it would have in the `std`
  crate but suitable for embedded things, some simple flag maybe attached to
  every buffer indicating whether it can be used or not. The same implementation
  is shared by both kernel and userspace, so a mutual respect to who is
  accessing what at the type level.
- Leon: I think that is a distinct issue because we do have our sort of pseudo
  concurrency problems solved by using memory barriers prior to switching to
  processes and after switching from processes so I think that is not an issue.
  What we are concerned with here is aliasing and the optimization Rust does
  based on whether memory is aliased or not.
- Vadim: Yeah, same solution would prevent this if we are using `unsafe` but
  there will always be `unsafe` when we cross the boundary.
- Phil: It looks like Alyssa was just able to join.
- Jett: We've been talking about the solutions for the Miri issue and going into
  this zero-sized type approach. Amit can't ask now, but I think Amit was
  wondering if this problem is specific to our situation or a wider problem for
  Rust.
- Amit: I'll try to clarify, it's just a busy time for me. I'll try to speak
  quickly. My high-level question is that it seems like we are morally doing the
  right thing, and is this something we might be able to resolve upstream in
  Rust? It it something where this behavior is not well-defined enough or there
  isn't an appropriate construct in the language that we may be able to
  contribute such that we can switch to a simpler solution in a future version
  of Rust.
- Alyssa: You're talking about where you're using cells for read-only data?
- Amit: That's right. It's data that is read-only for the user but wraps memory
  that may or may not change.
*Editor's note: Alyssa dropped off the call here (connection issues)*
- Jett: We're trying to represent two kinds of memory that's kind of connected.
  We can share from the app and it can be a mutable thing the app can continue
  to change or they can share truly read-only memory that isn't changable but
  we're wrapping it in something we can directly change it from in the kernel.
  Kind of like the short and sweet way of describing the two problems that are
  conflicting.
- Phil: I think when Alyssa re-joins she needs more context, because we're
  pretty deep into the discussion. Do you mind if I try to give her a bit of
  background and context for the question?
- Amit: Yes, please do, I'm multitasking.
- Phil: I think I understand the thing you're getting at.
*Alyssa re-joins here*
- Phil: Hi Alyssa, this is Phil, can I give you a bit of context of what we're
  talking about so the question makes more sense?
- Alyssa: Sure
- Phil: Amit's question is along the lines of "hey, we're seeing this problem,
  but it seems like a problem that other people will bang into too". Like if
  we're putting Rust in the Linux kernel, this idea that I want read-only access
  to memory which in C parlance is volatile underneath, which operating systems
  are going to need to do, is this something which maybe Rust should have better
  support for?
- Alyssa: I can talk a little bit about what the current status and what the
  actual problems are. The real problem is that in the kernel you are able to
  convert a read-only buffer to a read-write buffer. I don't see a huge problem
  interpreting a userspace-level read-only pointer as an interior-mutable type
  in the kernel. The problem is there's an interface that in the kernel you can
  convert from a read-only buffer into a read-write buffer and that's where the
  undefined behavior comes up. The only way we could possibly think of proposing
  to be able to transmute a read-only buffer to a read-write buffer as long as
  you don't write to it, I would be very skeptical we could get that past
  anyone. Stacked Borrows is the memory model that Miri uses, and they're
  dealing with much finer details than that at this point, so I would be very
  hard-pressed to convince them that transmuting a read-only referenc to a
  read-write reference is safe.
- Hudson: I don't think that fundamentally the thing we need here is to
  transmute a read-only reference to a read-write reference. Instead, we need
  some sort of read only cell, which Rust would know can change behind the
  scenes but I am not allowed to write. If a construct like that were to exist,
  it would not require transmuting anything from read-only to read-write, it
  would just require a Cell that Rust understands can change behind the scenes
  but Rust knows it can't write.
- Alyssa: We already have that with raw pointers.
- Leon: I have a question regarding the statement that this is only a problem
  because we are actually allowing kernel memory to be represented as a process
  slice because I agree with that fundamnetally. But in userspace we could be
  allowing a buffer as read-only into the kernel that resides in flash, which is
  only mapped as readable. I know that is a hardware implementation detail, but
  to confirm, do you think it is fine to treat that as mutable memory in the
  kernel in the sense of a cell we never write to?
- Alyssa: As long as they're protected by types I don't think the Rust compiler
  can tell the differenc.
- Leon: That makes sense, thank you.
- Alyssa: This is fully external memory, right? Who created the memory that is
  passed to the applications?
- Leon: I think the memory is, in a sense, created in the kernel, but it is only
  created in the sense that we are passing raw pointers down to the application.
  We might temporarily construct a slice in the kernel but we'll drop it before
  the application starts.
- Alyssa: You're creating it as mutable memory in the kernel, right? Like it
  should not be possible for a userspace application to tell the kernel that it
  wants to have a read-write Allow on read-only flash?
- Leon: That is disallowed, yes, we check against that.
- Phil: The canonical example of this is userspace has something like a key and
  it wants to store than in flash, not RAM, because it's part of the program
  image. It wants to be able to pass that to the kernel to say "please use this
  key" but also say it is read-only access so it can't accidentally invoke a
  system call which tries to write the memory.
- Alyssa: It sounds like you're pretty correct for using interior mutability,
  the primary problem is that you're able to take truly read-only memory in the
  form of an immutable reference in the kernel and cast it to this read-write
  interior mutable memory. That is a big problem if the kernel is able to have
  read only memory. Where is that allocated -- like, is the Rust compiler aware
  that is allocated a certain way?
- Phil: The way it works is userspace will make the read-only Allow system call,
  passing a pointer and length. The kernel will check the pointer and length are
  within the process' RAM or flash.
- Leon: We're never persistently creating in-kernel references to userspace
  memory. We're only creating it when we're accessing it. We're using a slice of
  cells because we don't check against overlapping buffers. We were hoping that
  using slices of cells we could deal with these aliasing problems in case one
  of them is mutable.
- Alyssa: I understand what you're saying here. I'm trying to think of a way,
  because I did a couple of tests to see if I could trick Miri essentially -- I
  tried a union of a `Cell<u8>` and a `u8` and that did not work, unfortunately.
- Leon: It sounds to me like if we are in agreement that this is purely stemming
  from using kernel memory, treating it as a process slice and therefore slice
  of cells, we may get away with using a composite type like an `enum` on top of
  that. Then we're never actually creating a process slice of kernel memory, we
  just have a composite type that can encompass both user memory and kernel
  memory if we know we need to treat them differently on the type level.
- Alyssa: There's still the problem that you probably shouldn't be representing
  a slice of cells for read-only flash memory. Miri probably would be incapable
  of detecting undefined behavior. I don't think it would cause undefined
  behavior as long as you're using it correctly, but it's still not great.
- Leon: Would it help to use `UnsafeCell` instead? That would make it clear that
  we have an `unsafe` interface and need to use `unsafe` to access the data.
- Alyssa: I don't think that will help. It would make just as much sense to
  expose the interface safely and have all the unsafe behind the scenes. The way
  that makes the most sense to me is to just use a slice of pointers or raw
  slices. That doesn't necessarily deal with it -- it depends how the kernel
  constructs it. If it knows for certainty that a raw slice could survive from a
  read-only slice then it knows for sure it won't be mutating and therefore
  cause undefined behavior if it does mutate.
- Leon: I think we can encompass that information in the struct we are
  representing this with.
- Alyssa: The difficulty is figuring out how to represent read-only memory while
  also allowing this read-write memory sort of situation. I'm thinking maybe
  instead of a userspace-kernel separation there's a separation between flash
  and read-write memory.
- Leon: But we were using the process slice explicitly on mutable and immutable
  memory because we want to treat mutable allows as immutably-shared memory in
  the same way as you can take an immutable reference to a mutable reference.
- Phil: My guess is this kind of thing is tough to talk over voice. We really
  want to be looking at code. It looks like we've been having good conversations
  on the issue. We were mostly wondering if this is a big problem we'll see
  Linux banging into, or is it something that is really kind of specific to us?
- Alyssa: It depends on whether Linux allows shared mutable memory between
  multiple faces of it -- I don't actually remember.
- Amit: Thank you for the detailed explanation of the issue too.
- Phil: I think now a lot more people will be able to sound off on the
  discussion online and understand it better.

## Flash HIL discussion

*The Flash HIL discussion was tabled until next week because there wasn't time
to discuss it.*

## App IDs
- Phil: This isn't so much about app IDs as it is about a detail in the Tock
  Binary Format. I put a summary of the issue in play in the agenda item. The
  question is we need some kind of new header to indicate where the binary ends,
  because that's where the footers start. We could create a replacement for the
  main header called the program header that has this field. It works great, I
  have everything working, it works with both the main program, `elf2tbf`,
  tockloader works. The challenge is the current semantics of the main header
  and TBF loading in the kernel are such that if you load a binary kernel before
  this update and somebody loads an application that has a program header, then
  when the Tock kernel loads the binary it will skip the header, see there isn't
  a main header, and default the init function pointer to be 0. In practice, if
  you install a binary with a program header on an old kernel, it will generally
  crash immediately, and the kernel can't tell that this has happened. So I
  think there are a couple options, for example instead of a program header we
  could just have an extra "binary ends" header. I don't like that because it is
  a "required optional" header. Another thing we could do is change the
  semantics about program headers, you can't have neither.
- Brad: I was thinking about this a bit. One question is will `elf2tab` make the
  switch, or will most apps be built with the main header?
- Phil: I made it as a positive flag in `elf2tab` -- you do `--footers` and it
  inserts a program header. Whether all the makefiles use `--footers` is a
  separate question. I was thinking yes, but I wanted to remain
  backwards-compatible.
- Brad: In the short term, anyone using that flag will have a new kernel,
  because they will be using those footers. It's more people who are not
  interested in using those signatures that will be using older kernels, we need
  to make sure those work. I suspect it depends on what we do with the
  `libtock-c` and `libtock-rs` makefiles. If we don't make it a default I
  suspect it won't be an issue. I'm inclined to think we can sort of avoid this.
  I also think it would be fine to have both, and just have newer kernels ignore
  the main when both are present.
- Phil: Then like I have a deprecated, old header because you might run an old
  kernel. For me, it has much more to do with the fact that the kernel expects
  that this thing is here but the error handling around it is tricky. Parsing
  doesn't fail when you encounter a header you don't understand, you ignore it,
  but the main header is always inserted by `elf2tab` but the kernel sees it as
  optional. What is an example where you don't have the main header?
- Brad: Padding. tockloader will insert "applications" that are just to get the
  linked list alignment to work correctly.
- Phil: How does the kernel know not to run the application?
- Brad: I think it's the missing main header.
- Phil: No, otherwise it wouldn't be running, it would fault. It might be
  there's two pieces of data there.
- Brad: That's why it's not in the base, because it wasn't assumed it would
  always be executable. That's something I didn't quite catch: the intent was to
  deprecate the main header.
- Phil: Not necessarily. My concern is that somebode sends you a TBF, you load
  it on an old kernel, and it crashes. As opposed to saying "hey, this is an
  application, I'm going to try to run it, but it doesn't have a main".
- Brad: That's why I think it would make sense to have both.
- Phil: Okay and so you can always reset and say "hey I don't want to run this
  on older kernels" and just put in the program header. We don't want the main
  header to stick around forever when people know they're running on newer
  kernels.
- Brad: I think we should absolutely have an intent, and it seems like we don't
  really need both. We should eventually plan on removing it, but it seems it
  makes everything easier to keep it around for now.
- Phil: Okay. So we'll change the semantics so you can have both a program
  header and a main header, and it's up to the kernel to choose which it uses.
  Newer kernels will use the program header. Over time we can change the
  toolchains to stop generating the main headers.
- Brad: Okay.
- Phil: Thanks everyone. Since it was quick, you sort of know what's going on,
  just a bit of an update but great. I will do that, Brad.
