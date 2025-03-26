# Tock Meeting Notes 2025-02-05

## Attendees
 - Pat Pannuto
 - Viswajith Rajan Govinda
 - Brad Campbell
 - Alexandru Radovici
 - Leon Schuermann
 - Johnathan Van Why
 - Hudson Ayers
 - Tyler Potyondy
 - Ben Prevor


## Updates

- Leon: Networking WG Update
   - Talked a lot about ethernet; it's at a pretty good state; hil merged into staging as planned; think there is still one open PR
   - Two of the four implementations are ported to the new HIL, others ready do go
   - [litany of other things that are done; too fast to catch in notes here; in networking WG notes]
   - Some library headaches around Makefile.setup, in particular cases where setup needs to fetch a submodule and in-tree Makefiles try to include from a Makefile (in the submodule) that doesn't exist yet.
   - Short term solution is to vendor some files
   - Brad: Sounds fine for now, but I'd like to look at some point; generally we don't use Makefiles from others
   - Leon: Yeah, these aren't traditional Makefiles per se, but are more prepared lists of files that need building
   - Now that we have two implementations of the ethernet HIL and two users of the HIL, now might be the time to merge staging into master
   - Brad: Makes sense
   - Brad: There's a couple open PRs that are ethernet related; get those merged into staging, and then open the PR to merge staging to master?
   - Leon: Yes, that's the plan; and that's actually a slightly smaller changeset than I first feared as it's just two of the implementations; and future implementations can merge directly to master

- Viswajith: Dynamic app loading PR 3941 is ready for review
   - Viswajith: There's a small issue related to total-size
   - Brad: That's on the agenda, we'll get to that later
   - Key update is that the PR is ready for deep review (as is the TRD PR)

## Tock World 8
 - Alexandru: Rust Foundation sounds generally on-board with co-marketing our workshop
 - Brad: What day is workshops date?
 - Alexandru: If they do the same as last year, would be Fri Sep 5
 - Alexandru: RustConf is 2-5, but there is no schedule available yet
 - Alexandru: Immediate priority is connect RustConf organizers and Amit
 - Alexandru: RustConf was two days last year, now it's three-four


