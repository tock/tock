# Tock Core Notes 2023-09-08

## Attendees
- Alexandru Radovici
- Amit Levy
- Brad Campbell
- Branden Ghena
- Hudson Ayers
- Johnathan Van Why
- Leon Schuermann
- Pat Pannuto
- Philip Levis
- Tyler Potyondy

## Merging PRs
- Amit: After a PR has been approved, who clicks the merge button?
- Brad: Has something changed with merge queues relative to Bors that changes
  the answer?
- Amit: I've noticed some PRs sit for a while without getting merged.
- Branden: It's hard to tell when we're done approving and should merge a PR.
- Brad: I'm pretty aggressive on merging PRs. I like to either merge, or tag it
  with something orange or yellow, meaning we're waiting on something.
- Leon: When I'm unsure if a PR is ready to merge, I think last-call is a good
  way to indicate I'm going to merge it soon.
- Amit: Two core team approvals has been a rule. I agree that something like
  last-call is a good "oh I'm not really sure". If there's two approvals and
  last-call is over 2 days old, just merge it.

## Updates
- Amit: The NSF funding finally came through. We're going to start hiring
  people.
- *Everyone*: hooray!
- Brad: I spent a few minutes trying to update the HOTP app from the tutorial to
  use our KV store. If we can get that added then that fixes the credential
  problem and means we can directly port it to `libtock-rs`.
- Amit: And it would be a good litmus test for the KV store.
- Leon: I have ported a minimal version of that to `libtock-rs`. There were some
  issues trying to read from console. I didn't want to break the USB interface,
  so reading the console requires a per-character read, which causes it to miss
  characters. If we can sort this out sometime, I think we could reasonably push
  a minimal example somewhere.
- Branden: Networking WG update: Spoke about buffer management, thinking about
  questions about goals and what the problems are. Discussed how buffers are
  used by hardware. Discussions still ongoing. Recurring topic is patterns that
  are sometimes async, such as at the bottom of Ethernet. On-chip peripherals
  respond instantly while peripherals over SPI fail asynchronously. Interfaces
  have to either support both or everything has to be async. We don't have a
  good answer or deeper thoughts, I just see it coming up repeatedly.
- Johnathan: At OT call, I discussed some work I'm doing on an alternate API for
  `libtock-rs`. Still ongoing, may not have time to finish.

## Transition to list.tockos.org mailing lists
- Amit: Our mailing lists are currently semi-scattered. tock-dev is basically
  dormant, I believe there's an OpenTitan one on Stanford's mailman, Helena on
  Stanford's mailman. It's been working fine, but if you want more it's a
  bummer. I created lists.tockos.org. It's mailman 3, which is way nicer than
  mailman 2, and I suggest we transition some or all of the mailing lists over
  there.
- Phil: I'm happy to send you the full member lists. It's a nice historical
  thing, but it's important that things transition to a neutral place.
- Amit: Can login with GitHub or Google accounts now in addition to creating an
  account.

## Uberconference
- Amit: We've been using the same room for all meetings. I created an
  organization called Tock. We can create specific rooms for different meetings,
  and have people who are not me be administrators. The downside is the URLs
  have random nonces in them so you'll have to store the URLs.
- Phil: We use Uberconference because it's what we've always been using. We can
  switch if something is substantially better.
- Amit: Personally I like Uberconference's ease of use by dial-in, but I don't
  feel strongly.
- Phil: Sounds like you don't see anything substantially better.
- Pat [in chat]: IIRC, the original motivation for UberConference was better
  (free) international dial-in support — I think that has long-since equalized
  however

## Tagging/devolving PR review by WG
- Amit: Increasingly, there are PRs that should be reviewed by a subset of
  people, which includes people not on the core WG. Maybe it's fine for some of
  these things to move forward in ways that do not meet core kernel standards.
  We occasionally tag things with WGs but we could imagine evolving that. Maybe
  if something is within the purview of e.g. the networking group, then the
  people in the networking group should be reviewing it.
- Brad: I agree. I think the networking group should open a pull request to
  automatically tag their PRs, which is what OT does.
- Amit: So there's a way to automate it?
- Brad: Yes. The only thing that is tricky is when there are other labels, they
  fall into a bucket, and not a lot of PRs only have WG labels. #3660 is a good
  example. If there's a single label and it's `WG-`, then it makes sense for
  only them to look at it.
- Pat: Everything should have a working group tag, with a default to `wg-core`.
- Hudson: You can tag the working group as a reviewer, as an alternative to
  using a label. We have OT and core GitHub groups. Could make one for
  networking as well. A reviewer from those groups will satisfy it. Downside is
  when you're scrolling the list of PRs, it doesn't show up. Label shows up in
  PR search results. Could do both.
