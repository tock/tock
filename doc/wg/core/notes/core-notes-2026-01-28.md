# Tock Meeting Notes 2026-01-28

## Attendees
 - Branden Ghena
 - Leon Schuermann
 - Amit Levy
 - Brad Campbell
 - Johnathan Van Why


## Updates
 - None today
 - Network WG didn't meet last session
 - Crypto WG plans to resume this Friday


## Documentation
 - Amit: pulled up some stuff that seems to potentially have been forgotten for today
 - Amit: On documentation, three pieces we said we'd write with various degrees of importance
    - Single-Thread Value Porting Guide, more-or-less complete in PR comments https://github.com/tock/tock/pull/4519 and https://github.com/tock/tock/pull/4676
    - PR Quality and Engagement, came from discussion on stalled PRs where author is not progressing in the direction we want. We could add something laying out what we expect from PRs in terms of completeness and how to engage. Amit was meant to write it and hasn't yet.
    - Forbid unsafe in chips, explaining the safety boundary for DMA-capable chips. Relates to the PR Leon recently made on a safer DMA interface
 - Amit: So, do we still want to write these, and who should do so?
### STV Documentation
 - Amit: STV documentation for porting may be blocker for Tock 2.3 release. How to port a board to use STV.
 - Brad: I think the STV porting guide is still relevant.
 - Amit: Yeah, even if we had everything upstream converted, downstream boards could use a guide.
 - Brad: I think we have porting guides somewhere already?
 - Amit: In the two PRs: #4519 and #4676
 - Brad: We took care of kernel. There's a comment there on updating boards. Chips could still be an issue, but STV isn't the fix there. So if we have Boards porting guide for out-of-tree boards, plus a small paragraph on just how STV works, that would be good. The comment on #4519 works here
 - Brad: Where do we put this?
 - Amit: Good question. We have a doc/ folder, but without a clear hierarchy. It could be a TRD. It could go in the book somewhere.
 - Brad: I think the book makes sense. It isn't documentation of the repository itself. We also have a guide now for porting to a new platform and for porting from 1.0 to 2.0 Tock. So we could add it there with some structure
 - Amit: It would be good practice here to have general upgrade guides on per-release basis
 - Brad: I'll take a first draft on the STV stuff.
### PR Quality and Engagement
 - Amit: On PR Quality and Engagement, it's still relevant but not immediately urgent. We could still use this. Probably still on me to do this.
 - Brad: How does this relate to the Code Goals document? https://github.com/tock/tock/blob/master/doc/CodeGoals.md
 - Branden: I see those as different. Code Goals is really about Tock itself, not about PR management
 - Brad: I see PR quality as intertwined with this though. Unless you think quality is about commit structure.
 - Amit: Certainly related. PRs should meet the Code Goals, for instance. But we do have some thoughts beyond the specific code
 - Brad: Contributing guide seems relevant too: https://github.com/tock/tock/blob/master/.github/CONTRIBUTING.md
 - Branden: This seems to be about the _start_ of a PR, not continuing support.
 - Brad: Code Review guide as well https://github.com/tock/tock/blob/master/doc/CodeReview.md
 - Amit: There's definitely overlap there
 - Brad: I could just see this as an expansion of the Code Review document. Or maybe a wholistic view of various guides and a meta-guide that's more intuitive.
 - Amit: Some of these docs are targeted to certain audiences. This doc would be somewhat of a contract between creators of PRs and reviewers.
 - Branden: I could see that as part of the Code Review document
 - Leon: That document is very out-of-date and could use updates. Also it has CI stuff which could probably be removed
 - Amit: So we could reframe as revising the Code Review document
 - Branden: Definitely a focus on engagement would be useful here.
 - Amit: Okay, I'll take the task of updating the Code Review document
### Forbid Unsafe in Chips
 - Leon: Part of this was exploring how far we could go in chips. Part of this is based on the efforts in https://github.com/tock/tock/pull/4702
 - Leon: So we could delay this until after the PR.
 - Leon: This could be a TRD about how to structure things.
 - Amit: Okay, so we'll figure out how to support DMA in chips first. Then Leon will be responsible for a TRD giving guidance in the area. We could make a tracking issue about this.
 - Leon: I'll note Brad was pushing for this. I was the one blocking this on DMA. So Brad and I should both have custody. I'm happy to put this on my agenda to initiate the TRD process. Brad can help with the vision of removing unsafe and I'll focus on where we do still need unsafe.
 - Brad: Is it valuable to have this proposal in advance of actually changing some code?
 - Leon: If there's an improvement to be made, we can just do that. But reorganizations across multiple chips seem like they could use a design document, or at least an example for one chip.
 - Brad: Thinking on the libtock-c restructuring. There was kind of a document I developed while making the changes. And when they diverged it was annoying to update it. I think it's still unclear what this will really look like. Maybe we should just try it with one chip and see what we're comfortable with. Then document that as the guide moving forward.
 - Leon: So this could delay until we're more sure about chip-wide changes.
 - Branden: I think this sounds vague enough that we don't want anything right now.
 - Amit: From the original meeting where we discussed this, it was related to https://github.com/tock/tock/pull/4626 which forbids unsafe in nRF5x chips. Brad notes "how unsafe sneaks into chips today, what's expected and good, and what isn't". So I mischaracterized this in the agenda. So this is really documented the existing tensions about making chips forbid unsafe. For instance, that would include moving the DMA stuff into separate unsafe crates.
 - Brad: What's changed from then to now is that we have a prototype of the DMACell stuff. So there may be less that needs to be done in terms of the document
 - Amit: Okay, so we'll still get the DMACell stuff in. Then we'll review whether we can forbid unsafe in chips. Then we could document that if we move forward
 - Leon: I'll still make a tracking issue for that.


