# Tock Meeting Notes 2026-06-10

## Attendees
- Amit Levy
- Brad Campbell
- Johnathan Van Why
- Leon Schuermann

## Updates

## tock-registers PR review process
- Amit: Basically, tock-registers is ready. There's a lot to merge, so Johnathan
  has broken that down into a DAG of PRs. We need to decide how we're going to
  review them.
- Johnathan: Basically, said DAG is 6 nodes deep. I've said like "oh, the
  critical path" -- there are several paths that are 6 nodes deep in the
  dependency tree, and it doesn't make sense to merge any two nodes along that
  path, so that's the depth of the project. In the past, significant PR in Tock
  tend to sit for a couple weeks before merge, because we're unsure whether
  we're ready to merge it, until someone brings it up on the core call. If we do
  that here and add two weeks to each PR, that adds 12 weeks to the project, and
  I don't think we want that. What we need to figure out is who all needs to
  look at it and be happy with it before we hit the merge button. I think we can
  decide that for these PRs as a set. Under our code review policy, if we
  consider them significant PRs, then we either need the entire core team to
  review and approve before merging or we need to wait a week each time. That's
  not the end of the world but definitely not the fastest way possible. The
  other thing is if we consider them moderate upkeep PRs, then we need two core
  team approvals plus a 24 hour last call window, which I think is pretty
  reasonable. That kind of makes sense when you think of the overall design as
  significant but already approved, and these are just implementations of the
  already-approved design. I do want to know who really wants to look at it -- I
  don't want to hit merge while asking the question "is someone going to be
  upset that they did not get the chance to review this". I think what Branden
  said last week is that he's busy for now but that he will have time to review
  them in the future and has committed to reviewing. I do want to know if there
  are any PRs that Leon really wants to look at before we move forward. And
  yeah, figure out what the bar for merging and who is committing to look at it
  and make sure they're compatible -- that we don't have a higher bar than
  committment.
- Amit: Do you have a concrete proposal?
- Johnathan: I think my concrete proposal would be to treat these as moderate
  upkeep PRs, which require two approvals and a 24-hour last call. It would be
  good to have volunteers to review the PRs so I know I'm going to get the two
  approvals. I do want to know who specifically will be reviewing the proc macro
  PRs because people might not want to look at them.
- Amit: It feels to me like answering the second question -- when do we click
  merge -- will help answer who reviews each PR. It's a different repository
  than Tock. It's obviously critical to Tock in certain ways, but merging stuff
  into this repo doesn't flip the switch over for the rest of Tock until we
  update the git revision or crate version. So my sense if we don't need to do a
  core team consensus for each PR. We would prefer to appoint a subset of people
  who are reviewing and merging them all. Does that strike others as reasonable
  here?
- Leon: I think I agree. The other thought was that ultimately it is good to
  review them in isolation for feedback on the implementation code, but my
  feeling is that we'll know whether it feels like once we're using it in Tock.
  I'm wondering what the story there will be. I'm not worried at all about
  merging something into the tock-registers repository, especially if we're not
  tagging a release just yet. It would be nice if we could get experience
  porting stuff over before we tag a release.
- Amit: Presumably Johnathan sort of had that, right?
- Johnathan: I mean, I like my design, but yeah. A different item I was
  expecting to work out later was "when do we tag a release/what series of
  releases do we do before we tag a 2.0 release that deletes a lot of the old
  code". But yeah, certainly we should be using it from Tock before we cut a 2.0
  release. There is room to change things.
- Amit: Yep
- Leon: Is it reasonable to take a small driver in Tock, create a draft PR
  against the commit hash of tock-registers with everything merged, then we'll
  get experience.
- Johnathan: I have that already.
- Leon: Oh great.
- Johnathan: It's linked from the tock-registers PR, I usually use the link from
  the tock-registers PR to find it. It's buried because it's 2 months old. It
  passes Tock CI, I've been keeping it updated.
- Amit: Here's a concrete proposal. Johnathan, Leon, and I merge these in.
  Either Leon and I divide and conquer the PRs as we go through them, or we both
  review each PR. I do the first iterations and detailed review, then Leon looks
  at it at the end and hits approve with more of a glance.
