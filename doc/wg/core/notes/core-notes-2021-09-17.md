# Tock Core Notes 2021-09-17

## Attending

- Hudson Ayers
- Brad Campbell
- Arjun Deopujari
- Branden Ghena
- Philip Levis
- Amit Levy
- Gabe Marcano
- Alexandru Radovici
- Jett Rink
- Leon Schuermann
- Vadim Sukhomlinov
- Johnathan Van Why

# Updates

- Johnathan: Tried to implement fuzzing against libtock-rs allow APIs to verify
  that libtock-unittest correctly panics before invoking undefined
  behavior. Turns out, catch unwind does with libfuzzer. Those tests can't be
  written. Keeping the infrastructure around for when we have something to fuzz.

  Tried to refactor mock drivers for system calls and combine them into one,
  realized that this was not an improvement.

  Also thought about read-only allow. Realized the previous design is unsound,
  allows that one can get both a mutable and immutable reference to some
  buffer. Unfortunately, the only sound design might be using slices of
  cells. Tried to avoid that. Opened an issue in the libtock-rs repo.

# Shortening libtock-rs' review policy

- Amit: Johnathan suggested shortening the review policy for libtock-rs from 7
  days to something shorter, such as 3 days, until we get to 2.0 compatibility
  for libtock-rs.

- Johnathan: there is a deep dependency tree between PRs. Don't want to send
  them all at once for review. Almost all PRs have been reviewed by Hudson and
  Alistair. Essentially the PRs get two PRs quickly and then sit and wait.

  Would like to shorten this delay to allow making significantly more progress
  on the design and iterate more quickly.

- Amit: seems reasonable to me. Anybody object?

- Phil: can we push it further and say, temporarily, that if there is one review
  we wait for three days, and if there is two approvals we can merge it
  immediately?

  Basically, if both Hudson and Alistair approve then we don't even need to wait
  the three days.

- Johnathan: would also be okay with explicitly saying "if Alistair and Hudson
  and Johnathan reviewed and approved" it's good for immediate merge.

- Hudson: can just say "two members of the core team".

- Amit: the purpose of the longer window is to let people chime in. Most of
  those people would be on this call. Would anybody actually look at one of
  these PRs within a day or two?

  ...

  This proposal is hereby blessed.

## PR #2759: Add a kernel config option to elide process printing code

- Amit: as I understand it, there is not necessarily many changes in terms of
  the high-level positions on this since the last time we discussed it. Hudson,
  want to give an update?

- Hudson: high-level positions have iterated. Brad presented a nearly working
  alternative PR: #2826. This creates a ProcessPrinter trait. Is this full
  functional, Brad?

- Brad: it's functional but it only handles the memory map.

- Hudson: right, so the idea is that the memory map is the most complex to print
  and the rest of the process state is equivalent or easier than the memory map?
  There's no weird issue with generic which we haven't found a way to address?

- Brad: no. The memory map case is the very easy case. The "print full process"
  is the more complex case.

- Hudson: high-level overview: basically, back when I submitted this PR for a
  kernel configuration option, there was push back for that: people don't want
  to have a large matrix of configuration options because this makes testing
  more difficult. Furthermore, it would require to set a flag and at the same
  time to not call `print_full_process`. It's undesirable to have to do both.

  The motivation for this is code size. Rust is not optimizing out unused
  methods on traits for which trait objects exist.

  In terms of alternate designs: Brad has proposed the process printer approach
  in his PR. The good thing about this is that it would allow for a
  null-implementation which would not have any negative impact on code size, but
  it could also allow for a different implementation which would, for example,
  log to flash. Main disadvantage: tricky to get this to work from a design
  standpoint. It requires a lot of information which is local to the process,
  either because it's not trivial to expose it or it wouldn't be generic over
  multiple process implementations.

- Brad: what makes this such a thorny problem is that it's three intertwined
  challenges with tradeoffs on their own, but they can't be solved
  independently. One question which would help us make progress is to decide on
  whether to support different methods of "printing". Currently, only human
  readable printing.

- Amit: because if we do decide that, something like a trait makes more sense.

- Hudson: and if we don't the trait approach probably has more downsides than
  the config approach.

- Leon: judging from the current state of the ProcessPrinter PR and the
  interface as it is designed there, we couldn't only use this for outputting
  process state or writing to flash, but rather as a generic process
  introspection API. This could be very useful for testing the system or
  fuzzing. Trait would allow us to get information for these purposes as well.

- Phil: the idea is that to influence the strategy of printing I would modify
  some initialization path code in order to instantiate a different instance of
  this trait?

- Hudson: I think if we follow Brad's approach the idea is to not call
  `panic_print` and thus not end up with the size associated with these
  functions in the binary.

- Phil: where is controlled whether you call `panic_print`?

- Leon, Hudson: controlled in the panic handler defined in the boards.

- Phil: so we'd modify the board code of the panic handler to change this
  behavior.

