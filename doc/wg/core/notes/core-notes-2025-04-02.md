# Tock Meeting Notes 2025-04-02

## Attendees
 - Branden Ghena
 - Leon Schuermann
 - Johnathan Van Why
 - Brad Campbell
 - Alexandru Radovici
 - Kat Fox
 - Hudson Ayers
 - Viswajith Govinda Rajan
 - Benjamin Prevor

## Updates
 * None!

## Tock x86 Support
* Amit: PR opened (https://github.com/tock/tock/pull/4385) that picks up work
  by Pluton team at Microsoft that opened PR for x86 support, but with external
  dependency removed (which was the main blocker at the time)
* Amit: What should our bar be for merging this PR? Do we want to be blocked or
  dependent on the Microsoft folks signing off pre-merge?
* Branden: Is this a replacement or an addendum to the Microsoft PR?
* Alex: This PR takes the other PR and then just adds commits on top of it, to
  remove the dependency and to fix clippy warnings
* Leon: One downside here is losing the connection to all of the discussion
  that happened on the previous PR. Could we just push those changes to the
  other PRs branch?
* Alex: We merged main into the PR — its not a totally linear history
* Brad: What are the comments? The only comment I see is a thing about
  components
* Branden: I do not see any significant comments on the Microsoft PR, I think
  it might be better to just close that PR
* Leon: I think there must have been comments on old commits but the PR has
  been rebased and those have been lost. Just confusion on my end
* Amit: Of course we will also back-reference the original PR from the new PR
* Amit: If it passes CI and doesn’t break anything else, is that all we need
  here? Or should we wait for Bobby’s team at Microsoft to verify they
  could/would switch to this branch tomorrow
* Alex: Depends how long we think that will take
* Alex: One other thing we need is some way to verify applications run, we do
  not have a libtock-c for x86. Microsoft has this but it is not upstream. But
  I do not want to wait 3 months for this
* Branden: Yeah seems we need to talk to Bobby and get some estimate of timing
  to decide
* Leon: Given Bobby commented they were doing internal validation just 17 hours
  ago, lets give them a few days at least
* Branden: If anyone is aware of documentation should be updated, we should do
  that. We might even want to look at the Tock book and see if it needs
  updating. Also, is there anything in arch/ that really needs the core teams
  eyes? Asking for those with experience from the RISC-V port.
* Leon: I think Rust’s crate-based isolation means we do not have to worry too
  much about anything that is wrong affecting non x86 builds.
* Amit: How do we have confidence in the virtual memory implementation?
* Alex: Really hard for us to test without apps. We could always make a simple
  app ourselves. Microsoft giving us a test binary would be sufficient 
* Hudson: Does Microsoft have a libtock-c or libtock-rs implementation?
* Amit: I know Microsoft has a libtock-c implementation, and a libtock-rs
  implementation in the works
* Alex: Once we merge this, we can pretty easily get a QEMU setup working and
  tests working
* Amit: Yeah I have run this several times, and it is refreshing how easy it is
  tooling-wise

## LiteX CI Failures and Compiler Bugs
* Leon: We used to have spurious LiteX CI failures in the past, right around
  when GitHub deprecated the 22.04 toolchain. Then we made it non-required. Now
  it consistently fails, presumably not because of the former intermittent issue,
  but a new one
* Leon: We have a hard fault in our panic handler because in the depths of
  Rust’s core formatting infrastructure there is an OOB dereference
* Leon: I think we are dealing with a miscompute in the LLVM passes that are
  being performed when targeting RISC-V
* Leon: Instead of loading the size of a string into a register, we are loading
  a function pointer into that register, and then interpreting it as the size
  of the string
* Leon: Brad has explored a few different options with different PRs and
  different nightly versions. It exhibits in some scenarios but not others,
  which to me points to miscompile
* Leon: This is not LiteX specific, this also panics in QEMU and also
  opentitan, so high priority
* Amit: It would be great for CI to pass. Bumping Rust nightly seems to work,
  and if this is indeed a compiler bug, then that seems like a fine fix
* Amit: What gives me pause is this behavior is also consistent with a correct
  compiler taking advantage of some UB in our code
* Leon: Also, the new compiler could have not fixed the compiler bug and we
  just happen to not be hitting it due to other changes in the compiler pass
  order or similar. So this could be a ticking time bomb
* Amit: If a compiler bug it would be great for us to track it down and help
  fix upstream, but that is orthogonal. My bigger concern is this is just UB in
  Tock.
* Hudson: Did the LLVM compiler change between the two Rust versions?
* Leon: I haven’t looked yet. I did look at the MIR in the buggy version and it
  looks right.
* Brad: This is related to the RISC-V mcause PR, in that if we revert that PR
  the issue no longer happens. Thoughts?
* Amit: That does not give me enough information
* Leon: The function pointer we are loading is the function pointer to the
  print_mcause function. So I am not surprised that change triggers this bug
  because it is adjacent to where we are seeing it. Could be UB in our
  tock-registers RISC-V CSR implementation, or not.
* Brad (chat): nightly where this happens: nightly-2025-02-19, nightly where
  this doesn't happen: nightly-2025-03-14, nightly where this doesn't happen:
  nightly-2024-11-16
* Amit: The bug is manifesting in the formatting of a panic string. The test
  that is failing is intentionally reading/writing outside the MPU memory
  regions, and expecting to read a particular panic message from the panic
  handler. The kernel gets stuck halfway through one of those panic messages.
* Amit: I think we should note down all of this info needed to reproduce this —
  is it deterministic?
* Leon: Yes, unless you make changes within that function. It seems stable
  across other kernel changes
* Amit: Once we have that, we update rust nightly with this PR so we can move
  forward. I think we just do not want to lose keeping track of whether this is
  a Tock bug or compiler bug
* Brad: On the topic of updating nightly, I sent something on Slack we need to
  resolve. There are some linking failures on newer nightlies
* Alex: Have we tried panic_immediate_abort?
* Brad: No
* Amit: This may work itself out, this seems similar to some bugs from the
  past. If not let’s talk with upstream.

## Dynamic Process Loading PR
* Leon: I have outstanding review comments that I have not submitted, I can
  voice some of it now. This is great infrastructure, but also really
  complicated. This warrants a close look because it touches so much in the
  kernel crate. After awhile leaving the code I re*read the TRD and then left
  some comments on that. I found some small issues in the implementation because
  the capsule for interacting with processes is not virtualized, and a process
  crashing could cause issues. 
* Leon: I do not think this is ready to merge today, for that reason.
* Brad: This only changes two files in the kernel crate, and one of those is
  new. The integration with the kernel is just changing the API for the
  sequential process loader — it is really an add*on in my opinion.
* Leon: I am just worried that once this is merged it will not be looked at
  again, and I want to carefully consider how the sequential process loader is
  used. The use of static mutable slices for process memory seems more concerning
  now that we will always have the process loader running in the background --
  this is straightforwardly UB. For that and other reasons I think this needs a
  very close look.
* Brad: I do want to point out this has been an open PR for a year. Don’t
  disagree with anything you said.

## Tockloader Release
* Ben: Treadmill fetches tockloader from pypi and we need the new openocd flag
  to work with multiple of the same board. So we need a new tockloader release.
* Brad: I will try to get a sense if there is anything in particular I need to
  test. Likely we can just do a release.