- Johnathan: That seems reasonable. Branden has volunteered to review, although
  I think that was motivated more by wanting to get the code merged than a
  particular interest in reviewing.
- Amit: We can still include other people. Branden can be in place of Leon in
  some cases.
- Leon: I really want to take a look at this because I am invested in it, but I
  do not want to be on a critical path. My ask, if people can hold me
  accountable, substituting me for Branden makes sense.
- Amit: Roughly, we are 6 levels deep?
- Johnathan: Yep
- Amit: If you and I have fairly high bandwidth, is there any layer here that
  you anticipate being more complicated than a five hour day to resolve and
  merge?
- Johnathan: No, I think high bandwidth could work. I'm mostly thinking of
  latency. Revisions take longer than other PRs, because I have to make the
  changes on the fully-merged branch, test them there, then port the changes
  over to the other branches and make some tweaks as a manual merge to make
  tests pass. That's slower, if I send that then walk away and do other things
  -- I'm juggling things in my life right now -- then you get the response,
  that's not super fast. We'd get through it relatively quickly.
- Amit: Looking at the first PR, I think one aspect that would be helpful -- the
  first PR is like 8 thousand lines of code changes between additions and
  subtractions, so high bandwidth to help navigate it for example
- Johnathan: Are you talking about PR 11? That's the fully-merged one.
- Amit: Oh. OH. Okay.
- Johnathan: Yeah don't review the draft PR.
- Leon: I don't know whether this helps, but Amit, would it help to dedicate an
  hour to reviewing this at our meeting tomorrow?
- Amit: I mean we can do it. My sense is that will not eat into a big chunk of
  this. We're not going to get through more than one -- there's only two PRs
  open now and they're relatively small.
- Leon: Got it.
- Amit: But yeah, my question originally, is there anything that from your
  perspective might be controversial or risky?
- Johnathan: I definitely can see some of the trait stuff being controversial,
  and I can see the code generation being something that people don't want to
  review. We kind of have that resolved by people committing to review things,
  but those are mainly it. I do want to query -- Amit, you're volunteering to
  review, do you want them to be gated on your approval or are you just
  volunteering to push them through?
- Amit: Hmm, I'm proposing that the PRs should be gated on a small set of
  people. I am committing to be some of those people, and they should not be
  gated on all of those people. So if we divided and conquered, we'd be gating
  on the person who is reviewing that particular PR, if it's something like I
  review each more thoroughly then Leon or Brandon does more of a sanity check,
  then it would be on both of us.
- Johnathan: So you'll be part of a group of which there's a quorum.
- Amit: Yes
- Johnathan: Brad, what is your take. Do you want PRs to gate on you or do you
  just want to be part of a quorum?
- Brad: No, don't gate on me, but I will try to look at them.
- Amit: I'm also anticipating that these PRs will be much easier to review if
  you've reviewed the previous PRs. Because these all fit into the larger
  context, if that makes sense.
- Johnathan: Yeah. I mean if you understand how to navigate the 8 thousand line
  PR, these PRs will be easier to review, but that's such a large project that
  I'm kinda trying to not require that, you know?
- Amit: Yeah
- Johnathan: How about this proposal. Between the three of you plus Branden, we
  need at least two approvals out of that group of four people, then I'm
  debating whether to also include a 24-business-hour last call period.
- Amit: No.
- Johnathan: Okay, so two approvals out of the four of you.
- Amit: Yeah, that seems reasonable.
- Johnathan: Alright, I'm good with that.
- Amit: Johnathan, if we can, lets try to make sure that you and I make progress
  each workday.
- Johnathan: Certainly I can do that on each day I'm working.
- Amit: That's what I mean.
- Johnathan: For me, trying to get the PR reviews through is my top priority. I
  am also continuing development on the PR, I have some other things that are
  not necessary but make it better, but I can sideline that.
- Amit: From your side, it might look primarily like "hey Amit, this this and
  this PR are open, please review, what do you need from me?".
- Johnathan: I certainly can ping you once I've revised PRs and see CI passing.
- Amit: Yeah
- Johnathan: I'll add I still have a trip planned starting late July, but it
  sounds like we might actually be able to get this merged before then.
