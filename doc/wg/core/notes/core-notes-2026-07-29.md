# Tock Meeting Notes 2026-07-29

## Attendees
- Alexandru Radovici
- Amit Levy
- Brad Campbell
- Branden Ghena
- Johnathan Van Why
- Leon Schuermann

## Updates
- Johnathan: I have a working safe DMA API, but it was a bigger change to the
  register_map design that I wanted. I should be able to send PRs within two
  weeks.There's still one annoying hack. The safe DMA API I implemented only
  works for `'static` buffers, which requires unit tests to either leak buffers
  or use `unsafe` to deallocate them. I'm considering a design that adds
  lifetimes to the `Interface` trait to say "the hardware lives only this long".
  The Real implementation would be `'static`, but fake peripherals could have a
  shorter lifetime allowing unit tests to work with stack-allocated buffers. I'm
  not sure that design will work or is desireable, though.
- Branden: Is this mostly behind-the-scenes, or will it impact users?
- Johnathan: This adds lifetimes to register block field types in the trait, and
  changes the methods to take `&self` rather than `self`. You now implement it
  on the fake peripheral type, not a reference to the fake peripheral type. So
  significantly different. This is close to the "have unique register handles"
  idea that I discussed with Pat, I'll think about maybe moving the references
  to `&mut self` to make it really unique. Not sure if that buys us anything.

## Code review rules for safety

*Pat not here yet, pushed until he joined.*

## IPC Progress
- Branden: The goal is to make a new IPC ecosystem of capsules that are easier
  to use, can be extended downstream. Network WG has designed it, I've
  implemented two registration/discovery capsules and a message sending
  capsule. I have libtock-c implementations, can send messages to each other, it
  all seems to work. Need to do cleanup and will have PCs soon. Long-term, there
  are other things to have. We have a design for an asynchronous message system
  using StreamingProcessSlice, and have a shared memory capsule design that
  needs more thought. Idea is to get something that work then add more in the
  future. I ran into two things that I wanted to ask the group about -- any
  questions before I continue going?
- Amit: What's the trajectory for something like direct shared memory?
- Branden: Long-term, not short-term. We discussed it a lot, and it is
  complicated. Direct allow-to-allow transfer was a lot simpler and higher
  priority.
- Brad: What were you using in libtock-c for testing?
- Branden: I wrote all the drivers and made test apps for it.
- Brad: New test apps?
- Branden: Yes
- Brad: Have you tried any existing apps?
- Branden: No, that's a good idea.
- Brad: So you created new driver numbers?
- Branden: Yes, this is a whole new system with new driver numbers, not a
  drop-in replacement.
- Brad: How does the driver number map to the ecosystem? Is there a class of
  driver numbers or one new number?
- Branden: A class.
- Branden: The first issue I hit was a grant allocation issue with
  yield-wait-for. A yield-wait-for will hang forever unless you've already done
  something to trigger grant creation. If your first interaction with a capsule
  is yield-wait-for then that didn't happen. For the IPC mailbox, it made sense
  for a service to yield-wait-for on a message to occur, but it won't get the
  message. I can specifically fix this for IPC, like an initialize command to
  allocate grant space, but more generally should yield-wait-for ensure that
  capsule grants are allocated? Subscribe does.
- Brad: We've definitely ran into this problem. Do you remember that?
- Branden: I do not recall.
- Brad: The GPIO driver could as well.
- Branden: Yeah, I remember that now.
- Brad: My conclusion was that it was suitably strange that just a subscribe
  shouldn't be enough. The original fix would check in the kernel that if you
  called yield-wait-for, and there's not grant, then create it. But that's
  special-case, so we ended up with "make sure the capsule creates a grant if
  userspace wants to do anything if there's going to be an upcall".
- Branden: I didn't realize we already discussed this. I'm happy to conform to
  that. IPC is weird and hits things like this. Seems perfectly reasonable.
- Brad: The TRD spells out that Subscribe is not a Command, so we articulate it
  as Subscribe doesn't tell the capsule to do anything, so there needs to be
  some other command to instruct the capsule to do something. I agree this is a
  case where all you want to do is to subscribe.
