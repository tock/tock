# Tock Meeting Notes 2026-06-24

## Attendees
- Alexandru Radovici
- Amit Levy
- Brad Campbell
- Branden Ghena
- Johnathan Van Why
- Leon Schuermann

## Updates
- Johnathan: tock-registers work moved fast, stumbling block because our DMA
  plans are unsound. Need to decide whether we want to keep UnsafeRead and
  UnsafeWrite. Those may be removed or look different. Need to rewrite safe DMA
  example.
- Leon: Some context. It is pretty much impossible to perform sound DMA, even
  with DmaSlice, without atomic volatile reads and writes. The only way to do
  that right now is with `asm!`.
- Brad: This is getting silly. We have to draw the line somewhere.
- Johnathan: For the record, I agree, but all the formal semantics people are in
  agreement.
- Amit: Who are these people?
- Johnathan: t-opsem Zulip members.
- Leon: Generally most of the people defining the semantics of Rust.
- Amit: You can't do this in C
- Johnathan: C has volatile atomic operations.
- Amit: I think the state is Rust does not have an equivalent of volatile atomic
  operations, so in the formal world there is no way to tell the compiler in a
  way it formally guarantees to do the thing we want. There seems to be a move
  to introduce that. Is it the case that volatile, in practice, breaks things?
- Johnathan: No, no way to miscompile. More of a theoretical unsoundness.
- Amit: Can use the technically unsound but practically safe thing now, with a
  TODO to swap it once the volatile atomic operations land. The practical effect
  is the same binary will be output but with a more formal guarantee.
- Leon: Because tock-registers provides all the abstractions we need, we could
  have it default to the sound option implemented via `asm!`, then disable the
  cargo feature in Tock.
- Amit: Are you talking about having assembly conditioned on hardware we use in
  Tock?
- Leon: tock-registers has inline assembly snippets for architectures we support
  but Tock opts out.
- Johnathan: There are two separate things here. Long run we want to use atomic
  volatiles by default in tock-registers, but for now that can only be done with
  `asm!` with a 0.5% code size impact. So for now we keep using non-atomic
  volatiles for non-DMA stuff, and something else for DMA.
- Amit: Lets move on, this is getting in the weeds.
- Brad (in chat): I fully support ignoring theoretical unsoundness.
- Branden: I thought we were doing tock-registers 0.11 before doing all the
  updates? Did that get nixed?
- Johnathan: That's still the plan.

## Brad's PRs.
- Amit: How many of these need discussion versus attention?
- Branden: Is there an order you're looking for?
- Brad: Yes, I put them in order.
- Amit: It's not very efficient for us to go through them in order if we're not
  going to talk about them. What about looking the AI in the PR template
  discussion first?
- Brad: No, I put it last for a reason.
- Amit: Lets start from the top.

## PR 4338
- Brad: Documents our process loading strategy. There are a lot of comments, but
  they indicate they don't like the strategy rather than the doc being
  inaccurate. Documentation written a year ago is sitting there. I think
  documentation on what we have is better than documentation on what we want.
- Amit: I'm generally in favor of having a document that is descriptive of what
  is currently in main. You're saying that that is what this PR has?
- Brad: Correct.
- Amit: I think this got lost in the weeds because this was created concurrently
  with #3941.
- Branden: The trouble is, it always felt like a TRD is about how we're supposed
  to do things.
- Johnathan: I ran into that on the RISC-V 64-bit ABI. We don't have a TRD type
  for a specification. We're treating TRD 104, which is documentary, as a
  specification. Hole in the TRD process.
- Amit: We should look at Pat's requests and convince ourselves that they are
  arguments with the status quo rather than mischaracterizing the status quo.
- Branden: The comment on 150 about covering integrity and not mentioning
  confidentiality might be relevant. Line 196 as well.
- Amit: For 150, the suggestion is to remove the specific requirement. Is there
  a meaningful confidentiality guarantee?
