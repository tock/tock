# Tock Core Notes 07/10/2020

Attending
- Hudson Ayers
- Brad Campbell
- Samuel Jero
- Garret Kelly
- Amit Levy
- Pat Pannuto
- Leon Schuermann
- Vadim Sukhomlinov
- Jonathan Van Why
- Alistair

## Updates

* Brad: [IPC issue](https://github.com/tock/tock/pull/1976) does seem
  to be a [stack overflow](https://github.com/tock/tock/pull/2003)
  issue. Revisited Hudson's long ago issue to catch the stack overflow
  issue with the hard fault handler. IPC is a good way to test this
  handling behaviour. The actual IPC issue is fixed too.

* Amit: Small fix in terms of code size, debugging is difficult as
  there is not a large semantic difference in Rust. Impressed with the
  fix.

---

* Jonathan: A new, entirely empty crate added to libtock-rs to prepare
  for future PRs. Interestingly, it changed application sizes, whereas
  it should be a no-op change. Likely affects link time optimization.

* Amit: One of my students has discovered that performance & size
  optimizations in Rust and LLVM in general are extremely
  unpredictable, so this observation agrees with what we're seeing in
  general. It may not necessarily be something worth pre-optimizing,
  as the problem could go away in the future.

* Jonathan: Could be further diagnosed with small programs, especially
  using the LLVM outliner. Can forward to OpenTitan and LLVM
  developers.

---

* Amit: Traffic in the project has gained large momentum, several
  people have started turning off notifications for libtock-{rs,c} and
  tock. GitHub notifications tend to increase by ~30 every few
  hours. How to surface things which matter outside of purely GitHub
  notifications? Example: libtock-rs contribution guidelines. Avoid
  things getting lost in the noise.

* Alistair: Thought about not using GitHub for sending patches? Using
  a Linux-style mailing list based workflow. GitHub's interface is not
  well suited, especially for a large amount of patches. It is hard to
  manage.

* Hudson: Unsure whether it solves this issue. Regardless of how
  patches come in, the notification count doesn't decrease.

* Leon: Primary issue is how GitHub organizes long and comment-heavy
  issues PRs and issues badly. Email is a more transparent system; all
  contained in one inbox which is looked at anyways.

* Alistair: Email has no notification problem, just scroll
  through. You can be CC'd on topics you're interested in.

* Leon: Another way could be kanban style GitHub projects, especially
  areas of focus.

---

* Leon: While the [kanban project tracking
  board](https://github.com/orgs/tock/projects/1) I've set up works
  well technically, I unfortunately don't have the resources to keep
  it current. I don't think it is very useful as not many people
  actively used it and therefore gained value from it. Proposal: put
  it on hold / delete it, at least temporarily.

## Multiple System Call Return Values

* Amit: Met up as a group to organize topics for Tock 2.0, clarifying
  which things need further discussion and how to approach that.

  Leon and I have done exploration regarding the tradeoffs of multiple
  return values from system calls. Generally options are to return 2
  values (as that can be cleanly represented in C) or 4 values (as
  this is how many registers we're giving up anyways for passing
  arguments). Exploration done regarding the overhead of returning 4
  values in userspace.

  Current results: Fairly trivial to do in Rust because of inlining
  and syntax. In C it is possible to do, but potentially having a very
  unergonomic interface, passing inputs and outputs using the same
  pointers.

  Preliminary conclusion: if there is no practical performance
  overhead, 4 return values is better than two; if there is overhead,
  2 is better than 1. Different thoughts?

* Leon: In C with 4 return values we can use the clean syscall
  interface, keeping the current semantics of the wrapper
  functions. Maximum overhead would be two memory copies + two words
  on the stack.

* Brad: I like 4 more than two.

* Alistair: How to follow this discussion?

* Amit: These results are very recent, so this is not yet written up,
  just discussed via Chat. I'm including Alistair in the future
  meetings. Hopefully we won't be too dependant on the meetings and
  work asynchronously.

* Alistair: Is there any reason more return values are not widespread
  amongst other OSes?

* Amit: Phil explored this. Likely because it is tricky in C to do
  with low overhead and on UNIX style OSes system calls are more
  granular. The system call can assume a particular memory layout or
  write to a userspace buffer directly. In Tock, the syscall interface
  is generic across all drivers.

* Garret: Probably historic reasons. Because x86 doesn't have many
  registers, the register usage must be minimized.

* Amit: On ARM return and argument registers are the same, whereas on
  x86 they are different.

* Vadim: Major difference between ARM and RISC-V is that the ARM ABI
  is defined that a struct with two parameters, like a slice, only the
  first parameter is returned on the register, others will be returned
  on stack. Whereas on RISC-V if a struct can be represented in two
  registers, it will not be written to the stack. Therefore on ARM to
  return two 32-bit parameters they need to be packed as a 64-bit
  integer, whereas on RISC-V it will automatically do that.

* Amit: The way the optimizations in C and Rust work is explicitly
  copying between registers and the target struct (or tuple) and
  relying on inlining to optimize that out. Worst case therefore is
  copying to some struct on the stack on ARM, on RISC-V maybe the ABI
  takes care of that already.

* Vadim: If you want to return on stack from the syscall you will need
  to perform copying the stack frame from the syscall stack to user
  stack. Must be very careful with that.

* Amit: Yes. The plan is not to use the stack at the system call
  interface; the semantics for the system call are the return values
  being in the registers. Question is how that is translated in
  userspace.

* Vadim: Major difference between ARM and RISC-V is how returning 64
  bit values is handled, probably best to use a common
  denominator. Difficult with x86.

* Amit: On x86 we have to behave differently, as the argument and
  return registers are different. On ARM and RISC-V this system call
  could potentially be optimized to not move data onto the stack.

* Vadim: Key is to avoid moving between registers a lot. Calling a
  function in C or Rust will use the C ABI, therefore using the same
  order also in the system calls will save instruction.

* Leon: Yes. Part of the reason why this is important is that the
  syscall wrapper won't be inlined even with aggressive optimization
  and LTO. Maybe this is because the assembly is opaque and the CPU
  might have been in either ARM or thumb mode, returning from the
  function automatically switches to the appropriate mode. If it were
  inlined we don't have this issue, because inside of a function, no
  specific ABI must be adhered.

* Amit: Inlining actually works by putting it into the header and
  adding inline keyword.

* Leon: Interesting that LTO doesn't do that. I will investigate
  further.

## Scheduler Trait PR / Structure of Kernel Internals

* Amit: Hudson opened a PR for a new scheduler trait. There is a lot
  of discussion there. A particular point worth discussing is that the
  overall code structure of the kernel crate w.r.t. modules and the
  scheduler does not make sense anymore and could be redone.

* Hudson: The basic idea is that boards can choose different
  schedulers rather than a single default scheduler. Because of the
  kernel architecture, the file previously named `sched.rs` should
  have been named `kernel.rs` as it holds most of the core
  functionality. Now that the schedulers are moved out of this file,
  `sched.rs` does not make sense anymore. Opportunity to think about
  general reorganization about the kernel and interface to platforms
  and chips. For example: the systick definition is in `platform`,
  whereas watchdog is in `hil`, but used in the core kernel.

* Amit: Certain structures are exposed outside the main repository,
  for instance where certain capsules live. Changing parts such as
  categorizing those into submodules would require porting by our
  boards, but also external boards. Wonder how liberally we can make
  changes to these top-level exports, for instance in PRs just because
  it makes sense to.

* Hudson: We have been doing this approach, but it might make
  maintaining out of tree boards harder. We have been even changed the
  names of top level exports, so this would break any board using
  those. Not infrequently we have to change every board to reflect
  kernel changes.

* Amit: That is true. Though capsules also interact with the kernel,
  but less strongly coupled.

* Hudson: Capsules exclusively interact through HILs. We have been
  more careful about making HILs stable than the platform or chip
  traits.

* Amit: Maybe we should just do it.

* Hudson: Jonathan, you maintain OpenTitan out of tree. Has it been
  difficult to keep that up to date with the changing interfaces?

* Jonathan: Only performed one upgrade, it was not been bad. Only
  libtock-rs keeps breaking, but Tock itself is fine.

* Hudson: We are not at a point where we should promise to keep the
  internal interfaces stable.

* Amit, Jonathan: Agree.

* Hudson: What should be a platform trait, what should be a HIL trait?

* Amit: Probably HIL traits relate to optional hardware where a board
  may not implement any of those, whereas platform traits must be
  implemented in order for Tock to work.

* Hudson: So the reason why `Watchdog` is in `hil` is that it is not
  required by the kernel.

* Amit: Yes. If Tock does not work without `Watchdog` implemented, it
  would become a platform trait.

* Garret: Counterargument is that `MPU` and `Systick` are in platform,
  but for a long time RISC-V had neither of those.

* Amit: Neither of those were implemented in a meaningful way, but it
  did use a no-op implementation of those. But yes, it is an arbitrary
  distinction. I am not willing to commit that this is 100% the
  case. For example, I am uncertain that we don't use the `Time` HIL
  in the kernel.

* Garret: Very certain that `Time` is not in the core kernel.

* Amit: This is an emerging distinction, and was not yet explicitly
  discussed.


## Compiling on Stable Rust

* Brad: Compiling on stable Rust is a goal we want to achieve. If we
  can, we should do it. Observation: we are adding nightly features
  more quickly than we are able to remove them. It seems like it is
  still a long way to remove all nightly features. It is difficult to
  guess when a nightly feature will land in stable Rust.

* Amit: Did not notice nightly features being added. Are there
  specific examples?

* Brad: One added for TBF header parsing without resorting to unsafe.

* Hudson: `option_result_contains`, being not necessary and also easy
  to remove.

* Samuel: One or two unstable features that we absolutely need to
  fundamentally to what we are doing, without a particular timeline
  for stabilizing.

* Hudson: Yes. A big blocker is the inline assembly. The RFC is
  merged, but the timeline from RFC merged to stable is unclear. Other
  blocker is `const_fn`, being relied upon by `OptionalCell`. Removing
  could require re-architecting large parts. Other can probably be
  worked around.

* Brad: What about associated type default.

* Samuel: `naked_functions` and `core_intrinsics` seem possible
  unlikely to ever stabilize.

* Amit: `core_instrinsics` can be possibly avoided by relying on
  `UnsafeCell`. `naked_functions` can be avoided by moving assembly to
  separate files.

* Jonathan: I think there is a global asm-macro than can be used to
  replace `naked_functions` that can replace global assembly without
  external assembly.

* Samuel: Then primarily depending on stabilization of inline
  assembly.

* Hudson: And `const_fn` to avoid replacing every use of
  `OptionalCell`.

* Samuel: Looks like it is going to be stabilized.

* Hudson: We could get around inline assembly by moving everything to
  external files, but that would not be a good solution.

* Brad: How are we going to implement deferred call for RISC-V without
  `core_instrinsics` and `AtomicUsize`? The issue says this is very
  difficult to get rid off.

  It seems like it was too early to commit to this issue. Our current
  mode of operation looks like liberally adding useful unstable
  features. We could go back to that.

* Hudson: Proposing a policy: every introduction of an unstable
  feature must also state how to replace it with a stable solution.

* Alistair: Or justify why it is required.

* Hudson: There are many cases where using nightly unstable features
  allows for a more ergonomic or efficient implementation, such as
  `option_result_contains` or a feature to allow const array
  initialization in components.

* Amit: I support that policy. Although I would push back on features
  like `option_result_contains`, as that could be replaced by a helper
  function easily.

* Hudson: It may not help to write unergonomic helper functions for
  things that are going to be stabilized since we are blocked on other
  features anyways.

* Amit: Agreed. I support this as a general rule. Adding an unstable
  feature should get pushback and require justification, which should
  include a stable workaround and/or a justification on why it may
  block on going to stable.

* Brad: Seems reasonable. Hard to objectively argue whether the
  feature is important.

