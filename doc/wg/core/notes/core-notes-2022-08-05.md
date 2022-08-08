# Tock Core Notes 2022-08-05

Attendees:
 * Leon Schuermann
 * Arun Thomas
 * Branden Ghena
 * Vadim Sukhomlinov
 * Philip Levis
 * Chris Frantz
 * Johnathan Van Why
 * Hudson Ayers
 * Alyssa Haroldsen
 * Pat Pannuto
 * Brad Campbell

## Updates
 - Phil: AppID: More code size reduction; 900 bytes w/out checking. 300 in TBF, 300 kernel, 300 in strings (ish). Will look into strings to see if we can cut those, but getting to the point where it won't be much smaller.
   - Hudson: 900 much smaller than when started; much more palatable
 - Hudson: New approaches to deferrred call. First, tighter kernel integration, no globals, atomics. Then tried new pair of PRs that would eliminate dynamic deferred calls (capsules use same infra as chips) at cost of rough syntax; other PRR keeps globals + atomic usize, one more instantiation of this becomes available to capsules. Suspect that approach two will improve code size a lot, esp for those using dynamic deferred calls.
    - Phil: Deferred calls have always been a tricky wart; nice to see improvements here.
    - Hudson: +1, always strange to have both dynamic and static
 - Alyssa: About to push a draft unit testing example. Not perfect, but want folks to see what the necessary pieces are to get unit testing working.

## PR Review
 - Phil: We brought this up at TockWorld and other recent meetings. Idea is to keep folks up to date with big PRs that have been recently merged and/or are open/active.

### Merged:
  - https://github.com/tock/tock/pull/3108 - kernel: hil: sensors: AirQualityDriver improvements
  - https://github.com/tock/tock/pull/3113, https://github.com/tock/tock/pull/3096, https://github.com/tock/tock/pull/3087  - Series of improvements to print_tock_memory_usage
       - Phil: Symbol tables from llvm have changed some, so attribution wasn't so great; now fixed.
  - https://github.com/tock/tock/pull/3106 - Seven Segment Display Capsule

### Open:
  - https://github.com/tock/tock/pull/3092 - capsules: Add support for AES GCM
      - Phil: Discussed at OT meeting. GCM is a form of authenticated encryption, concurrent encryption and MAC computation. Some complex layering, e.g. current GCM layers on CBC, so what does virtualizers do?
      - Phil: Historically, we have the strongly anti-third party policy. But crypto is the one place where this really probably shouldn't be the case.
      - Johnathan: If we're talking about the external policy, this is also probably a factor of why Tock and the greater embedded rust universe are somewhat distinct.
      - Phil: Yes. But for the immediate, if we're going to have software AES, we should probably use a third-party.
  - https://github.com/tock/tock/pull/3067, https://github.com/tock/tock/pull/3055, https://github.com/tock/tock/pull/3011 - Display updates
      - Brad: I can give a very limited update. Best info is probably in tracking issue that was recently created, but it's still a bit hard to keep track of the priorities and ordering of steps ( https://github.com/tock/tock/issues/3079 ). Does seem like step 1 is largely HIL consolidation, consistency, and cleanup.
      - Phil: Often we think only in framebuffers. Here, we actually have to think about the display as well (how pixels are sent, etc); makes this a bigger task.
  - https://github.com/tock/tock/pull/2993 - RFC: hil: Add generic block device HIL
      - Phil: Trying to be the simplest view, but still running into the challenges are page granularity, erasing, etc. Something to watch.

