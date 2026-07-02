# Tock Meeting Notes 2026-07-01

## Attendees
- Amit Levy
- Brad Campbell
- Branden Ghena
- Hudson Ayers
- Johnathan Van Why
- Leon Schuermann

## Updates
- Brad: Pushed a new tockloader release.
- Johnathan: Decided to move forward on tock-registers without
  UnsafeRead/UnsafeWrite, can introduce those once we have a need and design
  input. Fundamental concern highlighted by Amit that unsafe operation traits
  don't know their safety invariants is still there. I have a few changes
  collected that I want to do after it's merged before Tock uses it.
- Amit: There are several outdated changes. Merging one with two approvals.
- Branden: Network WG has been in hibernation for two months, everyone super
  busy.
- Amit: What's the update on IPC?
- Branden: Same as before. There's an initial design for communication, needs
  engineering hours to implement. Big thing on my todo list for summer. The
  shared memory interface does not have a full design, just vague ideas, but the
  idea was to implement single-copy Allow-based API first and can then move onto
  a shared memory design.
- Amit: Any design doc to look at?
- Branden: There's a PR for that. #4680. Fairly up to date, Leon and I have been
  adding commits. Will eventually become a TRD.
- Amit: Focus of the cryptography WG long-term has been the cryptography stack.
  The first chunk, getting close to proposing a redesign, is a new AES HIL. We
  hope that to be an omen and testing ground for other cryptography HILs. Design
  spearheaded by Bobby, the biggest change is it looks like the interface
  Hussain presented at TockWorld last year. The underlying driver calls to the
  client to request key/IV/payload to avoid unnecessary copying. It's generally
  pretty good.

## AI Policy (#4838 and #4834)
- Amit: #4838 is a month and a half old, specific change to contributor document
  for AI policy. #4834 updates the PR template with different language, pointing
  to the contributor documentation. Probably makes sense to merge together. We
  were blocking on Leon last time, Leon can you reiterate where you stand?
- Leon: I think my concerns have been mostly addressed. I don't like us making a
  statement about the utility of AI. I also want a statement about copyright
  still being contributors' responsibility. I think there are agreement on
  those, if I'm not mistaken.
- Hudson: The first is addressed.
- Amit: It doesn't say anything about copyright.
- Brad: Leon, why don't you open a new PR that discusses the copyright. We
  should have a clear and direct policy.
- Johnathan: I'm shocked that it was not already obvious to everyone that
  licensing is the contributor's concern. That's probably an argument for an
  explicit policy.
- Amit: My one remaining nit about this is that AI is a really vague term. What
  does it mean? We mean something much more specific. Code completion is AI. I
  don't want people disclosing their use of LSP servers.
- Leon: I've been reading a lot of policies from a lot of different projects
  recently. I think the general takeaway is that a vague policy that is
  fine-tuned in the future is better than a hyper-specific one. We can make a
  similar observation about the naming of things. If anyone is confused, it is
  simple to refine the policy.
- Amit: Yes. Are we ready to merge the policy?
- Johnathan: I want to see the copyright thing addressed.
- Amit: Brad suggested that copyright should be a separate section.
- Leon: I disagree with that. There should be a cross-reference.
- Brad: I'm not trying to be prescriptive about what this new PR should be.
- Amit: It definitely is not limited to one sentence in the AI section.
- Branden: But it can add a sentence to the AI section.
- Amit: But it deserves another section.
- Hudson: It feels like our documentation is sufficient if we need to add a
  bunch of reminders everywhere about copyright.
- Amit: I agree with Johnathan's intuition that it is not confusing, that
  everyone who has opened a PR has been clear about expectations. There's a
  concern that adding this language is tainting previous contributions?
- Hudson: Yeah. Maybe that's ridiculous.
- Amit: It's clearly stated in the README, and it's okay to clarify elsewhere.
- Leon: What is different about the transition now is there is actual active
  debate about whether or not these tools in general readily duplicate existing
  copyrighted work. I wanted to avoid that debate, but to shift the
  responsibility to the person who is already supposed to be taking that on
  which is the contributor. From other projects, the fear that people have is
  the tools make it easier to shift the blame elsewhere. That is really the
  thing that I would like to see us address and what I see other projects trying
  to address. Ultimately, I agree it is a separate discussion and I am happy to
  open a PR.
- Amit: Johnathan?
- Johnathan: I agree with what Leon said. I can't take notes and also review the
  PR, I can approve after?
- Amit: Lets pause for a few minutes to take a look.
- Amit: (After a few minutes). I'm hitting merge. Next, to #4834. It changes the
  PR template language around AI use. I think it is asking the contributor to
  list the LLMs used for the PR and to tick a box saying they've read and
  followed the AI policy. As opposed to the current language, which is a tick
  box saying the PR description details AI use and I certify the contents of the
  PR. Brad, could you explain the reasoning/need for the change?
- Brad: We want a more direct thing that requires a response. This is more clear
  -- if you used AI, tell us.
