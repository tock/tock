# Tock Pull Request Process

## Abstract

This document describes how the Tock core team merges pull requests for and
makes releases of the main Tock repository.

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
   re-implementations, new traits, or changes to the build system.

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
This is the basic process we use now.

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

## 3. Reviews

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

The members of the core team are:
 * Niklas Adolfsson - [niklasad1](https://github.com/niklasad1)
 * Hudson Ayers - [hudson-ayers](https://github.com/hudson-ayers)
 * Brad Campbell - [bradjc](https://github.com/bradjc)
 * Branden Ghena - [brghena](https://github.com/brghena)
 * Philip Levis - [phil-levis](https://github.com/phil-levis)
 * Amit Levy - [alevy](https://github.com/alevy)
 * Pat Pannuto - [ppannuto](https://github.com/ppannuto)

## 4. Release process

Having periodic stable releases makes it easier for users to install
and track changes to Tock. Our intention is to release approximately
every two months, at the beginning of even months. One week before
the intended release date, all new pull requests are put on hold, and
everyone uses/tests the software using the established testing process.
Bug fixes for the release are marked as such (in the title) and applied
quickly. Once the release is ready, the core team makes a branch with
the release number and pull request reviews restart.

Release branches are named 'release-n-mon-year'.
For example, 'release-0.1-Feb-2018'.

Patches may be made against release branches to fix bugs.
