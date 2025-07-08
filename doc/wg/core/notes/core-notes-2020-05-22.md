# Tock Core Notes 05/22/2020

Attending
 - Brad Campbell
 - Alistair
 - Leon Schurmann
 - Pat Pannuto
 - Samuel Jero
 - Johnathan Van Why
 - Philip Levis
 - Vadim Sukhomlinov
 - Amit Levy
 - Branden Ghena
 - Hudson Ayers

## Updates
 * Pat: We're excited to move over to Github Actions soon. Anyone still care for Travis?
 * Leon: There was one Github Actions failure so far, but it was a Github outage and they responded quickly.
 * Pat: I would be happy to switch over from Travis today. And will push something later today.
 
## New Component Interface
 * Amit: https://github.com/tock/tock/pull/1618
 * Pat: Need eyes on this. It showed up right before the last release and got lost a little. My comment on April 13 (https://github.com/tock/tock/pull/1618#issuecomment-612957086) is worth reading. The high level issue is we want to impose structure on what components look like so they are consistent. At the same time they are often unique to what they are creating, so it's hard to make a common type. What we try to do is say we're creating a trait for what they look like with associated types to create structure. We would like to draw attention and get feedback from everyone and then if we're happy, move forward.
 * Hudson: When we talked about this last time, the new finalize interface was discussed to account for circular dependencies. This would be committing to an interface that does not support that, which is maybe fine.
 * Pat: Are there cases where the circular dependency issue still exists?
 * Phil: When the component interface was designed by Shane, there was an issue with Teensey that needed circular dependency support. And I could imagine that examples of that still exist.
 * Amit: Can you explain what you mean by circular dependency here? Is there where components would be circularly dependent on each other?
 * Leon: Yes. Circular dependencies are supported within the component, but you could create one component, initialize memory for the next, and then use that to finalize the first.
 * Amit: I'm confused how this isn't ubiquitous.
 * Branden: This feels like it should always be a problem or never a problem. I'm missing the weird case where it's sometimes a problem.
 * Leon: I think it's not currently a problem because all circular dependencies are contained within a single component.
 * Pat: If you dig up the PR for components, this comment from Phil (https://github.com/tock/tock/pull/1012#discussion_r201152009) explains circular dependencies aren't actually needed.
 * Amit: Wouldn't this be an issue for Thread where the network stack is a dependency of Thread overall, but other parts like setting IP addresses are dependent on Thread?
 * Hudson: But it seems like Thread would just take a reference to the 6lowpan stack and then call the set_client calls before returning the component. So maybe this isn't really an issue.
 * Leon: This could be an issue if we had sub-components. Splitting part of a component into something else. But I'm not sure that would be a good idea.
 * Hudson: Given that we don't use the components, just the capsules they create, it seems unlikely that components would depend on each other.
 * Phil: Remembering now, Shane incorporated finalize because he thought you shouldn't pass parameters into `new`. And then it all evolved so that feature was no longer needed. I think this is a case where we just didn't realize there was an easier solution.
 * Amit: Hudson's point that components are not ultimately the important part is good. It's probably fine even if there are rare edge cases where we might have to handle something specially.
 * Hudson: I'm pretty convinced by this conversation that the problem won't come up.
 * Amit: So should we be approving this PR?
 * Pat: The code is old. We just need consensus on the plan in the PR thread.
 * Amit: So we should look at it and okay the plan.
 * Pat: So this time next week we should agree to move forward or have bigger revisions to discuss.
 * Brad: This is also a significant amount of effort to make the change for. Should we be adding this in parallel to the existing infrastructure (`static_init` and component interface)? Or replace all in one?
 * Amit: How many components do we have? Looks like 22. Good question. Probably depends on how hard it is to replace them.
 * Hudson: Could we have a PR with the new interface without removing the old one? New components will move to the new one and eventually we'll remove the old one.
 * Amit: Yeah, that would work. `static_init` is namespaced in a module.
 * Brad: Well, static init is completely re-architected in this PR. So we would rename it.
 * Hudson: So we could have a PR for the new static init. And a separate PR to update components and remove the old static init.
 * Brad: Possible. It'll just be a lot of effort to update old components.
 
## in_band_liftetime
 * Amit: https://github.com/tock/tock/pull/1646
 * Pat: Less about this issue and more about pushing towards stable rust. This is an example of a feature that's on the line, where Rust might incorporate it someday, but it doesn't look like anytime soon. My proposal is that we remove features that aren't up for at least medium-term implementation in stable.
 * Amit: Does anyone dissent?
 * Brad: Moving to stable makes sense. I don't really care much about the syntactic things, really the big-ticket reasons like assembly that we originally had to be on nightly.
 * Hudson: Given that there's already a PR for this, the longer we wait the more changes there will be to make in the future.
 * Amit: It's also the case that things like using assembly are contained in non-general crates. It's almost exclusively in the `arch` crate. So it's weird, but doesn't affect most people. You could even imagine going to stable by just replacing it with a standalone assembly file.
 * Brad: Looking at the stabilization issue, this will be 2 out of 13 knocked off, but it's still a good goal.
 * Amit: Alright, so it looks like the position is now "yes".

## License headers in files
 * Johnathan: Last time a lot of back and forth, with questions about why we need Tock to have copyright header if Rust doesn't. Example: open titan wants to take code from Tock, modify it, and include it in open titan. With Tock lacking license headers and open titan having them, the result would be that file would only have open titan headers with no note about Tock. They really don't want code in the repository in that state.
 * Branden: Could there just be a header added to the file for this one example when pulling it in to open titan?
 * Johnathan: If you had specific instructions for doing so, maybe?
 * Amit: We could say, if you use or copy one of these files, you may do so by adding the following text.
 * Alistair: Don't you still lose information though? You would lose git history of authors.
 * Phil: We can't assume the file always lives in the repo. We can't behind thinks in the git history.
 * Alistair: So once it's moved, there's no way to figure out who owns the copyright. The original authors are lost.
 * Phil: What's the reason for not doing it? Just that it takes up space?
 * Leon: One question was rendered files. I did some inconclusive research and it seems like it's fine to have in just the raw markdown, but not viewable in rendered form.
 * Amit: Another issue was whether it was okay to have the file headers assign copyright to "the tock project".
 * Alistair: I don't think you can do that without a Contributor License Agreement (CLA).
 * Leon: It is possible that that you don't need to document who owns the copyright. But tracing it may help for a legal discussion. A CLA seems like it is a construct to give assurance but does not limit what you write in the file. And copyright "the tock project" seems that it wouldn't have legal impact at all since it means nothing.
 * Amit: What's the reasonable choice here? If 50 people have edited a file, we're not going to list 50 people in the header.
 * Alistair: It's only major rewrites, not just touching a file.
 * Amit: But that's super vague. And only really matters when it comes in front of a judge.
 * Leon: And that's not even touching international laws. I would be for an all-or-nothing approach. Saying these three lines do modify copyright but these three don't is vague at best.
 * Amit: I feel strongly that we don't want to be responsible for determining what does or does not constitute a substantial change. Or keeping track of exactly who contributed to any given file. It's plausible that having a label like "the tock project" in the license file or the headers gives us, the contributors to Tock, no legal recourse if someone uses our work in a way we don't approve of. But, all I really care about is giving people assurances that let them get past their legal department.
 * Alistair: The problem is that companies have contributed to this, and they own the copyright to those contributions. You don't want to do something that would take copyright away from them.
 * Johnathan: Open Titan handles this problem by saying "copyright lowrisc contributors".
 * Amit: Which sounds analogous to what we're saying. But maybe the subtle change is important. Seems fine. What do we say now?
 * Hudson: "The Tock OS Developers." So maybe "contributors" is more clear about "anyone who has contributed to the repo.
 * Amit: So popping up, one reason we want to avoid headers, is that it's not clear what we should be adding to the header.
 * Leon: Another problem could be when others want to contribute and we enforce this header.
 * Johnathan: If you require that the Tock header be the ONLY header, that would be an issue for Open Titan.
 * Alistair: This is what most other open source projects do, if you look at linux, they have lists of copyrighted contributors.
 * Leon: We should be careful. Linux may not be a good model for us for licensing, though.
 * Pat: We should compare ourselves to something non-GPL as it makes people paranoid.
 * Leon: I didn't mean GPL specifically, but just that we should be careful about negative side effects of anything we introduce.
 * Hudson: But right now, we don't have any concrete side effects to watch out for.
 * Alistair: Could we just add them to the files Open Titan cares about? Or is that weird?
 * Samuel: Would possibly be more uniform to add to, for example, all rust source files and skip files that are annoying like markdown.
 * Leon: Adding to one file could resolve this one conflict, but if it's not the right choice legally, it could be the worst of both worlds. 
 * Amit: Can you explain why?
 * Leon: What I mean is, if only some files have headers, and having a header or not has legal implications, we would have to deal with both outcomes. Which could potentially be worse.
 * Pat: Doing it to some might have weird intent implications. Such that we "intended" to not apply it to the others.
 * Amit: My original concerns were that one of the funding sources wanted Apache. And then we dual license with MIT when some other people recommended that it would make companies happier. And we said "Tock OS Developers" because we didn't want to name specific people as owners when many people worked on it. What would we want to do with copyright that we, the project, care about?
 * Leon: Given that headers don't change the license, in my understanding, I want consistency.
 * Pat: My thought is adding headers doesn't change license, but may de-risk things for collaborators. And if it helps collaborators and doesn't hurt us, we should definitely do it, even if it's annoying. But as a Tock project author I don't really care.
 * Leon: That doesn't say how we handle adding other people's headers.
 * Amit: That's true. But personally, it doesn't bother me as long as those headers don't add more restrictions on who can use the project.
 * Alistair: If we are adding things, we might have to add "copyright WD" too, below the Tock header. Just a Tock header _could_ be a problem.
 * Amit: I don't have a problem adding additional copyright notices as a way to solve that problem. As long as it doesn't substantively change how people use it.
 * Leon: I haven't seen open source projects with 200 lines of license before the code starts though. And I'd like to avoid that for Tock. Everyone adding headers could get out of hand.
 * Alistair: But it normally doesn't because you really have to make substantive changes. And people don't make that many substantive changes. And rewriting the file removes old copyright notices because it's a new file.
 * Amit: And this would only be for the handful of companies that require it. So likely it would be Google, WD, maybe lowrisc.
 * Johnathan: Google wouldn't need to add to copyright headers, just AUTHORS file. Also, I'm concerned that we can't ever move files from Open Titan to Tock because Open Titan isn't MIT licensed.
 * Samuel: And you think not having MIT would prevent it? MIT wouldn't need a special header, just AUTHORS file.
 * Branden: So it sounds like there are three issues we are discussing. First, how do we take a file from Tock and move it to another like Open Titan, which might need a header or some notice in our repo about how to put a header if you move it out. There's a second question of how to move a file in, or add a header if its necessary, and I don't know what our takeaway there was. And third, for moving files out, how do we account for authors of that file.
 * Branden: So for the pulling in and adding headers, is that a problem for you right now Alistair?
 * Alistair: I don't think so. But if there was a header listing copyright on each file, then WD would also need to be there.
 * Branden: Okay, so the pulling files in doesn't seem like a problem right now. Exporting files from Tock is a problem right now, but we could try an ad hoc solution of telling people to add their own header for us if they need to. Then we could sort-of skirt by for now and see if this solves the license problem for another two years.
 * Amit: For what it's worth, we have pulled in stuff from other projects that we just replicated the license or copyright header for.
 * Hudson: The nrf52 crt1 code is like that.
 * Amit: The same for the SAM4L stuff at one point. Which came from Michael Anderson.
 * Leon: Are we even sure that adding something about headers to the readme is sufficient? We should check that.
 * Amit: As in will that satisfy Open Titan. Yeah, good question. That does seem like the most attractive solution so far. Second one would be adding a simple short header everywhere.
 * Pat: I don't have strong feelings. I feel like the two liner is more adherent to what other people do. Which maybe stops us from having this discussion again. But really I just want this to go away and not think about it.
 * Branden: I'm just worried we're going to get the two liner wrong. I don't know what to put to make people happy.
 * Amit: Okay, I will resolve this offline, including a PR. And will take the wrath of external people who may or may not agree.
 * Hudson: Question for Johnathan about pulling Open Titan files into Tock. Because Tock uses MIT, you think if someone in Open Titan takes a Tock file and makes updates, they won't be able to contribute it back to Tock?
 * Johnathan: I'm not sure, but it's possible.
 * Hudson: But people at Open Titan have already made changes I thought?
 * Johnathan: Those are changes directly to Tock. Not taking an Open Titan file and moving it.
 * Samuel: The problem is that the contributions are differently licenses. So you can't move a file between the two, but the same person could contribute to both.
 * Hudson: If all changes were made to Tock initially, it would be fine?
 * Johnathan: Yes. Also if all changes were made by Googlers. I'm worried about lowrisc or the other contributors to Open Titan.
 * Amit: We could also include files in the repo as long as they explicitly state that they're licensed differently.
 * Leon: And you'd need to change the LICENSE file to clarify that it doesn't apply to _all_ files.
 * Amit: Yes. So the conclusion is that I'm going to think about this and do a PR.

