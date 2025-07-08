# Tock Core Notes 2021-10-01

## Attending

- Hudson Ayers
- Brad Campbell
- Philip Levis
- Gabe Marcano
- Pat Pannuto
- Alexandru Radovici
- Leon Schuermann
- Vadim Sukhomlinov
- Johnathan Van Why

## Updates

### HIL TRD

- Phil: finalized the HIL TRD. One question: what do we do when things
  change? Pointed to RFC2581 as a model for this (TCP congestion
  control, e.g. initial window size). It is a series of SHOULD
  statements which then have been amended and evolved. It is intended
  as a set of guidelines which can evolve and change, but not
  necessarily lightly.

### Tock Book & Presentation

- Alexandru: Had a virtual presentation at embedded Linux
  conference. Talked about how to use Tock for MicroBit. Will be on
  YouTube, can share the link.

  Finished the book, should be ready next week. Would like to thank
  Branden and Leon for reviewing some of the chapters.

- Johnathan: Prototyping some new async ideas for
  libtock-rs. Prototyping a full "Hello World" application.

### Hardware CI

- Pat: HW CI: Anthony is in the lab instead of remote. He was
  re-plugging wires for every test. We're figuring out how to reset
  all IOs such that we can test SPI, I2C etc. without re-plugging
  things. Thus slight delay.

- Phil: For the test don't you want to just hard-reboot the device?

- Pat: This is on the Raspberry Pi side. There are constraints on
  which pins can be used for which purpose. These need to be
  configured and cleaned up again.

## Process Console Shell Prompt

- Alex: Nothing changed much. Added the alarm-based method. Brad
  pointed out that applications printing immediately, it gets
  interleaved with the shell prompt. It's as if you had a background
  application in bash.

  Brad is saying that he doesn't know whether we want this.

- Phil: That's how all shells behave. When there's a background and
  forward process, this happens.

- Alex: Brad is probably right in that we require some type of console
  output multiplexing. Would take a while to come up with a solution
  to that.

- Leon: Remember at least two discussions where we talked about
  console virtualization. It's a long standing issue. Doubt that we'll
  have a quick solution to this.

  As far as I understand the current state of this PR, the prompt
  generation is entirely optional. If this messes with developer's
  expectations, they are free to turn it off. It's trivial to
  recognize and disable this.

- Alex: in any case, if an app requires input it must be disabled.

- Phil: it's not intended to be used alongside applications which
  require input.

  Just say that process console isn't to be used in these cases.

- Leon: was referring to output. For instance if another machine /
  program is attached to the console output and the shell prompt
  messes with this communication channel.

- Alex: if you have something connected to the output, you would most
  likely disable the process console entirely.

- Leon: correct. It might be slightly annoying that the process shell
  prompt will be generated, but the solution to disable it is pretty
  accessible.

- Hudson: to disable the process console, kernel needs to be
  modified. In tutorials, we're going to say "flash the default
  kernel", "flash blink", "flash hello world". For this simple use
  case, we don't want people to recompile their kernel.

- Leon: why would users need to recompile their kernel? The prompt
  would just be interleaved with the output of the hello application.

- Hudson: we would like Tock to be a system where it's not required to
  load a new kernel depending on the applications one is using (at
  least for the default Tock kernels in upstream).

- Leon: but the effect of this is just that there's additional output?
  It doesn't really interfere with any app's behavior. Apps which read
  from the console can't be used regardless of whether we have a
  process console prompt or not.

- Phil: it's weird that, from a tutorial's point of view, there is now
  an uncorrelated additional message. The process console was
  originally written as a utility to be usable in a tutorial for
  demonstrating power management, spin loops, etc.

- Alex: this is why I've opted for the keypress approach. The initial
  message would say "press a key to start the process console".

- Leon: it sounds like the root issue is that the process console then
  messes with the expected behavior of apps w.r.t. console interaction
  in general, especially in tutorial scenarios for UART input. Thus
  it's not the shell prompt, but the process console being enabled by
  default which is causing issues.

- Alex: one other option: build an app which can function as a process
  console replacement.

- Phil: if it's just printing information, that's one thing. However,
  it can control applications and fault the kernel. It would be tricky
  to have system calls which allow to fault processes or the kernel.

- Alex: yes, this is why I didn't try to develop this.

  Another discussion is that processes currently can exit, but there
  is no way to start processes.

