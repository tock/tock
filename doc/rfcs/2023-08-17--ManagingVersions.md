Managing Versions
=================

 - Initial Proposal: 2023-08-17
 - Disposition: Under Review
 - RFC PR: https://github.com/tock/tock/pull/3622

Summary
-------

This document describes Tock's versioning approach as well as the
development model for how we manage changes pending for next patch,
minor, and major releases.


Kernel Versioning
-------

We apparently never wrote down how our stability guarantees map to
kernel versions, so we should do that. I tried to capture the essence of
what we currently do in the new §2.1 and §2.2 for TRD104 attached to
this PR.

_Ideally, this is just writing down our existing policy, and is not too
controversial._


Master Branch Policy
--------------------

A fairly significant piece of the unwritten versioning rules is "what
changes are allowed in master?"

We have an answer from the 2.0 transition that we used: the default
development branch should maintain ABI backwards compatibility but not
necessarily forward compatibility.  I.e., in our versioning language,
the master branch has 'next minor release' semantics.

I think it's fair to say that we've basically stuck to that policy.
I tried to capture this in the updates to the Maintenance.md document
attached to this PR.

_Ideally, this is just writing down our existing policy, and is not too
controversial._

I also tried to write down how we managed the major version transition.
I think it worked pretty well last time, but open to more thoughts here.


Understanding the delta between master and last release
------------------------------------

While we have a CHANGELOG file, to-date we are not great about keeping
it up to date during active development. Rather, as part of release
preparation the core team retroactively looks through all of the PRs and
commit history since the last release and synthesizes the key changes.

> Indeed, I got really confusing writing this section when I looked at
> the [current CHANGELOG in `master`](https://github.com/tock/tock/blob/master/CHANGELOG.md),
> as I thought we were on 2.1.1, but in actuality only _some_ of the
> changeset included in 2.1.1 is actually in current master---one of the
> missing bits is the update to the CHANGELOG :/.

Given our fairly non-deterministic release process, we can end up with a
long time and a lot of changes that makes it very challenging for an
external person to understand what has been fixed, what has been added
since Tock's last release in December, 2022 (well, that's the 2.1.1
point release, which _doesn't_ have a bunch of stuff between it and the
September, 2022 2.1 release included). There have been 1,350 commits
since 2.1 as-of this writing.

Trying to update the CHANGELOG with each PR, however, would likely
create a cluster of merge conflicts not worth maintaining.

One possible idea is to require CHANGELOG updates for any PR tagged
`P-Significant`. I suspect that might be a necessary, but not
sufficient, policy.

Another idea, of course, is more frequent releases. Maybe that is
something more feasible with robust hardware CI, but in the near-term, I
don't see our release process getting significantly less painful or more
frequent.


Carrying planned changes in code not in ToDo lists
--------------------------------------------------

