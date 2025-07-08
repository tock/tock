# Tock Core Notes 2022-03-04

Attendees:
- Alyssa Haroldsen
- Brad Campbell
- Branden Ghena
- Hudson Ayers
- Johnathan Van Why
- Leon Schuermann
- Pat Pannuto
- Philip Levis
- Vadim Sukhomlinov

## Updates

* Hudson: Ti50 is trying to use a currently-unmerged LLVM patch that adds linker
  relaxation for RISC-V. Saves about 7% on code size, over 26 kilobytes.
  Suggests that not having relaxation for RISC-V is a major issue.
* Phil: What is linker relaxation?
* Hudson: It's something done by the linker that allows for collapsing certain
  instructions based on addressing modes. Currently, if you look at RISC-V Tock
  binaries, you see a lot of `auipc` instructions followed by `jalr`. That loads
  an address and then jumps to it, rather than just jumping to it. One of the
  optimizations that linker relaxation enables is replacing those two
  instructions with a single direct jump instruction.
* Phil: So basically link time optimization when you can bound values to certain
  ranges.
* Hudson: It was Vadim who initially noticed that a lot of these optimizations
  were missing.
* Alyssa: I'm planning to open an issue about it, but I think I found an
  unsoundness error in `register_bitfields!`. You can essentially create a
  reference to MMIO memory, which is generally not a great thing to do. Or more
  specifically, read from a reference.
* Hudson: Do we currently do this in the Tock kernel, or is it just possible?
* Alyssa: I haven't looked to see if we do this in existing Tock code. We
  essentially need to enforce that a trait bound that a type is MMIO-safe. Ti50
  does have code -- it creates a buffer in MMIO space and right now we're doing
  `.copy_from_slice()` which, from my understanding, is not guaranteed to work
  because of the way volatile memory works with MMIO.
* Hudson: That's interesting, I look forward to seeing the issue.
* Phil: I've definitely seen weird things with MMIO on ARM architecture.
* Alyssa: The key thing is to use volatile reads and writes.
* Phil: Yes, nice catch.
* Johnathan: There is known unsoundness, because if I remember right, the
  `tock_registers` crate creates a reference to a type that has the MMIO memory
  inside of it and LLVM can insert arbitrary dereferences to it.
* Leon: That's exactly right. We do need to address this in the kernel, but I
  would address it by dropping the Rust reference and using regular raw pointer
  operations. If that is insufficient, I may need to rework the design I have
  now.

## `usize`-sized syscall return values (#2981)

* Hudson: Does anyone else have any context about the want for using
  `usize`-sized return values? Jett posted issue #2981 -- he wants to have
  `FailureUsize` and `SuccessUsize` return variants.
* Alyssa: I can give a bit of background on this. Essentially, it is because of
  host emulation, where `usize` is 64 bits. You can't really return pointers
  from Command anymore. [Editor's note: Alyssa's last sentence was hard to hear]
* Hudson: Alyssa said that for Ti50's host emulation code, which is 64-bit code,
  they want to return pointers in response to commands, but there's currently
  not a `FailureUsize` of any sort that allows doing so. That makes it hard to
  write code that works on both the 64-bit host emulation platform and a 32-bit
  platform that returns pointers.
* Phil: I put this in a comment, but if it has to be 64-bit, we can find a way
  around it, but it feels like the tail wagging the dog. We have an emulation
  platform that needs 64 bits, therefore our operating needs to support 64 bits.
  One challenge is there can be bugs that appear in a 32-bit setting that don't
  appear in a 64-bit setting. Why not run the emulation in 32-bit mode? If that
  ship has sailed then it has sailed, but that's one thing I didn't quite
  understand.
* Johnathan: So Jett's question is about things returned from Command, not
  Subscribe and Allow?
* Leon: Yes. For Subscribe and Allow we already took that into account. We
  fundamentally have designed the syscall return enum to use proper pointer
  types for those return values. I suppose that a part of Jett's question is how
  they should in general be returned to userspace -- what a proper 64-bit ABI
  would look like -- and the second is how that translates to Command return
  values.