- Leon: this discussion would likely take place after we've decided on
  how to move forward with application IDs, ACLs based on application
  IDs, etc. So it's going to be a while until we get to privileged
  system calls.

- Phil: we're talking about tutorials. Maybe for this it's sufficient
  to have two kernels, one having the process console and one not.

- Alex: need a configuration option for this. Not sure whether this is
  the best way to go.

- Hudson: the reason why we wanted this PRs is that people new to Tock
  commonly use upstream board definitions. Those users are not aware
  that the process console exists because there is no prompt. With the
  prompt, those same new users will instead be confused that a basic
  app such as hello will have additional messages interleaved with the
  app's output. If the solution is that to avoid the interleaving by
  disabling the process console and documenting this, the users are as
  unlikely to read that as it is to read the documentation that there
  is a process console.

- Leon: given that process console already badly messes with process
  input, I think it's fine for it to mess with output as well. I'd
  like to raise the question whether it's a good idea to have process
  console enabled by default in the upstream boards at all.

  We don't want the default board files to be incompatible with some
  of the example apps we're shipping.

- Hudson: consider them to be different. Console output is ubiquitous,
  whereas for input there are only very few apps utilizing that.

  Seems that there are three preferred approaches:
  - Leon prefers that process console is not included by default
  - Alex prefers the shell prompt as in the PR
  - Brad / I prefer that we don't include the prompt

- Phil: different boards can make different decisions on this issue.

- Leon: should different upstream boards (i.e. included in the
  mainline Tock repository) be able to make these different decisions?
  And should we stick to those decisions of the original board author?

  Given that Tock should try to abstract board details, I would be
  confused if two upstream boards behave entirely different for no
  apparent technical reason.

- Hudson: that's already the case today.

- Alex: we could add a string-slice argument to start, which would be
  the process console shell prompt. Some boards could decide to leave
  this empty.

- Phil: it seems there is no right answer here. Approved the PR.

- Hudson: yes. We should merge as is. If it seems problematic we can
  change it later.

## `mut_imut_buffer`

- Phil: PR by Alistair. Use case: cryptographic keys may be stored
  either in flash or in RAM. When passing down to lower layers, we
  wouldn't want to cast away the mutability from the `'static`
  reference, given we can never retrieve a mutable reference from an
  immutable one again.

- Brad: I like this PR. Don't like the name. Seems like a simple way
  to resolve the issue.

- Leon: For a name, I'd propose "AnnotatedMutability". I think we
  should remove the requirement for it to be a slice. We'd just have a
  generic container type for either mutable or immutable references of
  lifetime `'a`.

  This seems like very useful infrastructure. Was surprised to see
  that Rust does not have such a type in the core library.

- Pat: how are we the only people in Rust doing something like this,
  and are we doing something inherently wrong to require this. This
  feels core to passing around buffers.

- Phil: two explanations. On other systems, RAM is not as scarce so
  it's viable to do a copy. Furthermore, in a threaded execution model
  one can just pass an immutable reference of a mutable buffer. In
  Tock's asynchronous model we require the buffers to be held by the
  underlying callee.

- Leon: right. We're commonly doing static mutable borrows of static
  mutable buffers, this is not common in Rust but works quite well in
  our model.

- Phil: with these new types, there are more runtime failure cases. If
  there is an immutable key and calling generate on it.

- Leon: if mutating the buffer contents is required, one can simply
  take a proper mutable slice. What's not possible is to prevent the
  callee from modifying a mutable buffer, where it would normally only
  require an immutable slice.

- Phil: the only other solution appears to use a closure-based
  approach. One could have read-write key object and read-only key
  objects implementing a common trait. The trait would have a method
  which gives access to the key in a closure.

- Hudson: can't sit on top of a virtualizer, because the key access
  would be limited to the lifetime of the closure.

- Leon: also wouldn't be a particularly generic solution. In it's
  current form, this could even be used for passing immutable buffers
  down to e.g. UART for large debug strings stored in flash.

- Phil: right. If we introduce this type, perhaps there are many
  things which we would do differently.

  Going back to RSA, there is going to be some point where the key
  would need to be copied into RAM which the hardware can access. For
  example, for OTBN the key would need to be copied into the OTBN data
  RAM region.

  If the hardware requires a copy, a lot of these problems go
  away. Because then synchronous access is sufficient.

  Need to look at what hardware expects, whether synchronous accesses
  to the key material are sufficient. We should look into crypto
  implementations with existing boards.
