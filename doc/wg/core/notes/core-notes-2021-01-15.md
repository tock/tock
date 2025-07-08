# Tock Core Notes 2021-01-15

## Attending
 * Leon Schuermann
 * Alistair
 * Philip Levis
 * Branden Ghena
 * Brad Campbell
 * Johnathan Van Why
 * Hudson Ayers
 * Vadim Sukhomlinov
 * Arjun Deopujari


## Updates
- Phil: Tock 2.0 is progressing. We have a barrier problem because we cannot
  move forward until all of the capsule updates are complete. So I may take
over some of them now.
- Leon: One update on the callback/AppSlice restrictions. I am finding it is
  challenging to find a solution that is not too invasive, that should take a
  few more days.
- Brad: The nano33ble bootloader is merged! That should now be a very
  functional Tock board that would be a good platform for new users -- it is
  inexpensive and reasonably ergonomic to use (though there is a somewhat complex
  setup procedure the first time you use it). Any issues that come up, please let
  me know!

## Yield 2.0 Discussion
- Branden: Linked PR: https://github.com/tock/tock/pull/2351
- Phil: I am applying the updates requested on the PR
- Phil: There used to be one yield, now there are 3 variants. We are adding
  yield-no-wait, which lets userspace attempt to run callbacks, and yield-exit,
  which terminates the process.
- Phil: For yield-no-wait I followed Jonathan's suggestion of yield-no-wait
  taking an address and then the kernel sets a bit at this address to indicate
  if a callback was called.
- Phil: I have not tested the yield code yet but have updated the TRD and will
  push that this weekend
- Branden: So yield-no-wait will only execute one callback?
- Phil: Yes, same as normal yield. There is an edge case where if a callback
  calls yield that more callbacks could run.
- Leon: Can we return an error if a process passes an invalid address?
- Phil: No way to return an error. This is not the only API that works this
  way.
- Brad: I looked at the PR, I think the changes look good. I have one thought
  and maybe one question.
- Brad: Phil your approach requires userspace to pass in the address it wants.
  Alistair on a different call at some point brought up a different design,
  where there would be a set region in a process' memory, which would be
  dedicated for the kernel communicating with a userspace app. This would be a
  general way to tell the process what is going on without an explicit system
  call exchange. I find this intriguing, as it matches this grant region concept.
  And we could potentially find more uses for it.
- Phil: Sounds like AppSlice
- Brad: Right, but this would let the core kernel do this.
- Phil: There might be cases where that is a good design, but in this case I
  don't think it is. Yield is a very special case where only in one case do we
  need/want a return value. The problem with multithreading or reentrant code is
  it can break this kind of thing, if you just have a single set location. So the
  right place to put this is on the stack because it really is unique to your
  current invocation.
- Brad: Okay that makes total sense, and realize what I just suggested may not
  be as easy as I made it sound
- Alistair: I was gonna write up an issue about it, its more for time.
- Alistair: Basically a user process wants the current time, rather than use
  the overhead of system calls, the kernel just writes to memory right before
the kernel returns to the app.
- Leon: What if there was a cooperatively scheduled app?
- Alistair: I will write this up elsewhere, I think these issues could be
  resolved
- Brad: Other question was on exit. We initially talked about 2 versions of
  exit. One is "im done forever" and the other is "kernel please restart me or
  whatever I am in some weird state"
- Branden: So exit-clean-return or exit-with-error basically
- Phil: so we could do this. What do we do differently in the kernel?
- Brad: basically for one of them the kernel might want to eventually restart
  and run the app again
- Phil: Okay. We could do that.
- Phil: Re: why make exit part of yield?
- Phil: What if you wanted something to atomically clear the callback queue and
  exit -- these things could start getting more tied together
- Branden: That makes sense
- Brad: I would not find it intuitive that there is a way to say yield that
  also says restart me. Even if the way it is implemented seems similar
- Phil: These are just system calls, we could hide this behind userspace
  libraries.
- Branden: I think the fact that the OS treats them very similarly and that you
  might want to couple them is a compelling reason.
- Leon: Think of exit as yield-forever
- Brad: I think it would be difficult to explain to someone used to python exit
  or whatever
- Phil: Those are runtimes! Who knows what system calls they use.
- Brad: Okay we can wrap things in abstraction I guess. Don't feel super
  strongly on this.
- Hudson: A little weird that yield is the system call to run callbacks, except
  in the case of yield-exit.
- Phil: Sure, but yield is also always what you use to give up control to the
  kernel.
- Leon: Yeah makes sense to me.
- Hudson: Okay cool.
- Phil: conclusion is I need to go back and reread this old discussion about
  adding a yield-reboot and then based on that detailed reading I will do
  whatever I conclude we agreed on before

## Initial stack pointer and Break
- Johnathan: Clarification on my email: at the end of the email I had a
  question and if anyone knows the answer lets start with that.
- Phil: I am not sure I understand the email
- Johnathan: I am hoping we can change how the kernel sets up the initial app
  break and stack pointer so as to make the userspace initialization easier.
  Also want to know how to support apps that require less than 3kB of memory.
- Brad: Background for those who are not aware. OG Tock kernel did all setup
  for the application (relocation and all that). We changed this around 1.0.
  But we still needed the kernel to setup the stack pointer so we could keep our
  promise of stability for apps compiled at 1.0.
- Branden: Even if the app is setting itself up, it needs some valid stack
  pointer, right?