- Amit: The goal would be -- if they are 6 levels deep and we can do a layer
  every day or two that would be nice and we could get it done in a week or two.
- Johnathan: *Laughs*. Uh, yeah, that would be nice.
- Amit: You're laughing, but like are these layers 60 lines each and
  independently relatively straightforward, or is there genuinely a week of
  discussion and fixing for each of them?
- Johnathan: Let me compute the average line count.
- Amit: If each of the PRs look like 16
- Johnathan: They average between 200 and 300 lines.
- Amit: So you know, it's a lift but I think we can do it with a nimble group.
- Johnathan: Yeah, that is true.
- Leon: I'm not saying that reviewing these PRs is not important, because they
  are a more gradual way to understand the codebase and allows us to catch
  errors only, but ultimately I think if something slips in that we want to
  change, it's not the end of the world. Especially if we wait weeks or months
  before we tag a release.
- Johnathan: Yeah. I did have -- and this was part of the laugh -- I had on my
  TODO list to hold the discussion of what versions we release, do we mark
  things `#[deprecated]`, when do we delete the old code, that sort of thing.
  But I wasn't bringing it up because this was so much higher priority to move
  the project forward that I didn't want to muddle the Matrix chat by having
  both conversations at once so I was delaying it. If we actually manage to push
  this forward in two weeks, that other conversation becomes a priority too.
- Johnathan: Just to reiterate, I failed to write it down, I think we decided
  that two out of four is our quorum to merge.
- Amit: Yeah. Okay, resolved.

## RISC-V 32-bit and 64-bit concurrent support
- Amit: Brad, is the goal here to look through and merge these PRs?
- Brad: I think that would be a good outcome. #4846 introduces our custom
  pseudoinstruction for XLEN store and loads. I'm really glad I opened this PR,
  because I think we cracked the code on how to do this. We should have done
  this years ago. I think that one is ready to go. Then, there is #4851, the
  handful of style one, which makes more things 32- and 64-bit because like the
  `nop` instruction for example, but mostly it is a cleanup pass. The next step,
  if we can get them merged, is to use the new pseudoinstructions to do the rest
  of the assembly that needs to be XLEN-sized. Then the piece after that will be
  dealing with the `static mut`s for tracking whether we're in an interrupt
  context or not. That's the status of things. So I don't forget, one question I
  have: Leon, you were working on a 64-bit board. Is that in any reasonable
  state where we can have it for CI compiling?
- Leon: Yeah, I was going to ask the same question. I have a much cruder version
  of all the things you did locally so I appreciate the effort. I can push that
  to a branch and throw it over the fence. It boots the kernel on QEMU RV64 and
  you can use the process console. It doesn't support loading userspace
  processes because that was a heavier lift and I was running out of time. I can
  throw that over the fence. It did require some tricky engineering to get the
  semihosting right, so I wouldn't say that it is ready for primetime in a PR,
  but if you're working in that direction I don't see why you wouldn't be able
  to base it off those changes.
- Brad: Yeah, that would be great.
- Leon: I can open a draft PR. You can close it or whatever and fork it, I don't
  care.
- Brad: Cool
- Leon: Let me do that tonight. My TODO list is overflowing so hold me
  accountable, but I will write it down.
- Amit: For #4846, I wish we had a review tool that told me which line changes
  are semantic diffs versus comments or spacing or replacing one const name with
  another, because it seems like there's a lot of that. And also, without
  reading the RISC-V manual, I don't know how to go about reviewing this. I'm
  taking Johnathan approval and Eugene's not-saying-anything as saying it's
  good. How should we handle that?
- Leon: For start_trap, I'm not concerned, because I did the change on my end. I
  know it was mechanical and did not need any real engineering effort.
- Amit: I'm asking about start_trap.
- Leon: I can take a look, just make sure it is equivalent to what I have
  locally.
- Brad: GitHub does an okay job of highlighting the characters and line that
  have actually changed.
