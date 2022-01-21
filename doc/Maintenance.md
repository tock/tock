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
