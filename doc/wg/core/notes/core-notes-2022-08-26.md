# Tock Core Notes 2022-08-26

Attendees:
- Amit Levy
- Branden Ghena
- Brad Campbell
- Johnathan Van Why
- Arun Thomas
- Alexandru Radovici
- Chris Frantz
- Pat Pannuto
- Alyssa Haroldson
- Jett Rink
- Leon Schuermann
- Hudson Ayers

## Updates

- Alex: oxidos.io funding round secured to use Tock in automotive space.
- Need a blog post!

## Tock 2.1 Testing

- https://github.com/tock/tock/issues/3116#issuecomment-1209792251
- Brad: We've done a lot of testing, I say we go forward with the release soon.
- Alex: STM almost finished. RPi tomorrow. Will be ready.
- Other Alex will test i.mx board.
- Branden: Let's make a deadline to finish testing and move on with release.
- Pat: redboard problem.
- Brad: should create 2.2 issue now. Goals: update cortex-m syscall handling and
  AppID.
- Branden: Other PRs on 2.1 to discuss? They seem pretty straightforward.
- Brad: Need release notes, possibly changelog.md.
- Leon: Happy to start that, would like to see if anything is missed.
- Jett: Mark major breaking change in changelog.md.
- Branden/Amit/Brad: September 1, 2022 release goal.

### redboard Problem

- https://github.com/tock/tock/pull/3139
- Pat: Technical debt from context switching in cortex-m. Assumptions that
  things would happened one at a time. Nested events and floating point violate
  these assumptions.
- We are checking link register, should be checking status register.
- Current summary: we need to re-write cortex-m bottom half handling, should fix
  this issue.
- Amit: Been talking about this re-write for a while, makes sense to do this
  change.
- Probably best thing to do is go ahead with 2.1, focus on re-factoring this for
  next release, and let this board be broken for this release.
- Pat: could apply PR for 2.1 which is a workaround hack. This PR still leads to
  stack corruption but that corruption doesn't matter.
- Brad: agree with merging PR. Should open revert PR now.
- Alyssa: add comment indicating hack needs to be reverted.
- Hudson + Amit: agree

## Tock Foundation

- Amit: 501(c)3 very difficult.
- Alternatives, just as 501(c)6 should be fine.
- Technically, could accept money now through Open Collective.
