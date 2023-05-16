# Tock Core Notes 2022-01-27

Attendees:
- Alexandru Radovici
- Alyssa Haroldsen
- Amit Levy
- Branden Ghena
- Jett Rink
- Johnathan Van Why
- Pat Pannuto
- Philip Levis
- Vadim Sukhomlinov

# Updates
* [No updates]

# PR #3384 Fixed i2c buffer len
* Branden: Working with a screen, want to send it many bytes. Ran into HIL
  limits. Had a discussion about whether the size should be a `usize` or something like `u16`.
* Phil: Initially we decided we should use `u32` everywhere and not `usize`,
  then we discovered Rust gets mad at us for that.
* Vadim: This is a challenge for host emulation as well.
* Alexandru: I suggested leaving `usize` because that's what Rust uses for array
  length and indices.
* Phil: I was a strong proponent of `u32` but the realities of Rust make that
  impractical.
* Alyssa: Does `usize` implement `Into<u32>` on systems where that works?
* Johnathan: It impls `TryFrom`, not `From`.
* Phil: I think we should do `usize` for consistency.
* Branden: I was sold on consistency before there was a Rust reason. I think we
  can be done. So `usize` it is.
* Alyssa: Both `usize` and `u16` are `TryFrom` each other, even when they're
  compatible.

# Issue #3383: Interpretation of "blank line"?
* Alyssa: I think having it in a comment is entirely reasonable.
* Branden: I strongly don't care about this issue.
* Amit: Likewise
* Alexandru: I don't care.
* Pat: Don't care.
* Johnathan: Hudson prefers it to mean an entirely blank line.
* Johnathan: If we need a tie-breaker then I'll vote for allowing a blank
  comment.
* Phil: Be generous in what you accept and precise in what you send, that argues
  for accepting comments.
* Johnathan: I'll send a PR that changes the license checker to allow it.

# Ti50 Tock Wants
* [If you want the slides, you can contact Alyssa]
* Alyssa: A quick overview of some discussions I've had over the last week. The
  biggest want I've seen is a new blocking command syscall. We've already
  implemented it locally and have gotten solid code size savings. Allows us to
  do some operations soundly that would otherwise be quite difficult. Common
  pattern:
  - Command
  - Subscribe
  - Yield-loop
  - Unsubscribe
* Alyssa: We're done this enough that we realized we want to have a blocking
  command. During a blocking command, all upcalls that are scheduled are queued
  and not invoked. Allows userspace to use global buffers without adding
  synchronization with upcalls. I think it is useful and deserves its own
  syscall class -- what are your thoughts?
* Phil: I'm pretty positive on this. In particular, I think it is important that
  calls are queued for similar reasons to what you mentioned. One of the reasons
  the system call API is fully asynchronous was for the original low-power use
  cases, where you want a lot of parallelism. In Ti50's use case, you don't care
  about doing a lot of operations in parallel.
* Alyssa: We're doing an operation that we know takes some time and don't have
  anything else to do.
* Branden: You don't Allow memory during this?
* Alyssa: We often do Allow as part of this sequence too. For example, our
  console print is Allow memory, blocking Command, un-Allow memory.
* Jett: The Allow/un-Allow part is just about flash size and runtime savings.
* Alyssa: I think blocking Command is the fundamental piece, we can build other
  things on top of it. Upcalls being queued is the special part.
* Jett: I agree. I think blocking Command is less controversial, but taking it
  to the next step is worth looking at.
* Alyssa: I know we discussed combined syscalls, what was the problem?
* Alexandru: The prototype works, but I haven't had time to implement it.
* Phil: There were issues with Yield.
* Alexandru: Yield had to be the last command.
* Jett: This works better with blocking Command because Yield is not involved.
* Phil: A couple of issues come up with doing batches of syscalls. What happens
  if you do a batch of 5 allows and allow 3 fails, what's the error handling?
* Alyssa: On the app side or kernel side?
* Phil: On the kernel side. What's returned if you do 5 Allows and the third
  fails. You can return that a couple succeeded and exit, or keep going.
* Alyssa: You could also have it unwind the sequence.
* Phil: That doesn't always work.
* Johnathan: We're optimizing for a common case. In more rare cases with more
  complex error handling, the app can do that itself.
* Phil: What if there's a non-blocking Command in the sequence? Can't roll that
  back. There was research on batching in NFS, and the conclusion was you stop
  immediately. Rolling back state changes is hard.
* Jett: Allow/Subscribe are special as they are kind of a `try`/`finally` thing.
* Phil: I'm skittish because we're talking about a specific use case, and to
  build a general mechanism to solve a specific problem -- we want to be careful
  there.
* Alyssa: There are different levels of complexity. We could define setup,
  execute, and teardown sections. If I do Allow, blocking Command, un-Allow and
  the blocking Command fails, I would expect to still do the un-Allow.
* Alexandru: Can't the library in userspace do the rollback?
* Alyssa: Yeah, but why do the combined syscalls?
* Alexandru: It's a bit faster.
* Alyssa: I'm mostly concerned about code size.
* Phil: What if we made something narrower, like batched Allows? That makes the
  error handling simpler.