- Amit: Both might be fine. In general, the labels are useful -- I like to
  filter -- but maybe we can experiment it. I wonder if something like #3643
  with a ton of labels — it's not really an OT WG thing. Networking group notes
  (#3661), has the doc label but it is not documentation but WG notes. If the WG
  members feel it accurately reflects the meeting then it should be merged. Are
  there PRs where a specific WG tag wouldn't be able to indicate who is
  responsible?
- Brad: How about #3597?
- Pat: I think it's wg-opentitan but it is P-significant which pulls in more
  attention.
- Brad: It's a good example of a related issue because it also has the kernel
  tag, but maybe not a good example for this purpose. With this PR, the kernel
  changes are not significant.
- Amit: My judgement is that this is a wg-core PR. Because the important
  stakeholders care about RISC-V, then they should also be consulted. So this is
  like a both. There are going to be things that touch design-level things in
  the kernel and significantly affect the design of certain subsystems.
  Hopefully not frequent but I expect a lot around major releases.
- Brad: There's an advantage to the person opening the PR to have them be
  smaller-scoped. I'd like to incentivize that behavior.
- Amit: Generally, WG members have people who we trust in them. So we should be
  able to notice important changes in PRs sent to working groups.
- Brad: I like the auto label, I don't think it's scalable for people to tag
  their own.
- Pat: Also they can't, you need triage permission in the repo to apply the
  label.
- Brad: The labeler will do what the labeler will do.
- Hudson: Yes, it will add and remove tags, but that's not nice when people want
  to manually add labels themselves.
- Branden: I would note that automatic labelling will be woefully insufficient.
  I'm trying to do it for the network group. I have to find every file in the
  repo that has something to do with networks. It's not as clean as OT. E.g.
  there's a radio driver in `chips/nrf`, so it's a bunch of files not
  directories.
- Pat: The labeller is going to have to parse files and look an the HILs inside
  of them.
- Branden: I can manually list them, it'll just get out of date. I'll have to
  keep updating them.
- Brad: Either way it'll be out of date. I think automatic is better.
- Branden: I'll have a PR soon.
- Brad: I wonder if we should get rid of the kernel tag, which we kinda use but
  don't really, and replace with wg-core. My proposal is a bit different from
  Pat -- I don't think we should tag *every* PR -- but we should tag important
  ones.
- Hudson: I think manual tagging is ideal for the core WG.
- Brad: I disagree. We often see the title says one thing, and the PR changes
  others. Having the labels show up as a warning is important.
- Pat: My motivation for everything having a tag is so everything has someone
  responsible for it. Currently, there's the fallback of the core WG having
  responsibility for untagged things. Putting tags on all makes it more
  explicit.
- Brad: I see what you're saying. I don't quite like that because I want it to
  be automatic. I'm kinda reinventing `significant` but doing it automatically.
- Amit: So wg-core would be at least everything in kernel/ and arch/, right?
- Brad: Right
- Amit: I'm just trying to clarify what you're suggesting.
- Brad: That's a reasonable starting point. Perhaps I've sidetracked the
  discussion a little bit. My main takeaway is that imperfect automation is
  better than manual processes we can do perfectly.
- Brad: Hudson left, but an open question is whether we can have the labeller
  tool not remove tags? I'm wondering if we can make it leave manually-added
  `wg-` flags.
- Amit: I move that we switch towards Brad's version of the suggestion, with the
  open question of exactly how to do it, but auto-labelling WGs. Have wg-core be
  auto-labelled for everything in kernel/ and arch/. Essentially evolving the
  decision about things tied to a particular WG to that WG.
- Brad: SGTM
- Amit: Okay. Maybe an action item is to figure out how to do with w/ or without
  GitHub's auto-labeller. If we can, can we make sure that the auto-labeller
  will not remove a wg- tag.
- Brad: Right
- Amit: If it's impossible to do then maybe we figure something else out. We can
  probably fork the labeller action and modify it, e.g.

## Allow Command 0 to return Success w/ value or lock it down (#3626)
- Brad: I added #3626 to discuss.
- Phil: Sounds good to me. Should we finalize TRD 104 before we add updates?
- Pat: I have 2 PRs open against 104 right now.
- Phil: I think it is important to document the 2.x system calls.
- Pat: I could update this to finalize 104 and create a draft PR that deprecates
  104 and target against that.
- Phil: I don't think we have to deprecate, it can be an extension. Command 0
  should be in the finalized version, but adding system calls should be an
  extension.
- Phil: I think the 104 text was written to accomodate that.
- Brad: It sounds like #3626 should be included in 104 before finalizing it.
- Phil: I think so. Incorporate #3626 then finalize.
- Brad: Okay
- Phil: I'll read through 104 to make sure we can do the yield-wait-for.
- Brad: Are people willing to approve #3626
- Amit: I've already approved it.
- Phil: I am happy to approve it.
- Leon: I'll do a final review today, but generally LGTM.
- Brad: I don't know about this RFC thing. If we deleted the line, I approve. We
  have one TRD with one bullet point with this, which seems ad-hoc.
- Phil: I agree the bullet point is weird.
- Pat: The rationale is a lot of the TRD has exposition on the motivation behind
  stuff, which is quite long.
- Phil: So write 3 sentences that summarize it. The TRD tries to be concise in
  giving reasoning.
- Pat: Maybe a two-sentence reasoning and a pointer to the RFC discussion?
- Phil: There are huge discussions about 104, and there aren't pointers to it.
- Pat: I think that is really unfortunate. How will people find it?
- Phil: Go look at the PR.
- Pat: I tried to include links to the PR.
- Phil: Maybe this should just be a reference at the end of the TRD?
- Pat: Alright, moving to a reference.
- Brad: Makes sense to me.
- Phil: Example, in TCP, why is the initial window size 2? There was a long
  discussion which is not in the RFC. You have to go look at the mailinglist to
  see it.