- Brad: I don't know of any confidentiality, or what this has to do with
  confidentiality?
- Amit: Presumably, it would be something about not being able to read other
  binaries?
- Brad: But that has nothing to do with process loading.
- Amit: I don't agree with Pat's comment.
- Johnathan: With no context, I also do not agree with his comment.
- Branden: Then mark it as resolved.
- Johnathan: I don't want to do that now, I need to know more context. Kinda
  burnt out on PR reviews this week, so would rather focus on other PRs.
- Amit: Line 196, what happens when finalize fails.
- Brad: I don't know.
- Amit: Lets see if we can find out.
- Leon: FWIW, what I was trying to get at is not whether our current
  implementation does in or whether we should give guarantees. And we should
  state that.
- Amit: `finalize()` doesn't fail.
- Amit: I think the answer is no, it doesn't guarantee anything, based on what
  we see in the PR. Deallocating resources is up to the caller.
- Leon: We should probably write that down.
- Amit: Is that relevant for abort and setup as well?
- Amit: Leon, can I resolve 164?
- Leon: I just resolved it. I don't have time to review the full document, but I
  don't want to block on this. I think my pet peeve is that the semantics of
  finalize weren't clear, but I think that is mostly resolved at this point.
- Amit: I'm looking at line 230 now, it looks like Brad just responded.
- Amit: My sense here is that either the paragraph is unnecessary completely, or
  that Pat's suggestion is much closer in meaning.
- Brad: Okay, lets commit it.
- Branden: DynamicBinaryStore is not layered on top of DynamicProcessLoad.
- Amit: I agree with Branden's original comment, that it's not clear what this
  diagram is trying to show, but I don't understand Brad's response.
- Brad: How do you draw a struct that implements two traits?
- Amit: What does this diagram show?
- Brad: The struct `SequentialDynamicBinaryStorage` needs a reference to
  `SequentialProcessLoaderMachine` to implement the `DynamicBinaryStore` and
  `DynamicProcessLoad` traits. And it needs something that implements
  `hil::NonvolatileStorage`.
- Amit: I would argue that's not hard to say, and saying it would be more clear
  than this diagram. I will propose some text.
- Branden: I think that makes sense to me.
- Brad: For me, this is so much easier to understand.
- Branden: Neither Amit nor I understood it at all.
- Amit: I will make a suggestion to add the text.
- Branden: I made a suggestion to merge lines 252, 253 and separate them with a
  comma. Do you think that is reasonable or unreasonable?
- Brad: Yeah that's fine.
- Amit: Okay, how's that?
- Branden: That text makes sense to me.
- Amit: This is another suggestion in place of the diagram, just Rust.
- Branden: I think it's already merged.
- Amit: Yes.
- Branden: I think all the other comments are arguing about the design, not the
  documentation.
- Amit: I approved.
- Branden: I approved too.
- Amit: There's formatting issues, because we edited the Markdown.

## PR 4841
- Branden: There's one comment by Amit on 325.
- Johnathan: This is the recurring topic that we're discussing on Matrix.
- Brad: There's no DMA here.
- Amit: Is this safe? If it's safe, is it because it's read-only, because
  there's no DMA, or something else? I think previously this was unsafe and the
  invariant is you only did it once.
- Brad: That's a question for Leon. He's been saying it's okay as long as
  there's no DMA.
- Leon: I'm trying to be careful with these general statements. I think there's
  a high likelihood of this being fine. If we have a const ficr_base staticref,
  does that allow mutation?
- Amit: Yes, except I believe there is no actual mutation in this particular
  interface.
- Branden: This driver ends up being read-only.
- Leon: StaticRef does not give you an exclusive reference to something.
- Amit: It does. It gives you ownership over the FICR instance.
- Leon: I thought our register types only need shared references. We also need
  the registers to be shared. I think that's true for StaticRef.
- Amit: Can you say that again?
- Leon: If I hold ownership over the StaticRef.
- Amit: There are no StaticRefs.
- Leon: FICR_BASE is. I think I can only dereference to a shared reference, not
  a unique reference.