* Alyssa: If we wanted to have a combined Allow, blocking Command, un-Allow?
* Phil: That would be 3 system calls.
* Alyssa: If you're always doing Allow, operation, un-Allow, it makes sense to
  just declare I want an Allow wrapping this. If it's just for Allows, it would
  be combine this syscall with some number of Allows. If it fails, the kernel
  rolls back.
* Alyssa: Sounds like blocking Command would be well-received?
* [Thumbs-up appeared in emotes]
* Phil: I think the semantics makes sense, devil is in the details.
* Alyssa: Should we send an implementation PR or a TRD first?
* Amit: I would find it easier with an implementation but either is probably
  okay.
* Alyssa: Moving on, one team member wants a completely redesigned console
  capsule, with:
  - Order printing of strings in apps and capsules
  - Line buffering in the kernel, with app printing to a kernel buffer rather
    than an app-side buffer.
  - Single console output buffer for all modes. Each USB/UART/etc has its own
    head in and common head is min() of all heads -- moves w/ slowest.
  - Micro-optimization: a print that always appends `\n` to reduce string
    literals.
* Alyssa: Does anyone have thoughts on this so far?
* Branden: The last one seems easy. For line buffering, if you move it into the
  kernel, you have to do a system call for each partial line. That has a speed
  implication, though it seems like you care more about memory than speed.
* Alyssa: Yes. Personally, I want to keep line buffering in maps, but I'm
  expressing a want of my team member's.
* Phil: Can you walk me through that? You do a series of writes from userspace
  that are buffered until I send a newline?
* Alyssa: Right now our writes are buffered in userspace. In theory, the capsule
  could do that instead. It would be more syscalls but less code and simpler
  memory management.
* Phil: Because the kernel does it?
* Alyssa: Yes.
* Alyssa: That optimization using blocking Command saved 4kB by the way.
* Phil: If two processes are writing partial strings, the order will depend on
  the order of the syscalls that triggers newlines?
* Alyssa: Yes.
* Alyssa: I should add another ask from the team. For our automated tests we
  occasionally look at the console output, but if we end up having an interrupt
  in the middle of a series of line prints the test fails. We have a couple of
  possible solutions. I wanted to bring it up, because there was an idea of
  being able to lock the console (with a max timeout) for a particular app. My
  primary issue would be DoS.
* Amit: All of these seem like reasonable things, one concern I have about where
  this is going is the variety of needs we are trying to serve. For multiple
  apps, do we want a single shared console, or per-app virtual consoles? Maybe
  printing arbitrary-length strings separated by lines is not the ideal
  interface. I think all of these are reasonable, but I worry the existing
  console is the wrong starting point for it.
* Alyssa: Yeah, essentially it would be an entire redesign.
* Phil: One of the challenges is many of these look good on their own but I'm
  not sure their interactions are good.
* Alyssa: I'd want to make these requests more concrete.
* Johnathan: While designing app IDs I envisioned something that looks more like
  IRC, as a way to separate app messages without having completely virtual
  consoles.
* Amit: That would require line buffering.
* Phil: Not interleaving code helps in a test case, but avoiding interleaving
  may make debugging harder, if the interleaved prints indicate unexpected
  ordering. How do we do this in a general way? Doing a point fix for one
  problem causes issues for other problems.
* Phil: Number one seems kinda obvious.
* Alyssa: I personally don't want it but see some advantages.
* Alyssa: We also have some muxing -- Ti50's console outputs to UART and USB and
  they are not the same speed. What if prints run at a speed between that of the
  two devices?
* Alyssa: Also want code sharing between apps, I know it's hard. Has there been
  progress?
* Johnathan: No
* Phil: It's a linking problem, right?
* Alyssa: Yes. You can't share statics between processes but do want to share
  code.
* Johnathan: FDPIC
* Alyssa: We want FDPIC but statically located.
* Johnathan: Right. That doesn't exist, and Ti50 is alone in wanting it.
* Alyssa: ufmt in the kernel, has Tock considered taking over its maintenance?
* Johnathan: Pending an evaluation, maybe. Ti50's experience with ufmt can help
  here.
* Alyssa: I don't have numbers yet, kinda tricky to get because this was going
  parallel to other work. Seems to be quite a bit more efficient. Needs more
  TLC. There are specific use cases where I noticed it could be more efficient.
  If Tock owns it I would like to help improve it. I don't want to work on ufmt
  if I will upload a PR and it will be ignored.
* Johnathan: My main concern with ufmt was whether it was fit for our use, not
  code size. I assumed it would help with size. I also think having it
  maintained -- whether by Tock or another group -- is a necessity.
* Alyssa: There is want for more flexibility in syscalls -- how can we change
  the ABI or add syscalls? We also have a team member who wants a fully
  preemptive kernel with a simpler API.
* Phil: How does that interact with Rust?
* Alyssa: This person is not a Rust expert, I don't think they've thought it
  out.
* Phil: When we first designed Tock we decided that interrupts would always be
  unsafe. We'd have mechanisms to do interrupts but all bets are off.
* Alyssa: I think being fully preemptive in the kernel is desired. The safety
  questions aren't easy to answer.
* Alyssa: End of presentation questions?
