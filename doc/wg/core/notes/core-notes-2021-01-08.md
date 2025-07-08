# Tock Core Notes

Attending:
- Hudson Ayers
- Brad Campbell
- Branden Ghena
- Philip Levis
- Amit Levy
- Pat Pannuto
- Leon Schuermann
- Vadim Sukhomlinov
- Joanthan Van Why
- Alistair


## Updates

### 64-bit Tock PRs

* Leon: I've seen some PRs opened by Brad regarding RISC-V 64-bit
  support in Tock. I appreciate that and think it's very
  interesting. However, in the Tock 2.0 discussions we concluded that
  the ABI is only defined for 32-bit platforms for now. Question is:
  whether, if this makes it in prior to 2.0, should it be included in
  the release or rather some follow-up release? I fear it might
  further complicate the review of changes in `tock-2.0-dev`.

* Brad: Right now, there are no plans to add more RISC-V 64-bit
  PRs. The current changes don't add a 64-bit architecture, just make
  the CSR interface more generic. I don't foresee 64-bit support any
  time soon.

* Leon: Thanks for the info!

### State of Tock 2.0

* Phil: On Tock 2.0, there are 45 outstanding capsules to port and 23
  are done. I might start bugging people whose PRs are outstanding.

  There are a couple capsules (especially sensors) where we don't
  necessarily have the hardware to test, so we need to track down who
  has hardware to do the port and test these capsules, or whether the
  capsules should be removed if nobody has hardware.

* Branden: Hudson had asked about the Signpost sensors, I'm on the
  side of deprecating them from Tock.

