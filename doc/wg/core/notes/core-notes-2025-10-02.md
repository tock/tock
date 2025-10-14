# Tock Meeting Notes 2025-10-02

## Attendees

- Amit Levy
- Brad Campbell
- Branden Ghena
- Hudson Ayers
- Johnathan Van Why
- Leon Schuermann

## Updates

- Leon: Cargo.lock PR (#4605) is ready to merge, which might
  be one step towards the external dependencies discussion.
  Should merge fast because it needs to remain in sync with
  our build system.
- Amit: Is there a reason they're linked?
- Leon: Yes, because when we have external dependencies we
  will want to be able to control the version.
- Brad: I think the history is tied but they're standalone
  now.
- Leon: Sounds good.

## SingleThreadValue (#4519)

- Brad: Panic infrastructure has been copied everywhere and
  now it's a lot of work to update.
- Amit: The strategy shared in Matrix seems sound to me.
  We'll have two examples of doing this that will hopefully
  show that it is fairly mechanical -- or if not, figure out
  the non-mechanical stuff. Then realistically we can do it
  incrementally including soliciting help from outside the
  core WG.
- Brad: We already have examples, so I'm not sure why we
  need to wait for more examples. Both of simple things
  that use UART and a more complex one that uses the USB
  stack.
- Amit: I did not know that, I don't know where to look for
  it.
- Brad: That's why I brought it up two weeks ago. If we have
  `static mut` we can't update Rust. We can't do a release
  if it's half done, and I don't know how to track this.
- Amit: Previous plan was for Leon and Pat to do examples,
  then send out a call for help. If you don't think we need
  to wait on Leon and Pat, then perhaps you can write a
  porting guide then we can ask for help.
- Leon: The reason we suggested two follow-up boards, is
  this PR uses an older interface that has changed. Also, I
  don't think this has been tested yet and we want to make
  sure that panic handles work in all the way that you can
  reach them. Want to make sure there's no unexpected
  breakage.
- Brad: This PR uses the updated interface, it's both ARM
  and RISC-V, it could use more testing. If what we're
  missing is a porting guide then maybe that's what's
  needed.
- Leon: I can do the tests on a RISC-V platform to verify
  that works. Once I've done that and we're verified ARM
  works too, we can write the porting guide.
- Amit: Can I ask that you document the process as you're
  testing? Maybe start on the porting guide.
- Brad: I don't really see what's holding us up, I think we
  did a good job of designing SingleThreadValue.
- Hudson: It seems the steps are Leon tests on hardware,
  mark the PR as non-draft, merge it, then that PR can serve
  as a guide for the other boards.
- Amit: Leon, is that your understanding?
- Leon: Yes, I don't care whether that is this PR or a
  different PR.
- Amit: Leon is suggesting one PR that serves an an example
  of how to do the rest.
- Leon: Okay
- Hudson: And my understanding is the reason we don't submit
  it is that it's not tested.
- Leon: Might have to have a single large PR because the
  boards have to track some changes in the kernel crate.
- Amit: That would suck
- Brad: That's the reality
- Amit: Is the problem that we cannot do this incrementally?
- Brad: Yes
- Leon: I did not realize that
- Amit: Same
- Amit: Here's a slightly different suggestion. I suggest
  that Leon still does what he suggested, and I will
  dedicate that time to try to come up with a strategy to
  make this incremental, then write something down.
- Johnathan: Can we move the panic handler implementation
  into the kernel crate, perhaps via a macro?
- Leon: I've worked in on the Arty and QEMU, and there are
  many subtle differences in how they're implemented, so I
  don't think there's a simple way for us to split this out.
- Amit: Could maybe try to elide those differences, then
  restore them incrementally. But currently talking in the
  dark. Does it seem fine for Leon and I to follow that
  strategy?
- *General agreement*

## External dependencies (#4616 and #4589)

- Amit: Have two competing proposals for updating the
  external dependencies policy to support tock-registers
  being an external dependency. We should decide between
  them and move forward with one. Could each PR author make
  an argument for their PR?
- Brad: Mine is #4616. It does not change the policy, except
  to add a very explicit exception for tock-registers and
  explain why it is an exception.
- Leon: I rescind my proposal, I think Brad's is better.
  It's a subset of mine, and I don't have any issues with
  enumerating the dependencies that we make an exception
  for. I was proposing a catch-all rule, which I don't think
  we need.
- Branden: Given a sample size of one.
- Leon: Yeah. What this does not discuss is how the
  dependency is depended upon, but maybe that's deliberately
  explicitly excluded.
- Brad: What does "how" mean?
- Amit: git dependency vs crates.io vs submodules vs ...
- Hudson: I think that's fine
- Johnathan: I think that's an engineering concern not a
  policy concern.
- Brad: This is also coupled with Cargo.lock because cargo
  verifies the hash is correct.
- Leon: cargo is stronger than git hashes because cargo also
  considers transitive dependencies.
- Amit: I propose that Leon closes his PR and we approve and
  merge Brad's PR as-is.
- Hudson: Agreed. I already approved it.
- Leon: Closed my PR.
- Amit: Hitting the merge button.