One challenge with ABI stability, especially ABI stability in the master
branch, is that we can't fix 'small things' when they come up
(#3375, #3613, etc as motivation here). Currently, we simply close such
PRs and/or merge partial, non-ABI-breaking fixes, and put a TODO comment
in the code or maybe a tracking issue. I see several negatives to this
approach:
 - It is easy to miss/overlook/forget something
    - When I linked the stabilized syscall document, I noticed the
        comment that 'GPIO is slated for renumbering with 2.0', but GPIO
        was [00004 at v1.6][driverNum1v6] and was still
        [00004 at v2.1.1][driverNum2v1v1] (and is 00004 currently).
 - The cost of missing something is _very high_
    - It means that a change must now be deferred until the _next_ major
        release, which history tells us is rare.
 - We create a giant todo-list that blocks/slows releases
    - When something 'major' does motivate a new major release, in
        addition to that big, complicated thing, we have a giant list of
        tiny busy-work style things we need to 
 - It's off-putting to new users
    - Some of the latent consistency issues are (rightly) tagged a
        `good-first-issue`, yet when folks make PRs to fix them
        (#3397, #3613, etc) we reject the changes.
 - We throw away good code and developer context
    - This follows from the last two points---we should fix things when
        we are touching the code to make changes. Otherwise when we come
        back later during release crunch time, we're relearning context
        of what needs fixing and how to fix it that was already in
        someone's head a few months ago.


### What can we do instead?

1. Keep PRs with fixes tagged and open until a major release process starts.
   - Pro: Have the "real code" and "real fix" already there.
   - Con: Likely that the PR will not merge cleanly by the time the next
       major release occurs.
   - Con: Lots of noise in the PR queue

2. Always keep a 'next-major-release' branch active
   - Pro: Logical home for ABI-breaking changes
   - Pro: Developer-friendly, easy to ask people to change target of PR
       to accept useful changes
   - Con: Will diverge, likely significantly from master branch, making
       sync hard when release is ready
   - Con: Avoiding prior con would add non-trivial maintenance burden of
       periodically merging master with 'next-major', and likely
       carrying a large set of merge conflict resolutions along the way.
   - Con: Somewhat hidden, and unlikely to see any testing

3. The `cfg` option (I can't believe I'm suggesting it either)
   - Pro: Code is 'right there' such that wide-area interface,
       renaming, etc style changes will update next-release code as well
       [assumes CI builds both current and next-release, which is easy]
   - Pro: Release transition is very clear, simply remove all code where
       `cfg` is no longer relevant.
   - Con: It's `cfg`. It means we're carrying a bunch of code which in
       the common case is untested and unused.
   - Pro: Can result in space-savings for up-to-date downstream users.
       If we are carrying code for backwards compatibility that
       downstream folks know they won't use, they can remove that part
       of the kernel.
       - Con: The exponential number of kernel versions we just created.
       - Pro: Could act as indicator for when a release is merited, i.e.
           if we released a new version we'd shave off XX% of kernel
           size for large enough X.
   - Nuance: It may be worth distinguishing between "next minor" and
       "next major". In particular, "next minor" `cfg`'d stuff can be
       default-on, and largely no change from current practice, except
       for this sentinel sprinkled in the codebase identifying what's
       not in the released version [though, this could arguably also be
       accomplished with some type of comment keyword].

4. ...?

[driverNum1v6]: https://github.com/tock/tock/blob/e8d0a28d86897c91b6747be357abfcfa7e86688f/capsules/src/driver.rs
[driverNum2v1v1]: https://github.com/tock/tock/blob/44f39d7c8cf5db0038606f08640bfae670127eef/capsules/src/driver.rs


Being more explicit about ABI changes and surface area
------------------------------------------------------

It's great that we have things like the compatibility header in the TBF,
but if I wanted to understand which version of Tock my application
requires, how would I figure that out? We have the list of [syscalls
stabilized with 2.0](https://github.com/tock/tock/tree/master/doc/syscalls),
but does that mean an app can't rely on _anything_ else?


**Proposal:** We need something between "stable" and "unstable". Maybe
not the [7 tiers of Python stability](https://pypi.org/classifiers/),
but something that helps folks get a sense of 'this has been around for
a while' versus 'this is brand new and we are still figuring it out'.


**Proposal:** We should not stabilize whole driver interfaces, but
rather individual syscalls within them. Low-level-debug Command 2,
"print a number", seems pretty safe to call immutable; I don't think
we want to lock the LLD interface yet, however.


As we have learned time and again, it's also really hard to
keep such documentation up to date. We basically subsist now on @bradjc
going on documentation rampages.

It is also likely not obvious to new users when something is part of the
kernel ABI. In practice, the surface area of the kernel is strewn across
many files.

**Dream:** We should write a tool that enforces documentation, including
stability/maturity classification, for anything that creates a
userspace-facing interface (i.e. `SyscallDriver` impl's) and
auto-generate documentation, with info like "since Tock 2.x".
