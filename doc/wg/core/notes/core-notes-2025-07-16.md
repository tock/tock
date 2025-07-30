# Tock Core Call Notes 2025-07-16

# Updates

- Amit: Created GitHub sponsors page, going to the Foundation. Have a PR adding
  a `FUNDING` file to make those links visible on the repository sidebar.

- Leon: Network Working Group discussed IPC. First meeting, discussed a survey
  of other operating system's IPC mechanisms that Branden prepared: Hubris, Rust
  primitives to communicate between threads, and Redox. Valuable insights in the
  various different semantics of these interfaces, and their implications on the
  broader context and threat model of these systems.

  Also, Tyler gave a presentation about the existing Tock IPC interface and its
  shortcomings.

  Next step: reason about the various different interfaces that we've studied in
  the context of Tock.

# GH Merge Queue -- Octopus Merges

- Amit: The merge queue can get quite long, esp. if we merge a bunch of PRs at
  the same time. CI takes a while, hardware CI takes longest.

  When we were using Bors, we had octopus merges, where concurrent PR merges get
  combined into a single merge commit. Only triggers 1, as opposed to `n` CI
  actions.

  Major downside: if CI fails on a combined merge, then we'd need to disentangle
  that.

- Brad: When that happens, you could just merge the one that's not failing.

- Amit: Problem -- we don't know which one is failing.

- Brad: Many cases where we have a PR that's very simple, and another one that's
  very large and touches many files. Then we'd know.

- Johnathan: I'm having trouble finding documentation of this. Octopus merge is
  single merge commit with multiple merges at once. There's also merge limits on
  a merge queue.

- Leon: Insight primarily based on changes to the UI for configuring merge queue
  settings. Have not tested whether these changes result in actual Octopus
  merges being performed, or whether this works at all.

  When we switched to GH actions, they always had a way to run multiple entries
  in the queue concurrently. However, if one of those failed, then all
  subsequent ones would have to re-run, as the merge base then changes. Also,
  we'd still be running `n` CI jobs for `n` PRs merged. This has already been an
  issue with resource limits for GitHub's hosted runners; more pressing issue
  for hosted runners.

  Added a setting to the ruleset: not require every single PR the queue to
  succeed, but only require the new HEAD to pass CI. If this works, it would
  lower our CI workload a lot.

- Johnathan: Will look into the documentation on how this works.

- Leon: GH actions improved over bors on the merge UX, but otherwise
  stagnant. There are many other features we'd like for more seamless hardware
  CI, but there's no apparent development on those for years now.

- Amit: [screenshares GitHub settings UI] "When this setting is disabled, only
  the commit at the head of the merge group (i.e., the commit containing changes
  from all of the PRs of the group) must pass required checks."

- Johnathan: That seems reasonable to turn off.

- Leon: This seems to be a relatively new setting.

- Johnathan: Are you calling that Octopus merges?

- Amit: Leon and I assumed that this is using Octopus merges in the
  backend. Perhaps its actually performing them linearly, but the high bit is
  that it actually doesn't matter. The point is that it ensures that the new
  HEAD of the main branch passes CI.

- Leon: We're calling it that because bors did this underneath. And it's the
  better tool to use: when you do a bisect over just merge commits, you want
  every one to pass CI.

  The downside of enabling this without proper Octopus merges is that we might
  have commits in our history that are broken, which can be an issue for things
  like bisects. On the other hand, right now we're waiting an hour on CI.

- Johnathan: Necessary tradeoff for hardware CI.

- Amit: Does anybody disagree with disabling this setting? *no opposing votes*
  Is disabled.

# VCOM discovery in Tockloader

- Amit: What to do about non-free dependencies in Tockloader, and more generally
  in tools that we use.

  Tockloader has at least two dependencies that are non-free, which means they
  have a non-OSI-compliant license. One is SEGGER J-Link, the other is Nordic's
  `nrfjprog`.

  J-Link is non-problematic, as it's depending on that package to be installed
  separately. Tockloader just executes a binary. If somebody doesn't have that
  installed, Tockloader still works.

  The second case, Tockloader depends on `nrfjprog` as a direct library
  dependency, mandatory to be installed. Included in `pyproject.toml` as a
  non-optional dependency. If someone doesn't want to or can't install the
  dependency, they can't use Tockloader at all.

  In this particular case, it seems to be relatively trivial to replace the
  Python library with a binary tool that's maintained from Nordic.

  Two questions:
  - should we try to replace non-free library dependencies with external
    binaries?
  - should we disallow non-optional, non-free library dependencies more
    generally?

  Proposal is: yes, for both.

