# Project Instructions

This project defines an operating system for embedded microcontrollers which is written in Rust.
All code should take into account that this is an OS for constrained systems, with tight limits on
code size and memory.

New files should attempt to follow the coding style established by existing examples of similar
functionality and should adhere to the rules in the [Tock Style Guide](./doc/Style.md).

## Human-Facing Communication (Issues, PRs, Comments)

Tock's [AI Policy](./.github/CONTRIBUTING.md#ai-policy) permits AI-assisted code, but not
AI-written prose addressed to humans. If you are an AI coding assistant working in this repository:

- Do not write or rewrite the prose of issue descriptions, pull request descriptions, PR/issue
  comments, or code review replies. That text must be the human contributor's own words.
  (Exception: tools used specifically for accessibility or translation.)
- Summarizing a diff, explaining reasoning, or listing technical points for the human to write up
  themselves is fine. Handing them finished, ready-to-paste prose is not -- that is the same
  violation with an extra step.
- Posting text the human wrote themselves (e.g. via `gh`) is fine. The restriction is on
  authorship, not on the mechanical act of posting.
- AI-generated content may appear in a PR only inside a collapsed
  [`<details>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/details)
  drawer, and only as supplement -- the PR must stand on its own without it.
- The PR description must disclose which AI tool was used, what portion of the patch it produced,
  and how that output was reviewed. You cannot write that disclosure, so state those three things
  plainly to the user and let them write it up.

## Commit Attribution

Commit messages are *not* covered by the restriction above; they may be AI-assisted, but must then
carry attribution as a trailer, e.g.:

    Co-Authored-By: Claude Sonnet 5 <noreply@anthropic.com>

Name the specific tool and model actually used, not a generic placeholder.
A human remains the commit's `Author`. AI is credited only via this trailer.
Preserve these trailers when squashing or rebasing before merge; squashing drops them by default.
This is in addition to, not a substitute for, the PR description's AI-use disclosure.

## Rust Code
- All rust code (except in `tools/`) is embedded Rust code and limited to use of the core library
  (e.g. `use core::cell:Cell`). The std library (e.g. use `std::x`) is not allowed.
- Tock does not allow dynamic allocation in the kernel. There is a limited mechanism for dynamic allocation
  available via the `Grant` mechanism, which is only available to capsules.
- Tock does not allow unwinding panics.
- Tock heavily discourages panicking -- Results should be used whenever possible to convey error states.
- Tock uses a nightly compiler, but does not allow any new unstable features.
- Conditional compilation and `#[cfg]` are heavily discouraged. These must
  be clearly motivated and documented, and are only permitted in specific cases. One of the main cases
  is to ensure that all crates build in CI, even during documentation and test builds.
- All `unsafe` usage MUST be accompanied by a comment starting with `### Safety`
  that discusses exactly why the unsafe code is necessary and what checks are
  needed and completed to ensure the use of `unsafe` does not trigger undefined
  behavior.
- All new exports from the core kernel crate must be carefully examined. Certain
  functionality is only safe within the core kernel. As essentially every crate in
  Tock uses `kernel` as a dependency, anything exported can be used broadly.
  Functionality which is sensitive but _must_ be exported must be guarded by a
  capability.
- Uses of `#inline` directives should explain in an adjacent comment why they
  are needed.

## Tock-specific restrictions
- New code in `capsules/`, `chips/`, and `libraries/` may not use `unsafe` *at all*.
- Capsule code MUST NOT issue a callback from within a downcall. Callbacks may only be called in response to
  an interrupt or a deferred call. If you find you need to issue a callback within a downcall (e.g. within the
  implementation of a command system call handler), schedule a deferred call which can issue the actual callback.
- New functionality which is both publicly exported and has invariants which
  cannot be enforced by the type system or other automated means (e.g., they
  provide access to sensitive core kernel data structures) should
  likely be guarded with a capability.

## HILs
- New HILs should follow the [TRD on HIL design](./doc/reference/trd3-hil-design.md).
- HILs should be well documented and not specifically matched to a single hardware platform.
- All valid errors should be enumerated.
- HIL naming should be reasonably consistent and clear.

## Syscall Drivers

Syscall drivers implement `SyscallDriver` to provide interfaces for userspace.

- These drivers must support potential calls from multiple processes. They do
  not need to be fully virtualized, e.g. a driver which rejects syscalls from
  all but the first process to access it is acceptable, but drivers must not
  break if multiple processes attempt access.
- They must return `CommandReturn::SUCCESS` for `command_id==0`.
- They should use the first argument to any upcalls as a ReturnCode.
- They should only provide an interface to userspace on top of some resource,
  and should not implement additional functionality which may also be useful
  within the kernel. The additional functionality should be a separate capsule.

## Virtualizers

Virtualizers multiplex an underlying resource for multiple users. They are primarily used
in capsules which provide a system call interface to userspace applications.

- The `Mux` struct should handle all interrupts, and route callbacks to specific
  virtualizer users.
- The virtualizer should provide the same interface (i.e. HIL) as it uses from
  the underlying shared resource.

## `static_init!()`
- `static_init!()`, `static_buf!()`, and similar must only be called from board crates.
- `static_init!()` should only be called within macros, or functions that are guaranteed to only ever be
  called once (e.g. `main()`). For the most part, it should be called either directly within main() or
  from the `xx_component_helper!()` class of macros in `boards/components`. `static_init!()` should not
  be called from within a component `finalize()` method.

## Dependencies
- Tock does not allow external dependencies. Do not add code which relies on external dependencies.

## Building code
- Test code by running `make -C boards/<my-board>` for a board which includes the code under test.
  This will call cargo under the hood, and resolves some issues with calling cargo from the top-level workspace.

## Linting code
- All code should pass rustfmt.
- Tock uses clippy for code linting, but only enforces a specific subset of clippy rules. Check that these pass
  by running `make clippy` from the top-level after making changes. Running clippy directly will use
  Clippy's default ruleset, and will fail on existing code.
- Do not silence warnings / errors about dead code using `#![allow(dead_code)]` unless there is a specific
  reason that the compiler is unable to detect that the code in question is actually being used. This
  is quite rare.