* Leon: To give some background on the current design, when we thought about it,
  we acknowledge the fact that we would need to transfer pointers as part of
  Subscribe and Allow, but for Command we decided that we really want to fix the
  size of parameters passed to Command and return values from Command to make
  syscall drivers portable. As soon as you add variable-width returns or
  parameters, you start having issues where you write a driver on a 64-bit
  system and it doesn't work on a 32-bit system. I think that this discussion
  should still be interesting because fundamentally -- I suppose -- in the
  future we would be interested in having a 64-bit ABI. How to integrate that
  into the concept of a driver that fundamentally only has 32-bit values for
  Command could be very interesting.
* Alyssa: I think the easiest way to do this is to have the `usize` variant
  always be the `u64` variant, and on 32-bit systems just drop the top half.
* Leon: I'd be strongly against that. It feels like a hack, and you still have
  issues where this could affect portability.
* Alyssa: Having only 32-bit responses is directly affecting portability right
  now because we cannot run on 64-bit systems.
* Leon: Maybe the better term should have been consistency. We want to have
  consistency in what can be returned on different platforms through the driver
  trait.
* Phil: Why not run 32-bit emulation?
* Alyssa: I would like to be able to run it on Linux native without a dependency
  on 32-bit runtime libraries.
* Phil: It seems a little weird -- Tock was designed for 32-bit systems, and to
  change the kernel architecture to be able to emulate it on 64-bit processors
  seems weird.
* Alyssa: Was it designed from the beginning to be 32-bit? I wasn't aware of
  this.
* Johnathan: Definitely yes
* Phil: Yeah. That doesn't mean we couldn't -- someday we may want to run this
  on 64-bit platforms as well, but that will involve a new ABI like Linux has a
  different ABI on 64-bit systems. Those are different ABIs, and trying to make
  them transparent is a way to run into tricky edge cases.
* Leon: If we were to want to support a physical chip with a 64-bit RISC-V core,
  would that be more motivation to work on this issue?
* Phil: I think so, especially because whatever we do we want to ensure that we
  can just run on 64-bit platforms. That means we need to figure out a 64-bit
  ABI.
* Johnathan: I will point out that `libtock-rs` has encountered a similar issue
  which I resolved without noticing it. `libtock-rs`' unit test environment is
  portable and runs on 64-bit systems. The advantage we have is number 1,
  Command doesn't return pointers -- simply don't support that in `libtock-rs`.
  The other syscall types use like Success with u32, consistent with TRD 104,
  and shove pointer-sized values into the registers anyway. It ends up all kind
  of working because `libtock_unittest` and `libtock_platform` are part of the
  same project and have their own ABI between them. The ABI is identical to TRD
  104 in the 32-bit case but a little bit different in the 64-bit case. I didn't
  even recognize that when I wrote it.
* Phil: Does RISC-V say anything about memory maps in its specification? If you
  never have more than 4 GB on your embedded system, then would that ever be
  more than 32 bits? On an emulated system that makes sense, as sure you can
  have more than 4 GB.
* Leon: I think once you start having a 64-bit RISC-V architectures, then your
  pointers ought to be 64 bits wide. I think that RISC-V does not make any
  guarantees regarding physical memory maps, as fundamentally RISC-V is an
  instruction set, not a microarchitecture.
* Phil: The ARM ISA says things about memory maps.
* Phil: If this is the beginning of the push to support 64 bits, then we should
  just support a full 64 bit ABI.
* Johnathan: Something that's been bothering me with this whole conversation is
  the potential miscommunication. My understanding is the Tock core developers
  -- even before I joined -- none of you wanted pointers to be returned from
  Command, and Ti50 is returning pointers from Command. I feel like there was a
  documentation weakness or a lack of communication that lead to that design.
  It's nagging at me because that feels like the root cause -- they wrote a
  bunch of code based on an expectation that disagreed with our expectations in
  an undocumented contract. I'm not really sure what to do with that observation
  but I think it's sort of the fundamental issue here.