- Leon: one could also build infrastructure to pass this in from the board
  initialization. A similar thing is done for some UARTs which are passed into
  the panic handler from the board's main.

  Thus it seems entirely reasonable to do all of this configuration in the board
  initialization.

- Phil: at some point there is a reason why, at some point when they get
  complicated, applications have configuration files rather than modifying the
  source. In this case it's different given it is not dynamically
  loaded. However, at some point modifying the source becomes a greater cost
  than the complexity of configuration. Not saying we're at that point
  yet. Completely agree with the point of crazy configuration options leading to
  complexity.

  We probably don't want to encourage users to modify the panic handlers.

- Brad: This is all true today. Not sure why this particular effort would change
  this.

- Phil: I think this is not sustainable going forward. We don't want users to
  edit executable code as configuration options. Not saying that now is the time
  to change this.

- Brad: missing the connection to this issue.

- Hudson: my impression is that, if every time an option for a configuration
  comes up we say that we don't like config and instead use traits, then one day
  we'll end up with many traits which might be difficult to maintain.

  It might be easier to fuzz different configuration options instead of
  different traits, which must be implemented in the source.

- Phil: the argument of an exponential cost for testing with configuration
  options is valid, but is the flipside that we don't actually explore that
  space because it requires changing code? Are we going to have test cases where
  have the panic handler call this function and test cases where we do not?

- Amit: what's our answer to this?

- Phil: to give an example of where configuration options would have been nice:
  the size of a SPI buffer used by a SPI implementation. Turns out there was a
  bug in the virtualizer because it happened to be that the system call buffer
  was smaller than the underlying peripheral buffer. It never tested the case
  where the peripheral buffer was smaller. The only way to test that would have
  been to modify some constants in some files.

  I'm of the opinion that coming up with a clear way to think about
  configuration options would give us a lot more clarity.

- Leon: actually, your example is also a good illustration as to why
  configuration options are sometimes not the solution. For SPI buffer sizes,
  you'd likely want to set it on a per-instance basis, where as configuration
  options are in their concept a very limited feature. If we start to shift
  towards configuration options for all things we do not consider to be
  changeable at runtime, we would also quickly run into issues, given they are
  not as flexible as the configuration approach we're using currently.

- Phil: I was think not in the sense of Rust `cfg`, more generally in the sense
  of configuring the kernel.

- Leon: right. I agree we should probably have a broad concept about
  configuration options. I do interpret the current question regarding printing
  of process state as being one of deciding whether a Rust `cfg`-macro based
  approach would work.

- Phil: basic though is, when we have the trait-based approach, then do I use a
  configuration option in the board to configure whether this method is actually
  called.

- Brad: sure. If you want to have configurable panic handlers, that seems fine.

- Amit: between these two things, do we foresee supporting other ways of
  serializing human-readable debug text. It seems to me the answer is almost
  certainly yes. In a production system one probably wants to collect
  information about panics, and human-readable text is neither useful nor
  efficient. It seems that, in the long run, the trait approach is probably
  better.

  Maybe there also is a more time-sensitive use case for having configs. The
  default for those configs should be off, such that most boards can ignore
  them. They can be replaced with a trait when there is both sufficient
  motivation to implement it and an actual implementation.

- Hudson: for now, we are carrying downstream patches for that. The preference
  would be to avoid downstream patches and be able to directly rely on the
  upstream kernel crate.

- Amit: personally uneasy with downstream patches. Sometimes necessary,
  hopefully only temporarily. I'm uncomfortable forcing to continue that,
  especially in OpenTitan.

- Phil: I'm not strongly against them. There can be situations where the patch
  cannot be reasonably upstreamed.

- Hudson: if we need to wait two weeks to get a trait implementation in, that
  won't be an issue.

- Amit: do we believe that it will actually take only two weeks.

- Brad: is there an issue which explains this time criticality?

- Hudson: no, no specific time criticality. We're running into the edge of flash
  for our downstream code. We'd like to remove the downstream patch as soon as
  possible, such that we don't have to rebase these patches on top of other
  changes and can stay up to date with Tock.

- Brad: makes sense.

- Amit: my desire from having an imperfect, short-term solution is that,
  OpenTitan using and not forking Tock is important to the project. Code size is
  a sticking point. We should help them with this.

- Leon: to confirm, is the approach with the traits compatible with the
  configuration options? In the sense that the configuration options would give
  us the option to disable these debug code paths downstream, and we can then
  continue developing the traits based approach with the upstream boards in the
  meantime?

- Hudson: I don't think there's anything forcing us to commit to one approach in
  the long term.

- Leon: I wouldn't want to keep both of them in the kernel indefinitely, but
  whether we can add the configuration options now and buy us time to figure out
  the trait based interfaces, and not require downstream users to adapt to these
  changes constantly because we iterated on the traits-based interface.

