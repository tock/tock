# Tock Core Notes 2022-12-16

Attendees:
- Alexandru Radovici
- Alyssa Haroldsen
- Amit Levy
- Brad Campbell
- Branden Ghena
- Johnathan Van Why
- Philip Levis
- Vadim Sukhomlinov

# Updates
- Branden: I had a student who was digging into the microbit and particularly
  working on libtock-c this quarter. It turns out every sound application was
  doing value times 3 to play a note. We spent time tracking it down and it
  seems to be an arbitrary constant. He will send a PR to make those
  non-arbitrary.
- Alexandru: I have the answer. I copied the Arduino app and it wouldn't work.
- Branden: It's a minimum frequency. Those notes won't play because they're
  below what the speaker can do.
- Alexandru: I multiplied by 3 and saw it works but forgot to go back.
- Phil: Discussed 64-bit time with Alistair. His PR is primarily about a 64-bit
  timestamp. Given that TRD 105 says that platforms should provide the API, I'm
  going to start making sure things follow it. The current proposal is to add a
  new Command to retrieve 64-bit time to the time capsule. The difference from
  Alistair's suggestion is this adds a new command rather than changing the
  existing command to 64-bits. I'll start looking at that.
- Alexandru: I started working on the display trait and TRD. Hopefully I have
  more time next week to finish it. Phil, if your offer to discuss it is still
  valid I would be happy to do so.
- Phil: Happy to talk.
- Alexandru: I'll send an email.

# Ordering userspace print statements with kernel debug statements (#3327)
- Phil: This PR from me came in response to Alyssa's comment that it is
  frustrating that writes to the console can interleave, particularly
  user/kernel space stuff. We don't see causal ordering in the output of
  Console. This PR introduces a new capsule that makes userspace printfs append
  to the debug stream rather than going through a virtualized writer. So it's
  all synchronous and in order. The tradeoff is that because it uses a fixed
  size buffer it is possible that your messages are truncated. The question that
  came up in this PR is we now have an asynchronous console driver and a
  different driver which is synchronous -- what do we want to do? Print log does
  not allow you to read like console, should that be changed? Do we want to just
  have both, have one, should they be separate drivers, separate userspace APIs?
  How do we want to manage these two different semantics? I'd love to get
  people's thoughts.
- *[47 second silence]*
- Phil: Okay, any people have questions?
- Brad: How does the print log capsule tie into the debug buffer?
- Phil: It literally calls `debug!()`
- Alyssa: Is it possible to write to the same buffer as debug without being
  synchronous?
- Phil: The point is it doesn't block, in the sense of that being synchronous,
  but we write into the buffer at exactly that moment. That's why if you reach
  the end of the buffer it doesn't block, it just truncates.
- Alyssa: We could have the userspace make sure it doesn't send too much at a
  time.
- Phil: Hudson raised some questions about the size of the debug buffer, but I
  think that's trivial stuff we can sort out. This is really the question of: we
  want two implementations; one that allows arbitrary length writes from
  userspace but can interleave, and one that may drop writes but is synchronous.
  The question is whether they should be the same system call API with two
  implementations, or two different system call APIs? If there are two different
  system calls, then there can be two different userspace APIs representing
  them? If there are two implementations of the same system calls then there can
  be a configuration option and printf goes to both of them.
- Amit: The way it is currently structured -- same interface but different
  implementations -- seems better for the stated use (debugging) and a good
  thing to have. Arguably, part of the point of Tock is for it to be extensible,
  and part of that is we should have places where different platforms have
  different implementations with different tradeoffs. To a degree, I can imagine
  issues that can crop up -- what if one syscall API evolves and there's a
  mismatch -- but that almost seems like a good thing to have surface so we have
  to solve it. I think this design is good for both this use case and for the
  kernel to have a main capsule with this feature.
- Phil: My one software engineering concern to that approach is that while the
  write paths are very different, the new syscall doesn't implement receive. I
  would like to avoid having the synchronous API copy all the receiver code --
  would want to refactor the implementation to deduplicate the code.
- Brad: I agree with Amit, and there's a part of me that thinks it is strange to
  take something as fundamental as console and implement it on top of `debug!`.
  I would find that difficult to parse and understand as a user. The other part
  is -- I think there's still an idea out there that we want to have a more
  robust channel between a host and a device. By having two console
  implementations we are not making the stand that we always want sequential
  behavior to happen.
- Phil: The name of `debug!()` is historical. We could add an alias
  `synchronous_write!`.
- Alyssa: Yeah, we have three aliases, `console_info`, `warning`, and `error`.
- Brad: I think it's more than a name, but I could see some argument that
  `debug!` could be rewritten as an integral part of the kernel. `debug.rs` has
  things in it that are not okay for non-debug code.
- Phil: Like what?
- Brad: Like the unsafe hacks that make everything easy to use and nice.
- Alyssa: Which unsafe hacks?
- Brad: How the different buffers are connected -- off the top of my head I'm
  not sure. Definitely to get the user interface to work the macro required a
  bit of "okay, yeah, let's just get this to work because it's just for
  debugging".
- Phil: That's true, `debug.rs` has more than just the debug writer, it has
  debug I/O and the panic handler.
- Alyssa: To me, it matters what you're trying to use it for. The previous
  design is better to have a reliable data stream. This would have fewer of
  those properties, so it seems to be a tradeoff between easy-to-understand
  behavior or resilience against apps attacking each other.
