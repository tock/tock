# Tock Maintenance

This document describes some elements of how the Tock [core working
group](wg/core/README.md) maintains the Tock project.

<!-- npm i -g markdown-toc; markdown-toc -i Maintenance.md -->

<!-- toc -->

- [Roadmap and Feature Planning](#roadmap-and-feature-planning)
- [Outreach and Education](#outreach-and-education)
- [Tock Release Strategy](#tock-release-strategy)
  * [Release Tasks](#release-tasks)
    + [Before the release](#before-the-release)
    + [Release testing](#release-testing)
    + [Tagging a release](#tagging-a-release)
    + [Starting the next release](#starting-the-next-release)
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

## Tock Release Strategy

Tock releases are milestone-based, with a rough expectation that a new release
of Tock would occur every 3-12 months. Before a release, a set of issues are
tagged with the `release-blocker` tag, and the release will be tested when all
of the release-blocker issues to be included in the upcoming release are closed
(release blockers for an upcoming major version do not need to hold up a minor
release).

Once all such release-blocker issues are closed, a new release branch is
created. Release branches are named `release/$major.$minor`, for example
`release/1.4`. This branch is used to perform tests and validation for the
upcoming release. Once the release branch is deemed read and stable, a release
is tagged. The branch is not deleted post-release; future fixes will be merged
into this branch and it will be used to tag new patch releases.

After testing, a release is tagged. Release tags must be created as *annotated*
git tags, not lightweight tags as they would be created for GitHub releases.
These are created using `git tag -a` and are considered release-quality tags by
Git, e.g. with `git describe`). Tags should contain the same release notes as
are attached to the corresponding GitHub release. Tags will be named using the
format `release-$major.$minor.$patch`. The initial release for a given minor
version carries patch number `0`.

> Note: Previously, Tock operated with a time-based release policy with the goal
> of creating a release every two months. The intent was these periodic stable
> releases would make it easier for users to install and track changes to Tock.
> However, the overhead of keeping to that schedule was too daunting to make the
> releases reliably timed, and it often did not fit well with the inclusion of
> major features which might be in-flight at a release point.

### Release Tasks

#### Before the release

- Decide on what features should be included in the release.
- Mark relevant issues and pull requests with the `release-blocker` tag.
- Open an issue titled "Release <version>" with:
	- A list of the goals for the release.
	- A template checklist for testing each board.
	- A sign-off checklist for each core working group member.
- Work through issues and pull requests with the `release-blocker` tag.

#### Release testing

During the release testing period, members of the core working group and
maintainers of various boards will run release tests, checking off individual
tests as they are run for each board. These tests are to be run on the branched
off release branch. Rather than maintain a list of tests in the repository, each
release involves running all of the tests that were run for a board at a
previous release, plus any new tests the maintainer of that board wants to run.
Accordingly, the release testing process is generally as follows:
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

Once these steps are complete:

- All tests pass for all boards.
- The changelog is updated.
- The `KERNEL_PRERELEASE_VERSION` version number in `kernel/src/lib.rs` is set
  to 0.
- The version number in the root `Cargo.toml` file is updated.

then a release can be tagged.

#### Starting the next release

Immediately after branching off a release, the minor version number on the
`master` branch is incremented and the patch version number is set to 0. This is
set in the root `Cargo.toml` file by setting the version to `<new version>-dev`
and in `kernel/src/lib.rs` by updating `KERNEL_MAJOR_VERSION`,
`KERNEL_MINOR_VERSION`, and `KERNEL_PATCH_VERSION`, and setting
`KERNEL_PRERELEASE_VERSION` to 1.

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
