# Tock Core Notes

## Tock 2.0

* Phil: First step towards releasing Tock 2.0 is [the
  alpha](https://github.com/tock/tock/pull/2446) (merging tock-2.0-dev
  into master). Once that is done, we can tackle the rest of the
  things on [the list](https://github.com/tock/tock/issues/2429) which
  Hudson curated.

* Amit: Can these be concurrent?

* Phil: Fundamentally have to be sequential: first merging
  tock-2.0-dev into master, then getting the release candidate
  ready. But we can already prepare the release candidate by doing the
  tasks of Hudson's checklist. There is an ordering to the tasks'
  completion, not necessarily an ordering to when we can work on them.

* Amit: What's the state of the alpha? How can people help, for
  example by more reviewing?

* Brad: We've been reviewing the pull request for the alpha. We need a
  lot more reviewing. It's a large PR, so split up the files and take
  things one at a time.

  More people looking at the changes and making sure we don't miss
  anything is a good thing. Now is the time to get things straightened
  out, such that we don't need to revise it afterwards a lot.

* Amit: We're primarily concerned with changes to the kernel and arch
  crates?

* Hudson: Yes. My approach for reviewing is just checking the "viewed"
  checkbox for all capsule changes, because they have all been
  reviewed as PRs to `tock-2.0-dev` already. It becomes much more
  manageable then.

* Phil: I think we can separate reviewing arch and kernel.

  Regarding the kernel, we're going to want to look at it both on a
  per-file basis, as well as whether the changes as a whole make sense
  and are clear.

* Leon: I found the changes of the `arch` crates to be specific to the
  ones done in kernel. Once the latter have been agreed upon, the
  changes in `arch` are very mechanical.

### System call filtering

* Phil: I'd like to bring up some questions regarding system call
  filtering.

  Currently we have system call filtering, with the only system call
  which we cannot filter is yield. It's clear we want to be able to
  filter _Command_, _Subscribe_ and _Allow_ calls. My question is, do
  we want to be able to filter calls to _Memop_ and _Exit_?

  This has particular implications for default-off behavior (so
  requiring the TBFs to explicitly enable these calls). If _Memop_ is
  always enabled anyways, then we shouldn't require it to be stated in
  TBFs explicitly.

* Johnathan: Also, I'd like to make the argument that _Exit_ should
  not be filtered, because it would be great to have userspace not
  require handling the case that a call to Exit returns (at least for
  _exit-terminate_ and _exit-restart_).

* Amit: I agree with Johnathan on _Exit_.

  It also seems to me that no call to _Memop_ should affect the rest
  of the system, just the individual processes. In practice, no app
  can run without Memop.

* Phil: `crt0` explicitly invokes _Memop_.

* Johnathan: I expect it to work on specific _Memop_ operations
  only. We need at least the _set break_ operation.

  If we constrain _Memop_ to only have per-process implications then I
  don't think we need to filter it.

* Leon: Is there any way one could abuse _Exit_ or _Memop_ to affect
  the rest of the system state? I was under the impression those calls
  had policies around them which made sure that they cannot cause the
  system to go into an unsafe state and apps would still be mutually
  distrustful.

* Brad: Right, _Memop_ really only exposes setting the break pointer,
  other operations are just querying information about yourself and
  the system, and setting debug information.

* Johnathan: It's really a question of future evolution, what
  operations could we be adding and might we want to add an operation
  which one could want to filter?

  If so, we'll want to leave the option open in the future to be
  adding filtering to _Memop_. We could also rule out those changes
  entirely. Maybe we can not add filtering now, but presumably
  implement it later.

* Amit: If we don't implement it now and add it later, and if we set
  those permissions to _default off_, it would break existing apps.

* Phil: I agree with _Exit_, and the fact that _Memop_ is invoked in
  `crt0` also indicates that we might want to not filter these calls.

  There are 4 _Memop_ operations which set things:
  - set `brk`
  - set `sbrk`
  - specify app stack
  - specify app heap

  We can define _Memop_ to only relate to the address space of the
  current process. Later, if we do need memory operations which relate
  to other memory regions, we would make it a new system call class.

  This allows us to make these impact constraints around _Memop_,
  while still allowing to in the future implement operations outside
  of these constraints.

* Brad: I agree to not filter _Exit_ and _Memop_ and just constrain
  _Memop_. It makes for a nicer filtering interface, by not requiring
  application developers to think about these other system calls. Just
  filtering _Command_, _Allow_, _Subscribe_ is a more intuitive
  filtering UI.

* Phil: It defines a clear line -- system calls implemented by the
  core kernel cannot be filtered, but calls to individual capsules can
  be filtered.

* Brad: IPC makes this distinction fuzzy, but in general I agree.

* Phil: Right. I was referring to calls handled by the scheduler
  loop. Technically IPC is part of the kernel, but it's implemented
  using the standard `Driver` interface as a peripheral driver.

* Brad: If we were to implement the [new system calls proposed by
  Alistair](https://github.com/tock/tock/pull/2381) and do that
  similar to IPC works today (essentially through an _Allow_), it's
  going to be the same thing.

### Callback -> Upcall

* Amit: Phil, you wanted discuss changing `Callback` to be `Upcall`?

* Phil: Yes. This is essentially just a name change. The motivation is
  that _callback_ is a generic term, and we use it both for calling
  functions in the kernel and for calling into userspace.

  It would be helpful to have a distinction such that _callbacks_ are
  internal to the kernel, whereas calls to userspace would be called
  `Upcall`s.

* Amit: Yes, we would use it to specifically refer to invocations from
  the kernel to userspace.

* Leon: I would've expected it to work the other way around. In [some
  discussions some time
  ago](https://github.com/tock/tock/issues/1736#issuecomment-610015515),
  there were some thoughts about requiring an "upcall capability",
  which was referring to calls from the hardware to upper layers in
  the kernel?

* Phil: Upcalls do specifically mean calls from the kernel to
  userspace. For example, signals in UNIX are upcalls.

* Leon: I see. If that's universally accepted terminology it's
  completely fine.

* Pat: Would that only be a userspace change or introduce an Upcall
  type in the kernel?

* Phil: It would mean renaming the `Callback` type (which encapsulates
  a function pointer and userdata) to `Upcall`. Everything else would
  still be called a _callback_.

* Brad: And we actually call the kernel-internal callbacks a _client_
  and _interrupts_.

* Pat: Is that true for deferred calls?

* Phil: Deferred calls are delayed invocations of something not on the
  current stack frame.

  We have a deferred procedure call mechanisms in Tock to be able to
  provide callbacks not on the same stack frame, for instance in error
  cases.

* Leon: But it uses the exact same mechanisms which are used for other
  callbacks.

* Amit: It's not necessarily distinguished. The thing a deferred call
  calls is a _callback_, which in our case is a function on a client
  trait.

* Brad: To your original question Phil, I think this is a good a
  change. One can define the _Subscribe_ system call to be
  specifically for registering an `upcall`. Its a straightforward
  terminology which can be explained easily.

* Phil: Right. In userspace we're still going to be referring to
  _callbacks_, because we do not necessarily want to leak whether a
  function invocation in userspace has been the result of an upcall
  from the kernel or not. Think of virtualizers in userspace.

* Brad: I agree. In userspace it all looks like regular callbacks.

* Phil: The only distinction is that we have a specific function
  signature for upcall handlers in userspace. We could introduce an
  additional typedef such as `subscribe_callback`.

* Brad: That sounds good.

### Fix unnecessary stack use

* Amit: Next up, [issue by
  Hudson](https://github.com/tock/tock/issues/2425). Everyone should
  read it first.

* Phil: Related to this, I updated the `print_tock_memory_usage` ([PR
  #2448](https://github.com/tock/tock/pull/2448)). If there are bugs
  or issues, don't hesitate to contact me.

* Hudson: The reason I wanted to talk about this issue is that there
  are several solutions to this issue, each with different
  tradeoffs. I wanted to get a survey of the four solutions I
  proposed, as well as the "components in each board crate" solution
  by Brad.

* Pat: Is there a substantial difference between your solution 1 and 4?

* Hudson: Slightly: for solution 4, we're moving everything into the
  separate function.

* Amit: Presumably, solution 4 is easier to write, but the stack would
  need to be at least as big as all of the structs we want to
  initialize. If that is bigger than the stack we'd need otherwise,
  that's not great.

* Hudson: Yes, but the advantage is that at the end of the board
  setup, nothing that is allocated remains on the stack. For the first
  one, everything allocated except the peripherals with `static_init!`
  still remains on the stack frame.

  I've verified that each of the solutions substantially reduces the
  stack size, and I've tested allocation of a large array which does
  not any longer cause a stack overflow.

* Amit: I'm surprised Rust scopes do not have the same effect as
  putting them in a separate function.

* Johnathan: LLVM does not understand scopes, and hence it does not
  recognize this.

* Phil: Could we compose option 1 and 4, so we're not forcing the
  stack to be the sum of all `static_init!` things, and we can push
  everything out from the root stack frame.

* Hudson: Yes. But the stack must still be as big as all of the
  peripherals.

* Amit: For option 3, was there a significant impact on code size? For
  4, there is a significant benefit, but for 3 I would expect there to
  be an impact.

* Phil: My take is, if we had to choose a single option, I would go
  with option 4. It reduces code size and there is no prolonged stack
  use. There is just a one-time diff that requires review.

  Also, within the call to `board_setup`, the board authors can try to
  better manage stack use by for example, decomposing peripherals,
  etc. It gives the board authors more flexibility and room for
  optimization.

* Brad: Hudson, do you have a sense of how big the peripheral structs
  are?

* Hudson: They make up the majority of the stack frame of the reset
  hander on the analyzed boards. For imix, it is around 1200
  bytes. For the stm32f4, it is about 2300.

* Brad: So this is still only around half of our usual stack size, so
  it isn't a problem.

* Branded: I'm wondering if there is a way to do option 4, without a
  big diff. Couldn't we cut off the last few lines of `reset_handler`
  and have what calls `reset_handler` also call some extra function?

  So we would essentially move the scheduler call into its own
  function.

* Hudson: Is `reset_handler` called from assembly?

* Brad: No, on Cortex-M platforms it's called by the hardware. On
  RISC-V it's called by assembly.

* Hudson: We might be able to do option 4 without a large diff. Git
  might recognize it as a function rename.

* Phil: I'm not worried about it being a big diff. It might be large,
  but it's a relatively straightforward change.

* Brad: Since we use components, do they avoid this issue? So that the
  stack frame doesn't grow to everything you are initializing?

* Hudson: I believe that is true. For the most part, components code
  is not inlined.

* Brad: What that means is that, if we do option 4, the new function
  would have a stack frame size approximately equivalent to the
  peripheral size.

* Hudson: Yes, a little bigger: there is a few other things which we
  `static_init!` directly in `main.rs` for several platforms and other
  local variables some of the time.

* Brad: It wouldn't be the sum of all things initialized therefore
  there would not be the need to break it up more, unless one were to
  optimize a board.

  So option 4 seems pretty compelling, what about my idea, making the
  peripheral instantiation a component so that it doesn't get added.

* Hudson: Downside is that every board needs a component crate inside
  the boards crate, or a per-chip component crate.

* Brad: And that reset handler stack frame is still there.

* Hudson: If the peripherals are initialized in a component, it's
  essentially just option 4. So long as we ensure that the component
  initialization is not inlined.

* Leon: Is component initialization not being inlined a guarantee or
  just the way Rust compiles it currently?

* Hudson: We could add an `#[inline(never)]` there, but currently we
  don't.

* Amit: This strikes me as a compiler regression. It's obvious that
  this is not what one wants to happen, and it probably affects other
  allocation mechanisms in Rust as well. This might (should) be fixed
  eventually.

  If that is the case, is moving to per-chip components something we
  will want to do anyways? Is that a move in the right direction? Or
  if Rust/LLVM would have a fix, would we go back to the old approach?

* Brad: We do have one chip-component crate for the nRFs. This is a
  legacy of the component switch-over and the difficulty of doing
  generic components. I don't have anything against using per-chip
  components, but it's probably not worth the overhead.

  I don't know of any other usecase for it, and we'd prefer to have
  generic components.

* Hudson: Yes. The main use case for components was to take the long
  initializations in `main.rs` and make them brief. Right now, it's
  already a one-line initialization, so adding a component won't help.

* Amit: Hudson, can you outline a way to test this?

  I feel like there is a solution which is not hacky.

* Hudson: Sure.

  The main way I've been testing this is adding the `-Z
  emit-stack-sizes` flag to `rustc`, and then using `llvm-readelf
  --stack-sizes` and `grep`ing for the stack frame of the reset
  handler.

  To validate this, I've been testing code with large allocations
  which would cause a stack overflow prior to these fixes, but works
  after them.

* Amit: Is it fundamentally the case that the fact that the reset
  handler does not return means that it's stack will not be popped?

  I suppose, cannot be popped currently, because the board kernel
  variable is at the bottom of the stack. In theory, if you needed
  nothing else on the stack, one could pop everything prior to calling
  the last function (as in a tail-call).

* Hudson: My understanding of LLVM is that this is not an optimization
  which LLVM makes.

  There have already been complaints when people tried initializing
  large arrays using `Box::new`, and the Rust developers did not have
  a solution.

* Phil: Is this true for GCC as well? GCC will dynamically increase
  the stack in scopes, but I don't know whether it will also retract
  it. I suppose it has to?

* Hudson: (From a StackOverflow post, might not be reliable
  information) GCC will, for variables that are allocated in a stack
  frame and no longer used, reuse this memory for later stack
  allocations.

* Amit: What is the takeaway? Does option 4 sound good?

* Brad, Hudson, Phil: Yes.


## Callback swap prevention PR #2445

[Link to PR](https://github.com/tock/tock/pull/2445)

* Leon: This PR prevents capsules from swapping `Callback`s. We
  already elaborated in previous calls that the kernel returning the
  exact callback information which was previously provided to the
  kernel/capsule are important semantics for userspace.

  In my opinion, by returning the previous `Callback / AppSlice`
  information as part of the ABI, we are emphasizing that a process
  can use them in a meaningful way. I think it only makes sense to be
  returning those if we can guarantee their correctness.

  This PR is one attempt of many to solve this issue, by implementing
  restrictions and tweaking the type system in ways such that the
  kernel is able check whether a capsule returns a `Callback` that was
  previously given to the capsule under this driver number, subdriver
  number and process id combination.

  One of the primary issues with this PR is that, as a process spawns,
  the kernel will call a hook in each capsule, replacing the generic
  callback placeholders with process-bound placeholders (being
  specific for a driver and subdriver number).  This implies that we
  only ever have one instance of the `Callback` structs per `(driver,
  subdriver, process)` combination.  However, as a side effect this
  makes `Grant`s useless given they are lazily allocated but the
  drivers will "use" each Grant immediately on process startup.

* Johnathan: I think it's important to prevent capsules from swapping
  callbacks between different processes. I don't think it's important
  to prevent them from swapping callbacks that all belong to the same
  process. As long as `Callback` is non-copyable, we can check a lot
  of these properties from userspace.

* Leon: Yes, I believe we can resolve these issues by just
  guaranteeing that we don't have actual data leakage between
  process. I suppose this is the minimum level of protection we need
  to give, with regards to the threat model.

  This level of protection should be achievable by a somewhat
  intrusive change, namely switching the Grant allocation to use an
  alternative `Default` trait which gets the `AppId` passed in. This
  allows creating process-bound callback instances which we can
  validate to match the process currently issuing the system call.

* Phil: This only matters for the initial callback, right? After that,
  all of the callbacks are generated from system calls from userspace.

* Leon: Yes. There is a subset of these guarantees (which were covered
  by an [earlier version of
  this](https://github.com/tock/tock/pull/2282)). This has the effect
  that a capsule can always return a _default_ (non process-bound)
  `Callback` instance, which the kernel will always trust. Because we
  can't limit the number of _default_ `Callbacks`, a capsule could
  essentially always return a _default_ `Callback` to userspace.

* Phil: Right. And we can't modify a _default_ `Callback` (ptr and
  appdata are both always zero), so there is no information
  leakage. We have a kernel-built and trusted struct which will be
  returned to userspace.

* Leon: Correct. This is a valid solution, but cannot entirely
  guarantee that the information returned to userspace is always
  correct. Why do we then even pass this information back?

* Johnathan: If a `subscribe` call returns a callback, and you only
  ever pass that `Callback` to the kernel once, you know that the
  capsule is never going to schedule this `Callback` again.

* Amit: Yes. Trust is not binary. For a large class of applications,
  if the guarantee that I have is that the callback I get back will
  never be scheduled again.

  The thing that we cannot guarantee is that a capsule may not cause a
  resource exhaustion because it will keep hold of the callbacks
  somewhere and not hand them back to processes.

* Leon: One way to make all of the guarantees of the [current
  PR](https://github.com/tock/tock/pull/2445) without having the Grant
  allocation issues could be to make the `process_init` call on
  capsules an explicit call from applications.

  What would be potential issues if we were to require processes to
  register themselves upon startup with the capsules they want to use?

* Amit: Does it have to be all at once?

* Leon: No, prior to capsule usage. The capsule would need to check
  whether it's already initialized, could require a non-lazily
  allocated `Grant` variant.

* Phil: So this would be a new system call?

* Leon: I suppose so, yes. It might not work, just a question of
  whether something like this could be feasible.

* Phil: It's pretty rough on a process. We could do it as part of the
  libraries though.

* Hudson: Leon, you feel pretty confident that we can't construct
  these process-bound instances as part of the lazy grant allocation
  mechanism?

* Leon: Yes. Because we don't know the number of Grants for a process
  and driver upfront, we can't track global state and ensure that we
  don't create duplicate `Callback`s per `(driver, subdriver,
  process)` combination.

  I think you have some interesting ideas there Hudson, I'll reach out
  to you.

* Phil: From a security standpoint, the critical thing is that a
  capsule cannot leak a `Callback` from process A to process B.

  All of the other guarantees are valuable, but there are a lot of
  ways to do that. In priority, those come after preventing the data
  leakage issue.