- Amit: It's a bit silly, but a lot of the changes are just comments. There's
  also a lot of changes like lines 355-370, it's changing 4 to XLEN/8, and then
  the instruction mnemonic. This is a side note for NSF SafeOSE. It's a lot of
  cognitive code to review for the level of change that it is. It'd be nice to
  have one of the tools that we discussed building, that's what I'm saying.
  Ultimately I think this is a straightforward PR.
- Brad: What would help is rustfmt, and not what they're designing. I've spent
  so much time trying to get the assembly in a consistent format because these
  people won't do it.
- Amit: Who's these people?
- Brad: The rustfmt people. They write trivial one-line assembly statements and
  come up with a crazy way to document them, and anyways.
- Amit: Yeah. Okay, I'm going to hit approve on this. Then we have #4851. I hit
  approve on it, Leon said he would review it. I think if he doesn't review it.
- Leon: If I don't review it my tonight, hit merge.
- Amit: #4851. This seems like something I should be able to.
- Brad: The only interesting changes from a functionality point of view are
  making WFI and NOP 64-bit compatible.
- Amit: That's just in the cfg?
- Brad: Yep. Everything else is spellcheck, assembly formatting, and making the
  configs consistent throughout the crate. And simplifying the mcause debugging.
- Amit: Another place where semantic awareness for the PR would be nice.
- Leon: One of the really heavy lifts I did for 64-bit is I ported the PMP to be
  native-machine-word-size agnostic. I think that took me like 3 days so I think
  you want to re-use that Brad. That's in a fairly clean state so I can make a
  PR. That was insane, that took ages.
- Brad: Good to know.
- Amit: #4851 looks good to me. I'm inclined to merge it. And so I did. #4865 is
  merged. Now there's a follow-up.

## Alarm/time HIL overflow
- Brad: Can we talk about #4867?
- Leon: I have to drop off now.
- Brad: Our Alarm/Time HIL has a counter overflow mechanism. You can set a
  client, and presumably get a notification when it overflows. Apparently we
  have not been implementing that because nobody uses it. And so there's this
  idea that we should keep it because SOMEBODY might use it, but we have a
  single chip out of like 30 that actually supports it.
- Amit: Yeah
- Brad: I think the fix in the PR of making it a separate trait is reasonable,
  so at least you get a compile error if it's not supported, which is better.
  But gain, who's going to add it?
- Amit: It sounds like you are making an argument that we should get rid of the
  API.
- Brad: I am, because I think the status quo will always carry weight. I'm
  making the argument -- I don't know exactly what I think -- I'm kinda like
  this was a bad idea. Nobody used it, lets get rid of it. Alarms are hard
  enough, basically.
- Amit: Yeah.
- Johnathan: I'm still team 64-bit timer.
- Brad: Exactly, then it wouldn't overflow.
- Amit: The PR description is misleading now.
- Brad: It is?
- Amit: The PR title.
- Brad: Ah. Okay, lets fix that.
- Amit: Okay. I have absorbed this now, I think. I basically agree, Brad, that
  this probably shouldn't be there. Also, I think that this current version --
  moving it into a different trait -- is the right incremental engineering move.
  Both because we don't know that no-one will use it, and we don't know that
  nobody does currently use it. If downstream users currently use it then this
  additional trait is an easy port. I still don't remember if there's an easy
  way to mark things as deprecated to raise an alarm bell. Maybe we mark the
  trait as deprecated/experimental/whatever, so if people do use it they get a
  warning and pipe up. Then if nobody pipes up by the next-next release we can
  remove it. Is that a thing? Johnathan, do you know?
- Johnathan: Yeah. I'm pretty sure you can, and you will get a warning when you
  try to use something that is deprecated and you will have to manually mark
  that you know it is deprecated.
- Amit: Can you do that for a trait as well?
- Johnathan: A trait I would expect yes. I am wondering about an individual
  function in a trait.
- Amit: You can definitely do a single function, but this would be for a whole
  trait. I'm proposing to keep this PR as-is, which moves the function from the
  regular Time trait to a new CounterOverflow trait, but we would mark that
  trait as deprecated.
- Brad: Well, lets just try it.
- Amit: I'll start trying it right now.
- Brad: That's fine, because it's a HIL, we should have some decision.