- Phil: I'd be in favor of that. Have a short-term solution to buy us some
  time. If we can do this with a traits-based approach and it has roughly the
  same impact on code size etc., it's strictly better.

- Amit: right. I'm not as opposed to configuration options dogmatically, however
  here it does seem that there is some more flexibility we would want to have.

- Brad: are these ever going to the removed again? Probably there will be pull
  requests adding more configuration options in the meantime?

- Amit: there is no reason why we shouldn't immediately work on the trait
  approach.

- Brad: there is a pull request for a trait right now.

- Amit: but it's not complete yet.

- Brad: right, the issue is that, there is not a lot of incentive to solve the
  hardest part if there is not an agreement that this will ever work.

- Amit: we do agree that it is viable and it would work. We've been saying that
  the trait is the better long term solution.

- Phil: right, said that it is "strictly better".

- Brad: the catch is that every component we want to print from is going to
  require a trait. For instance, Process, MPU, UserKernelBoundary, ...

- Amit: that seems quite in line with how Rust works more generally. There's
  often a `Debug` trait which most things end up implementing for a slightly
  different use case. It seems very reasonable that things we want to be
  displayable implement that. It's also not an infinite number of structs that's
  going to have to implement this.

- Hudson: generally agree with what Amit said.

# ProcessConsole

- Alexandru: Brad just replied before the meeting that an Alarm would be
  okay. Was my original idea and is fine with me. Only concern: it will increase
  code size, the process console needs to add an Alarm client and there needs to
  be another virtualized Alarm. Switched it to be based on pressing a key
  because it has less impact on code size.

- Phil: seems fine. Generally systems using the process console are not the most
  concerned about code size.

  Have been exploring the different options here and in the end it seems the
  timeout-based approach really is the easiest one.

- Alexandru: this clears it then. A timeout of 100ms should be fine. Not sure
  how it will work with USB CDC but will test that.

# UART HIL

- Phil: we have `ReceiveAdvanced` which allows you to do receive with a
  timeout. It's in the UART HIL, but only one chip implements it (SAM4L). The
  bootloader has an emulated version of it for when the hardware does not have
  such as mechanism.

  It's weird in that's a trait which is not used anywhere in the main tree and
  only one chip implements it. My opinion is: if it remains in the tree, we
  should include an emulator. If we don't want to have uniform implementations
  in the tree, we want to remove it from the HIL.

- Leon: from experiences with previous implementations using this trait, what is
  the actual utility of having `ReceivedAdvanced`? I'm aware it's used in the
  bootloader. Has anyone ever tried to use it anywhere else?

- Branden: background is that I wrote some previous version of the UART HIL, was
  trying to come up with generic capabilities which a serial interface would
  have. It used to have `receive_until_timeout`, `receive_until_character` and
  `receive_length`. We remove some of them given they were never properly
  implemented.

- Phil: this seems like a good example of something which hardware doesn't
  generally provide but we can emulate in software. It seems weird to have this
  HIL which we can't generally use in the main tree.

- Leon: what I'm worried about in your comment, Phil, is that your saying one
  could just work around the HIL and interact with the hardware driver directly
  for these features. That seems like a valid approach when there is a specific
  or special piece of hardware one wants to interact with. However, presumably
  we wouldn't want to use endorse this as a general paradigm. It invalidates the
  purpose for which we use HILs, which is hardware abstraction. When we force
  users to interact with the hardware directly, we might have much more code
  which will be hardware dependent.

- Phil: there can be other HILs than `kernel::hil`. If we don't have it in the
  main UART HIL any longer, but the bootloader still wants to be hardware
  independent, it can still introduce its own abstraction for that.

- Brad: I'd like to argue for having this behavior in the tree, however I'm not
  sure that is a good idea. I'm not sure that emulating this behavior works well
  or is desirable. I'm not sure whether it's a good idea for the bootloader to
  use this any longer. The best option is probably to remove the HIL. It's main
  benefit would be to, if someone asks for how to implement a specific feature,
  it would be a good pointer to a model which we like.

- Phil: "specific feature" is representative for "expose some hardware
  functionality"?

- Brad: correct.

- Phil: what's the particular issue with this one then?

- Brad: naming or documentation? It suggests that it lets you connect a chip to
  a capsule but it's not something every chip can do. It's an extension of
  existing functionality. We want something which we can point people to in
  advance.

- Leon: this refers to a discussion quite a while ago: we want to treat the UART
  HIL as a "core" or "important" HIL, we've been talking about various
  interfaces with different "statuses" in the Tock repository, where only some
  would be properly documented, standardized, "core" HILs.

  Could this extension be part of a less-strictly standardized and maintained,
  optional interface?

- Phil: it again comes down to whether or not there are implementations which
  you can use.

  Brad, your point is right. There place where that is going to manifest is in
  one of the most complex HILs: ADC. That might be a good place to have these
  extensions and configurations.
