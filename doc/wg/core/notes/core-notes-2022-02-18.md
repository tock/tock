# Tock Core Notes 2022-02-18

Attendees:
- Alexandru Radovici
- Alyssa Haroldsen
- Amit Levy
- Brad Campbell
- Branden Ghena
- Johnathan Van Why
- Leon Schuermann
- Pat Pannuto
- Philip Levias
- Vadim Sukhomlinov

## Updates

 * Brad: Just a quick update on stable Rust with Tock. There's not been a ton of
   progress, but things are moving in the right direction. Some new
   stabilization PRs have popped up, and the Rust procedure for getting approval
   to merge at least one of them (the `const_fn` one). We remove the one use of
   `option_result_contains`, so that's one feature we've removed. The major
   limiting thing is `const` `mut` refs, which we might have to dig into a bit
   more. Some progress, things are looking good from the Rust point of view.
 * Amit: Seems like bit-by-bit progress but promising.


## `libtock-rs` API design
 * Amit: If I understand the discussion topic correctly, the question is "what
   should, generally speaking, userspace drivers -- libraries for `libtock-rs`
   interacting with kernel drivers -- should their APIs generally look like?".
 * Alexandru: I started writing userspace drivers for `libtock-rs` and faced a
   dilemma on how to write them. I tried to get inspiration from `libtock-rs` 1
   but they can't be done exactly the same. There's no `async` in `libtock-rs`
   2. Basically, I could see two ways of doing this. Either write drivers as
   super low level, mimicking what the capsule does. The second is to write
   higher-level drivers which would be meaningful, and the best example is the
   GPIO driver. We could either go with functions like `set_pin`, `clear_pin`,
   `set_input`, `read_input`, or we could have a higher-level driver which
   exports structures like `Pin`, `InputPin`, `OutputPin`. Or we could have a
   mixture of the two -- low level drivers which mimic the capsules and build on
   top of them optional libraries that use memory allocation and stuff that
   requires more memory.
 * Amit: Could you expand on the differenc between having a low and high level
   API and having a low level API plus additional high-level APIs in presumably
   separate crates.
 * Alexandru: I'll take for instance the GPIO. What we could do for the GPIO is
   like the LEDs -- simply offer static functions in the driver meaning `set`
   which gets the pin number and `clear` which gets the pin number. For
   interrupts, we register a single callback. Whenever there's an interrupt for
   the pin, the same callback would be called always. The callback would have a
   parameter for the number of the pin and the state of the pin. The
   higher-level API would be to have a structure called `Pin`, from which we can
   get an `InputPin` and an `OutputPin`. You can't read from an `OutputPin` as
   the function would not be there. Whenever we want to set a callback, you
   would set the callback on the pin, like `pin.on_interrupt(callback)`. That
   would mean the high-level driver would need to register one single callback,
   have a dynamically-allocated array of pins -- we don't know at compilation
   time how many pins we have -- and send the event to those pins. It would
   probably require memory allocation -- I'd be happy to talk about how to avoid
   that -- and probably more memory. Some applications would need that, some
   would not. If this is one single driver, then you can't opt out. If you split
   them into two, the users would have a choice not to include this upper level.
   The GPIO driver is written in a strange way right now -- it has some basic
   functionlity (global callback) and some structures for pins. I don't like it
   very much.
 * Amit: Johnathan, it seems like you have one perspective on this, so maybe you
   can share what you said in the comment.
 * Johnathan: My view is kind of that when you design high-level APIs you end up
   making tradeoffs, and making correct decisions there is essentially
   impossible unless you understand the use case for those APIs. If we try to
   build high-level APIs now without using the APIs as we're building them,
   we're just going to end up with APIs that no-one uses because they don't fit
   their use case. The low-level APIs will work for any use case the syscall API
   will work for. Building high-level APIs now is doing a bunch of work that may
   never get used. My take is to focus on low-level APIs now and if somebody
   wants the high-level API for their application, they can design and
   contribute that API.
 * Amit: I was going to say it seems like one similar way to view this is "what
   do we want the role of `libtock-rs` to be and what should be done elsewhere".
   A reasonable model to me is how Rust divides the `core` library and the `std`
   library. The `core` library contains the stuff -- it's not exactly 1:1
   because the `core` library doesn't contain system calls -- but nonetheless,
   the `core` library contains the stuff that any Rust application needs.
   Similarly, maybe `libtock-rs` should contain the bare-bones glue that
   effectively should apply to any Rust application for Tock. The `std` library
   for Rust contains not only mapping to drivers, but layers of abstraction like
   the sockets API, which is higher level than the C socket API, some
   portability, etc, at the expense of not being appropriate for all
   applications. It's reasonable to have multiple standard libraries, like the
   `async` ecosystem has a few alternative standard libraries. Maybe we don't
   have to go as far as having a separate crate for each high-level driver.
   Maybe it's reasonable to have a `libtock-rs-std` -- maybe let's not call it
   that -- that is one point in the design space for how a Rust application
   would be written against a common set of high-level drivers. In fact, maybe
   that should focus on relative ease of use and good abstractions for common
   applications + prototyping, and doesn't need to be concerned about absolutely
   minimizing footprint or performance or whatever. Critical applications, like
   OpenTitan, might not use it, but many of us would use it for applications
   that are less sensitive.
 * Johnathan: You talked about a division between a core part of `libtock-rs`
   used by all applications, and a separate part where some of these APIs
   belong. We already have a division there, where `libtock_platform` contains
   stuff used by all `libtock-rs` applications, and GPIO and LEDs are in their
   own crates. We already have a division there, but maybe the lines aren't
   drawn in the right place. Like `libtock_platform` is suitable for all users
   and these APIs could just all be high-level APIs, and anyone who needs a
   low-level API could write their own.
 * Amit: At least to me it does seem like it's worth having a division in
   project too -- for example, the kind of scrutiny and process for merging PRs
   for `libtock-rs` might have a different bar than a library that is focused on
   providing usable abstractions. The same way that the bar for merging stuff
   into the kernel is different than for merging stuff into userspace.
 * Alexandru: For me it's not clear. Johnathan, you said the base would be the
   platform and then everything else would be opt-in, but the platform does not
   provide any drivers. It's just access to system calls.
 * Johnathan: That's correct. It sounded like Amit didn't realize the drivers
   were already in their own crate, so I wanted to point out that we do kind of
   have a division there. That's not the same as low-level APIs versus
   high-level APIs.
 * Alexandru: For instance, for me I don't like how the GPIO crate is right now,
   because it has some things that are more or less low level like the
   interrupts and some things like pins that are high level. I'm seeing this as
   a mixture and I'm not sure it's the right way to go. For LEDs it was clear,
   set on/off, two functions on and off, and for buttons it was just read the
   button and set callbacks. For GPIO it's more complicated, I'm not sure if
   this is the right approach. It adds some code size -- very simple
   applications may want to just use the system calls. Those structures like
   `Pin`s add some code size. I'm not sure if it's significant, but it does add
   some code size.
 * Phil: Can you talk about how it mixes high level and low level stuff? You
   can't have an input without pull up.
 * Alexandru: Let's say you want to set a pin to `1`. The simplest way you could
   do it is `Gpio::set(pin_number)`. If the pin is valid and it was previously
   set to output, it would work, otherwise it would fail. If it is an input pin,
   it's now allowed to be called like that. The way it is implemented now, you
   have `Gpio::get_pin(pin_number)`, which returns a `Pin` structure that has no
   useful methods, except for `make_input` and `make_output`. `make_output` will
   give you back an `Output` structure, on which you can `set` and `clear`.
 * Phil: So using the type system instead of runtime checks.
 * Alexandru: Exactly, but it adds some code size. On the input, it's a
   templated method with a generic, so depending on the data type you state on
   the generic, you have to specify the type of pull you want.
 * Phil: I understand the idea that what we'd like is a bunch of simple methods
   to minimize code size, but I could also imagine cases where you want the type
   checking rather than having to handle those runtime errors. The flip side is
   that when you set the pin, you have to check the return value. I think both
   cases are useful. For your particular case, A may be better than B, but for
   others B is better than A.
 * Alexandru: The problem I have is that due to this more complex API, the
   driver has the simpler functions, but they're private, not public. In the
   back, those structures will call the same function on the driver. My idea is
   we only call the low-level one and provide the higher-level one in a separate
   library, someone could build another library which is better suited for their
   job.
 * Leon: One thing to consider is that when you have these high-level APIs, one
   can always make the argument for extending the functionality covered by these
   APIs. For instance, when I say `make_output` but I have another process that
   shares that PIN, I could make the argument that now the API also needs to
   check whether the pin is still an output since I've created it. I think that
   exposing these lower-level methods could be a good common ground to build
   upon, where everyone can agree on a flat and clean API surface, and then
   develop it in these different directions with regards to how much
   functionality the high-level APIs actually cover. There's much more space for
   exploration there.
 * Alyssa: I don't like the whole high-level/low-level distinction, it's about
   statically-checked versus unchecked.
 * Phil: I agree, it's about where is the checking occurring. Is it occurring
   within the library using types, or is it required by the caller.
 * Alexandru: I think there is a problem that Leon exposed very well. We do the
   type checking, and if there's one single app that uses the pin, it's find --
   kind of, because you can call `get_pin` twice. There's no way to stop that
   without using allocation. At least I couldn't find a way, probably somebody
   more experienced than me might find a way. The problem gets trickier once you
   have two apps using the same pin. In one app you set it to an output and
   start using it, and the other app might set it as an input and the first app
   wouldn't know about that. Unless the GPIO driver allocates pins to the apps,
   and then we'd need to modify the GPIO driver in the kernel.
 * Amit: I lost the thread on where we stand in this discussion. One question
   is, is there an objection to doing this stuff in a separate
   library/crate/repository?
 * Alyssa: I think we should push people towards the static-checking versions
   while providing the unchecked versions. I think that only providing the
   unchecked versions is not the way to go.
 * Amit: Can you clarify, when you say the unchecked versions, is that the stuff
   that's currently in `libtock-rs`, or the stuff that Alex is proposing?
 * Alyssa: I believe the proposal, but I would need to see the transcript of
   this conversation to be sure.
 * Alexandru: The problem is users will always be able to directly call the
   system call, so the low-level things will always be available.
 * Alyssa: It should require `unsafe` to do so, does it not?
 * Johnathan: No, `command` is a perfectly-safe API.
 * Alexandru: I think the GPIO problem is deeper in the kernel because the
   driver does not allocate pins to apps, but that's another discussion. My
   vision was that we could build the low-level API which is not statically
   checked, and advise users to use higher-level libraries which are statically
   checked. If some user has some really constrained app, or wants to build a
   library differently, they could. If we only provide the statically-checked
   ones, the user will not be able to optimize their library, but will always be
   able to use Command and go around the driver.
 * Amit: To give another comparison, `libc` in Unix is offered as the
   recommended way to interact with the kernel through a set of abstractions,
   and sometimes people choose to go around that. Like the Go runtime doesn't
   link against `libc` and interacts directly with system calls. Now again, the
   division might be a little bit different, but to me it sounds reasonable to
   say "the recommended way is to use this crate with safe abstractions, but
   applications that are more constrained are free to use the core `libtock-rs`
   interface with little abstraction".
 * Alyssa: I would call it a "raw" interface to match with `RawSyscalls`.
 * Leon: I agree with the Alex's sentiment because of another reason: we
   currently have a perfectly-fine way to standardize the ABI between the kernel
   and userspace at the system call layer. When we expose these raw APIs as part
   of `libtock-rs`' public API surface, there is no ambiguity.
 * Alyssa: I like exposing a raw interface but pushing people towards the
   statically-checked one.
 * Johnathan: I was initially opposed based on the high-level versus low-level
   framing, but looking at it as statically-checked versus unchecked, I'm
   totally fine with exposing a statically-checked interface, as long as it
   doesn't require dynamic memory allocation. I do think that
   dynamically-checked via dynamic memory allocation is a separate thing -- if
   you expose that, you also need to expose an API that doesn't rely on dynamic
   memory allocation. As for the low-level raw version, that could just be
   defining the constants for the different Subscribe/Allow/Command numbers, and
   telling the users "hey, if you don't want to use the high-level APIs, you got
   to go read the syscall documentation". Ultimately, that sort of low-level API
   ends up being a bunch of functions that just call Subscribe/Allow/Command
   directly, and that ends up being pure boilerplate. Providing the constants
   and saying "don't use these unless you really need them" seems like a pretty
   reasonable way to go. It's not going to be much harder for the users.
 * Alyssa: I think if we put it in a submodule called `raw` and make it clear
   that these are the unchecked interfaces, and have functions that are really
   thin wrappers over the syscalls, so you don't have to do a ton of research to
   figure out how to do a raw output on a pin.
 * Alexandru: I think the functions can be inlined there, it will be transparend
   in the code size.
 * Johnathan: I don't think we need to bother maintaining the functions.
 * Alyssa: I think it would be good for us to provide somewhere in between the
   statically-checked API and just doing syscalls. Something a little bit more
   friendly.
 * Alexandru: I see this as the difference between the standard C library and
   POSIX. Of course, the standard C library works on several platforms, but if
   you use the POSIX API you're not directly using system calls but it's very
   close. If you use the standard library, it has more functions but takes more
   space.
 * Alyssa: I mean, unused functions in `libtock-rs` are basically free.
 * Johnathan: Yeah, I think it should be possible to avoid code bloat. The main
   thing is the statically-checked version will probably not support every
   possible usage pattern -- it will probably be a bit over-restrictive.
 * Alexandru: At least for pins, I cannot put an interrupt on a pin structure
   unless I can allocate a vector of pins. If there is any other way of doing
   this, and not having static things and `unsafe`, I would be really interested
   to talk about it.
 * Alyssa: I don't think that should be necessary.
 * Johnathan: I take back what I said about extra cost. You can totally do that,
   but it would involve like a linked list, which is going to be larger than the
   raw API.
 * Amit: I think this discussion should be another vote for doing this in a
   separate library. Folks in this call have a history of designing good APIs.
   One way to make progress on this sort of question is to go explore and maybe
   design an API that makes sense for a particular kind of application, and when
   there is some usable library -- maybe covers more than just GPIO and LEDs
   because that's not necessarily the totality of drivers than an application
   would use -- then we could have a look at it. We could then look it it and
   tweak it to cover more applications or decide it is application-specific and
   we need different abstractions for other applications. That would give us a
   sense of whether we want an additional library or to tweak this one.
 * Johnathan: That might be useful from a maintainership perspective. I've
   wanted to review all the code in `libtock-rs`, but I honestly didn't want to
   review the high-level APIs also.
 * Alexandru: We can build an additional repository with libraries for
   `libtock-rs` and push there.
 * Johnathan: I don't think it should be an additional repository.
 * Alyssa: Why not draft one under `apis/`?
 * Alexandru: Okay
 * Johnathan: We currently have one crate per API. That's probably an
   unnecessary amount of division. We can probably combine all the low-level
   interfaces into a single crate and all the high-level interfaces into a
   different crate or something instead.
 * Alyssa: As long as those don't require any statically-allocated memory. I
   know that Console might require that.
 * Johnathan: I honestly wouldn't expect that because it will not work well in
   the unit test environment.
 * Alyssa: Static memory?
 * Johnathan: Yeah. A mutable static is going to be a challenge in the unit test
   environment -- that's a thread-safety issue. An immutable static, either way,
   reachability analysis will not compile it in if that API is not used.
 * Alyssa: Hopefully.
 * Johanthan: Actually, the bigger concern is HMAC or CTAP support, which
   doesn't currently exist but exists in libtock 1, and depends on large
   external crates. That's a big dependency tree thing. We could handle that
   through the use of features, or perhaps that should be a separate crate.
 * Amit: One reason I might advocate this being in a separate repository is it
   would be a shame if Alex ends up blocking on discussions about particular
   drivers while there's a design exploration process going on. If there ends up
   being a whole discussion about whether this way of doing things is optimal
   for every driver, or it might be the case that a few drivers work together a
   bit better, then maybe it's not worth having that in the same workflow as
   more mature `libtock-rs` work.
 * Amit: Alex, was this discussion useful enough to move forward, or is there
   still a remaining open question about how and what to do.
 * Alexandru: It's not really clear how I should move forward. The next drivers
   would be Alarm probably, and the simple drivers which are really easy to
   implement, but I'm not sure how to implement them. For instance, for Alarm, I
   could go with structures and something similar to `libtock-c`, where part of
   the alarms are set in userspace. The driver only accepts one alarm per
   process, and in `libtock-c` you can have several. Or simply go with a simple
   raw interface as Alyssa said. I'm not really sure how to continue and I'm not
   really sure if I should change the GPIO driver.
 * Amit: I'm curious what other people who haven't chimed in think about this.
   My sense is that if we want to move towards building applications in Rust by
   default, then we ought to have a user-level library layer that is very
   convenient for writing applications against. I think it's clear that
   `libtock-rs`' primary goal should not be to do that, that we should have an
   advocated-for but optional separate library for exposing a porcelain API that
   prioritizes convenience and ease of development over performance or memory
   efficiency. Obviously there's a tradeoff and we need to find a reasonable
   design point, but yes. For alarms, I think it should be similar to
   `libtock-c` -- it makes sense for apps to have multiple alarms, and managing
   that on your own is tricky, and that logic shouldn't be in `libtock-rs`. I
   think it should be in at least a separate crate or separate repository that
   takes much more freedom than `libtock-rs` should about implementing
   heavyweight abstractions.
 * Johnathan: I don't think it should be a separate repository, as there are some
   blurred things between there, like does dynamic memory allocation belong in
   `libtock-rs` or does it belong in that crate? That has much more tie-in with
   the runtime than it is optional in `libtock-rs`. I'm not sure if a separate
   repository is right, a separate crate seems good thought.
 * Alyssa: From the Ti50 perspective side, if `libtock-rs` doesn't implement
   something that makes sense, we'll implement our own, and I'd rather not
   duplicate that work.
 * Amit: It's quite likely that Ti50 will have different requirements of a
   runtime library than non-Ti50 applications.
 * Alyssa: I suppose, but I think designing for alloc-less first is a good idea.
   Have that be the default, and have some nice-to-haves that require
   allocation.
 * Alexandru: In the pin library, my thought was to add allocation as a feature,
   so if you don't use allocation you can't set interrupts on the pin structure.
   If you use allocation, you can set interrupts directly on the pin structure.
 * Alyssa: I feel like a lot of the time, `Box` can be replaced by a static
   reference.
 * Alexandru: If you have time we can chat about that, I would be interested. If
   you have any idea how to do that without allocation, you have more experience
   than me in Rust, I would be happy to talk to you about that.
 * Amit: I suspect that at high level the answer is the library doesn't know how
   many timers the application will need but the application will, so the
   application can statically allocate a structure for storing timers and pass
   that to the library. Then you don't need to dynamically allocate on the heap,
   you have the application allocate statically and pass to the library.
 * Alyssa: Just have it all be borrowed.
 * Alexandru: I see what you mean.
 * Amit: Leon, Brandon, Pat, Brad, and Vadim, and Phil to some degree -- Phil
   talked a little bit -- thoughts on this? What's the library that you would
   want to use?
 * Phil: One thing I'm worried about is the type checking that's provided by the
   higher-level library is only valid in the context of kernel drivers that
   provide exclusive access. Like I might get an output pin struct, but if
   another process makes it an input it's no longer valid. We can't necessarily
   avoid runtime checks within the application. That suggests to me that the raw
   interface is what we want, but doing something that leverages Rust's type
   checking might require changes to the syscall API. That's a bigger can of
   worms.
 * Leon: I'm personally not fundamentally convinced that the static type
   checking approach will work, but I agree with the general sentiment of having
   the raw API. I think that is important to establish as a ground truth from
   which you build higher abstractions. What Alyssa said about building from
   something that does not require allocation is a good idea, and it plays
   nicely with whath Rust's `core` and `std` libraries do, and the division
   seems very natural to me.
 * Vadim: Yeah. I would say I prefer to have both raw interface and type-checked
   interface and let basically developers choose and see how it works, what are
   the use cases for the raw interface, maybe we can generalize. I usually like
   don't want to force people to do some specific way, so I prefer to have both
   ways of doing that and see adoption dynamics and see what new ideas may pop
   up.
 * Amit: My summary here is that since Alex, you are kind of the one doing the
   work for these drivers and hopefully you have some use in mind, I know you
   have applications you are building, essentially it is up to you and the rest
   of us can choose -- I suppose -- to offer feedback and use it or not.
 * Alexandru: My take of this would be that LEDs and Buttons are written exactly
   like that. I would modify the GPIO interface to have a fully-raw interface
   and split the libraries somehow out. Johnathan, do you prefer it in some
   place in `libtock-rs`? Are the APIs library, or how do you see these being
   organized?
 * Johnathan: No strong opinions, other than that I don't want two crates for
   every syscall API. I would personally prefer to merge the raw drivers
   together and all the high-level -- well, I don't know.
 * Amit: I generally agree with that, for what it's worth. We don't need a
   million different crates, we need crates that offer different entry points.
 * Alexandru: Do you want me to rearrange the raw interfaces into one crate and
   submit a PR for that? Maybe we can merge the button PR which seems to be
   ready, and then I can rebase the GPIO PR and split them.
 * Johnathan: I think merging this into a raw crate -- I'm just questioning,
   maybe this belongs in `libtock_platform`.
 * Alyssa: Maybe the raw APIs should be a submodule of the higher-level APIs
   that currently exist. Like there's the `apis/leds` crate and maybe there
   should be a `apis/leds::raw` module.
 * Alexandru: I understand. So basicallly if you don't use the higher level,
   then you don't pay the cost of it, it won't be compiled into your app, so you
   can always use the raw ones.
 * Alyssa: Yes, I believe so.
 * Alexandru: But it would be something use `apis/leds` raw.
 * Alyssa: I guess it's up to the group whether you want a single raw crate or
   have a raw module in every API crate.
 * Amit: I personally prefer a separate raw crate or in `libtock_platform` or
   something.
 * Leon: If we determine that we want to have single crates for each syscall
   driver, then this takes me back to the question I have a few minutes ago
   about feature flags. Would the feature flags be disabling an entire module,
   or is that going to just disable a few functions? I think feature flags are
   the wrong tool to disable a few functions.
 * Alexandru: Probably a few functions, but it was just an idea. I'm not if you
   want to use that.
 * Leon: I'm scared that we will have -- depending on the combination of feature
   flags that are enabled -- we will have a very incoherent API.
 * Alexandru: I was referring to a single feature, alloc, if allocation is
   available or not.
 * Alyssa: That's entirely reasonable.
 * Leon: I understood that you'd essentially have two ways to achieve one goal
   rather than two APIs. I was worried you'd have multiple functions to do the
   same thing, one richer than the other, and then we have to deal with the
   implications of people using two functions interchangably. Whereas when we
   have two very separate and encapsulated APIs, where one is the raw one and
   one is the more rich one which perhaps uses allocation, then we can think
   about all the interactions of these calls to these APIs.
 * Alexandru: I like Alyssa's ideas to have a raw submodule in the API because
   that API will use that raw module.
 * Alyssa: Yeah, exactly,
 * Amit: Great, that sounds like some sort of way forward. In my mind, if that's
   what Alex prefers, that's what we should do. It seems coherent.
 * Alexandru: Okay. I can restructure the GPIO driver like that and send another
   push. Maybe a different PR, just drop PR and people can actually state some
   opinions on that.
 * Amit: I think we're overtime. If I can offer some predictive advice about
   this, I think it is worth leaving ourselves open to changing these sorts of
   decisions once we get to more complex drivers APIs. I strongly suspect that
   for example, without commenting about the specific choices so far, doing
   alarm and user-level networking library, maybe LCD or screen stuff, will
   elucidate more challenges. Those are more complex and involved APIs than
   things like GPIO and LEDs. We should be open to revisiting decisions like
   these once we get to those APIs.
 * Alexandru: Yeah, makes sense.