- Phil: I agree with that framing, which is why my initial thought is having
  separate system calls. It is the case that if the kernel can provide either
  implementation, the userspace API must reflect that data given to write may
  not be written. That would push that logic to userspace, so even if you were
  sitting on top of console you would have logic to complete a write that was
  not completed, though console would always complete it.
- Alyssa: Console could just write up to a line.
- Phil: True, but that's not what it does. One other side of saying they're two
  implementations of one system call is you can't have both -- something that
  can write arbitrary-length stuff from userspace and something that can write
  synchronously.
- Alyssa: If we can give them the same API with different guarantees, that seems
  ideal. I think they should be exclusive -- one or the other.
- Brad: I agree, I worry about complicating this for userspace and userspace
  examples. This seems like an advanced thing that most users don't need to deal
  with. I wouldn't want to see different forms of printf calls in different
  examples, as that's a barrier for new users. I would prefer unifying them in
  the syscall interface and making this board-specific.
- Alyssa: We could make the console capsule take this choice as a generic
  parameter. Users would have to choose between one or the other, but we
  wouldn't have conditional compilation.
- Brad: That's what we have now but with two different capsules.
- Alyssa: I want to make it obvious these perform the same purpose but you have
  to choose between them. I think most people expect console to be
  temporally-ordered, so if we have to have a default I think it should be
  temporally-ordered.
- Phil: It sounds like there's pretty good consensus that option one,
  alternative implementations of the same device is the way to go.
- Brad: Phil, did you look at putting a layer between the bottom of console and
  the UART or debug?
- Phil: That's totally something to dig into. I would be wary of making calls to
  `debug!` look like UART, especially if that is the default one.
- Brad: Yeah, because the issue is the virtualization is at the "wrong" layer.
  The virtualization needs to be above the central pool, not below it.
- Phil: There are other issues that come up. Not just interleaving, but also
  userspace being able to take a lock. Once the client is operating on the UART,
  it gets to keep on writing until it has nothing left. I worked out some cases
  where clients can starve others.
- Brad: That seems like a different issue, a console issue.
- Phil: This is at the UART virtualization layer.
- Brad: Oh I see.
- Phil: I think that would probably be the right way to do this. There will have
  to be some refactoring for the receive path.
- Phil: Are there any arguments for having separate syscall APIs?
- Alyssa: I think that would fragment the community.
- Phil: That sounds like an argument against, I'm looking for an argument for.

# Fixing MapCell safety (#3325)
- Alyssa: Looks like it may double drop, I'll add some comments.
- Brad: The issue with `MapCell` is you can put something in the `MapCell`, take
  it out, and take it out a second time, ending up with two references. This
  pull request tracks that state, preventing you from taking something out a
  second time. The tricky question is "what do we do with `replace`"? `replace`
  should put something in the cell and return the old thing, but what if you
  call it while you've taken something out? There's nothing to return, so it's
  invalid, so this PR panics. You know I'm not the biggest fan of adding panics.
- Phil: You and people who care about code size.
- Brad: True. Maybe the mistake was having `replace` and we should just get rid
  of it.
- Branden: Why can't `replace` just return `None`?
- Brad: Good question.
- Branden: You don't have to answer.
- Amit: I see four options. Return `None`. Return a `Result<Option<>, Error>`,
  it could panic, or we could remove `replace`. Do we use `replace` anywhere?
- Brad: It looks like we do use `replace` but we never do anything with it, so
  we can just use `put`.
- Amit: I think that a fix is necessary, that panicing is not a good option. If
  we don't need `replace`, then lets get rid of it. If we really need it we can
  resolve the API issue. Basically what Brad said, right?
- Phil: `MapCell` is not used in that many places, right?
- Alyssa: I may be wrong but I don't see `replace` update `self.occupied` when
  inserting a new item.
- Amit: That seems right, so it's maybe just a bug.
- Alyssa: I'm not comfortable changing `MapCell` at all until there's rigorous
  unit tests.
- Amit: That seems like a reasonable ask for the PR.
- Branden: I'm a little confused about the `self.occupied.set` problem, it
  wasn't there before.
- Alyssa: It did it via `self.put`, which sets `self.occupied`.
- Phil: Alyssa, can you comment that on the PR?
- Alyssa: Yes. It's also doing a double-drop on values.
- Branden: I think that's two more votes for removing `replace` as we've managed
  to put two more bugs into it.

# Rasberry Pi Pico USB (#3310)
- Alexandru: Are we ready to merge #3310?
- Phil: Brad, can you take a look at #3310?
- Brad: Yup.
- Brad: So we have `make program` that doesn't program.
- Alexandru: It's a bit tricky with the board, because you need to use a UF2
  file. At the moment you cannot program it with OpenOCD unless you have a
  patched version of OpenOCD and another Raspberry Pi (either Pico or the big
  one) and we want to avoid this. So we just use the UF2 now. We couldn't use
  this because it has no serial so it needed the USB to print something.
- Brad: So the copy command copies the UF2?
- Alexandru: To a fake USB drive.
- Brad: That ends up programming it?
- Alexandru: Exactly. It has a bootloader in its ROM which cannot be
  overwritten.
- Brad: Okay. Doesn't have to be OpenOCD as long as it does something, great.
- Alexandru: I think we still left the comment in the README file. If you really
  need OpenOCD and want to debug it, you can. I think in the Makefile it's the
  UF2. As soon as we understand how to program the flash on the Pi we'll port
  the Tock bootloader to it, but for now that's a flash-less chip so it's a
  little bit tricky.
