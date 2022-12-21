# Tock Core Notes 2022-09-23

Attendees:
- Adithya Anand
- Brad Campbell
- Chris Frantz
- Branden Ghena
- Alyssa Haroldsen
- Amit Levy
- Pat Pannuto
- Alexandru Radovici
- Jett Rink
- Leon Schuermann
- Vadium Sukhomlinov
- Johnathan Van Why

## Significant Pull Requests

* Amit: Brad submitted a substantial amount of pull requests for
  component updates porting to new static format.

  Hudson submitted PR [#3239](https://github.com/tock/tock/pull/3239)
  to remove `StaticUninitializedBuffer` and `UninitializedBuffer` in
  favor of `MaybeUninit`.

  Furthermore:
  - [#3225](https://github.com/tock/tock/pull/3225)
    BBC HiFive Inventor board ported to Tock
  - [#3218](https://github.com/tock/tock/pull/3218)
    ESP32-c3 Build Fixes, add docs for flashing apps

## Dependabot and `tock-teensy` PR #18

* Branden: Are we still using the [tock-teensy
  repository](https://github.com/tock/tock-teensy)?
  Dependabot noticed a vulnerability and submitted [a pull
  request](https://github.com/tock/tock-teensy/pull/18).

  Seems like no one with knowledge about this repository is on the
  call, might punt on it for next week.

* Leon: Teensy 3.6 is already significantly outdated. Not sure whether
  makes sense to continue support.

* Branden: Suspect we need to wait on Phil and Hudson. Suspect they'd
  vouch to archive it.

### Dependabot Alert on `elf2tab` for `structopt`

* Branden: Dependabot also sent an alert for our usage of
  `structopt`. Migrate to `clap`. Opened issue
  [#53](https://github.com/tock/elf2tab/issues/53).

## `static_init` Updates

* Brad: `static_init` has been around for a long time in Tock. A few
  years ago, to make components work for the kernel, we needed a
  version of it which would separate the actual declaration of static
  memory from initializing this memory. This led to creating
  `static_init_half` macro, which separated creating the static buffer
  from initializing it with the object type. The idea was that
  creating the buffer could be in the boards main, whereas the
  initialization happens in the component itself. About 2 years ago,
  we updated `static_init` to make it more sound and safe to use.

  PR [#3239](https://github.com/tock/tock/pull/3239) adds a few new
  things:
  1. Components can generally only be used once today. This is because
     some of the components use `static_init`
     internally. Instantiating these components multiple times would
     then alias the underlying buffer.
  2. The infrastructure currently has grown organically, with
     different parts of it being in various states.
  3. Rust types for uninitialized memory have been improved.

  This PR updates the `static_init` and `static_init_half`
  implementations to check whether one is aliasing the underlying
  memory. When this happens, it will issue a panic.

  Furthermore, we are getting rid of a lot of machinery which we added
  to Tock, in favor of the core library's `MaybeUninit`.

  Finally, we are introducing a standard practice way to write
  components, which we can check against in PR reviews. We should
  disallow `static_init` to be called in components, which makes them
  reusable.

  This PR should be a good step forward for `static_init` and
  components in Tock. It does not, however, enforce the fact that we
  cannot use `static_init` in components explicitly through types, but
  just by convention.

* Alyssa: We definitely need to keep the unchecked, raw version
  around, which just writes without any checks. This is for places
  where we want to use unsafe to eliminate the cost of these
  checks. We cannot afford this `Option` cost at this exact moment.

  Also, it does need to be sound on `std` systems, e.g. in unit
  tests. This means that, if it is running on a system with threads,
  it must perform synchronization. If it is not performing
  synchronization, it is unsound.

* Brad: We do not have any systems with multiple threads.

* Alyssa: Unit tests do run on systems with multiple threads, and we
  want to unit test. Also have host emulation running on x86-64. If it
  is `unsafe`, then it's fine to not synchronize. However, if it is
  safe and does not perform synchronization, it's unsound.

* Amit: Is there a way to perform synchronization on these things in
  test environments, but is not expensive on our single-threaded
  target platforms.

* Alyssa: I would go with the safest option, which is to enable
  synchronization by default and turn it off using a flag. This can be
  implemented using `build.rs`, for instance.

* Amit: Basically, this translates to `#ifdef`s which would incur
  `Mutex`es in these macros on platforms where it would be required
  for safety.

  Is there a major benefit from having these new macros not be marked
  as `unsafe`? These are all used in the board definitions, where we
  are working with a global unsafe on the function definition.

* Brad: Right, this is because we are not changing the component
  trait. If we were to mark `static_buf` as unsafe, things would
  continue to work.

* Amit: It seems that the vast majority of this PR is a clear
  improvement. A version of this PR which does not remove the `unsafe`
  seems uncontroversial enough.

* Alyssa: I have been thinking more about initialization. Given that
  initialization happens all at once during initialization, it would
  be nice for us to transition to a function or a macro which
  generates a function, which is guaranteed to only be executed
  once. Instead of tracking the initialization of each single object,
  you are tracking the initialization of a group of objects.

* Amit: Seems like a good idea. Suspect that synchronization on
  multi-threaded systems would still be an issue. Anything which
  enforces the invariant that the reset handler is only called once
  seems like an improvement.

* Brad: Board initialization is 90% unsafe. Hence using unsafe as a
  marker of caution is not working for us; just have 100s of lines of
  code of unsafe. Making `static_init` would help us to specifically
  tag distinguished part of the initialization as unsafe.

* Alyssa: The benefit is only truly there if it is actually safe
  though. It is not fine for us to make it safe it it isn't actually
  safe to use in every context.

* Amit: On the target systems for the OS, what synchronization would
  look like is a no-op, right?

* Alyssa: Would also need to disable interrupt.

* Amit: Even if the interrupt is not reading / writing it?

* Alyssa: No, only a problem if the interrupt can read / write it.

* Amit: In the context we're in, we are exposing things which are only
  safe because of the environment we are working in and assumptions
  made (e.g. hardware specifics, memory layout, etc.), but not
  necessarily in the larger Rust context.

  How strongly are we willing to keep never allowing this type of Tock
  code to be used from running in a violating environment. It seems
  like a reasonable discussion whether this is a case in which we want
  to make weaker assumptions about the underlying system (e.g. to
  safely run on multi-threaded unit tests), or say that developers
  must pay attention to emulate specifics of the particular system one
  is working on.

* Alyssa: If a given piece of code is safe, and is only sound given
  certain properties of the surrounding system which are not checked,
  then it is unsound. It must not compile on systems where it is
  marked as safe and is unsound.

* Alex: How is this different from accessing a buffer from within a
  capsule? An interrupt can fire and conceivably, on a different
  system, modify the buffer contents. This is not using unsafe to
  access the buffer.

* Johnathan: It would not be safe to call into a capsule from within
  an interrupt context, as capsules are not `Sync`. In the board's
  initialization function, there is nothing preventing it from being
  accessed from multiple threads.

  The synchronization primitive used in `static_init` can be modeled
  through dependency injection.

* Amit: To summarize: we do not all have to agree on the statement as
  to whether it would be _only_ permissible to have `static_init` be
  safe when it is sound in a multi-threaded (i.e., non Tock board)
  context as well. We can certainly agree that this is desirable.

* Jett: Inside `static_buf` macro, we currently dereference a
  buffer. If we remove that and move it out of the function, we'd use
  unsafe in all the call location. This might remove the controversy
  around this.

* Brad: Moving the unsafety up does have a cost. This is not related
  to the academic notion of safety, but the software engineering
  notion of safety and what it means for users to develop for this
  system.

* Alyssa: We are deferring the decision and figuring out a solution
  for later. We are introducing new checks which inform users when
  they have violated the assumptions of these macros, even if it does
  not make them entirely safe. This itself is a massive improvement.

* Amit: Defer the discussion of the context switch code to next
  week. Also move remainder of this discussion to the pull request and
  an issue about removing the `unsafe` marker now or in the future.

