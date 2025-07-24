# Tock Pull Request Process

## Abstract

This document describes how the Tock [core working group](./wg/core/README.md)
merges pull requests for the main Tock repository.

<!-- npm i -g markdown-toc; markdown-toc -i CodeReview.md -->

<!-- toc -->

- [Introduction](#introduction)
- [Pull Requests](#pull-requests)
  * [Significant pull requests](#significant-pull-requests)
  * [Upkeep pull requests](#upkeep-pull-requests)
- [Continuous Integration](#continuous-integration)
  * [CI Organization](#ci-organization)
    + [The short answer: `make prepush`](#the-short-answer-make-prepush)
    + [The complete CI setup](#the-complete-ci-setup)
      - [`ci-job-*`](#ci-job-)
      - [`ci-setup-*`](#ci-setup-)
      - [`ci-runner-*[-*]`](#ci-runner--)
      - [`ci-all`](#ci-all)
- [Comments and Review Criteria](#comments-and-review-criteria)
  * [General Review Principles](#general-review-principles)
  * [Review Guide by Repository Subsystem](#review-guide-by-repository-subsystem)
    + [Core Kernel (`/kernel` crate) Not Including HILs](#core-kernel-kernel-crate-not-including-hils)
    + [HILs](#hils)
    + [Capsules](#capsules)
    + [Chips](#chips)
    + [Boards](#boards)
    + [Arch](#arch)
    + [Libraries](#libraries)
- [Reviews](#reviews)
- [Other Tock Repositories](#other-tock-repositories)
  * [Userland Repositories](#userland-repositories)
  * [Tertiary Repositories](#tertiary-repositories)

<!-- tocstop -->

## Introduction

As Tock supports more chips and services, changes to core interfaces or
capsules will increasingly trigger bugs or integration problems. This
document describes the process by which pull requests for the main
Tock repository are handled. This process is not set in stone, and may
change as problems or issues arise.

Active development occurs on the master branch. Periodic releases (discussed
more below) are made on branches.

## Pull Requests

Any pull request against the master branch is reviewed by the core Tock
team. Pull requests fall into two general categories:

1. **Upkeep pull requests** involve minor changes to existing implementations.
   Examples of upkeep requests involve bug fixes, documentation (that isn't
   specification), or minor reimplementations of existing modules.
1. **Significant pull requests** involve new modules, significant
   re-implementations, new traits, new kernel components, or changes to the
   build system.

Whether a pull request is upkeep or significant is based not only on the
magnitude of the change but also what sort of code is changed. For example,
bug fixes that are considered upkeep for a non-critical capsule might be
considered significant for kernel code, because the kernel code affects
everything and has more potential edge cases.

The core team decides whether a pull request is upkeep or significant. The
first person to look at the pull request can decide, or defer based on
other core member feedback. Pull requests by a member of the core team need
to be reviewed by a different member of the core team. If a team member
decides that a pull request is significant but another team member decided
it was upkeep and merged it, then the merging team member is responsible for
backing out the merge and resolving the discussion. Any team member
can decide that a pull request is significant. The assumption is that the
core team will have good consensus on the boundary between upkeep vs.
significant, but that specialized knowledge means that some team members will
see implications that others may not.

Core team members can change their votes at any time, based on discussion,
changes, or further thought.

### Significant pull requests

These PRs require review by the entire core team. Each
core team member is expected to respond within one week. There are three
possible responses:

  - **Accept**, which means the pull request should be accepted (perhaps
    with some minor tweaks, as per comments).
  - **No Comment**, which means the pull request is fine but the member
    does not promote it.
  - **Discuss**, which means the pull request needs to be discussed by the
    core team before considering merging it.

Significant pull requests are relatively uncommon, however, they will generally
require extensive discussion and several revisions from the pull request author
or other contributors. Significant PRs are often discussed during working group
meetings, with feedback captured in meeting notes and commonly summarized in a
comment on the PR thread. While the goal is for all significant PRs to receive a
full core team review, sometimes members are unavailable for extended periods,
in which case a subset of the core team may elect to merge a significant PR.

### Upkeep pull requests

These pull requests are more informally handled compared to significant pull
requests. Commonly, upkeep pull requests fall into one of three groups:

1. Very minor: these PRs fix typos, improve a comment, fix formatting, fix a
   clippy lint, or make other inconsequential changes that do not affect the
   code or are trivial to reason about.
2. Minor: these PRs edit only a handful of lines of code to improve clarity, fix
   an obvious bug, add functionality to a Tier 2 or 3 board, update the version
   of a tool, add a component, add a test, or make other changes that are
   straightforward to understand and reason about, do not make changes to the
   kernel crate or depend on any specific hardware functionality, and are
   limited in the scope of Tock code that is impacted.
3. Moderate: these PRs add a new capsule, make minor modifications to the kernel
   crate, modify a Tier 1 board, implement a HIL for a chip, modify existing
   chip code based on how the hardware functions, modify the logic for an
   intricate capsule, change a `SyscallDriver` interface, or make other changes
   that could have impact to many Tock subsystems or users, require expertise in
   a particular platform or subsystem to review, or add a moderate amount of new
   code to Tock.

These groups are not generally explicitly distinguished but instead are
understood by core team members based on experience and discussion.

Generally, very minor PRs will be handled and merged by the first core team
member that reviews the PR. These are typically merged immediately to avoid
unnecessary time spent reviewing very minor changes.

Generally, minor PRs will be reviewed by two core team members, and if neither
reviewer has any comments the second reviewer will merge the PR.

Generally, moderate PRs will reviewed by at least two core team members and left
open for several business days to leave time for any core team member to raise
any potential issues. In the case that a moderate PR modifies specific
subsystems, the PR will likely be left open until the core team member most
familiar with that subsystem is able to review the PR. After two approval
reviews with all comments addressed, the PR will often be tagged "last call",
starting a roughly 24 hour clock for any last-chance reviews. If there are no
additional comments the PR will generally be merged the next business day.

All upkeep PRs can be merged by any member of the core team. That
person is responsible for the merge and backing out the merge if needed.

## Continuous Integration

Tock leans heavily on automated integration testing and as a project is
generally willing to explore new and novel means of testing for hardware
reliability.

With exceptions for drafts or works-in-progress, generally it is expected that
a pull request pass the full continuous integration (CI) suite before core team
members will perform an in-depth review.

One frequent challenge with CI setups is replicating failures in local
development environments. Tock goes to great lengths to mitigate this as much
as possible. Within reason, the inability to replicate a CI test in a local
development environment shall be considered a bug (however, it is reasonable
that local CI requires the install of non-trivial tooling, so long as there is
a well-documented, reliable path to set up the tooling locally).

### CI Organization

All CI is driven by `make` rules.

Generally, there are a series of fine-grained targets that do the actual tests,
and then a meta layer of rules that are invoked depending on context.

#### The short answer: `make prepush`
This is a meta-target that runs what Tock considers the "standard developer CI".
This is the rule that should be run locally before submitting PRs.
It runs the quicker jobs that catch the majority of small errors.
Developers are encouraged to consider wiring this to the
[git pre-push hook](https://git-scm.com/book/en/v2/Customizing-Git-Git-Hooks)
to run automatically when pushing upstream.

#### The complete CI setup

All CI is required to support the possibility of parallel make invocation (i.e.
`make -jN`), but is not required to handle multiple independent make processes.

##### `ci-job-*`
To the extent reasonable, individual tests are broken into small atomic units.
All actionable `make` recipes that run tests **must** be in `ci-job-*` rules.
These perform individual, logical actions related only to testing.
These rules **must not** perform any one-off or setup operations.
No automated tooling should invoke job rules directly.
If a CI check fails, developers should be able to run the failed `ci-job-*`
locally, although in certain cases this may require installing supporting
tooling.

##### `ci-setup-*`
These are rules that run any required setup for jobs to succeed.
They may install arbitrary packages or do any other significant labor.
Many jobs may rely on the same setup target.
To the extent possible, setup targets should cache their results to avoid
re-execution.
Setup targets **should** handle upgrades automatically; this may include
automatically clearing caches or other artifacts if needed.
Setup targets **may** handle downgrades, but developers working on experimental
branches may be required to handle these cases manually.
Setup targets are permitted to expect "total ownership" of the directories they
create and manage.

Setup rules may vary between runner and local environments, as they may perform
automatic and possibly invasive (e.g. apt install) operations on runners.

When run locally, setup targets **must** prompt users prior to system-wide
persistent changes.
These prompts **should** be rare, as example, asking the user to install
system-wide development packages needed for a build.
These prompts **must not** generate on every invocation of the setup rule;
that is, setup rules **must** first check if the install has already been
completed and not prompt the user in that case.
If an update or upgrade is required, setup targets **must** prompt before
installing.

##### `ci-runner-*[-*]`
These are targets like `ci-runner-netlify` and `ci-runner-github`.
They represent exactly what is run by various CI runners.
For platform with multiple CI rules, like GitHub, the `ci-runner-github` is a
meta target that runs all GitHub checks, while `ci-runner-github-*` are the
rules that match the individual runners.
These targets **must** execute correctly on a local development environment.
Small deviations in behavior between the runner and local execution are
permitted if needed, but should be kept to a minimum.

##### `ci-all`
A meta target that runs every piece of CI possible.
If this passes locally, all upstream CI checks should pass.


## Comments and Review Criteria

Most pull requests will receive comments and reviews via Github from core team
members and other interested parties. Likely, only trivial pull requests (e.g.
spelling/formatting fixes, test fixes, or documentation/comment updates) will be
merged without discussion.

The types and detail of the comments will vary based on the type of change and
the location within the repository of the changes. Changes marked "significant"
will receive a more thorough review. Similarly, changes to code that is used
widely (e.g. changes within the core `kernel` crate) will be more scrutinized.

To help pull request reviewers and pull request submitters alike, we document
review principles that will be used when evaluating pull requests.

### General Review Principles

**PR Mechanics**

- Is this a self-contained change, or should portions of the pull request be
  split into separate PRs? Note, this is not evaluated by number of files or
  changed lines, but rather by the semantic meaning of the changes.
- Are the commits all relevant to the change, or are there possibly unrelated
  branches that are unintentionally included?
- Does the PR provide enough explanation to help reviewers understand its
  purpose, and to explain the change for future readers of the code? Does the PR
  link to relevant tracking issues or other discussions? Are there existing
  discussions that should be referenced?

**Documentation and Comments**

- Many core designs of Tock are documented in specific markdown files. Does this
  PR change any of those designs/details, and are the corresponding documents
  updated?
- Does the change include proper rustdoc comments for new files and new data
  structures?
- If the change is user-facing (i.e. part of a public API or command line
  interface) does it include enough helpful information for users to understand
  how to use it?
- If documentation is removed, is it clear that it is no longer accurate or
  needed? Or is there justification for removing the documentation?

**Code**

- Is `unsafe` used? If so, the changes in this PR need to be carefully
  evaluated. The purpose of `unsafe` must be documented and why it is used
  correctly must be explained in a comment.
- Is `unsafe` used when code is not actually memory or type unsafe (i.e. does
  not violate the Rust safety model)? This should not be marked `unsafe` and
  instead should likely be marked safe or use a capability.
- Does the code use interrupts and callbacks? If so, the code MUST NOT issue a
  callback from a downcall. The callback may ONLY be called in response to an
  interrupt. Using deferred calls is often necessary to remedy this.
- Are Rust features (i.e. conditional compilation and `#[cfg]`) used? These must
  be clearly motivated and documented, and are only permitted in specific cases.
- Are `lib.rs` or `mod.rs` files added? In general these should only be used to
  reference other modules and setup exports. Actual OS logic should be in
  descriptively named files.
- `static_init!()` (and similar) must only be called from board crates.
- Is any new functionality both publicly exported and have invariants which
  cannot be enforced by the type system or other automated means (e.g., they
  provide access to sensitive core kernel data structures)? If so, this should
  likely be guarded with a capability.
- Uses of `#inline` directives should explain in an adjacent comment why they
  are needed.

### Review Guide by Repository Subsystem

In addition to general code review practices, certain review principles are only
applicable in specific portions of the tock kernel repository.

#### Core Kernel (`/kernel` crate) Not Including HILs

In general, any substantial changes to the kernel crate should be accompanied
first by a discussion issue. This permits discussion on if the change should be
included in the Tock kernel and if so how it should be implemented.

All substantial changes should be clearly documented. New files should use `//!`
comments to explain their purpose, and all functions and data structures should
be clearly documented with `///` comments. Often additional documentation in
discrete markdown files is required as well.

Additionally, particularly subtle or extensively discussed rationale should be
included in the source file directly (often with a `//` comment). This leaves a
clear trace of how key design decisions in Tock were decided and why certain
aspects may not use the most intuitive design. This helps avoid re-hashing
discussions and assist new users with understanding the kernel.

All `unsafe` usage MUST be accompanied by a comment starting with `### Safety`
that discusses exactly why the unsafe code is necessary and what checks are
needed and completed to ensure the use of `unsafe` does not trigger undefined
behavior.

All new exports from the core kernel crate must be carefully examined. Certain
functionality is only safe within the core kernel. As essentially every crate in
Tock uses `kernel` as a dependency, anything exported can be used broadly.
Functionality which is sensitive but _must_ be exported must be guarded by a
capability.

#### HILs

New HILs should follow the [TRD on HIL design](./reference/trd3-hil-design.md).

HILs should be well documented and not specifically matched to a single hardware
platform.

All valid errors should be enumerated.

HIL naming should be reasonably consistent and clear.

#### Capsules

Capsules should explain what they do in comments but do not need to be
rigorously commented.

**Virtualizers**

Virtualizers multiplex an underlying resource for multiple users.

- The `Mux` struct should handle all interrupts, and route callbacks to specific
  virtualizer users.
- The virtualizer should provide the same interface (i.e. HIL) as it uses from
  the underlying shared resource.

**Syscall Drivers**

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

#### Chips

Often changes within chip crates are difficult to test as not many reviewers may
have the specific hardware. Review comments often rely on visual inspection of
the code.

Files in a chip crate should avoid giving the impression of functionality which
is not actually implemented. This means avoiding peripheral files which only
contain registers or return `ErrorCode::NOSUPPORT` for all methods. A peripheral
must implement at least basic functionality to be merged in mainline Tock.

Chip crates should be properly named. Many chips use nested crates to represent
families of chips and to share implementations.

Rust `cfg` features should be avoided. However, in circumstances where different
chips differ in very small ways, and those differences are well understand and
likely documented in a datashet, chip-variant configs may be used. They should
be contained to a single file (i.e. not scattered throughout the crate). It
should be entirely unambiguous whether a feature is set or not (i.e. it should
be based on physical hardware where it is obvious which chip a user has).
Generally, this means `cfg` directives should be an explicit list of chips or'd
together. Rarely, if ever, is a `cfg(not ...)` the correct approach for anything
outside of unit tests.

#### Boards

Changes to boards are generally left to the maintainer or original contributor
of the board. Generally boards are thought of as examples or starting points,
and may vary in terms of what functionality is exposed.

New boards should explain how someone can get the hardware and how to get
started running Tock and applications.

#### Arch

Changes to the architecture crate are somewhat uncommon.

Any assembly should be clearly documented and explained why it is needed to be
in assembly.

#### Libraries

The libraries folder in the tock repo contains which is used by the Tock kernel
but is also logically distinct from the kernel and could be used outside of
Tock.

If code is added to libraries from other sources it should be clearly
attributed.

Changes to libraries may affect other (i.e. non-Tock) projects. Certain changes
may require discussions on how to include them without breaking downstream
users.

## Reviews

To be merged, a pull request requires two Accept and no Discuss votes. The
review period begins when a review is requested from the Github team
`core-team`. If a member does not respond within a week, their vote is
considered No Comment. If a core team member stops responding to many
significant pull requests they may be removed from the core team.

Core team members enter their votes through GitHub's comment system. An
"Approve" is considered an Accept vote, a "Comment" is considered a "No
Comment" vote and a "Request Changes" is considered a "Discuss". If, after
discussion, non-trivial changes are necessary for the pull request, the review
window is re-started after the changes are made.

## Other Tock Repositories

This document covers the procedure of the core Tock repository
([tock/tock](https://github.com/tock/tock)). However, there are several other
repositories that are part of the greater Tock project.

### Userland Repositories

Tock has two userland environments that are heavily developed and supported:

 - [tock/libtock-c](https://github.com/tock/libtock-c) The C/C++ runtime was
   the first runtime developed. It is fairly stable at this point and sees
   primarily maintenance support as needed. Its development process follows
   the main tock repository, with the same core team.
 - [tock/libtock-rs](https://github.com/tock/libtock-rs) The Rust runtime is an
   active work-in-progress. While basic application scenarios work, there are
   still major architectural changes coming as it converges. Thus, the Rust
   runtime follows a slightly less formal model to allow it to move faster.
   Primary owners of the Rust runtime are:
    - @alevy
    - @Woyten
    - @torfmaster
    - @jrvanwhy

   However the Tock core working group reserves the right to make final
   authoritative decisions if need merits.

### Tertiary Repositories

Tock has several additional smaller support repositories. These generally do
not have any formal contribution guidelines beyond pull requests and approval
from the primary maintainer(s). Any member of the core working group can merge
PRs in these repositories, however, generally things are deferred to the owner
of the component.

 - [tock/book](https://github.com/tock/book) Getting start guide and tutorials
   for Tock.
   Primarily maintained by @alevy and @bradjc (Dec 2019).
 - [tock/elf2tab](https://github.com/tock/elf2tab) Tool to convert apps from
   `.elf` to Tock Application Bundles aka `.tab`s.
   Primarily maintained by @bradjc (Dec 2019).
 - [tock/tockloader](https://github.com/tock/tockloader) Tool for loading Tock
   kernel and applications onto hardware boards.
   Primarily maintained by @bradjc (Dec 2019).
 - [tock/tock-archive](https://github.com/tock/tock-archive) Components of Tock
   (often hardware platforms) no longer under active development.
   Maintained by the core working group (Dec 2019).
 - [tock/tock-bootloader](https://github.com/tock/tock-bootloader) Utility for
   flashing apps via USB; works with tockloader.
   Primarily maintained by @bradjc (Dec 2019).
 - [tock/tock-www](https://github.com/tock/tock-www) The tockos.org website.
   Primarily maintained by @alevy and @ppannuto (Dec 2019).

Other repositories under [tock/](https://github.com/tock) are either
experimental or archived.
