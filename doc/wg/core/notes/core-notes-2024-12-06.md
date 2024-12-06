# Tock Meeting Notes 2024-12-06

## Attendees
- Branden Ghena
- Hudson Ayers
- Johnathan Van Why
- Benjamin Prevor
- Pat Pannuto
- Leon Schuermann
- Kat Watson
- Brad Campbell


## Updates
 ### Isolated storage PR
   * https://github.com/tock/tock/pull/4258
   * Brad: I tried the isolated storage stuff and it worked. Looks good, but we should put it off past the tutorial at this point.
   * Brad: I think this is the kind of ABI that's really important and would be beneficial to stabilize.
   * Brad: I've also been thinking about OpenSK and having support for it from the upstream kernel. And part of that would require persistant, non-volatile, isolated storage. So this would be really useful
   * Branden: And this is an updated version of an old PR: https://github.com/tock/tock/pull/4109A
   * Brad: Yes. I wrote the non-volatile storage that exists a long time ago. That was pretty clunky code, so this version is just better!
   * Hudson: I know we've had discussions about various HILs and interfaces for storage in the past. Can you say how this is different or solves an issue today?
   * Brad: This PR uses the new storage permissions/interface to allow the kernel to provide individual non-volatile storage for applications. It does not assume a particular backing storage hardware, so it doesn't optimize flash writes or use particular properties of it. And it's the only isolated non-volatile storage that we have. As for the data structure, it uses a linked-list-esque thing where each element is a header specifying who owns the block. And new applications are appended to the end.
   * Hudson: So, if someone calls write and the backing storage is Flash, it's the responsibility of whoever implements the backend to do the erase?
   * Brad: The non-volatile storage API would have to translate a write into an erase+write series of operations
 ### EWSN Tutorial
   * Pat: Happening at EWSN early next week. Order 20 people involved


## PR Check-in
 ### MachineRegister type
  * https://github.com/tock/tock/pull/4250
  * Brad: Hopefully we've agreed that we want a datatype for "data that fits in a machine register". This is one implementation. It's, I think, the minimal change to add this type.
  * Brad: The discussion: is it worth doing a more invasive change. And if so, what should it look like?
  * Branden: Should MachineRegister be the base and define CapabilityPtr? That's the discussion on the PR. Do you have an opinion on that?
  * Brad: I do agree. I didn't want to re-litigate the prior PR though if it was going to be a fight.
  * Johnathan: I pinged Lawrence about it. One concern would be if a swap caused huge merge conflicts to him.
  * Pat: Shouldn't everything still be fine, if we're just changing the type definition?
  * Johnathan: I guess it would just be the PRs that define it? So maybe it wouldn't be so bad. I know when we discussed it, he liked the current way where register wraps CapabilityPtr, but I'm not sure why he feels that way
  * Brad: I'm hearing that we need input from Lawrence
  * Pat: Agreed. I think one reason we didn't push this previously is that it was a small piece of a larger CapabilityPtr PR. So I wouldn't be too afraid of making the change now. I think it should be switched, although it's not critical if there was something unexpected that made that difficult
  * Pat: So the action item would be that the core team is pro-switching, but we don't want to make Lawrence's life miserable.
 ### EXC_RETURN ARMv7m
  * https://github.com/tock/tock/pull/4256
  * Branden: This is forward-thinking, right? Not a bugfix
  * Brad: Yes. I think we would have done it this way from the start, but we just had to get something working back then
  * Pat: Agreed, this has been on my mind for years
  * Brad: Since we have the beginnings of support for hard-float where this will be important, and architectures advance we will increasingly want this
  * Pat: I haven't had the chance to look or think about this, but I will this weekend hopefully
  * Brad: I think I found some reasonable ARM instructions for the implementation
  * Branden: Do you have ARM documentation on these changes?
  * Brad: It was a hodge-podge of various documents and references. Each architecture is sort-of documented in isolation, and they're not always clear about what's stabilized or shared across architectures and what might change
  * Brad: They also use this idea for what they put in the link register, but I don't think they have a name for it.
  * Branden: What do we need to merge this PR? Just a couple more eyeballs?
  * Brad: I think so
 ### CONTROL Register ARMv6
  * https://github.com/tock/tock/pull/4259
  * Pat: I also need to look into this. The verification team found that our ARMv6 could return to userspace while still in privledged mode
  * Leon: Didn't we fix this like 6 months ago?
  * Pat: I think we did in v7-land, but the fixes were improperly applied to v6
  * Brad: I looked at this quickly, and the code changes seem fine. I didn't check that it makes sense in the whole context though. Hopefully there aren't additional missing locations
  * Pat: The question there the verification team might still be able to find. I will definitely look at this though
  * Branden: Same question, just a few eyeballs to merge this? Changing the assembly is always scary
  * Pat: The only platform for v6 right now is also the RPi Pico, so it won't affect too many things and Alex's team can test pretty reasonably
  * Branden: The Pico is a cortex-m0+, right? Is that different?
  * Pat: The core architecture is the same. Some implementation details are different


## Release Plan
 * Brad: Github really highlights how long it's been since our last release, so it's been on my mind. We want to use Treadmill and the hardware CI to do a release, what's the outlook for that?
 * Leon: We're still in the phase of getting tests running on the hardware CI repo migrated into the Tock repo. There's a PR for this. https://github.com/tock/tock/pull/4252 The tricky thing with this PR is that it needs to be merged in sync with another PR in the other repo. Then once that's in, we can have testing definitions for all of the nRF52 tests which run on all Tock commits. If that's good enough for a release, we should be able to hit one this calendar year
 * Brad: I'm happy to merge that PR now that the docs are switched over.
 * Leon: A couple of things were broken on the very last commit. We should figure that out on our end before pushing. We can look, coordinate, and then merge whenever
 * Brad: No problem. You all should feel free to hit merge when ready
 * Leon: After my deadline soon, I can really focus on release and testing
 * Brad: Awesome. So we're really close to hardware CI being able to test the nRF. Is there more discussion on that?
 * Leon: We have a list of the tests that we currently support
 * Benjamin: Located here: https://github.com/tock/tock-hardware-ci/actions/runs/11926833583 (see the Test Overview section)
 * Leon: We have some of the most important ones that run on a single board without human interactions. We are going to do multi-board tests at some point (like BLE) but we could just do those manually for this release
 * Brad: This is still _such_ an improvement over the status quo. So great
 * Leon: I'll post on the release issue when we're good to go
 * Leon: We'll also have to look at the release blocking pull requests?
 * Brad: Here: https://github.com/tock/tock/labels/release-blocker
 * Brad: But all of these are for Tock version 3, so they're not actually release blockers.
 * Leon: We might want the ARMv6 fixes in this release too
 * Brad: Okay, so post on the release issue when hardware CI is ready. We also don't want to exclude targeted testing and checking that basic things run still. Maybe we could have a release window, do a tag and have people do testing? We can keep things stable for just a little bit and then put it out
 * Leon: We could also establish a process for more frequent releases. Where we have two-week windows and check things
 * Leon: Is there anything we need to get in for this release?
 * Brad: CoreLocal is big. MachineRegister is unfortunate. But I think we should think about this release as nothing major and not wait on things
 * Branden: I agree with that
 * Leon: Okay, so could someone else tag a release candidate and open the two-week window?
 * Brad: I'll do that
 * Branden: Does that need to wait on Treadmill?
 * Leon: No. We can target Treadmill at any branch/tag.

