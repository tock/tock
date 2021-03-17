# Tock Maintenance

This document describes some elements of how the Tock [core working
group](../wg/core/README.md) maintains the Tock project.

<!-- npm i -g markdown-toc; markdown-toc -i Abstract.md -->

<!-- toc -->

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
academic conferences.

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

Before the release.

- Decide on what features should be included in the release.
- Mark relevant issues and pull requests with the `release-blocker` tag.
- Open an issue titled "Release <version>" with:
	- A list of the goals for the release.
	- A template checklist for testing each board.
	- A sign-off checklist for each core working group member.

During the release testing.

-
