Tock Core Notes 2023-10-06
==========================

## Attendees
 - Alexandru Radovici
 - Alyssa Haroldsen
 - Amit Levy
 - Brad Campbell
 - Branden Ghena
 - Chris Frantz
 - Johnathan Van Why
 - Pat Pannuto
 - Phil Levis
 - Tyler Potyondy

## Updates
 - Amit: We're making some progress on finding someone to hire to work on CI
   infrastructure, security, testing, etc.
 - Brad: I just pushed a bunch of releases. `elf2tab` and `tockloader` have new
   public versions. There've been some changes that are helpful to have in a
   release. From the OpenTitan side, not too many updates, but Johnathan has
   been working on I2C.
 - Branden: I can give a network WG update as well. This past week, we were
   looking at the tock-ethernet branch. It's existed for a while, the open STM32
   ethernet PR is open against the branch. It's probably time to start pulling
   the branch into main. It made sense when we were trying to move fast, but
   we're stable enough that it doesn't need to be its own branch anymore. The
   other topic of discussion is buffers, discussing how sk bufs work in Linux
   and how we may make our own version. Discussed how the type system could help
   here, such as preventing passing buffers that aren't big enough to hold
   headers. Looking to have Leon prototype something small. Alex and I were
   discussing implementing it on non-network drivers. Display driver has this
   problem, as does the SD card driver. Those are more mature, seems like a good
   place to prototype it then move into the less-mature networking stuff.

## PR #3701 (scheduler trait)
 - Brad: We don't have Hudson so maybe it's not a good idea to discuss now, but
   it's a pretty low-level change.
 - Branden: Do you have a feeling it'll be controversial?
 - Brad: Either nobody is going to say anything and it'll get stuck or people do
   have feelings and we're not going to know.
 - Phil: I'd love to hear Hudson's thoughts. I'm skittish about things like
   "this interface works, but it's not ideal" -- sounds like churn to me. I try
   to suppress that, but I don't always do so.
 - Brad: Where are you seeing that?
 - Phil: #3702 is the issue that describes the problem solved by #3701. It's
   basically like I think we should have a different idea.
 - Brad: You're responding to the general idea it isn't ideal.
 - Phil: Yes
 - Brad: I thought you were talking about a change where someone said "oh it
   works but". I think this discussion started at TockWorld. I didn't quite
   understand what was happening. I thought that was referring to a previous PR
   not a new issue, there was another thread on this.
 - Phil: #3701's first sentence says "this pull request reimplements the
   scheduler interface as discussed in issue #3702". #3702 explains the
   reasoning.
 - Brad: Correct. I didn't even look at the number, I just assumed it was this.
   So that was my confusion. It's a more general version if you will. But I
   suspect it's thinking about the same thing.
 - Branden: We might have to kick the can until we get Alex and co or Hudson and
   co or both.
 - Brad: That's fine, I just think it won't make progress unless we discuss it
   on a call.

## Blog
 - Brad: This is a minor update, but I'm trying to revitalize the blog a little
   bit. Just to kind of maintain a sort of presence for the project. I'm kinda
   sacrificing like depth for more frequent updates. It's only taking me 5-10
   minutes to write a post to highlight things. If there's an interesting PR
   merged you can make a blurb about it and send it my way.
 - Amit: That's really good.
 - Branden: Yeah
 - Brad: It's also helpful to have the `significant` tag to filter on.