* Leon: I think returning pointers is actually very reasonable. For instance, if
  you have a shared buffer between the kernel and userspace and you want to
  point to arbitrary data in the buffer. You would like to use actual
  pointer-sized values for the indexing operations and not rely on the fact the
  buffer is limited to 32 bits.
* Phil: I disagree, the TRD is quite clear. It says "Command calls should never
  pass pointers". It doesn't say anything about return values, so we should
  check what the return value specification says, but it's very clear you should
  never pass pointers.
* Leon: That's very interesting, because how would you implement a structure
  like a ring buffer shared between userspace and the kernel?
* Phil: You don't then.
* Alyssa: You would need to use indexes rather than pointers. I've suggested we
  do that right now in host emulation -- use a 32-bit offset from a base
  address, but that's been difficult. I wish Krzysztof were here right now
  because he could better explain the use cases.
* Hudson: Hopefully Jett or Krzysztof or someone will chime in with more details
  on the use case in the issue that already exists.
* Johnathan: We may want to shelve this until next week, as we're having a
  conversation about an unexpected use case and the people with that use case
  aren't here.
* Hudson: I don't think that's the worst idea.
* Phil: We had some discussion, but obviously we should not reach any
  conclusion here because we want to hear from the folks are encountering this
  and understand the issues in play. The writeup in the TRD says you never pass
  pointers, in retrospect it probably should've said you don't return pointers,
  but it doesn't. We can talk through that and sort that out.
* Johnathan: Yeah, part of this is that I think the problematic code predated
  Tock 2.0.
* Johnathan: Oh, next Friday is a company-wide holiday, so it might be shelved
  for two weeks.
* Hudson: We can continue discussion on the issue.

