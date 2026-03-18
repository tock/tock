# Tock Core Call Notes 2026-03-11

Attending:
- Johnathan Van Why
- Brad Campbell
- Alexandru Radovici
- Leon Schuermann
- Hudson Ayers
- Branden Ghena

## Updates

None.

## Threat Model Appendix PR (https://github.com/tock/book/pull/81)

- Johnathan: Brought up few weeks back. https://github.com/tock/book/pull/81 Has
  seen reviews from Brad, but no other responses yet. What's the state of that
  PR? Ready to merge, anything else needed? Should try and either get this PR
  ready to merge, or consensus on what needs to change.
- Brad: seems like a sign that we should merge it.
- Johnathan: only designed for one AppID per host OS app. Probably fine to leave
  as is.
- Brad: most comments just clarifications.
- Consensus: let's merge.

## `DMASlice` Discussion

- Brad: Amit opened the Cortex-M implementation. I decided to try changing an
  NRF chip. There's not currently a use of DmaSlice, right?
- Leon: We're using DmaSubSlice in VirtIO, but not DmaSlice.
- Brad: I see. Is that what people should use?
- Leon: It depends on if you have a subslice or a regular Rust slice.
- Brad: Whether you have it, or whether you should have it.
- Leon: I think you're always able to create a subslice from a Rust slice. Maybe
  we don't need DmaSlice for regular slices, because we can always put them in a
  regular slice. The thought is if you have a Rust slice, as we do in many HILs,
  then you use vanilla DmaSlice. If you have a DmaSlice and want to use part of
  it, then you use DmaSubSlice. Amit was similarly confused about how to use it.
  Similar complexity about `immutable_from_into_bytes`. In Amit's WIP code, what
  didn't feel right was you have a bunch of `unsafe` method calls in your
  driver. E.g. `restore_mut_slice_ref` is unsafe. None of that complexity or
  unsafe existed before in the driver. That's odd, we're trying to remove
  unsoundness but are adding unsafety. Our conclusion, preliminarily speaking,
  was that DmaSlice and DmaSubSlice do make sense, and the safety is required,
  but we have an impedance mismatch between registers being safe to write but
  we're encapsulating fundamentally-unsafe operations.
- Brad: Okay
- Leon: A takeaway is that this PR is okay as-is, but we need to re-think the
  registers abstraction. We should encapsulate the DMA abstraction inside the
  registers abstraction to provide an overall safe API.
- Brad: I think I understand. On the surface, writing a register and the fact it
  happens to use a memory instruction, to me is perfectly fine. Is it really so
  bad to write a number that happens to be a pointer to something in the
  hardware that we call a register?
- Leon: I don't think that's the issue. The problem is that we have two
  philosophies on how we are thinking about safety that happen to collide.
- Brad: So it's more the unsafe
- Leon: Yes. I don't know if Amit has pushed this yet, but now in the driver,
  when DMA is finished, you write to a register that cancels the operation. That
  is safe. Then you use an unsafe operation through DmaSlice to assert that the
  operation is done writing the buffer, which gives you a Rust slice back. It
  would be nicer if this registers object had a safe method you could call to
  start and stop a DMA operation, that the driver can safely call, which
  internally does the two unsafe operations (writing the pointer to a register,
  and doing the DmaSlice operations).
- Brad: Branden, you just joined in the middle of this discussion. Do you
  understand, do you have any questions?
- Branden: I don't understand, but I don't want to make you have to recap.
- Brad: There's confusion between modifying registers being safe but DmaSlice
  operations being unsafe. Wouldn't it be nice if we could similarly wrap that
  complexity somehow so we don't have to add unsafe everywhere.
- Brad: It makes sense for that complexity to not be part of this. Right now,
  we're doing a bunch of unsafe stuff, without making that explicit. DmaSlice
  makes that explicit. [...]
- Leon: I think that's fair to say. There's a clear pathway to transition
  between those steps. When we port a driver to use the DmaSlice infrastructure,
  we solve concrete soundness issues at the cost of introducing more complexity
  and more explicit uses of Rust's `unsafe` into drivers. In a second step, we
  can reduce that complexity by introducing a new layer of abstraction that
  hides that `unsafe` and uses safe operations on registers. That new layer
  would assert about the state of the hardware.
- Brad: Do you know what that would look like? That would still be
  chip/peripheral-specific code.
- Leon: Amit was thinking about whether the tock-registers rework could allow
  for that. Maybe you could extend the registers struct -- add a new method like
  `start_dma`. A challenge of this is the registers struct would need to keep
  extra state around. It's currently just a pointer to device memory. On a
  conceptual level, we know how this could work.
- Brad: Even what you just described, there's a method. It has to be implemented
  somewhere. That has to be in the chip crate and unsafe.
- Leon: Thinking one step ahead, we have the goal of having drivers be
  `#![forbid(unsafe)]`. There's some point where we have register definitions in
  one crate, and the driver implementation in another crate, and this would
  belong in the unsafe registers crate.
