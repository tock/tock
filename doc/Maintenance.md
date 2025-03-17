# Tock Maintenance

This document describes some elements of how the Tock [core working
group](wg/core/README.md) maintains the Tock project.

<!-- npm i -g markdown-toc; markdown-toc -i Maintenance.md -->

<!-- toc -->

- [Roadmap and Feature Planning](#roadmap-and-feature-planning)
- [Outreach and Education](#outreach-and-education)
- [Preparing a Release](#preparing-a-release)
  * [Release Tasks](#release-tasks)
    + [Before the release](#before-the-release)
    + [Tagging a release candidate](#tagging-a-release-candidate)
    + [Release testing](#release-testing)
    + [Tagging a release](#tagging-a-release)
- [Stabilizing a Syscall Driver](#stabilizing-a-syscall-driver)
  * [Syscall Driver Stabilization Process](#syscall-driver-stabilization-process)

<!-- tocstop -->

## Roadmap and Feature Planning

The major long-term planning efforts occur at periodic (roughly yearly or so)
"Tock World" workshops where core working group members and other stakeholders
discuss designs for new Tock features and overall project goals.

Other planning occurs on the weekly core working group calls.

## Outreach and Education

Beyond being an open-source project available to anyone to use, the Tock core
working group periodically hosts interactive tutorials to give interested users
hands-on experience with Tock. These have been hosted in conjunction with
academic and professional conferences.

The project also maintains a [book](https://book.tockos.org) which includes
self-guided tutorials for various Tock features.

## Preparing a Release

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

> Note: Previously, Tock operated with a time-based release policy with the goal
of creating a release every two months. The intent was these periodic stable
releases would make it easier for users to install and track changes to Tock.
However, the overhead of keeping to that schedule was too daunting to make the
releases reliably timed, and it often did not fit well with the inclusion of
major features which might be in-flight at a release point.

### Release Tasks

#### Before the release

- Decide on what features should be included in the release.
- Mark relevant issues and pull requests with the `release-blocker` tag.
- Open an issue titled "Release <version>" with:
	- A list of the goals for the release.
	- A template checklist for testing each board.
	- A sign-off checklist for each core working group member.
- Work through issues and pull requests with the `release-blocker` tag.

#### Tagging a release candidate

- Once most blocking PRs are merged, the core working group will often decide on
  a "freeze", where any new PRs will not be merged until after a release.
  Typically, this freeze should only last about a week.
- Once all issues and pull requests marked `release-blocker` are closed/merged,
  a release candidate can be tagged by a member of the core working group. The
  tagging of a release candidate marks the beginning of the release testing
  phase.
- Release candidates are named release-<version>-rc-x, where x is the release
  candidate in question.

#### Release testing

During the release testing period, members of the core working group and
maintainers of various boards will run release tests, checking off individual
tests as they are run for each board. Rather than maintain a list of tests in
the repository, each release involves running all of the tests that were run for
a board at a previous release, plus any new tests the maintainer of that board
wants to run. Accordingly, the release testing process is generally as follows:
- Select a board to test
- Copy the release testing checklist for that board from the tracking issue of
  the previous release, but uncheck all the boxes.
- Post this copied checklist in the new release tracking issue. This indicates
  that you have taken ownership of testing this board
- Add any additional tests that you would like to run this release. For example,
  if that board has added a new capsule since the last release, it is reasonable
  to add tests for that new capsule.
- Run the tests, checking off each item as it is completed. If a test fails,
  edit your comment on the issue to mark that test with an `X` to indicate that
  the test failed, and add a description of how it failed.
- For all failing tests, either submit a PR with a fix, or post an issue
  describing the failure to ask for help

If significant changes need to be made to fix bugs discovered by the testing
process, additional release candidates can be tagged. The core working group
will decide whether new release candidates require re-testing all boards or not.

#### Tagging a release

Once all tests pass for all boards, and the changelog is updated, a release can
be tagged.

## Stabilizing a Syscall Driver

Tock maintains a list of stabilized system call drivers that the kernel
guarantees not to break in subsequent releases of the kernel with the same major
version number. These are the drivers that implement `SyscallDriver` and are
commonly found in capsules. Note, these stabilization guarantees are separate
from the stability of the system call interface itself (i.e., `command`,
`allow`, etc.).

The goal of this stability guarantee is to ensure that applications that use a
particular system call driver will continue to work with a kernel of the same
major version number. However, Tock does not prohibit expanding a stabilized
interface within the same major version number. This enables new functionality
to be added, or for bug fixes to be added without breaking backwards
compatibility.

### Syscall Driver Stabilization Process

The general process is a syscall driver, identified by its driver number, is
proposed to be stabilized. The driver must have a documented interface in the
`docs/syscalls` directory. The interface has a waiting period (which can have
started in the past) where if no changes have been made to the interface the
interface is marked as stable as of the next release of Tock.

Syscall driver stabilization process:

1. The driver has complete documentation in the `doc/syscalls` directory.
2. A Tock developer proposes that a specific driver, as identified by its driver
   number and source code in the upstream Tock repository, should be stabilized
   by creating a pull request that moves the driver to the `capsules/core` crate and
   marks the stabilization column in `doc/syscalls/README.md` with a ⏰.
3. Stabilization PRs are always considered `P-Significant`, which requires the core
   working group to support the stabilization.
4. The syscall driver must be unchanged for a period of four months. This period
   may start from a point before the stabilization process started. The driver
   must be reasonably tested during this period.
5. If changes are required the waiting period resets.
6. After the probationary period, the syscall driver is marked stabilized at the
   next Tock major or minor release. The syscall driver is marked stable by
   updating the stabilization column in `doc/syscalls/README.md` with the Tock
   release version number.