[Editor's note: Phil had to leave the meeting at this point]

## Updating the Rust toolchain

* Brad: Can we merge the update to the new nightly?
* Hudson: I was not clear on if we were going to wait for the
  `const_fn_trait_bound` stuff to work itself out, but it's been long enough at
  this point that I guess would should merge now. We can update again if we want
  to.
* Brad: That PR becomes unmergable every two days, then it takes a few days for
  someone to fix it, perpetually.
* Alyssa: Is there a major update on `const_fn_trait_bound`?
* Hudson: The update is they accidentally stabilized a workaround that allows
  you to do it, and then everyone decided that instead of dealing with the fact
  they stabilized a workaround, they'd go ahead and stabilized the feature.
* Johnathan: But they haven't merged the stabilization yet?
* Hudson: Correct
* Johnathan: I'm subscribed to the stabilization issue now.
* Alyssa: This is one of the only things our codebase actually depends on, so
  I'm excited for us to be able to use in on stable.
* Hudson: It feels like we're really close to stable Rust, as there's also been
  progress on naked functions and `asm_sym`.
* Alyssa: I think the only thing we'll end up depending on is a couple of
  nightly-only intrinsics.
* Hudson: We've talked about getting rid of the intrinsics in the core kernel,
  but haven't pulled the trigger yet.
* Brad: That const mut refs though, I don't know.
* Hudson: Where is that used again?
* Brad: That is used in a lot of the library code -- a lot of the cell code --
  and propagates through the capsules.
* Leon: We also used it when we had to use `const` functions in the chip crates,
  I think for peripherals?
* Hudson: Yeah that sounds right.
* Brad: And that one there was like some progress, then there wasn't process,
  and I think it could take another year.
* Alyssa: I couldn't even figure out how to implement it for my
  `impl From<Option<T>> for const T`.
* Hudson: I `bors r+`'d the PR, Brad.
* Brad: Fantastic.
* Brad: I think the summary is that yes, there is optimism, but const mut refs
  seems like it will be a real issue.
* Hudson: Yeah
* Alyssa: Can we have a separate initialization pass for `const` contexts that
  doesn't use mut refs?
* Hudson: The problem is we have a lot of cells, like `OptionalCell`, and you
  can contain a reference in an `OptionalCell` or a `TakeCell`. You may want to
  initialize it as empty, and use a `const` constructor so you can initialize
  something as a static mut global, then later the type still needs to be a
  mutable reference as it is a global you may change it to a non-`None` value. I
  believe it is less of an issue now because we got rid of a lot of the `static
  mut` peripherals, but I'm guessing there's still a couple `static mut` things
  that require `const` constructors and also contain
  `OptionalCell`/`TakeCell`/`MapCell`, which therefore need to have `const`
  constructors. It's maybe possible there's something we can do here, but I
  don't think it will be very straightforward.

## ProcessSlice raw pointers PR (#2977)
* Leon: Brad, you said you wanted to port over some more code, otherwise I would
  spend the weekend looking at some capsules. How should we progress on that?
* Brad: I just looked at the first capsule, which is the app flash or something,
  which looked like it had a straightforward "copy between two buffers using the
  indexes" and I said "I can change that to use `.copy_from_slice()`". The
  problem is that the Rust `.copy_from_slice()` API is that the slices must be
  the same length, and in a lot of cases we don't want that. We want to copy as
  much as we can, and then tell me how much you copied, and we'll iterate
  anyways because it's event-driven. I tried to add a new API that will handle
  all that internally for you. I got a little ways into that and got distracted
  or something. I can share what I have or find some time today to try to get
  that working, but that's as far as I got.
* Leon: I think the usual approach is to calculate the minimum of the two slices
  lengths, and use that for subslicing, then use `.copy_from_slice()` or
  `.copy_to_slice()`. I've also been using Rust's iterators, which are
  implemented by the ProcessSlice types, which you can combine using `zip()`.
  That may be an option to avoid re-implementing the API.
* Brad: I'm remembering why I got stuck. If you don't want to use the `[]`
  operator, then you have to use `Result`.
* Leon: If we manage to use `zip`, then it's automatically bounded to the
  minimum of the length of the two slices lengths.
* Brad: Great, that sounds like that is encapsulating that sort of
  `unsafe get_unchecked`, which would be great. If you've got a prototype of
  that or something, that sounds great.
* Leon: I'm assuming it works, I'm not sure if it works on two different types
  of iterators, so I'll try.

## App completion codes (#2914)
* Brad: Were there any outstanding issues on app completion codes?
* Alyssa: I thought that was merged after an adjustment was made. Did it not?
  Otherwise I can get it merged.
* Brad: It's not merged.
* Alyssa: I don't think there's anything else. I'm okay with the new language,
  and it sounds like everybody else is.
* Hudson: It seems like we should merge that.
* Brad: I'll do that.
* Brad: We can always edit it.
* Alyssa: I did have one question about the wording. It says that the
  specifications and exceptions must be written in a TRD. Can people just write
  TRDs and not share them publicly, as long as they stay on a specific version
  that works for them? Can Ti50, if we say we're going to stick on a particular
  Tock version can we write our own TRD that says the kernel can assume semantic
  meaning for completions because we wrote our own specification for our
  application and our own branch of the kernel. Would that be violating this TRD
  to do that, out of curiosity?
* Leon: As far as I interpret it, I think the TRDs only mandate things for
  upstream changes, so what you do downstream is not necessarily compatible with
  how Tock development continues. I don't think TRDs are meant to limit what
  downstream users can do. I'm not an authority on this, though.
* Brad: I think this would be a question for Phil.
* Alyssa: It seem Phil has the strongest opinions for the legal language here.
* Brad: I would concur.
* Pat: And just the most experienced, but I do agree with Leon's assessment.
* Alyssa: It's basically contract law.

## Storing kernel-managed grant values in one `usize` (#2958)
* Hudson: I'm planning to come back and think through some of those soundness
  issues after a conference deadline.
* Brad: Sounds good. I would like to try to measure it -- to measure what the
  size change looks like. As long as there's no other major updates to `Grant`
  in the meantime it should be okay.