## SHA (#4855)
- Brad: It looks like this capsule is a combination of copy/paste and merge
  conflicts. We want to use it for a userspace services demo, and we gotta
  rewrite it. So I did. That's 4855. The failure is because we have not merged
  #4854, because I needed some way to test it, so I wanted to have a kernel
  build that included the SHA. It was never included because it didn't really
  work, and libtock-c needs changes, and all this stuff. This adds a
  configuration board which exposes SHA so we can add a userspace test. But then
  I didn't to copy the configuration board again so I made it a library, which
  is the other PR.
- Amit: #4854 is on its way. #4855 will need to be rebased?
- Brad: Yes. I also renamed the capsule file because we have so many different
  SHA things, but I did basically change everything so I don't think the diff
  would be helpful. I guess the downside though is I didn't have any way to test
  it with more SHA algorithms because we only have support for 256, so it does
  remove the other algorithms if you take the assumption that it ever worked.
- Amit: I added the cryptography WG to look at this, which includes me so I will
  look at it as well, and we'll try to get this through.

## PanicWriter #4831
- Brad: How do we feel about that? The longer we wait, the harder it gets,
  because people keep contributing new boards with the old style. I have to go
  back and ask them to implement PanicWriter.
- Amit: Yeah
- Brad: I will say AI did it, and I don't have these boards. I've looked at it
  and it looks reasonable. It's most boards because I don't know what to do
  about the UART-over-USB boards. These are all the ones that just use UART.
- Johnathan: I mean, I don't really want to look at a 1,500 line long PR that an
  AI wrote for boards that we have not tested anything in. I think there are
  bugs and I think I will miss them. At the very least, have we already ported
  all of the boards that we can test?
- Brad: I'm not sure I can say all, but we have definitely done some, yes.
- Johnathan: It would be really good to prioritize the boards that we can test.
- Brad: That's what we did.
- Johnathan: We should have a standing requirement documented somewhere that
  like "hey, don't" -- we should probably put a comment into all of the boards
  that we cannot test saying "do not copy this, look at X other board for how to
  do it now".
- Amit: This is all so mechanical.
- Brad: When you say "we", are you including more than just me? Presumably Alex
  could test some, but I don't think he's going to.
- Johnathan: Yeah, I was including more than just you.
- Amit: A lot of these boards do exist in the ecosystem, almost all of these
  boards. This is the kind of thing where it is a matter of we need to rally
  people -- usually around release time -- to test these boards that we (the
  three of us on the call) don't have.
- Johnathan: Yeah, okay.
- Amit: This is all like pretty mechanical.
- Brad: Right, it is, that's why I was like let the robot churn.
- Amit: Yeah. To your concern, Johnathan, if somebody copies code for a board
  that doesn't currently work, the parts that don't work would have to be
  changed anyway. It's like the baud rate is wrong, or the wrong USART is passed
  in, or whatever.
- Johnathan: This is kindof a project issue of we have a lot of code that
  bitrots because it's not easy to test. This PR is the worst case example of a
  really large PR that touches a bunch of boards, generated by a kind-of
  unreliable tool, and we don't have tests for it. This is a worst-case scenario
  in a lot of ways for this setup. I'm not worried about this PR breaking a
  board then people copying it, because they'll be testing it on their board and
  they'll debug it and fix it for their board. But I don't know where to stand
  on the whole "just leaving the boards alone and hoping that they keep working"
  versus like doing these refactorings. We probably need to do these
  refactorings, so, I guess I don't have a better way to do it.
- Amit: We would need to have a centralized or distributed test harness so that
  we could at least run tests for all these boards.
- Johnathan: Absolutely. Or, having a lot of the board stuff being more
  centralized and more library based and you invoke a macro to invoke a lot of
  this, which is its own huge engineering project.
- Amit: I think that wouldn't, in practice, help with this PR. If you look at
  it, that's basically what's happening. The changes to each board are of the
  kind of changes that you would make if it was totally macro-ified.
- Johnathan: Alright. Okay. I can take some time to look at this PR. I don't see
  a better solution, so I can take some time to review it.