- Amit: I don't recall.
- Leon: Under that assumption, there's a chance this is safe assuming it is
  instantiated on the correct board, and it contains no type with interior
  mutability, then I believe it is safe. The safety invariant that is not
  discharged is that it is only instantiated on a chip whose register layout
  matches and doesn't invoke UB.
- Johnathan: I think we need to figure out how to construct drivers in general,
  debate that policy once, then we don't need to debate this on every PR
  separately.
- Amit: There are two invariants here. One is general, one is specific to this
  peripheral. The thing specific to this peripheral is that every register is
  read-only. The other is that this is only instantiated on boards where
  FICR_BASE is valid and it has the shape of the FICR peripheral. That is also
  something we've been asserting through `unsafe`.
- Brad: Why are we making a point of this one? We do this exact same thing in
  400 places in Tock.
- Amit: First of all, because this is where I caught it.
- Brad: I've been merging boards that do this. Why this PR?
- Johnathan: #4856 has the same hangup.
- Leon: This is the way that we've been doing things.
- Amit: But `static mut` requires `unsafe`. We're historically used `static
  mut`. Major difference.
- Brad: This is a weird one-off case.
- Leon: First of all, this is not a public const. This is a const scoped to the
  module. That's been used forever. What I'm arguing for is that we have never
  formally reasoned about when and when not that is safe. This is popping up
  because we're more aware of the issues around it now.
- Johnathan: One thing that's motivating us to look at this is the desire to
  unit test drivers, which requires us to run these drivers on a host system.
- Amit: I think this pattern is pretty unsafe, so I commented, and didn't get a
  response, so I didn't merge it. I think it's safe now, so we can merge this.
- Brad: We can block a different PR that's explicitly about this.
- Leon: This is not about any particular PR, this is about a general shift in
  how we think about these things.
- Brad: There was no response because there were two PRs at the same time and we
  were having the same discussion on both.
- Branden: The PR has a conflict.
- Brad: I'm writing code that is SOP for Tock and then getting comments on that.
- Johnathan: One issue is I don't know what is SOP for Tock. I haven't reviewed
  much kernel code until recently, and now I'm seeing these patterns and
  objecting.
- Leon: We have the issue that this has grown organically, copying what other
  chips do. There's not a formal way to judge which ones can be safe and unsafe.
- Amit: It's very clear they ought to be unsafe, because it's still possible to
  create multiple and still possible to create this on chips that don't have
  this hardware. So this should be marked unsafe, and the board instantiating it
  should attest to those variants.
- Leon: That's my position, but I think we shouldn't litigate this now.
- Brad: I would be interested in seeing an example of the second one. How do you
  instantiate this on a chip that doesn't have the peripheral?
- Amit: You could import the crate. It's possible for a chip implementation to
  have an implementation of peripherals that exist on some variants of chips but
  not others.
- Brad: That shouldn't be possible. It shouldn't be enforced via unsafe, it
  should be enforced via other mechanisms. cargo and features.
- Leon: That's not how Rust reasons about this. Whether or not something depends
  on a certain crate should not be relevant.
- Brad: I have a different opinion on that.
- Amit: I think my singleton argument is sufficient anyway.
- Leon: That applies to most drivers, but we have a potential solution for that.
- Johnathan: tock-registers has a type to represent a singleton register handle
  as well.
- Leon: There needs to be a place for the developer to discharge the invariant
  that it's running on the correct chip. But I think we'll be stuck for an hour
  if we litigate this now.

## SHA capsule #4855
- Brad: Can I have a request for someone to look at this? It's blocking my
  student.
- Leon: Don't we gave a crypto WG?
- Amit: This is somewhat blocked by that WG's dysfunction.
- Brad: It's been a month.
- Alexandru: Similar issues here, can we join as an observer. My colleagues have
  a lot of questions.
- Amit: I will look at SHA.