- Brad: Yes on ARM (hardware uses the stack), no on risc-v.
- Brad: This being arch dependent makes me think this belongs in arch/. The
  kernel doesn't need to know where the stack is. Then we could do this
  differently for ARM and RISC-V. But this question remains -- what is the
  contract between arch/ and the c-init code in userspace.
- Johnathan: For system calls the kernel might still need a stack.
- Branden: But should only need a small stack
- Johnathan: How easy is it to determine just how much stack is needed
- Brad: There is a fixed size, but it is not the size of StoredState (as you
  asked in your email)
- Brad: But it is still arch dependent! This may even depend on whether
  floating point is enabled!
- Alistair: It is very different (8 vs 26 registers)
- Branden: Okay Brad's argument for arch specific makes a lot of sense. Unless
  this is really hard because of how the kernel interacts with all this
- Brad: I don't think it will be too hard, but might be harder to keep the
  debug info about all that.
- Leon: I like the idea of arch specific. But if we have an arch which does not
  need these things setup I would say we should make the least promises to
  userspace as is reasonable.
- Johnathan: The trick is that if making a system call on risc-v saves stuff to
  the stack, we might fall into the situation with ARM where we need the heap
  break high enough.
- Leon: I thought we stored the stored state on the kernel stack
- Johnathan: Ok maybe we would not need that then.
- Brad: Yeah I agree with Leon
- Brad: From my perspective each arch should define this however it wants.
- Johnathan: So the API there would be that maybe the chip provides a function
  that computes the initial heap and app pointer value?
- Brad: Good Q - I am not sure how to set the `app_heap_break` or what we should
  do about that
- Branden: Why do apps need a heap to start off with?
- Brad: They need accessible memory so if there is a stack to start with it
  needs to be in accessible memory
- Johnathan: Honestly it could probably be set equal to the stack pointer, why
  set it differently on arm/risc-v
- Brad: Maybe we can get that through the stored state? I imagine that does
  need to be st in process.rs
- Johnathan: Yeah process.rs has access to the chip!
- Brad: I suspect there is some solution we can write. Maybe what is more
  important at this point is that the kernel should not have anything to do
  with the stack but it does have something to do with the accessible break, so we
  could either set that to some fixed value that we think is conservative enough
  or we could ask the arch crate what we should set it to
- Leon: Setting "conservative enough" could break future architectures. Would
  be nice to not do that.
- Johnathan: I think I can come up with a PR that forwards the decision through
  to something arch or chip specific and then let those of you that actually
  know the necessary sizes for ARM set that offset -- it seems someone already
  knows that. I will write the code and see what people think
- Brad: Sounds great!
- Branden: Yeah if this is passed through to arch its easy to let some
  architectures pick an arbitrarily large enough value and then we could easily
  change this later.
- Johnathan: Yeah we could document that apps should not rely on any extra. One
  of my concerns is very small processes that use very little memory and if
  process assumes lots of memory this could break compatibility
- Brad: Great

## On 2.0
- Brad: I feel it is getting close-ish (maybe not next week, but still). But I
  think when it makes sense we should open a PR from the 2.0-dev branch so we
can more clearly track what the diff looks like and where there is complexity
here.
- Leon: I talked to a git master who deals with Linux release management. He
  suggested a release branch, and that once we are ready to consider this we
  should merge master and 2.0 dev to the release branch, then let people give
  feedback and test on the release branch. Basically PRs are hard to work with
  with big changes, integration branch is easier
- Phil: how is integration branch different from tock-2.0-dev?
- Leon: Merge in the other direction, and this would be a stable point of
  reference for people to test. Basically just a snapshot.
- Phil: My hope for 2.0 is end of the month. Not that much left!
- Leon: I think we feel good about the ABI we worked out, but many things that
  could be improved (callback swapping prevention etc.) -- the PR I will open
  will have rough edges. The key thing is a stable ABI so we can release 2.0
  without perfecting all of that.
- Phil: Agreed. We should pare this down to the smallest possible diff.
- Brad: My preference is to merge the smallest possible thing. Maybe have a
  month where we are still developing on 2.0-beta-1 before we do the actual
  release. Then once we get all these things in and improvements figured out we
  can do a big testing thing.
- Leon: Think we should still do an integration branch and test it before a
  beta release.
- Leon: I think just an atomic switch for ABI and capsules is all we need
- Brad: Right. And it sounds like we are close.
- Phil: Sounds good.

## Bootloader for nrf52
- Brad: Post 2.0 I think we should have a workshop and get people engaged on
  using Tock. We can port the bootloader to other nrf52 boards and it should be
  easy enough for people to buy them remotely.
- Phil: Workshops are hard remotely
- Brad: We support some accessible hardware, I think we could really broaden
  our reach for this sort of thing.
- Branden: agreed. Lot of people at northwestern I would invite to a remote
  thing but not an in-person workshop
- Leon: We would need an updated and fresh tutorial
- Brad: Yeah we should update the Tock book so we have an always improving
  de facto "get started with Tock" material
- Hudson: Would we do this as a workshop at some remote academic conference, or
  as some standalone thing?
- Brad: Standalone thing IMO, but haven't thought a ton about it.
- Phil: I think it depends what we want the content to be. If there was
  accessible risc-v hardware it would be cool to focus on that, but we are not
  ready for that yet. I think there will eventually be a lot of people at
  OpenTitan interested, but maybe not yet.
- Brad: I think multiple sessions might be easier for remote (though that would
  be more setup work).
- Hudson: Sounds like a good thing as first priority post-2.0