- Branden: It felt strange to add a `bool` to indicate "I'm ready for stuff",
  but maybe just the existence of the grant space is good for that and there
  should still be an init command.
- Brad: What about stop?
- Branden: Just unsubscribe. Though we could make a command for that. I think
  this is again going back to subscribe-is-not-a-command.
- Brad: It could be "buffer this for me", basically.
- Amit: Subscribe doesn't inform the capsule. I can sort of see this for IPC,
  but for things like network packet filters you would want the unsubscribe
  command to be explicit. The capsule would do things like "nobody is listening
  anymore, let me turn off ..."
- Branden: Okay. I think that's totally reasonable and I don't feel the need to
  discuss this a ton more. I wanted to know if my design is in a weird space or
  just a problem, and I'm in a weird space and should just change the design.
- Branden: That was one of two. I have one more, which is process state change
  callbacks. Typical scenario: client wants to send a request to the server.
  That command verifies the server is a valid destination. Later, the server
  will send a response indicating the transaction is complete. A client can't
  send two messages at once; it can wait or cancel. The problem scenario is if
  the client sends the request, and the server is valid, but the server faults
  before responding, the client is stuck. It can implement a timeout, but it
  won't be aware the server died. Ways to fix it: all clients must set timeouts,
  or number two is we can make a mechanism to notify the capsule when process
  state changes, then the capsule can go through and update clients. Is that a
  good idea?
- Amit: I have complex feelings about this. The short version is yes, but it
  should be gated by a capability and we should be careful to not let it balloon
  into other things. I'm weary of it becoming a resource-cleanup signal.
- Branden: I agree that would be a reasonable place for it to fall into.
- Amit: What would you do alternatively? This is the moral equivalent of setting
  a timer and periodically checking for liveness.
- Branden: Yeah.
- Alexandru: Wouldn't the yield-wait-for block forever if the server died?
- Branden: You can send an error upcall to wake the client.
- Brad: The server in this case is a process? And when you say died, the kernel
  is aware that it has faulted.
- Branden: Yes, it has entered a faulted or terminated state.
- Brad: And not that the server has lost track.
- Branden: Yes, the kernel cannot be aware of that.
- Brad: How can the client know?
- Branden: A highly-reliable client would need a timeout.
- Brad: It feels like this is redundant with that.
- Amit: What about having a timeout in the IPC capsule? Can use that mechanism
  for both.
- Branden: Yeah, we could do that. We could also put the timer in userspace. In
  either case, clients would have knowledge of how long it should take the
  server to respond, could be implemented in either place.
- Branden: I do agree with you Brad. When I initially thought about it, I
  thought it was necessary, then I realized there are a lot of other failure
  cases and I eventually wobbled back-and-forth on it. Based on Amit's concern
  about how other capsules might use it, and the weird way I'd have to implement
  it, I'm leaning against implementing it and using timeouts instead.
- Brad: This doesn't seem like a terrible idea. I do agree with Amit's concerns.
  It would be nice to decouple this from IPC, consider it separately.
- Branden: I was going to make the IPC PR without this, and discuss it
  separately. Given that feedback, I'll push this down the list. It's a maybe,
  but I'm weary about adding it if IPC is the only use case and we'll need
  timeouts anyway as I think timeouts will be more useful.

## 64-bit RISC-V
- Amit: You basically want us to merge a bunch of PRs, right?
- Brad: More-or-less
- Amit: We can look through this. Last I understood, yesterday Leon was going to
  review a couple, and he merged a couple.
- Brad: He merged the PMP fix into a branch.
- Amit: Awesome. How should we do this? Should we go through the PRs one-by-one?
- Brad: Sure
- Amit: 4873 adds helpers for 64-bit arch stuff.
- Brad: Yes. Disclaimer that this does not match the TRD. My plan is to get
  everything cleaned up and merged, then adjust it to match the TRD.
- Amit: You're copying the syscall register function and then modifying it for
  64-bit.
- Brad: Yes.
- Amit: Really tight within itself.
- Amit: Looks decent. MachineRegister has an `as_usize` method which should be
  an Into implementation. Wait, I approved then the merge base changed.
