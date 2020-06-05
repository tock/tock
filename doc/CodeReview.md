# Tock Pull Request Process

## Abstract

This document describes how the Tock [core working group](../wg/core.md) merges
pull requests for and makes releases of the main Tock repository.

<!-- npm i -g markdown-toc; markdown-toc -i Abstract.md -->

<!-- toc -->

- [1. Introduction](#1-introduction)
- [2. Pull Requests](#2-pull-requests)
- [3. Continuous Integration](#3-continuous-integration)
  * [CI Organization](#ci-organization)
    + [The short answer: `make prepush`](#the-short-answer-make-prepush)
    + [The complete CI setup](#the-complete-ci-setup)
      - [`ci-job-*`](#ci-job-)
      - [`ci-setup-*`](#ci-setup-)
      - [`ci-runner-*[-*]`](#ci-runner--)
      - [`ci-all`](#ci-all)
- [4. Reviews](#4-reviews)
- [5. Release Process](#5-release-process)
- [Other Tock Repositories](#other-tock-repositories)
  * [Userland Repositories](#userland-repositories)
  * [Tertiary Repositories](#tertiary-repositories)

<!-- tocstop -->

## 1. Introduction

As Tock supports more chips and services, changes to core interfaces or
capsules will increasingly trigger bugs or integration problems. This
document describes the process by which pull requests for the main
Tock repository are handled. This process is not set in stone, and may
change as problems or issues arise.

Active development occurs on the master branch. Periodic releases (discussed
more below) are made on branches.

## 2. Pull Requests

Any pull request against the master branch is reviewed by the core Tock
team. Pull requests fall into two categories:

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

**Upkeep pull requests** can be merged by any member of the core team. That
person is responsible for the merge and backing out the merge if needed.

**Significant pull requests** require review by the entire core team. Each
core team member is expected to respond within one week. There are three
possible responses:

  - **Accept**, which means the pull request should be accepted (perhaps
    with some minor tweaks, as per comments).
  - **No Comment**, which means the pull request is fine but the member
    does not promote it.
  - **Discuss**, which means the pull request needs to be discussed by the
    core team before considering merging it.

Core team members can change their votes at any time, based on discussion,
changes, or further thought.

## 3. Continuous Integration

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


## 4. Reviews

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

## 5. Release Process

Tock releases are milestone-based, with a rough expectation that a new release
of Tock would occur every 3-12 months. Before a release, a set of issues are
tagged with the `release-blocker` tag, and the release will be tested when all
of the release-blocker issues are closed. One week before the intended release
date, all new pull requests are put on hold, and everyone uses/tests the
software using the established testing process. Bug fixes for the release are
marked as such (in the title) and applied quickly. Once the release is ready,
the core team makes a branch with the release number and pull request reviews
restart.

Release branches are named `release-[version]`. For example, 'release-1.4.1'.

Patches may be made against release branches to fix bugs.

Note: Previously, Tock operated with a time-based release policy with the goal
of creating a release every two months. The intent was these periodic stable
releases would make it easier for users to install and track changes to Tock.
However, the overhead of keeping to that schedule was too daunting to make the
releases reliably timed, and it often did not fit well with the inclusion of
major features which might be in-flight at a release point.

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