- Leon: This is a follow-up to a PR I created. I agree with the motivation. I
  don't think this delivers an indication of whether gen AI coding tools were
  used. The Rust project deliberately want a data point they can track, so they
  use check boxes for it. My proposal had that data point because it had two
  checkboxes, versus this checkbox that I would always need to check.
- Branden: Yes, you do always need to check the box. Also, it does have a place
  where you write down when you used LLMs. It's just freeform, which I think
  fits better than a checkbox. It's harder for trivial statistic collection.
- Leon: Thinking about it, maybe I should rescind my comment. Maybe it's easy
  enough to trivially detect a freeform response and do some basic analysis. We
  could denote -- via Markdown comments or code backticks -- an area for them to
  respond.
- Branden: It has that.
- Leon: Yeah.
- Branden: If you look at the whole template, it has that in every section.
- Leon: I meant a set of delimiters. But sure, I think that works. How should I
  change this if I don't use LLMs.
- Branden: I would put `AI use: none` and then check the box. In the same way as
  I would delete the `This pull request is tested by...` and then write an
  answer.
- Leon: Sure
- Branden: Other people might do different things.
- Amit: One downside is it does make it more work if you're not using AI.
- Johnathan: I think that is necessary to get an accurate signal about AI use.
- Branden: I agree, this is intentionally raising the bar by a few character
  strokes.
- Amit: I'm thinking purely about UX. With checkboxes, you can open the PR and
  just click on the checkbox, as opposed to scrolling down and editing text.
- Branden: I think ideally this would be a radio button, they just don't exist.
- Leon: We're having this discussion because when we proposed having two
  checkboxes, all PRs are marked incomplete in GitHub's task tracking. I
  personally blanket ignore them, because many other projects have many
  conflicting check boxes. I understand if this breaks people's workflows, but I
  don't know if that's a concern we should have. It's superior in every other
  way.
- Johnathan: Is actually important for anyone? Someone brought up the concern
  but it might've been for someone else when that someone else doesn't exist.
- Brad: I thought it was important.
- Johnathan: Can you clarify?
- Brad: For checkboxes to be complete.
- Amit: I don't people see what problem arises when checkboxes are complete.
- [Brad said he wasn't in a good spot to talk much]
- Amit: Maybe we can defer this, and instead use the remaining time to review
  the next PRs.

## tock-www PR 133
- Amit: tock-www PR 133 is adding a DmaSlice blog post. DmaSlice blog post is
  not super deep academic, but I don't think we have time to do better.
- Leon: Agree. I left one comment, can we merge once that's addressed?
- Amit: I think so.
- Leon: My suggestion is about a comment that I wouldn't have said anything
  about if it didn't exist, but it does and it's not quite correct.
- Amit: I've created and applied a suggestion. Approved and merging.

## libtock-c PR 568 (YWF)
- Branden: If we've got a second for YWF, I have a question for Hudson/Leon.
  PR 568 is almost entirely uncontroversial, Brad did it with Claude because it
  is very mechanical. It's weird in two places, I think because of existing
  weirdness. UDP send is weird. It takes a buffer, length, and destination. The
  destination has been totally unused for 8 years now.
- Leon: Is this with 6LowPan.
- Branden: net/udp.
- Hudson: I believe it is over 6LowPan.
- Branden: Do you have any memory of what is going on.
- Hudson: This is slightly familiar. Man, it really has been 8 years since. I'll
  look at this some after the meeting, I won't have an answer right away.
- Branden: I think we should raise this as a separate issue because this is not
  the PR's default. I'm not even sure it is *wrong* to not allow it. We've
  changed the interface, so it might be correct, but we should change the
  interface.
- Hudson: It might be done when you call `bind`.
- Branden: I looked at example apps, and they only passed it into UDP send.
- Amit: You'd expect bind to give the local address. What's the likelihood that
  this is broken?
- Branden: IDK, do we ever test UDP?
- Hudson: Recently, no. But since 2018? I would expect yes.
- Branden: I think you'd notice. Send with no destination? Should do nothing.
- Amit: Unless it defaults to broadcast, or the receiver doesn't check
  destination address. It will be received if it's a single hop.
- Hudson: Tyler added these `hack: this is not static` comments in a few places.
  Maybe he knows. I'll keep looking after the meeting.
- Amit: This is probably a semantically-fine transformation, but we don't want
  to do it.
- Branden: I think it's orthogonal to this PR. It points out a problem that we
  should probably care about, but it's not the transformation's fault. The
  transformation is still valid.
- Amit: Yeah.
- Branden: I think that's all this PR needs.

## PR 4899
- Branden: Brad spotted an issue in our asm uses. Looks like an obvious fix to
  me. Should I merge?
- Amit: I think the assembly makes sense as well. The only thing is r3 isn't
  being used after the test of
- Branden: You've confirmed that?
- Amit: Yeah, it's not.
- Branden: Cool, you can click merge on that.