- Brad: What about QSPI support?

- Leon: `nrfutil`, the new tool, is supposed to be able to do all the things
  that `nrfjprog` can. Haven't checked yet. (Also, `nrfjprog` is deprecated at
  this point.)

  QSPI is interesting, because it's not a frequently used feature. If this
  needed a proprietary Python dependency, perhaps we could make the user install
  it after the fact?

- Brad: Is there hope for using an external tool for QSPI in the future?q

- Amit: Yes, there are tools. First of all, the `nrfjprog` binary works. For
  `nrfutil`, seems supported according to Nordic documentation.
  https://docs.nordicsemi.com/bundle/nrfutil/page/nrfutil-device/guides/programming_external_memory.html

- Brad: For the short term, the fix is: Leon prepares a patch for Tockloader to
  use `nrfutil`, the external binary.

- Amit: Seems like answer to the first question is yes: if possible, we should
  replace these dependencies. More tricky if something's only available as,
  e.g., a Python library.

- Brad: That case seems unlikely.

- Amit: Question #2: Should we avoid these proprietary dependencies in general?
  Not an issue right now, won't resolve.

# `libtock-c` Style Changes

- Amit: Brad wanted to raise two suggested changes to the style in `libtock-c`:
  1. Avoiding reexporting raw system call symbols from higher-level libraries.
     If apps wanted them, they'd need to import these directly.
  2. Better error handling.

- Brad: First one is the most stylistically changing. The original development
  doc / guide specified that you include the `syscalls.h` file and the
  `libtock.h` file. This would undo that, and say that generally, header files
  shouldn't include other header files. `.c` files should include everything
  they use.

  Applications have to include everything they use. Motivated from the
  `libtock-sync` split: with `yield-wait-for` it behooves us to have a clear
  indication that you're using both the synchronous version _and_ the
  asynchronous version. So if you're also using raw system calls, that should be
  visible as well.

  The complexity here is that we have a driver exists API for each system
  call. That was only in the system calls file. So, for this change, we'll have
  to duplicate this in the main library.

  Coupled with this: there's some sharing for `libtock` and `libtock-sync`,
  e.g., with enums (such as rising/falling edge triggers). Then we need a new
  header file (e.g., `_types`) that could be included by both.

  Thoughts? Changing a lot of drivers.

- Leon: We've always been saying that downstream users can instantiate drivers
  on non-standard driver numbers. We don't have these problems upstream, as we
  just use one instance of a driver to test it.

  Is there a point to be made that the low-level system call functions shouldn't
  know the driver number, and instead take it as an argument? Might be no, might
  be too big of a refactor. But would that be a cleaner abstraction?

- Brad: Definitely don't want to export that to the application. Could add
  another library like `libtock-syscalls` to generalize this interface, but
  seems like a lot of work.

- Amit: Refactoring header files seems trivially good.

- Amit: For the error handling around `allow`s, split on it. Conceptually
  reminiscent of Go's defer pattern. `goto`s triggers alarm bells. Both this and
  the alternative are easy to get wrong.

- Pat: Is it the case with the proposed change that you can include a peripheral
  header file, use exclusively a function from the header you included, and then
  get a compiler error? That shouldn't happen.

- Brad: No, it wouldn't happen.

- Pat: Because the C file underneath includes the required symbols. Seems more
  idiomatic in retrospect.

- Brad: And any mistakes there would be detected when compiling the underlying
  library.

  What about the error handling?

- Amit: Deep-link to the relevant file:
  https://github.com/tock/libtock-c/pull/532/files#diff-44e7e5e06b72107aee5761e0efd4c4b1c04a8920df52352939baf838ff57b6b6L21

- Pat: What is the specific problem we have?

- Brad: Basically, we want to try to require that `libtock-sync` looks like this.

- Amit: Problem is that we need to unallow things we've allowed, but the allows
  themselves may fail. So we want to only unallow things we've successfully
  allowed.

- Pat: Quintessential use of `goto` in Linux kernel code.

- Leon: Agree that this looks sensible. Have seen `goto` labels be more
  descriptive, to indicate exactly at which stage you failed.

- Amit: Like that the numbers implicitly add what looks like lexical scopes. If
  this was automatically generated from some syntax that's less bug-prone, this
  is exactly what it'd generated.

- Brad: A value in having not too much subjectivity in code. Should feel like
  its written by the same person.

- Pat: Don't know of any better approach. First suggestion is to have deeply
  nested if/else statements.

- Brad: Will continue down this path. HMAC is also one of the worst cases,
  usually just one or two allow buffers.