- Brad: What we have with the registers interface right now is all the
  complexity is shared. Each chip just has to name them. With DMA, understanding
  the complexity of which registers you have to write cannot be generalized. It
  does seem like there's a relatively small amount of code there.
- Leon: That's the only invariant we need to maintain for DmaSlice. If you look
  at the documentation — which is currently cryptic — there's a no-drop
  invariant, and a guarantee that the hardware will no longer touch the buffer
  after it is done. These have to be coupled, but when you couple them you can
  wrap it in a safe abstraction.
- Branden: I have two thoughts. One, I don't buy it. I think this is really hard
  to encapsulate. I'm happy to be proven wrong, but I think that is pretty
  high-effort. However, I think there is a second point that chips don't have to
  be safe, we just thought that they could be and want it. I see this as an
  experiment — could we make them safe? Maybe yes, maybe no. Or maybe we can
  break them in half, with half unsafe and half safe.
- Brad: To your first point, yeah, good concern. To your second point, I'm
  pretty optimistic — `unsafe` is a virus, is too tempting, it's a thing we
  always want — and I'm pretty convinced with more careful abstractions and
  better tools, we can get it down to about 100 lines of code per chip. It's
  only hard because we've dug ourselves such a big hole, like using `static
  mut`.
- Branden: But I think that DMA is going to end up being a big chunk of that
  ~100 lines.
- Brad: Basically, yes. That's why it's 100 lines and not 10 lines. Leon, you
  said this is tangential but I guess it really isn't. I think `DmaSlice` right
  now is designed for people who think about programming language, and
  compilers, and safety, and it's not designed for the users. I don't have a
  problem with writing things for PL people, but I'd rather they be encapsulated
  in the kernel crate. For something to be exposed to chip authors, I feel like
  it's our obligation to hide this complexity from them, so they get it right,
  not try to teach them why the complexity makes sense.
- Leon: That's a fair point. In that sense, I don't think it's tangential at
  all. Amit seems to be very confident that, for a large subset of peripherals,
  we can encapsulate this safety properly. There's already a lot of complexity
  in the DmaSlice layer I'm proposing here. It sounds like it is an abstraction
  that you can use to build sound drivers. At the same time, it is definitely an
  abstraction that is exposing PL-style issues to users. Your concerns are very
  valid. This is a step we need to get to the bigger picture, but we would want
  to build abstractions around it.
- Brad: That's what I think we need to talk about then. It's not clear to me
  that there's another layer of abstraction on top of DmaSlice that chips use.
  There's an additional tool that we would add, that merges with the registers,
  so that when you're writing a pointer to a DMA register, and you're doing all
  this.
- Leon: I don't know whether calling it a tool on top or a layer is
  substantially different. All of this complexity, right now, exists or ought to
  exist in all the drivers that do DMA. We're trying to unify those abstractions
  into one we can reason about and make the complexity explicit. It either
  already exists or it doesn't but the crate is unsound.
- Brad: What does the chip author see? They see DmaSlice/DmaSubSlice. They'll
  have to implement that method we were talking about earlier. If that's not the
  case, that there's another layer to be exposed, I'd like to see that. I don't
  want to port drivers over to this interface then port them again later when we
  introduce a new abstraction.
- Johnathan: I think I see clearly that, if we build a layer that integrates
  with registers it will build on top of DMAFence, not necessarily DMASlice.
  Because it might use Rust slices at its abstraction boundaries, so effectively
  DmaSlice would be internal only.
- Branden: it would essentially re-create parts of DMASlice.
- Hudson: seems impossible to only use DMAFence, need to use slices somewhere?
- Johnathan: but if we have an operation on the register it would still just
  take a regular Rust slice?
- Hudson: It would take that as its input, but then still need to avoid storing
  the slice.
- Brad: cause it's the chip author that needs to reason about all these
  guarantees.
- Johnathan: If this is part of a registers abstraction, we don't necessarily
  need to store the slice, or can only store it in registers.
- Leon: I think that's an optimization, that's not necessary, we might be able
  to store it. Either way, there are two lines of thought here. I fully agree
  that the wrong move is to merge this, completely port over, then change how
  we're exposing this to developers. I'm also relatively confident that we have
  a correct, sound set of abstractions that we can use to build sound DMA
  peripherals and that is potentially also useful in constructing this
  high-level interface that developers can use. Breaking this apart at the level
  of this PR is still the right strategy and seems like the right layer of
  abstraction. I don't want to hold this up, because it's been brewing for a
  while and we're getting confident in it, but I also understand if we want to
  take a step back and look at what the higher-level abstractions look like
  first. I'm conflicted, I'm not sure if we should move forward.
- Brad: After this discussion, we don't want to convert and convert back. It
  does seem like when we implement drivers, we can implement `start_dma`
  functions and do this wrapping ourselves to get a feel for it. That would then
  presumably be a direct port if we add a more standardized method. That seems
  like a reasonable balance. Based on what we discussed, we don't need to wait
  to have that mechanism to move forward. My proposal is to rename the functions
  in `DmaSlice`. Instead of `from_whatever`, just `new`. Get rid of
  `immutable_iterable_buffer_thing`, `from_slice`. I like the `revoke`
  terminology from `DmaFence`, something that implies that you no longer have
  it. That to me would be the intuitive name. To me it's just that, changing the
  docs to be user-focused instead of explanatory and using names that either
  mimic subslice or feel more intuitive.
- Leon: That has also been the consensus of everybody that I've talked to about
  this. This is not news. I don't think I should be the one trying to make those
  changes because I don't have a good feel. `immutable_to_from_bytes`, apart
  from the name, I think we need it. That is also not externally exposed.
- Johnathan: Yeah I can't revise the docs or names either because it makes
  perfect sense to me. That probably means I'm too deep down the PL rabbit hole.
- Leon: There's no overlap between people who can write these docs and people
  who can revise them for usability.
- Brad: *missed*
- Leon: *missed* could take a . The fence operation is meaningless, because you
  can still expose inconsistent data to the DMA hardware. We need the Ts to not
  have interior mutability. Additionally, there is or used to be a guarantee
  that the user needed to assert that the Ts can be safely converted into a
  fully-initialized sequence of bytes. Rust types can have uninitialized bytes
  in it. There's also a requirement because the DMA hardware can write to them,
  that they can be converted into from an arbitrary array of initialized bytes.
  Like if you interpret a network packet as an enum with limited variant. So we
  have to limit what types can be used, which immutable_to_from_bytes
  represents. This combines the restrictions of zerocopy's FromBytes, IntoBytes,
  and Immutable. Having this trait bound allows us to make some operations safe
  that otherwise would not be safe.
- Brad: Everything you said makes sense. However, why do we need T? Why can't
  this just be a u8 buffer.
- Leon: I've received that question a couple times. One example I know where a
  u8 wouldn't work is OpenTitan's crypto accelerators, which require 32-bit
  alignment. Those do use DMA.
- Hudson: Could those be cast to u8 and back?
- Leon: Yes, but that would add either runtime checks or add unsafe. Rust
  doesn't have safe transmute between u8 and u32 in the standard library, so we
  would have to use unsafe. But the point is this trait is only exposed to users
  when we use a T that we don't have a *missed* for.
- Brad: *missed*. T has to be this thing, which is not imported from a library.
  What is this thing -- a DmaSlice thing? What am I missing here. As opposed to
  a u8, which is clearly a buffer. It is nice to have a concrete example we can
  implement today. This type does not seem like something we should have to
  create.
- Leon: Many argue it should, especially the zerocopy authors, who are doing a
  lot of hard work reasoning about the safety implications. We're doing
  something much less sophisticated, taking a few types and manually asserting
  the invariants hold.
- Hudson: Is there a reason you only did the unsigned types?
- Leon: There is not.
- Brad: I get we don't need the zerocopy crate, but is there an argument this
  should be a library in Tock.
- Hudson: But the library would just be a worse version of zerocopy.
- Leon: It would be much much worse.
- Johnathan: Just use zerocopy.
- Brad: That's why it would make sense to do it as a library, we would just be
  doing a Tock version of this external thing. Maybe now, maybe in the future,
  we get to the point where we need something more powerful and we replace it.
- Leon: My concern is less technical. What we're doing here does not rise to the
  level of us wanting to make any sophisticated claims.
- Johnathan: I'm hearing us spend a lot of time trying to avoid zerocopy because
  of our third party dependency policy when zerocopy is better-maintained than
  the Rust standard library.
- Brad: I think we can just have 2 DmaSlices. We only have u8 and u32 cases.
  Keep it simple.
- Leon: It would be a whole lot of code duplication. Would it help if we had
  either a default type assignment to a u8 for that T, or a type alias for a
  DmaSlice over u8?
- Brad: No. I think the better solution would be to move that trait into
  something that makes it feel more general, so that when you're reading it you
  can see where it comes from.
- Leon: I'm more than happy to move it, maybe renamed, to utilities. That's
  perfectly fine. I think you're right that this is not necessarily only
  applicable to DMA.
- Brad: I guess we could talk about, do we really need it to be so generic, or
  could we call it U-something? At the end of the day, that's less of a concern
  than the function names and moving documentations around. There should be a
  part you read if you want to understand why and a part you can read if you
  want to just use it.
- Leon: I think we should workshop this in GH. I agree the naming is bad now.
- Brad: I'm at a point where I feel much more confident. I think we should check
  if Amit pushed his code, I would be curious to see what that looks like. I
  feel like we have a plan.
- Leon: Very cool
- Brad: Did you see my reformatting PR? https://github.com/tock/tock/pull/4759
- Leon: I did not
- Brad: I just created it an hour ago. Does anyone else have thoughts or
  comments on this?