## PRs
 - Update nightly (https://github.com/tock/tock/pull/4348)
    - Brad: Mostly just clippy changes, biggest thing is the register macro recursion limit, which we can just increase as workaround for now
    - Leon: Rust 2024 edition has been stabilized. Biggest challenge we will have is that static muts are now hard errors. This nightly (and corresponding stable) would let us upgrade to 2024 edition
    - Brad: Yeah... the static mut issue is hanging over our head still
    - Leon: Yeah, so now is probably the time to revisit this and figure out a solution
    - Brad: Agreed, we don't want to fall too far behind, and don't want to be stuck on 2021 edition
    - Pat: I have a follow-on WIP patch removing yet more semicolons would like to add before merging
    - Brad: Sounds good!


 - Dynamic Process Loading (https://github.com/tock/tock/pull/3941)
    - Viswajith: Experimenting with writing the app but not writing the header (or header getting written in pieces?)
    - Viswajith: Currently we guarantee that the first 8 bytes are known
    - Viswajith: The current TBF threat model is that we only rely on the total_size field in the header and nothing else
    - Viswajith: Thus ultimately we were considering that header parsing not rely on checksums
    - Leon: Don't quite follow when this is an issue?
    - Viswajith: If you have a new app being written to flash, the currentDPL implementation doesn't make assumptions about writing app atomically, nor the order that segments of app is written to flash
    - Viswajith: If an app is partially through being updated, when current code scans flash, it might look like a padding app but the checkshum will fail (as the contents isn't empty padding)
    - Brad: This is a question for Johnathan. Currently, we need the total size to find the next app in flash. 
    - Johnathan: Didn't see any discussion in this issue?
    - Johnathan: Two issues here: There is integrity/availability. App loading can't break the linked list. The other issue is confidentiality, if, say all the apps are 1K in size, then if an app sets its size to 10K then it gains the ability to read the (presumed secret) app binary
    - Johnathan: That said, I don't really understand where the issue is in the context of this PR
    - Brad: The second issue, confidentiality, is easy to handle and is handled
    - Brad: Right now, the implementation respects what the threat model says it does. One option would be to update the threat model. Other change the loader somehow.
    - Johnathan: Let me see if I understand right, there is also a checksum on the header, and if that checksum fails the loader doesn't trust it and the list breaks?
    - Viswajith: Not quite...
    - Johnathan: Conceptually there should be something file-system like that lets you find where apps are located that are trusted, and then app headers which have to be untrusted; the intrusive linked list mixes two trust levels in the same header
    - Brad: I interpret that is that we should stick with the threat model we have; and the issue is the current kernel implementation
    - Leon: I think it's generally very useful to have a checksum on the linked list itself; useful to have as protection against incomplete writes to flash
    - Pat: I need to interject here as note-taker—this is too hard to follow in real time. People are talking past each other and it's not clear what exactly the issue is. This needs a write-up folks can digest
    - Brad: Agreed, will take that as action and bring this back.

 - Isolated nonvolatile storage (https://github.com/tock/tock/pull/4258).
    - Brad: Anthony Tarbinian at UCSD got this going, trying to get across the finish line now
    - Brad: Some issues came up around 64-bit address space for storage on 32- and/or 64-bit platforms, I've put in potential solutions in the PR, but want to see if there is anything controversial here
    - ...
    - Brad: Silence as not much to discuss now, but would appreciate folks looking at the PR
    - Hudson: Mentions it's an updated version of 4109 which points to 3095 etc; and there are docs in the PR, but pointer-following and rectifying is hard. Could the top-level be updated with motivation here?
    - Brad: We have the issue, is that sufficient?
    - Hudson: Issue 3905 handles this? That's okay, I can read that.

 - Stabilization documentation (https://github.com/tock/tock/pull/4329)
    - Brad: This is a procedural proposal for how to stabilize syscalls/interfaces
    - Brad: Are there any questions, things we should change?
    - (various): Good to me
    - Pat: I still think we need to update the table from checkmarks to 2.0 for consistency, but otherwise good

 - NonZeroU32 baud (https://github.com/tock/tock/pull/4255)
    - Brad: I thought we could have an associated type where the chip can define the type that would be valid. So STMs could define baud to be NonZeroU32, but some other chips have like eight valid bauds, so they would need an enum
    - Leon: I've heard this brought up, but I'm not sure this is possible or feasible in the way our types work right now
    - Brad: What gives you pause?
    - Leon: I'm worried about how this type information would be propagating across all of the layers of virtualizers, etc; especially possible that some traits would no longer be object safe; probably only need this type info on the downcall path maybe?
    - Leon: You're not going to get any assurance from me unless I or someone else tries to hack it together and implement it
    - Leon: As much as I like static checks, one of the problems we have is the virality of these types bleeding through the whole code base. And the one tool Rust has to help with that doesn't work well with associated types
    - Leon: Don't think there is much value in mandating any baud rates be supported
    - Leon: The only baud rate that can never be supported by any hardware is zero.
    - Brad: The benefit we would get from what I am proposing is that we eliminate runtime errors; but the question is whether that would actually be possible
    - Brad: We have some HILs which specify what types of things have to be feasible can be chip-specific, e.g. https://github.com/tock/tock/blob/master/kernel/src/hil/flash.rs, https://github.com/tock/tock/blob/master/kernel/src/hil/spi.rs
    - Leon: Even if we had this, there's some layer somewhere which takes an arbitrary baud and tries to convert it to one of these types, and must have an error path
    - Brad: It was a lot of churn to implement the chip select in SPI, but we got somewhere really capable
    - Brad: So my takeaway here I just have to try to find some time to go do this
    - Pat: In the immediate term, what does that mean for this PR? My disposition is that this is better than what we have (even if not as great as strong types could be) so we should merge the bird in hand, but I'm guessing you would rather close this and do the full type thing?
    - Brad: Yeah, the PR description says it gets rid of errors, and we should actually do all of that
    - Pat: Sure, but the current PR removes half the errors, which is an improvement now until the strong type thing comes someday
    - (meta): Meeting ran out of time, will continue on PR.