## x86-next Branch
 - Amit: We may not have the right people in the room for this
 - Amit: There was an issue with the x86 arch crate when Alex and company wanted to do virtual memory for x86. It didn't mesh well with the non-virtual-memory version that's currently in use. Rather than try to resolve that tension, the thought was to create an x86-next branch, akin to the ethernet staging branch, where virtual memory could be worked on separately and experimented with before officially merging into the main repo.
 - Amit: That never happened. There is no x86-next branch. As far as I know, that kind of stalled. So I'm wondering if virtual memory in x86 is still something we want to pursue
 - Johnathan: Shouldn't this go to the x86 working group instead of Core?
 - Amit: Good point. I'll plan to bring it up there. Alex is the most relevant person for the question.


## Tockloader Cleanup
 - Amit: Main thing is that in early summer it became really difficult to install and use Tockloader in distros that consider nrfjprog deprecated. Plus nrfutil is the replacement.
 - Amit: We got a promise to move over to that new one, but it never happened. I think this still needs to happen, but who can take over the task?
 - Amit: Also, generally this raises a question about Tockloader maintenance and sustainability. I think the status quo is that it's all on Brad's back. Also it's not the most exciting thing to work on, so sustainability is a worry.
 - Leon: I'm still interested in the nrfutil port, but it's not high on my task list. So timeline is vague
 - Brad: As for maintainability, I think the main challenge is quirks of different boards. Trying to support different platforms with various programming tools out there. We don't have the ability to test all versions of tools on all boards. There's also some "fringe" capabilities Tockloader has that doesn't get used very often.
 - Brad: Tockloader does still work for common cases on common distros
 - Amit: There are various outstanding PRs on Tockloader. Some new boards to add, plus some small fixes
 - Amit: The point isn't to call out Brad. He does _plenty_. Just generally to think about sustainability here.
 - Amit: A re-architecture could make this more sustainable to develop. But tockloader-rs work seems to have stalled.
 - Amit: I'm just worried that core Tock tools can't just be in maintenance mode, especially given changes about relocations that need to be supported.
 - Branden: I see this as an issue of developer effort here. There are only so many of us, and a working group for Tools would be great, but it would realistically just be staffed by the same group of us and not create extra hours in the day.
 - Brad: I see this differently. Elf2tab and Tockloader are integral to many workflows, and they need to do things correctly or we break stuff. We also add features, more often to Tockloader. Elf2tab is pretty maintenance over the last year, while Tockloader has gained features in the last year. There's also who knows the code well enough to consider additions as good/bad, and perhaps that's the thing that's an issue, where the people who know it is pretty low. Many PRs are "quick fixes" which would make it hard to maintain these tools over time. But I don't think there are many of us who could review those reasonably.
 - Brad: One way of looking at this, is that the greatest bang for our buck would be stabilizing portions of these tools, and testing that they do these stable parts consistently over time. We sort of do this implicitly already. Formal separation could be useful. But the challenge is that I understand this for Tockloader, but Elf2tab is harder to reason about.
 - Leon: What would be stabilized? The interface to users? Internal code?
 - Brad: User facing functionality and interface.
 - Amit: I'm in favor of that. How would we do it?
 - Leon: Rust has something similar it calls "UI test" for its tools. We could generate a bunch of invocations, check that the outputs don't change. We could use Treadmill for that.
 - Amit: But how do we get someone to do it?
 - Brad: Harder.
 - Leon: One thing worth discussing is that the interfaces of Tockloader and elf2tab do have quirks, and we'll want to think about which parts we do want to stabilize versus which we don't.
 - Amit: Yeah, that would be the initial effort, for sure. We could make decisions and back them up with CI
 - Amit: That remains up-in-the-air to create a task-force or working group that includes a person who has time to implement stuff for Tock tools. 

## Overflow Items
 - Amit: Won't get to these. Async in Tock, and LLM policy in Tock.