## PR #3576 Scheduler
 - [Alexandru joined here].
 - Alexandru: I'm not particularly attached to it, so if there's a better idea,
   I'm all years.
 - Branden: Can you give us some context comparing this to #3701 -- is that
   unrelated or tied into the same issue?
 - Alexandru: I don't know
 - Branden: Oh if you don't know it's related then it's probably a separate
   effort.
 - Amit: It is unrelated. #3701 is about process scheduling, and #3576 is about
   capsule scheduling. If I can summarize this correctly, when there is an
   interrupt, we go into a chip-specific scheduler to call into capsules based
   on which interrupts have fired, but that is unaware of priority or any other
   scheduling choices. You'll basically always prioritize which one you put in
   the match statement, or something like that. Maybe that's not guaranteed, I'm
   not sure. As opposed to doing something more clever. Correction: it's
   whichever one has the lowest number.
 - Phil: This discussion came up two months ago, and I pointed back to something
   that came up four years ago. We've run into it in Cr50. We had a state
   machine, and two interrupts would fire with timing tight enough it would
   always handle a particular interrupt first. There have been other solutions,
   being able to re-order in terms of priority and such.
 - Amit: Do you mean the queue?
 - Phil: #1181.
 - Alexandru: This was done using hardware tools? I see NVIC
 - Amit: I don't remember the specific details, but an approach would be to mask
   the interrupt bits with NVIC priorities or something. Not mess, like filter.
 - Pat: I think NVIC stuff was an optimization. At the time, we weren't doing
   any prioritization, so I don't think these changes rely on having them.
 - Amit: My take on this is as we discussed at TockWorld, I think there are more
   efficient solutions to do this, given some potential changes to how interrupt
   handling overall works in Tock. At least, my recollection of this proposal
   seems like it is probably good enough for now.
 - Alexandru: The discussion on the PR is if we need more structure than my
   ad-hoc approach. This is connected between the chip, arch, and kernel crates.
   Another observation my team has had from writing scheduler specs, the
   scheduler not only makes decisions but also triggers kernel work. Somehow the
   scheduler does more than just makes the decision.
 - Amit: I'm not following that. You're saying w.r.t. your PR that a critique of
   your PR is that it leans into that coupling even more?
 - Alexandru: Exactly. Right now, the interrupt handling is a mixture between
   the arch crate, the chip crate, and the scheduler. The scheduler, besides
   making decisions, has this function to do kernel work. The scheduler runs the
   kernel work, instead of the kernel passing scheduling decisions to the
   scheduler. The scheduler needs to know the chip crate.
 - Amit: I see.
 - Alexandru: I think this is Ioan's PR from yesterday. I think he was saying
   something related to that. This is connected with my PR, because in my PR the
   problem of knowing the interrupts is an intricate problem between the arch,
   chip, and scheduler. Brad's suggestion at the time was to have a better
   infrastructure for that.
 - Amit: I think I understand what you're saying about the inversion of control
   for the scheduler, but that in itself doesn't seem to impact the ability to
   use the system. It's more of a design improvement?
 - Alexandru: Right
 - Amit: Hopefully it would pay dividends down the line, but it's not like "oh
   we can't do this thing because of the design". The interrupt priority for the
   scheduler is an important feature to have.
 - Alexandru: You have a good observation that my PR is not about interrupts but
   about executing specific capsules.
 - Amit: Right. But there are, in cases that you're interfacing, there are
   certain kernel functionalities that need to take priority. I don't know
   they're necessarily contradicting each other, there's a form that's getting
   things done now. May not be the most beautiful in the long term, and a more
   sweeping change would include revising how this is done, but within the
   current framework we can evaluate on whether it improves more than it breaks.
   If we change the scheduler more broadly, maybe that impacts how this is
   implemented. Basically, I'm suggesting we shouldn't block Alex's PR on a
   potential longer-term design discussion. Timelines are different.
 - Alexandru: I'm okay with that, but
 - Phil: There are mechanisms for this in the hardware but we're not using it,
   we're just scanning the bits. To toss out an idea: we have interrupt priority
   levels, we can keep multiple sets of bits, as long as the number is small,
   and go from there.
 - Amit: We're currently using the hardware bits.
 - Phil: When you scan them, you don't get the priorities. If instead of one set
   of bits, you had say four.
 - Amit: Broadly speaking, that's the change Alex is approaching.
 - Alexandru: The problem with the hardware approach is I might not want to
   execute one of the interrupts. Executing the interrupt is just the bottom
   half.
 - Phil: That's why you disable them.
 - Alexandru: Well, yeah. Technically I could disable them and mask them in
   hardware. You're saying, if I don't want to execute that, I could disable
   them in hardware, but then I don't know if they're pending.
 - Phil: What I'm saying is that instead of using the interrupt pending
   register, you'd maintain software state. In the interrupt handler itself, you
   would index into the bitmask for that priority to set the bit. You could
   always disable the interrupt, then it wouldn't set the bit.
 - Alexandru: I need to do scheduling decisions like "as long as I have CAN
   interrupts I will never execute UART interrupts".
 - Phil: The way I would expect that to manifest is to put the CAN interrupts
   into a higher-priority bitmask, then when I check interrupts, check that
   bitmask first.
 - Alexandru: I need to take a closer look on how this works.
 - Phil: Maybe we should exchange some email or something. There are some tricky
   parts to it, but this makes more sense. This is a problem we have encountered
   many times. In Cr50 it used to be that if any interrupt fires your interrupt
   handler can be triggered. I could disable interrupts 4, 5, and 6, but if I
   then enable interrupt 4 and 4 fires, the system will see them all pending.
 - Alexandru: I need to take a closer look at how the hardware interrupt
   handling works. Thanks for helping.
 - Phil: This is a recurring problem so it would be great to solve it.
 - Alexandru: Regarding the second PR, I'll have to take a look. Can't currently
   do a detailed design. I'll read the PR and comment on that.
 - Phil: I have a comment where I said I was worried, but I read through it and
   am not concerned.
 - Alexandru: I'll take a look.