- Brad: What do you mean?
- Leon: The target branch just moved.
- Amit: Yeah, it says Brad dismissed it by changing something.
- Brad: I'm not running git commands, I'm not sure what GitHub is doing.
- Amit: Something is going on with GitHub.
- Amit: Alright. Now we have 4873, which is big.
- Amit: I do not understand the logic in what separates qemu-rv64-virt-chip and
  qemu-rv32-virt-chip -- is that also a thing?
- Brad: Yep
- Amit: I don't understand the logic of what goes where.
- Brad: I think there should be three crates: the shared qemu-rv crate, with
  drivers for QEMU peripherals, and the two variants for the two boards. We
  would call those microcontrollers if those are real hardware. Those would use
  the shared crate and allocate peripherals at the correct memory addresses.
- Amit: I guess like, for example, the UARTs don't look different. The CLICs
  don't look different.
- Brad: There's UART files on both of the microcontroller-specific ones because
  those need `unsafe`.
- Amit: Why is it a separate rv64 and rv32
- Brad: Because they're separate microcontroller things.
- Amit: Is the code identical?
- Brad: No idea. Is the code identical in this PR? Probably. Maybe. I didn't
  think about it. Will they remain identical? No idea.
- Leon: These are generated from a single QEMU definition with different ifdefs
  for sizing and alignment constraints. In practice, they're likely to be
  identical. There are references to rv64 and rv32 that we would need
  compile-time flags to switch between. But we don't have that guarantee from
  QEMU.
- Amit: Okay. Some of this is not QEMU-specific, like the CLIC.
- Leon: Right, it's just instantiating it for that board at that board's
  address. They happen to be the same between the two boards, but that's
  effectively coincidence. This is the same way we're doing it for other chip
  families like STMs and NRFs. There's a 10-line-ish instantiating module that
  asserts that this peripheral exists at this address.
- Brad: In my dream-of-dreams, QEMU would give us a way to change the memory map
  so that we could use the upper bits.
- Amit: I'm noting that Brad has not approved this.
- Brad: I've written most of this PR at this point, by some definition. I didn't
  open it thought.
- Amit: If you approve, we'll have two approvals.
- Brad: Fair.
- Leon: I agree this is effectively Brad's PR at this point.
- Amit: We have two approvals now so we get to merge.
- Amit: The last PR, 4874, I think there are unaddressed comments.
- Leon: In essence I agree that we're getting rid of some clear comments about
  how the PMP works. But a lot of those have moved into shared methods that we
  use to write the registers, which are at the top of the diff. We might want to
  add a couple comments back, which is an easy change. I can do that this
  evening.
- Amit: I think that's fine. Brad, you should feel empowered to merge after
  approving, I feel.
- Brad: Okay.
- Amit: How will we solve the 4904 issue. It's waiting for approval even though
  I approved.
- Branden: Brad managed to automatically cancel the auto-merge on a PR he wanted
  to merge.
- Amit: Do you have a coding agent running?
- Brad: No.
- Amit: I'm going to try something.
- Branden: If we tried to merge it while other PRs aren't in the merge queue,
  will it be okay?
- Amit: Maybe
- Leon: Typically closing and reopening flushes GitHub's internal state in
  pipelines.
- Amit: I'll try it.
- Brad: Should I try it?
- Leon: It's re-running the proper CI.
- Amit: I'm wondering if GitHub things are vibe coded.
- Johnathan: Should we be running a service that scrapes our PRs and verifies
  they are merged correctly?
- Leon: I'm running a system that archives PRs.
- Alexandru: Had merge queues completely break a repository of ours.
- Amit: Well, it's a good reason to keep local replicas of the tree.
- Alexandru: That wouldn't have helped. They force-pushed without a force-push.
  Even if you have the replica. The merge commit was linear, it never showed on
  the master branch that it would've been a force push.
- Amit: Oh
- Leon: They squashed hundreds of thousands of lines of changes into the
  supposed merge commit without telling you. Rolled back thousands of PRs on
  public repositories silently.
- Alexandru: They didn't roll back ours.
- Amit: How did this happen? Like `git merge` is `git merge`, ultimately.
- Alexandru: PRs looked fine, the merge commit just deleted.
