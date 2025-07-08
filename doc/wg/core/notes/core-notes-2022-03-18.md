# Tock Core Notes 2022-03-18

Attendees:
- Alyssa Haroldsen
- Amit Levy
- Brad Campbell
- Branden Ghena
- Hudson Ayers
- Jett Rink
- Johnathan Van Why
- Pat Pannuto
- Vadim Sukhomlinov

## Unscheduled chat about RLS/`rust-analyzer`

* Brad links [Rust's nightly component
  history](https://rust-lang.github.io/rustup-components-history/) in chat.
* Hudson: Brad, that's an interesting chart there. I'm surprised there has been
  no RLS support since February 23rd. I wonder if it was officially deprecated
  in favor of `rust-analyzer`? Is `rust-analyzer` the official option? I'm not
  sure.
* Brad: That's what makes keeping track of this so hard.
* Johnathan: I see a `rust-analyzer-preview` which suggests that it is not yet
  the official option, but I can see motivation for maintaining RLS drop over
  time.
* Alyssa: Has anyone figured out how to make `rust-analyzer` descriptors for
  non-`cargo` projects? Can I run Clippy using the same config? That's what I
  want.
* Hudson: I've definitely never used `rust-analyzer` on a non-`cargo` project.

## Updates (which became a discussion about #2958)

* Hudson: I looked back at #2958 (compressed grant resource counters). My issue
  with the original PR is not that it did anything unsound but it exposes
  functions that if called in different places could create unsoundness. I'm
  trying to figure out what to mark `unsafe` to regain the safety invariants
  that Rust needs. Will hopefully have a commit out today.
* Brad: I'm curious to see what you will come up with. I don't think I agree
  that the current approach distributes this unsafety in a lot of places because
  we already have this guarantee that you can only enter a grant once. That
  provides a lot of uniqueness properties, and we're already checking it so we
  can leverage it. My changes centralize all of the tricky work that if you did
  it wrong would be very bad. It makes it less distributed across the `grant.rs`
  file, and the APIs are easier to use. That doesn't mean there isn't a
  different way to achieve those same goals.
* Hudson: I think the current state still has issues where there are a lot of
  places where we construct the `KernelManagedLayout` type that don't require
  entering the grant before constructing it. For instance the subscribe, allow
  RO, and alow RO functions that are called by the kernel, you don't have to
  enter the grant to call those.
* Brad: No, but those enter the grant.
* Hudson: Yes, but right now it's possible to rewrite the
  kernel without using `unsafe` to both enter the grant and call one of those
  functions.
* Brad: Yeah, that's fine.
* Hudson: We don't currently do that, but I could call Subscribe from within an
  entered grant and create unsoundness before the panic would occur.
* Brad: How would you do that?
* Hudson: I would pass a closure to some call to `Grant::enter` from within the
  kernel and call the grant subscribe method from that closure.
* Brad: So if you've entered the grant, and you've called subscribe on an upcall
  that is in that same grant, then when the subscribe function tries to run
* Hudson: I'm calling the subscribe method defined in `grant.rs`, not the
  subscribe method on `Upcall`.
* Brad: Correct. I'm not sure how easy that is to get -- I guess I could pull up
  the link. The first thing that subscribe function has to do is to enter the
  grant.
* Hudson: Does it not have to create the kernel-managed layout first? I thought
  it did.
* Brad: No, because it doesn't know the pointer to use until it has entered the
  grant.
* Hudson: Maybe I misread this and what you have now is fine then. I'll look
  back through that.

## Pointer-width syscalls (#2981)

* Jett: I came across this while implementing host emulation stuff which runs on
  64-bit Linux. It's not upstreamed, but we have a custom kernel-owned buffer
  that is passed back to applications which uses pointers. We ran into a problem
  where we are trying to return a pointer from Command and we don't want to have
  to fork in our code and return a different success type on 32-bit and 64-bit
  architectures. That made me think "what is the plan for a 64-bit ABI", in
  general, as there is 64-bit RISC-V variant. We could add a "success with
  pointer" return type which would give us what we need without having to add
  `usize` to the ABI. It would only be for a pointer so it could be
  architecture-independent. Would keep the property that each syscall returns
  one success variant.
* Hudson: The person who most strongly pushed back against this is Phil, who is
  not here. I suppose you probably saw his comment on the issue about having a
  separate 64-bit ABI. That doesn't solve the issue of writing capsules that
  want to pass pointers, because then they would need to switch depending on the
  ABI.
* Jett: Exactly. It's weird to write capsule code that is chip-independent --
  therefore architecture-independent -- that has to fork based on what it is
  compiled against. Subscribe already returns pointer-sized values. I think
  there should be something for Command which gives the ability to pass back a
  pointer. I think we should make that first class.
* Hudson: The issue is the 2.0 syscall TRD says that Command should never pass
  pointers, so we're discussing changing that wording.
* Jett: Editing that text and allowing this thing solves that 32-bit/64-bit ABI
  thing.
* Amit: What again is the expected scenario where we would have a 64-bit
  architecture.
* Jett: We have host emulation, which uses the kernel and applications and runs
  on a Linux system. I believe in the tree there is a 64-bit RISC-V port -- not
  fully fleshed out, but it exists.
* Johnathan: Is the question "what is host emulation?" I'm not sure we've ever
  really explained that. It is spinning up a Tock kernel in a standard Linux
  process and spinning up Tock processes as separate Linux processes and making
  them communicate and pass syscalls between each other as a way to simulate a
  Tock system.
* Alyssa: Essentially a "Linux HIL".
* Amit: And this is for testing?
* Jett: Yes, this is for testing in our CI. Running tests in our application
  with our kernel and make sure it works without deploying to hardware.
* Amit: To channel my sense of Phil's pushback, do we want to make architectural
  changes to the system to make testing more convenient?
* Jett: I think this testing is bringing up an issue that would exist in the
  future with 64-bit RISC-V or something else.
* Amit: I think there's skepticism from Phil and others on this call that we
  will realistically see 64-bit embedded chips.
* Hudson: It seems very possible that we'll see 64-bit embedded chip, but it
  doesn't seem like Tock will be a good fit.
* Pat: We fundamentally wrote a 32-bit syscall ABI, and to support a 64-bit
  platform then we should write a 64-bit ABI rather than shove 64 bits into our
  32-bit ABI.
* Jett: That makes sense, but it still doesn't solve the problem of capsules
  having to know about the architecture. We can have a 64-bit ABI and I think
  that makes sense, but we still run into the problem that there needs to be a
  Success variant that is specifically called Success with Pointer or Success
  with `usize` for it to have a corollary on both ABIs.
* Pat: Leon might know better because he worked with Phil a lot more on the
  syscall ABI surface. I don't think we intended to be able to pass pointers
  across the ABI, did we?
* Branden: Leon is not here.
* Brad: I think we need to separate these two issues. There's a 64-bit syscall
  ABI, and there's transferring pointers. If we did have a 64-bit syscall ABI,
  then all the widths would be 64 bits, and you could put a pointer in the
  normal "success with u64" field. This probably really wouldn't have come up in
  the same way, as it would be pretty easy to work around. It wouldn't be as
  elegant, but it wouldn't be as much of a blocker.
* Jett: Suppose a 64-bit ABI exists. What does capsule code specify for a
  success pointer? It has too choose between success with u32 and success with
  u64 depending on the architecture. Agree that it is related, but even if we
  have a 64-bit ABI it doesn't solve the problem of what a capsule should use
  for a pointer-sized variant.
* Alyssa: I'm thinking the capsule could switch which variant it does based on
  the size of usize. In kernelspace, when you ask for a `usize`, it treats it as
  the variant that is pointer-sized for this architecture. Handle it at API
  level rather than ABI level.
* Jett: That's how you'd have to do it now -- have to do some forking in your
  capsule code or the success variant class has to do the forking based off
  architecture as well. Is that actually breaking the "one success variant per
  return" rule?
* Alyssa: Then it would be one success return variant per syscall per platform.
* Jett: Is forking logic the expected way, or do we want to have a unified path?
* Alyssa: Are you asking "should we have a `usize` variant"?
* Jett: Yes.
* Branden: Another question is should Command ever return pointers. If yes, then
  we need a `usize` variant. Another workaround here -- and maybe it's not
  possible because of the way the documentation is written -- but you could use
  Subscribe to return a pointer, which already supports `usize`.
* Jett: I was having trouble making that work.
* Pat: Is this a kernel pointer that is being returned or a userspace pointer
  that is being returned?
* Jett: It is a kernel pointer, into a kernel-owned buffer that is leased to the
  application.
* Pat: I'm trying to make an analogue to Linux, where you get an index into a
  kernel-owned data structure. Can you do the same thing and avoid leaking
  kernel pointers?
* Jett: When the application needs to read it, it is reading from memory. On
  Unix processes we open up a memory-mapped file and pass pointers back and
  forth. This emulates how it happens on an embedded device.
* Branden: So the analogy Pat mentioned breaks because you don't execute a
  system call to do the reads and writes.
* Pat: The fundamental issue is you're giving applications access to kernel
  memory rather than the kernel access to application memory, which doesn't
  match with Tock's original design where the applications own all memory they
  access.
* Jett: Right, and the kernel fundamentally owns the memory, so it can detect if
  an app dies and revoke it to reuse it.
* Amit: To ground this, are you doing this as a testing thing, or is there a
  production-use argument for having userspace use kernel memory?
* Jett: We have to receive 1K-sized I2C messages, and they are passed to
  multiple applications. If we were to have application-based buffers, each
  application would have to have its own 1K buffer in RAM. I2C only sends and
  processes one packet at a time, so we have the kernel own the buffer and
  dispatch it to an app for processing.
* Amit: The most similar thing I can think of that exists in Tock is IPC. It's
  kinda like IPC, except one of the services lives in the kernel.
* Jett: Yeah, I see your analogy and I agree there are similarities.
* Amit: I wonder if the right solution to this, rather than bending
  Subscribe/Command/Allow or whatever, is to acknowledge that this is a
  different kind of pattern. Maybe if we had a decent IPC mechanism, I would
  recommend we would add this here. We don't, so maybe we should come up with
  something separate. Sharing kernel memory via an additional API that is
  specifically about sharing pointers (or a pointer + a length). Then Command
  could return something like file descriptions -- e.g. return "I'm referring to
  buffer 3" or something like that.
* Jett: To make sure I'm understanding: you're proposing creating a new class of
  syscall that just deals with kernel-owned buffers.
* Amit: Yes. Not Command or Subscribe, something like "kernel allow".
* Alyssa: What about when we want processes to share memory with each other?
* Amit: That was the intent of the IPC capsule, which was trying to fit that
  into Tock's existing APIs. Maybe we need to acknowledge that this is a
  legitimate enough use case to have ABI support for it, and a set of system
  calls that cleanly support sharing memory from the kernel with processes and
  between processes.
* Jett: I think that is a good way to go about this to make it first-class. That
  would also solve some problems -- there have been other PRs where we want to
  add and remove MPU accesses that was hard to put into the kernel.
* Amit: The origin of not using `usize` for `CommandReturn` and other things is
  that in the common case, Command is returning a value, not a pointer. If it's
  a pointer-length value, then capsules or applications can't rely on the width
  being something. It's harder to assume that e.g. the capsule will always
  return something 32-bits, and know it won't be truncated by hardware. It's a
  different enough use case that maybe it warrants a different system call API.
  We want IPC and know IPC has reasonable use cases, and the existing IPC
  mechanism sucks. We should probably treat IPC in a more specialized way than
  glomming it onto the existing syscall API.
* Jett: Yeah, that seems like a reasonable way to go. I'm curious what other
  people think -- if we were to write this up, would you support this idea?
* Hudson: I think it's worth noting that before Amit voiced this idea, Branden
  put the same idea in the chat (which Amit can't see because of how he's signed
  in to the call).
* Branden: I also think this is a reasonable idea. Maybe extending subscribe
  will work, bet perhaps a new system call is the way to do that. I will
  devils-advocate this for a second, and say that if extending Command is
  sufficient for everything that is needed, then that is way easier. Just add a
  new return variant.
* Jett: Yeah, so I agree we can solve this by adding one variant for pointers,
  or we can add all of this architecture for this first-class concept and
  dispatcher.
* Pat: I'm hesitant to say it's "just that simple", because it will have the
  contract that the kernel is providing access for some time, and it's not clear
  what that time is. I think tackling the memory sharing problem directly is a
  more robust way to do it.
* Alyssa: We need to do thing that the base Tock OS doesn't support so we need
  to add functionality. It would be nice to have something that's officially
  offered by Tock, but for just the ability to pass pointers back and forth for
  people who are hand-rolling their own IPC, I think it makes sense.
* Jett: Another way to say this is I don't think the solutions are mutually
  exclusive. We can implement both.
* Branden: I think it may be worth doing the simple thing and playing around
  with it for a while to see what it would take to have first-class support for
  IPC.
* Amit: I'm generally empathetic to that kind of approach. It seems that we have
  this use case that we want to do in the future and also it would be nice to
  have something like this now so we could do something really hacking for
  testing. We could make changes that we mark as experimental and have that be
  concurrent with figuring out what we need to be more robust rather than
  blocking progress on a months-long design project.
* Jett: I think having a pointer-sized Success variant can stand on its own. It
  has its own use cases. It's not hacky.
* Hudson: I think the one thing that's a little strange about adding a success
  pointer variant is there's no reason it's useful unless you have a capsule
  that can directly manipulate the MPU, which most cannot.
* Jett: I don't think that's true.
* Amit: I could imagine a capsule that returns information about a process'
  address space.
* Hudson: Yeah okay, makes sense.
* Jett: I think this is a nice piece of functionality that is well-defined that
  we could add that would unblock us. I do like making it a first-class thing
  where you have kernel-owned memory that can be shared with userspace, but it
  would take months to get that working. We'd help design it and implement it,
  then switch to using it.
* Amit: I'm convinced, are there other concerns? Does anyone want to try and
  channel Phil?
* Hudson: Before Brad left, he pasted in the chat "I have to go, but: I think
  this is generally an interesting idea and worth allowing Tock to explore.
  However, the two leads on Tockv2 (Phil and Leon) are not on the call, and I
  think we need their thoughts." I agree and am also personally convinced, but
  we should ask them. Maybe we can send a PR and start the discussion with Phil
  and Leon there.
* Pat: I think Phil would want a specification PR -- a PR of semantics against
  TRD 104, which explains what guarantees are made around the pointers etc,
  which is where a lot of the challenge and risks come from.
* Jett: I think there are no guarantees with the pointer -- it will be a driver
  or capsule-specific guarantee on what the pointers are.
* Pat: That would be a place to start -- write down exactly that. Just say that
  "this is what the syscall would do, here are guarantees it doesn't make" and
  start the discussion with that.
* Amit: I would go farther, and say the changes to the TRD should say that this
  variant is experimental, and capsules should not rely on it existing.
* Jett: I don't think it needs to be experimental as it is well-defined.
* Hudson: I think if we introduce a superior first-class approach in the future
  then we'll want to get rid of this one.
* Jett: Yeah, but I think there are still scenarious where you could return a
  pointer that isn't just dispatching a first class thing.
* Amit: Experimental doesn't mean we won't stabilize it. Experimental means we
  don't need to convince Phil and Leon now that the API will be permanent.
* Jett: I would rather start with stable and negotiate down to experimental.
* Amit: I think that will make the PR review take longer, but that's okay.
* Alyssa: I think it's intended to mirror the Rust intention behind experimental
  and stable, where you do things as experimental so you don't have to commit to
  supporting all of the specifics for the forever future.
* Amit: There's a chicken-and-egg problem we need to solve. Have to answer the
  question "what are the use cases", but can't answer that because it isn't in
  place. Adding it as an experimental feature allows us (the royal us) to
  demonstrate use cases.
* Jett: I'm okay with it being experimental. With the guarantees being "there
  are no guarantees", then I don't think there is a legacy support burden.
* Amit: I totally get it. This was just a suggestion from me -- I would be okay
  with not marking it experimental.
* Jett: I'm out for most of next week, but after that I'll try to send a PR. It
  sounds like we have rough alignment, so the next step is to put it into TRD
  words.

## Console API change/bug fix (#2996)

* Johnathan: I mostly want to make sure people saw the PR, because it rides the
  line between a breaking change and a bugfix, and that's not obvious from the
  PR's title. This was a result of a `libtock-rs` code review, where a new
  contributor read the console documentation and believed that console could
  write less of the buffer than the process had asked it to. I.e. they
  interpreted that if you pass a 5 byte buffer to the kernel and told the kernel
  to write 5 bytes, it could write 2 bytes, and that is acceptable. That is now
  how I read the console docs, but it wasn't explicitly written in the
  documentation. I read the console source code to verify I read the docs
  correctly, assuming it would always write the full 5 bytes, unless the process
  told it to print fewer bytes. If the process passed a larger buffer and told
  console to print a smaller number of bytes, I assumed console would write the
  first bytes, but when I read console's source it prints the last bytes. I
  changed the wording to clarify that the console will always write the number
  of bytes you told it to write or the buffer size, whichever is smaller, which
  is its current behavior. I also adjusted the console capsule so that it prints
  the first bytes, rather than the last bytes, of the buffer. That is
  technically a breaking change, sort of. If anybody ever depended on the
  behavior of printing the last bytes, this change would break them. But the
  documentation never said what it should do, and it's kind of a question of how
  you read the documentation before whether this is a breaking change or a bug
  fix (making console behave the way it was specified to).
* Amit: It seems like there's no way we shouldn't do what you're suggesting.
  Whether we call it a breaking change or not is incidental. This is also like
  the console -- this is affecting `println`s basically, and as you're
  suggesting it is difficult to imagine that anybody relied on that behavior.
  Must of the people who have used Tock are on this call. While the community of
  users is relatively small, take advantage of that and fix things that are
  clearly bugs even if technically they are a breaking change to what some
  theoretical user might have relied on.
* Jett: I second that, for sure.
* Alyssa: Just document it in the release notes, that's all I care about.
* Hudson: Yeah, I agree that that's clearly just a bug fix.
* Johnathan: Okay, so I can probably just merge it as is, or should I ping Phil
  and Leon on the PR and ask them to review?
* Amit: Ping me, if I'm not already on it, I will look at it and will approve it
  quickly.
* Johnathan: Sure, I'll do that.
* Hudson: I think the discussion that started at the end of the PR about
  removing the bytes written field -- if somebody is particularly concerned
  about that can just be opened as a separate issue or PR and not hold up this
  bug fix.
* Johnathan: Okay, well I think it's settled.