## Discussion of 2.1 Release
 - Hudson: Brad made a tracking issue ( https://github.com/tock/tock/issues/3116 ).
 - Hudson: Brad and I are advocating to do a release in the very short term. i.e. I plan to try to do release tests on imix today. If no issues, then kick out the platform testing process. Some folks wanted to wait for a few others,  wanted to chat quick
 - Leon: I had wanted to get some of the open unsoundness issues merged, but given that there are no visible issues from  unsoundness yet, okay to go forward now
 - Hudson: Yes, and 2.2 should follow 2.1 pretty quickly, i.e. roughly post appid
 - Phil: Yes, because of libtock and soundness and such, doing 2.1 w/out AppID make sense. Less concerned  about how quick a release with AppID comes out, more concerned about how quick AppID gets merged, as there's a lot of maintenance work to keep it up as 
 - Leon: Can we merge the trivial PRs right now, and then do a freeze now?
 - Hudson: That sounds reasonable, I'll share screen and we can walk through these.
 - Leon: Quick question, what's our platform deprecation timeframe, if people don't step up to test?
 - Hudson: Maybe two weeks after the initial round of testing and tagging people; if we haven't heard anything (even 'need more time to test'), can  pull it for the release
 - Brad: There is also some judgement here; something 'near' a platform we support a lot can stay more, something that's less used can be more aggressively pruned
 - Hudson: That makes sense, I'll give please reply in two weeks message, but won't threaten removal. We can do judgement after that

### PR Triage
 - https://github.com/tock/tock/pull/3127, https://github.com/tock/tock/pull/3125, https://github.com/tock/tock/pull/3124, https://github.com/tock/tock/pull/3123 - Not ready yet
 - https://github.com/tock/tock/pull/3122 - waiting on OT bitstream update
 - https://github.com/tock/tock/pull/3120 - Hudson: could go? Phil: I'll look and merge.
 - https://github.com/tock/tock/pull/3119 - Hudson: Already approved; merge.
 - https://github.com/tock/tock/pull/3118 - Has changes requested (just spelling?). Not urgent. Don't block for this.
 - https://github.com/tock/tock/pull/3117 - Might be needed for tests to pass for this platform
 - https://github.com/tock/tock/pull/3114 - Not ready, 'could be months of back and forth;  hoping not...'
 - https://github.com/tock/tock/pull/3112 - Not a blocker.
 - https://github.com/tock/tock/pull/3110 - Not ready
 - https://github.com/tock/tock/pull/3095 - Leon/Pat will discuss after call
 - https://github.com/tock/tock/pull/3092 - Blocked on OT discussion
 - https://github.com/tock/tock/pull/3086 - WIP
 - https://github.com/tock/tock/pull/3085, https://github.com/tock/tock/pull/3056 - Not working? Still some PMP issues. Chris: Would prefer to delay bitstream until OT solidifies formal release process. This one is mine, can just close, I will open a new one when ready.
 - https://github.com/tock/tock/pull/3084 - Not blocking.
 - https://github.com/tock/tock/pull/3077, https://github.com/tock/tock/pull/3068 - WIP.
 - https://github.com/tock/tock/pull/3067 - RFC not blocker.

... At this point, all older enough to not likely be blockers; any folks want to highlight?

 - Leon: https://github.com/tock/tock/pull/2516 should be ready to go today; Leon will rebase, Hudson will review
 - Phil: https://github.com/tock/tock/pull/3045, lowrisc autogen register definitions? Chris: In a similar vein as the OT discussion, want to close this until the OT release process stable. You want the version of these files tied to the release tag you're going to support; these are an arbitrary day. Short term: convert to draft, indicate waiting for OT release process.


## Discussion of approaches to removing `DynamicDeferredCall`

 - Pre-meeting notes from Hudson:
    - https://github.com/tock/tock/pull/3123 is the first approach, and builds off of my older PR that removed the need for atomics in deferred_call.rs, helping us remove an unstable feature. However, if you look at the changes in capsules/src/ieee802154/driver.rs, you will see that the syntax required by this change is messier, and makes adding a deferred call to an existing capsule more of a chore.
    - https://github.com/tock/tock/pull/3127 also removes DynamicDeferredCall in favor of pushing capsules to a statically defined approach, but still keeps the general approach of using global (atomic) variables in the kernel. This does not help us move off Rust nightly, but lets the syntax in capsules be much nicer and requires less refactoring elsewhere. It also seems to produce smaller code, though I have not extensively tested this.
 - Hudson: Really want to do compare/contrast of these approaches.
 - Hudson: I think it makes sense to get rid of DynamicDeferredCall. In all the upstream boards/chips/capsules, we don't take advantage of the fact that at runtime you can change what structure gets calls. Things are effectively set up statically at boot. Really we have this dynamic thing because it's hard for capsules to set things up given the type limitations for the existing deferred call type (as capsules can't depend on specific chip crates, and chip crates don't depend on capsules) - so no way to create a list of deferred calls for capsules and chips.
 - Hudson: The https://github.com/tock/tock/pull/3123 approach has a generic deferred task that chips and capsules implement over. The board main.rs defines a mapper (`fn handle_deferred_call`) for what handles deferred calls. This also handles all the associated type definitions. Previously, this logic was contained in `chips`, now it's all surfaced to the board main.
     - One thing worth considering; look at the radio capsule example ported: Previously had to allocate, now take in the manager reference, which adds a generic over deferred call manager parameter that could propagate in a challenging ergonomic way.
     - Other thing: The mapping of deferred calls, and what comes back to capsule is in trusted main.rs code; but the triggering of deferred calls currently trusts capsules to call the right one.
     - Leon: Q how does this relate to abstraction layers over trait objects? Given that all the users must be generic over deferred call mapper, will this work okay with external uses, will this leak through all later types?
     - Hudson: Is Q what happens if radio driver needs to be object-safe?
     - Leon: It's really does this generic type propagate through trait objects?
     - Hudson: Yeah, actually think it might not be object safe in general
     - Leon: what happens when components from other crates, not components or capsules? i.e. downstream?
     - Hudson: Out-of-tree capsules crates generally seem to depend on our capsules crate, e.g. to get virtualizers or other needed pieces. Can define their own Task enum that is a superset, and pass that as the capsule task type in main.rs
     - Leon: Would that be compatible with the upstream use of the directly named type?
     - Hudson: No...
     - Phil: Skittish about this namespace. Not checked or protect; collisions are possible as well.
     - Hudson: Going to need to develop something to protect ids for different users and use sets
     - Hudson: Leon, the point is good that this harder with types for downstream
 - Hudson: The https://github.com/tock/tock/pull/3127 approach: Doesn't get rid of globals, atomics; instead adds capsule-deferred. Basically a copy in capsules of what we did in chips today. Changes of use much smaller. Still have the same namespacing  problem, but the generics issues go away. There are some small changes for generics in chips.rs; basically assume two kinds of deferred call. Still have a mapping in main, mostly  the mapping is defined in e.g. capsules/src/driver.rs, just instantiated in board main.rs now. Still doesn't have a great story for downstream extension.
     - Leon: Maybe we should look into Rust type ids? Could map to integers and guarantee uniqueness?
     - Phil: Compile-time count is good for array sizing etc; key thing is conflict, duplication issue
     - Leon: Would assume most capsules would break when given a spurious deferred call
     - Phil: Shouldn't be able to call a method on something if you  don't have a reference to it
     - Hudson: This approach shouldn't have the spurious call problem as they can't get reference, but still no good solution to the downstream extension
     - Hudson: Downstream extensibility wasn't really considered yet, seems like that's a big issue I need to / will think about next
     - Leon: Thank you for doing this. Lots of work. When writing first capsules struggled with dynamic
     - Hudson: Call for ideas for task registry with out-of-tree capsules, now or later. Today, it relies on an enum in the capsules crate that we can update upstream as things change, but doesn't really allow extensibility.
     - Alyssa: Wrapper around integer with associated constant?
     - Leon: isolation? Can craft integers, while enums give type restrictions
     - Leon: This might be where we can use rust type ids, as they are guaranteed to be unique and hash to unique integers
     - Alyssa: yes, but type ids will require vtables
     - Hudson: Even with unique types, not clear can be generic over unknown types
     - Hudson: Will try to create a more minimal example, discuss in small working group (roughly alyssa, leon?)

## Closing Comments

 - Alyssa: PR up for example test. Think we'll need some structures to help support test infra, esp static_init...
 - Hudson: Lots of discussion about how to make static_init safer; still open issue I think, will send along