* Phil: Can you strike those out on the [Tock 2.0 capsule porting
  issue](https://github.com/tock/tock/issues/2235)?

* Hudson: [I called out the three sensors in the
  issue](https://github.com/tock/tock/issues/2235#issuecomment-753331799).
  Probably none of us have hardware for them.

* Phil: Okay, so the `ltc294x`, `pac9544a` and `max17205`.

* Pat: I do have several signposts in my office, but I do not want to
  necessarily advocate for keeping them around.

* Brad: I do advocate for keeping them around. Our original vision was
  for capsules to be a big folder full of drivers. I don't think it's
  reasonable to expect many development boards have these chips. We do
  want to encourage people to upstream code, so I think it's expected.

* Phil: But who is going to test them? The hardware doesn't
  necessarily need to be on development boards, but we need someone to
  volunteer to test that the driver works as expected.

* Brad: But hardware does not change, so the driver should continue to
  work. Saying that we are going to make architectural changes but as
  a result have to delete a lot of code that we currently don't
  support [does not sound reasonable].

* Phil: But we can't test.

* Brad: We haven't been testing it for months. If the interfaces don't
  change then the driver will still work.

* Amit: Pat, if I write the code, will you test them?

* Pat: Sure.

* Amit: I'm finding that once I sit down to change the drivers,
  they're not an issue. My biggest problem is that I don't have
  hardware at the moment.

### nRF52840 USB Bootloader

* Brad: Update on the bootloader for the nRF52840 over USB. It's
  working, Hudson and Branden have been helping test it. Hopefully
  we'll be able to switch over to the nano33 board and expect that
  Tock will run on top of the Tock bootloader. This will make it a
  much more user-friendly board.

* Branden: It takes about three commands and 40 seconds to do these
  changes and the nano33 is a fully working Tock board. And the
  changes are reversible too, one can have it work with Arduino again.

### New Rust nightly

* Brad: We should be ready to move to a new nightly now, with some exciting new features.


## Return types for yield-wait and yield-no-wait

* Phil: Currently, when yield is called there is no return value. But
  with yield-no-wait, we're going to want to have a return value to
  indicate whether a callback was scheduled. The problem is the way
  that yield/callbacks are implemented:

  The kernel rewrites the application stack such that we jump to the
  callback function and the callback returns to the code that was
  running.

  In order to know whether a callback was invoked information needs to
  be passed either from the kernel, or from the callback itself.

  There seem to be three options:

  1. `yield` and `yield-no-wait` return `void` and there is some other
     way to get this information

  2. every callback must return `TOCK_SUCCESS` and that is propagated
     to the return value of `yield-no-wait`

  3. after the callback is invoked we return to the kernel and the
     kernel sets the return value accordingly

* Leon: Another idea could be to have a _shim_ function in userspace,
  which is stacked along with the callback by the kernel, and which
  the callback would subsequently return to. It could set the return
  value accordingly, such that we don't have to go through kernel
  space again. So essentially we stack two functions, where the
  "lower" function would set the return value indicating that a
  callback was scheduled and then jump back to the place where yield
  is executed.

* Phil: Challenge with that is that the kernel needs to know the
  address of the shim function.

* Leon: If the RAM was marked as executable we could potentially stack
  the entire shim function as well (so it'd be provided by the
  kernel). Not sure how that plays out with userspace ABI though.

* Amit: I think we don't want to involve the kernel, as it is a bit
  over complex (except with maybe enforcing the return value). Having
  a different way to indicate whether a callback was scheduled is
  interesting. I'm not yet sure how that would be realized, other than
  using a global variable. Out of these three options, number 2 seems
  like the cleanest.

  Maybe we can just define that callback functions cannot return
  `TOCK_FAIL`.

* Phil: The problem is that you can have nested callbacks, so one does
  not necessarily which callback executed. Tying the result back to a
  specific callback is tricky.

* Amit: Yes, then it could make sense that, as a convention, a
  callback would always have to return `TOCK_SUCCESS`.

  This could be useful, because if an application wants to pretend
  that a callback did not happen, this would be possible. For
  instance, if an application wants to filter on incoming packets or
  is subscribed to a broadly defined hardware event which it would
  like to filter.

  Even if this would be a larger change, what about not stacking the
  callback and instead use `epoll` or `select` style semantics? Such
  that `yield` or `yield-no-wait` would essentially return the
  callback to call.

* Leon: In this case we'd need to return the pointer, appdata and all
  callback arguments. In the early discussion around Tock 2.0 we
  talked about returning more than 4 registers from the kernel. With 5
  values we might go over the argument limit that can be passed in
  registers? It might be an expensive operation then.

* Jonathan: Isn't it just 5 registers then?

* Amit: Yes, it would require an additional register that we're
  currently not using.

* Jonathan: `yield` clobbers all caller-saved registers currently.

* Amit: Yes, its plausible that this could be implemented as a simple
  _jump to register_.

* Johnathan: It's actually only 5 clobbered registers on ARM, just
  checked.

* Amit: Do we currently use the return value of callbacks?

* Phil: Callbacks have a `void` return type.

* Amit: Option 2 does seem like a good option then.

* Johnathan: We could also pass a pointer to a boolean flag as part of
  the `yield` system call, which the kernel would write to if it
  executed a callback.

* Brad: I prefer either Leon's method (if there is a way to do that)
  or Johnathan's approach, where we can really enforce it. I'm worried
  that by making it a convention it'll work really well, until it
  doesn't. For instance, if a library has a wrong implementation.

* Phil: Yes, if all callbacks would be contained in `libtock` and
  everything was synchronous, that would be fine. But then there is no
  reason to have `yield-no-wait`. As soon as you have applications or
  libraries on top of `libtock` just one faulty callback is all you
  need to break these semantics.

* Amit: And Johnathans approach would only change the signature of
  `yield-no-wait` (not `yield`). Would it be easy to implement?

* Phil: I like Johnathan's approach. `yield` and `yield-no-wait` have
  a single parameter (no wait / wait). `yield-no-wait` takes an
  additional address to which it writes whether it scheduled a
  callback. That should be straightforward to implement.

* Amit: That means it is an additional memory read on the user side
  and an additional memory write on the kernel side, along with an
  additional check whether this memory is valid for the process.

* Phil: The check whether the memory is valid would be done when the
  system call was made. If a bad pointer is passed, simply return an
  error.

* Branden: It's only a read on the userspace side if they care about
  the result.

* Leon: We could also pass in a null-pointer to signal the kernel
  we're not interested in that value.

* Phil: Let me think about this more. I'll try sketch up an
  implementation over the weekend. Johnathan's option does seem like
  the best approach.


## Callback argument type contraints

_Posted [link to the issue](https://github.com/tock/tock/issues/2320)._

* Amit: What do we think about the proposition? Do we all agree that
  the idea is to remove `ReturnCode` from the kernel and to instead use
  `Result<_, ErrorCode>` to represent the same information?

* Brad: Question: `ErrorCode` does not have a success variant. How would
  we signal _success_ to a callback?

* Leon: The idea is to reserve a special value in
  `ErrorCode`. Currently, it is an enum over an `usize`, so we could
  for instance reserve `0` as a value indicating _success_, but have
  no variant associated with that value. Then we can implement the
  `From` trait on something like an `Option<ErrorCode>` which would in
  absence of any error be encoded to `0`.

* Brad: Gotcha, so a callback would return `Option<ErrorCode>` where
  `None` would mean success.

* Leon: Essentially yes. In a Rust userspace application we could
  convert it back to an `Option<ErrorCode>` whereas in C we continue
  to use it as an integer.

  A proposal by Phil is, if we'd like to further convey the error in a
  C application, we can just use the negated integer value to indicate
  an error (following the C convention of encoding errors as negative
  integer numbers).

* Hudson: It seems that Phil's final comment suggests to not take the
  approach Leon and I were suggesting and instead we pass _success_
  and _no success_ in a separate register.

* Phil: I think your proposal seems fine, my comment was just an
  idea. I was trying to address the issue of all three registers being
  used: there is a small number of callbacks where all the parameters
  are being used. I looked at these cases and it seems simple to
  compress them down to only two registers.

* Leon: Both options seem fine. The concern with this option is that
  we would essentially "waste" another register for what seems like a
  very small amount of variants.

  For `CommandResult` that seems fine, since we upgraded from 1 to 3
  registers for actual values, whereas here we'd downgrade from 3 to
  2.

* Amit: With Phil's option, what does the capsule pass, how is that
  translated to the ABI and what does userspace do?

* Phil: Not sure, it is not a complete proposal. My major point was
  that we can either make C return values look like the Tock kernel
  (`ErrorCode`s are positive numbers), or we go the POSIX route where
  errors are represented as negative numbers.

* Leon: What convinces me about Phil's approach (to return the variant
  information as the first argument) is that we then have the same
  semantics there and in the other system call return values. This
  way, userspace can treat the parameters differently depending on
  whether a _success_ or _error value_ would be reported.

* Amit: Can you further explain how Phil's proposal would look like?

* Leon: I might be off here, but I think the idea is to make the
  callback parameters mirror the semantics of `CommandResult`, where
  in the first parameter / register the variant information is
  encoded. The remaining parameters would be used to encode values in
  a layout depending on this first parameters. For instance, if the
  first parameter indicates an error, then the second parameter would
  have to contain an `ErrorCode`, etc.

* Phil: Currently callbacks get three arguments, as the fourth is used
  for appdata. Passing appdata results in clean semantics, we don't
  want to change that. For the other three arguments, it's up to the
  callback what values are passed in there.

  Leon and Hudson looked at the issue where the kernel uses
  `ErrorCode` to pass errors around, but there is no way to pass
  `ErrorCode` to userspace given it is not able to represent
  _success_. Their proposal was to reserve the value `0`, which is not
  instantiable as part of `ErrorCode`, but we can use it to indicate
  _success_ when we're encoding `ErrorCode` to an integer value.

  My idea was to represent _success_ and _failure_, maybe callbacks
  could use the same representation as the system call return
  types. This has the caveat that one of these has to be `appdata`, so
  we have to constain the variants that can be passed into a callback
  to not require 4 parameters.

* Leon: This is not particularly bad, just that we're reducing the
  number of parameters a capsule can freely choose from 3 to 2.

* Hudson: The other downside is that some callbacks don't pass
  `ReturnCode`s at all, as they don't indicate failure.

  If we want to continue to support that, we cannot use some low-level
  shim function to decode callback parameters automatically in
  userspace. `CommandResult` works really well for _Command_ system
  calls, since every _Command_ system call is guaranteed to return
  this type. The decoding of that type is not something individual
  userspace apps have to handle.

  If only some callbacks are encoding using these types, then we can't
  hide that from users.

* Phil: Doing this in a type-safe way would mean changing the
  signature of `Callback::schedule` to take this high-level type and
  encode it into the parameters.

* Leon: But even doing that we lose one register for passing the
  variant information.

* Phil: Right. So each callback that currently returns 3 values would
  not work anymore. The only things which are problematic are 3-axis
  drivers (which can be combined in registers) and SD card.

* Branden: If it's just SD card and 3-axis, both are fairly easy to
  fix. As long as that's outruled in the future we shoudln't run into
  issues.

* Hudson: Might've missed something. Phil, did you say we should
  enforce all callbacks to return some _success_ or _failure_ type or
  would it be optional?

* Phil: I don't know. This was just an idea, not a full thought-out
  design.

* Hudson: If it were a requirements, it seems like we would be losing
  two registers. Right now, there are three values to pass to
  schedule. If one were the variant and another the `ErrorCode`, we
  would be left with only one register.

* Leon: The latter two parameters would depend on the first one. If
  the variant were _success_, we would not encode an `ErrorCode` at
  all.

* Hudon: I see, makes sense.

* Amit: Should we be enforcing this. For example, in the tricky
  register-pressured drivers that were mentioned, none of those signal
  errors ever.

* Leon: With subscribe, one could be able to just register a separate
  success and error callback.

* Brad: Or what SDCard does currently. It has one callback handler,
  but the first parameter encodes the callback type, which can be an
  error.

* Leon: Unlike with system calls such as _Command_, we're not
  constrained on passing all required information to userspace in this
  single callback type. We could in principle also have a vectored
  callbacks.

  Therefore it might not be a good idea to always enforce being able
  to encode errors in callbacks.

* Phil: I think I agree. It's been nice that callbacks are flexible
  with respect to the encoding of values. On the one hand it's cleaner
  to have the same encoding everywhere, but on the other hand it does
  take away some flexibility. I could go either way.

* Amit: I sounds reasonable to say that callbacks can still encode
  values however the capsule chooses. The issue we want to avoid
  having the type `ErrorCode` and `ReturnCode` both in the kernel
  crate, which look very similar, but are used for very different
  purposes.

  We might want to encourage drivers to, if they want to encode
  errors, use the method proposed by Hudson and Leon. Which
  essentially is only a convention to encode that structure into a
  register.

  Some callbacks will just not use it, or handle errors differently by
  having multiple kinds of callbacks. We don't even have to change
  existing drivers, with the exception of using a helper to replicate
  the behaviour of `ReturnCode`, which is not encouraged for new
  drivers. We could also go through the effort of changing userspace
  or the kernel.

* Leon: This is a good summary. Want to emphasize that this is not an
  arbitrary decision. The reason we want to move away from
  `ReturnCode` is that `SuccessWithValue` encodes a 31-bit integer
  value by cutting off one bit from a 32-bit integer, which could be
  catastrophic. Furthermore, having both _success_ and _error_ encoded
  into the same primitive type is not idiomatic Rust (we should rather
  use `Result`).

* Phil: The flip side is that this is idiomatic C. Libraries might be
  expecting errors encoded like this. This seems like the fundamental
  tension here.

* Leon: I agree. Converting between them seems easy enough though by
  just negating the discriminator.



## Outstanding PRs

* Amit: Brad, we don't have enough time to go through the list of
  outstanding PRs. Do you want to raise the issue?

* Brad: My observation is that we have approx. 50 open pull requests,
  where maybe half of them are from the last month, and the others are
  significantly older in some cases.

  At some point we should probably address them and make decisions on
  either closing them as "timed out", we intend to update them, or
  close them if they don't get updated until Tock 2.0. I wanted to
  bring this up so that we don't have these lingering pull requests
  for long periods of time.

* Amit: Not only lingering, but at some point probably completely
  unmergable without redoing the pull request anyways.

* Brad: That's my concern, so that we only do the Tock 2.0 update /
  testing once, rather than on a series of individual PRs.

* Amit: Suggestion: we choose a small subset of us to meet once, to go
  through all of those to categorize and either act on them, or bring
  them to our intention. I volunteer.

* Phil: I can do it, but I'd rather pickup some of the slack on the
  Tock 2.0 port.

* Amit: Yes, that makes sense. Hudson, Brad and I can probably do it.

* Brad: While we're on the topic, what about #1624? Question to Phil
  and Leon, should that be merged into the `tock-2.0-dev` branch?

* Leon: Down in the comments I volunteered to port this to Tock 2.0
  eventually. Initially did not want to do this while we were in the
  progress of figuring out the basic system call semantics in fear of
  making things more complex. Might break something, so we could delay
  it until all capsules have been ported.

* Brad: Good so long it's on your radar.

* Phil: Johnathan, I believe the motivation for #1624 was from
  libtock-rs?

* Johnathan: Yes, the motivation was for a `panic!` to not execute a
  callback.

* Phil: Yes, we can just wrap #1624 into Tock 2.0.
