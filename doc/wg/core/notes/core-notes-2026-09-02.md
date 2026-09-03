# Tock Meeting Notes 2026-09-02

## Attendees

- Johnathan Van Why
- Leon Schuermann
- Pat Pannuto
- Alexandru Radovici
- Amit Levy
- Brad Campbell


## Updates

 - Leon: There is a second treadmill deployment up and running in Seattle
 - Leon: Want to do a more in-depth demo, but probably a week or two out to something live / big
 - Amit: This is not anything in the nrf family, it's an STM board
 - Amit: If people have requests for hardware platforms to add, lmk

# Naming scheme for Token types

## Context

As part of the `unsafe` cleanup effort, we've got two PRs in flight that
replace `unsafe` with a token attesting to the current execution context
(#5095, #5130).

The thing I'd like to talk about briefly on the call is Brad's most recent
comment on #5095 re: naming these things:
https://github.com/tock/tock/pull/5095*issuecomment-5443562519
the text here for simplicity as it stands alone reasonably well:

I'm on board to call all of these things "tokens": Capabilities, interrupt
context, panic context. What trips me up is the confusion on what "Context"
means:

   - Is it the state of affairs when interrupts are disabled, or the
      language has jumped to the panic handler (i.e., "This code is
running in an
      interrupt disabled context.")?
      - Or is it the thing that proves the the context is valid (i.e.,
      "This code has a context, specifically a InterruptDisabledContext, so
      it can prove that interrupts are disabled")?

Some brainstorming with claude, and I like "Ticket". ("Badge" is my second
favorite.) So, we would have this:

               Tokens in Tock

               /       \     \--------------------\

              /         \                          \

  *Capabilities*          *Tickets*                  *something else
in the future??*

  - MemoryCapability      - InterruptsDisabledTicket

  - ProcMgmtCapability    - PanicContextTicket

So, we would then rename context_tokens.rs to tickets.rs.

## Notes

 - Amit: Why not just capability
 - Pat: It's something semantically different; capabilities are largely about sensitive system operations, e.g. "do I have permission to start/stop processes", while these are more about providing a mechanism to enforce something programmatically that were previously comment-contracts on non-soundness-related `unsafe` documentation, e.g., "this function relies on interrupts being disable for correct execution"
 - Johnathan: Like "ticket"; "context" has other meanings in software eng
 - Leon: PL often uses "marker types" or "affine marker types", don't really like those
 - Amit: Still don't understand difference; capability is a immutable statement of authority
 - Brad/Pat: Think it's the "authority" that's different; they're not permissions
 - Amit: Are they not in the sense that they specifically permit you call functions that require that state?
 - Amit: Let me say differently: It's not the case (looking at impl) that the token or the type itself is actually enforcing the state in any meaningful way, it's just that we are saying that this is a thing that you should create or pass through only when the hardware state matches what the token describes; it's actually enforcing it
 - Amit: Sounds a heck of a lot like a capability
 - Brad: I think the implementation ends up similar, especially for interrupts
 - Brad: Panic is a little better, since it's not just where we call the macro
 - Brad: I don't really see this as an issue around the implementation or function control flow, and I did think about this -- my first comment that I typed and deleted was "these are just capabilities, let's just use capabilities", but as I thought about it more this is really something semantically different
 - Brad: A ticket is something that grants entry, while the capability is more about permissions
 - Leon: Ticket as a word presents some only-once semantics maybe
 - Brad: Let's not get hung up on 'ticket' as a word; the question is whether this is something different or just capability
 - Amit: With a panic, you enter the panic state and never leave it; with interrupts disabled, that state stops at some point, and so the semantics/terminology don't necessarily embody that
 - Brad: Could you say that again? It seems like a permission is something you keep, and a ticket is a fixed-window event
 - Amit: It doesn't seem like there's something tying this ticket being true
 - Pat: Correct; indeed at one point Claude tried to write code that re-enabled interrupts while still holding the Token
 - Pat: Ultimately, I don't think there's a way to have some type that can truly prove this---there's nothing that can compile-time prevent someone doing some unsafe inline assembly to enable interrupts while holding an object
 - Amit: Yes, no way to prevent that
 - Amit: So I guess that returns to whether this is the same thing?
 - Amit: Maybe tickets need to look more like read locks? Ignore arch/chip concerns for a second, when it's created, there is a singleton for the base thing, and then it can be cloned out and those clones reaped, and then interrupts can only be re-enabled in one spot by consuming the last instance of the ticket
 - Alexandru: Capability means something that I have and is not ever revoked; ticket is something that I have now, but can be revoked
 - Alexandru: And of course, there's always `unsafe` to escape all of this; transmute, etc
 - Amit: Agree with the vision of this, but in practice the only thing differentiating whether these are something you keep is that they're passed by reference
 - Pat: This has been helpful, and I will explore the "only can re-enable interrupts once you prove all the tickets are destroyed"
 - Brad: But what if you can't prove that?
 - Amit: Then it's just a capability
 - Brad: I'm a little uncomfortable blocking this on implementation details when our capabilities have technically always been a little in flux
 - Amit: I am proposing not introducing a new term to introduce something that is not new; and when/if it evolves to something that is new, it can get a new name
 - Brad: I don't agree, but I see your point
 - Pat: Is there any urgency to merging this? It just removes a bunch of `unsafe`, I don't think it'll take that long to explore the ref-count version
 - Brad: Not for the release or anything, but don't want to block the removal of `unsafe` effort on naming type nuance, e.g. #5063
 - Pat: I already clicked merge on that — it just had a CI hiccup
 - Brad: Okay, that works
 - Amit: While we're looking at #5095, I'm not sure how I feel about the `mint` macro, it hides an `unsafe`
 - Brad: Right, that was my request. Just like capabilities don't actually related to memory unsafety, this doesn't either; writing safety comments at each of the mint sites makes sno sense. The macro implementation is careful to ensure that only `unsafe`-allowing code can call it, so we get the protection still without the implication of memory unsafety
 - Amit: Isn't it kind of memory unsafety? The problem with interrupts is sometimes memory stuff?
 - Pat: None of them were actually soundness issues
 - Amit: To make my point again maybe, this feels like another smell of "it's just capabilities again" since we have the same magic `unsafe` hiding macro and explanation and such

## Decisions

1. Are we happy with omitting the word `Token` from all of our token types?
(I hope so, it's going to get wordy otherwise)
  - Yes.
2. Are we happy with `Ticket` as the term for "granting entry into a
function that assuming something about the state of hardware"
  - Sort of, it likely hinges on `Ticket` actually doing something different in enforcement than `Capability`. Pat will investigate.

## Actions

- Pat updates the open PRs with the settled-on nomenclature.

# Tock 2.3 release

## Context

Last week, we decided that the following were release blockers and we
expected them to be merged by this week's meeting:

1. 64-bit RISC-V support (excluding the TRD). From the notes, this was
already done, it's just a milestone feature.
2. QEMU boards for RISC-V and ARM tests.
3. STM32U support.

- https://github.com/tock/tock/pull/5125
- https://github.com/tock/tock/pull/5101
- https://github.com/tock/tock/pull/5098
- https://github.com/tock/tock/pull/5034
- https://github.com/tock/tock/pull/5022

## Notes

 - Johnathan: (reviewed context)
 - Alexandru: I need 1-2 days for STM clocks, just some error handling concerns, super close
 - Amit: Others? Let's just go through the PRs?
 - Pat: 5125—was blocked on tockloader release, went through this morning. Adversarial claude review might've found small nits, will incorporate those; should go within the week
 - Amit: Release blocker?
 - Pat: I'd say yes.
 - Amit: 5101—Looks like this adds something substantial to the PMP, and then is otherwise just some board stuff
 - Leon: The PMP stuff has me worried, since I think it reduces PMP space available to all boards, regardless of if we have the (?)flash(?) abstraction; want to take look before we merge it
 - Amit: Is this a blocker?
 - Leon: This is only used for CI right?
 - Leon: I don't see why this needs to go in before the release.
 - Amit: 5098: QEMU 32 virt add pflash driver
 - Brad: The PR adds a peripheral driver
 - Pat: If we've decided that 5101 isn't a release blocker, then 5098 and 5099 probably aren't either?
 - Pat: That said, I'd hope we can just resolve it in the 1-2 day window the other blockers are resolving
 - Brad: `pflash` is the QEMU term for flash, and this implements it for the QEMU board; it's just implementing the HIL for a chip
 - (some discussion on the PR itself)
 - Johnathan: Up-leveling, these are 'we hope they get in, but not release blockers'
 - Johnathan: Next PR is 5034, the STM32U one -- this is the clock configuration that Alexandru referenced at the start; needs 1-2 days; is a blocker
 - Johnathan: Last week there was a question about STM32U flash, do we need to block on that too?
 - Alexandru: Don't want to block release on that, can be in the "hope in" category
 - Johnathan: 5022—Not sure what this PR is, the description makes it sound like it might be filling something that was missed from another PR?
 - Brad: Amit already merged this PR as #4880 to a different branch, just didn't get merged into master; this isn't my PR, it kind of existed before stacks
 - Johnathan: So, does rv64 work on master right now?
 - Brad: Yeah
 - Johnathan: Right this is just supporting CI workflow
 - Pat: And what happened is when this changeset was merged into a branch PR, this had a bunch of pointers to specific commits and refs that were needed during bringup, but now can just point to libtock-c master, so those are the changes I've requested
 - Brad: Amit, you merged it last time, why not just merge it now
 - Amit: I think it should probably be merged. Now that I've copied the markdown from the original PR, I understand what's going on.
 - Amit: I understand Pat's comments / issues here, but if no one's going to fix them...
 - Pat: Fixing my comments is literally clicking the apply button
 - Johnathan: Does ci-job-qemu-virt not test everything that this tests though?
 - Brad: This is the quick version; fewer lines of code to just get things running
 - Leon: Okay, I'm very confused.
 - (lots of voices, summarized): This was a good PR at the time, but never got merged into master on accident; in the meantime, newer things in master have subsumed this, so now we don't really need it.
 - Brad: This was an external person, do we lose some goodwill that their code was never merged?
 - Amit: I'll write a nice comment explaining what happened.
 - Johnathan: I think we're out of time to discuss the rest of the release process questions / start cutting a release; seems like we will have to defer this to next week
 - (consensus)

## Decisions

1. Are we ready to cut a release candidate, or is there something we're
still waiting for?
 - Blockers: 5125, 5034
 - Non-Blockers (but expect/hope they land): 5101, 5098, 5099
 - Closed: 5022

--> Remaining Q's deferred to next week
2. Do we tag a release candidate or make a branch?
3. What tests do we want to run before approving the release?
4. Who runs the tests?
5. What is the deadline for testing?

## Actions

- Delay cutting a release candidate until a few of the context PRs above
are merged; solidify release plan next week.
