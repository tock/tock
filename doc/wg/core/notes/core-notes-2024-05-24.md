# Tock Meeting Notes 2024-05-24

## Attendees
- Branden Ghena
- Hudson Ayers
- Amit Levy
- Leon Schuermann
- Pat Pannuto
- Tyler Potyondy
- Alexandru Radovici
- Johnathan Van Why
- Brad Campbell
- Alyssa Haroldsen


## Updates
### Tock Peripherals Update
 * Johnathan: Tock register draft PR is out. https://github.com/tock/tock/pull/4001 Implementation only partially complete. Adds new `peripheral!` macro, which solves two issues: 1) unsoundness of having references to MMIO memory and 2) the existing registers implementation has no support for unit testing. It also allows for unsafe registers and allows for registers to be config'd in or out of existence.
 * Johnathan: Design documents are up for now. So that's what should be reviewed. Some proc macro parsing is complete, but would change if the design changes.
 * Johnathan: Goal is for most people to agree on design. Then I'll get to porting a driver over as an example. Then last a few people, Leon and Alyssa, can review some of the macro implementation.
 * Hudson: So this would be our first thing in Tock to use procedural macros? (yes) Are we pulling in external dependencies for that?
 * Amit: I believe these are build-time dependencies, not run-time dependencies.
 * Johnathan: Correct. 
 * Amit: So this is morally similar to pulling in the CC package to compile C binaries, or something like that. My perspective is that this does NOT constitute having external dependencies in Tock, but rather in the Tock build infrastructure.
 * Pat: We once decided against proc macros, like back in 2017.
 * Amit: I think they were very unstable in 2017, and aren't now
 * Johnathan: This definitely needs proc macros to be manageable.
 * Pat: Agreed. I just meant we should look to see what the old rationale was.
 * Amit: I do think it was just stability stuff at the time
 * Pat: Also, I don't want a full porting guide, but I do want an example of how much code really changes in practice. Seems to mostly be additional parentheses. Maybe a little paragraph explaining that would be nice
 * Amit: I think the comments in peripheral.rs show some examples of what this would look like.
 * Johnathan: Yeah, the README is supposed to be the high-level what it can do. Then the comments have all the features and syntax.
 * Amit: From the embedded working group discussions at Rust-NL, this design would address many if not all of the shared pain points that many people have about embedded Rust and MMIO. Those folks are primarily interested in automatically generated stuff from svd2rust. But both at the implementation level of avoiding unsoundness and syntactically of having a clear mapping of registers, I think this is in broad strokes a VERY good improvement. And hopefully not just for Tock.
 * Alyssa: Agreed. This is a massive improvement.
 * Hudson: Do you think the embedded Rust people would consider pulling it in as a crate?
 * Amit: Maybe. They definitely want auto-generated bindings from SVD files.
 * Johnathan: I would love the input to be SVD. But I don't really have time to understand that format. So an svd2rust that outputs this syntax would be useful. There could also be multiple syntaxes in proc macros, so we could add stuff if it would enable them
 * Leon: One takeaway from Rust-NL, they're dealing with huge register files, and they didn't want compile-time expansion because it took so long. Not sure if they actually measured this.
 * Amit: Interesting. I wonder if it's because they're pulling everything from the SVD no matter what, or if their chips are different.

### CPSWeek Tutorial
 * Tyler: Tutorial update. The people there really enjoyed the tutorial and the format. We had the screen working and touched a lot of parts of what Tock offers for apps and networking. Major plus is that we have the tutorial now. We did face some issues with the VM image freezing, and we need to figure out how to set people up with the build system. It worked fine for one person, but kept freezing and was unusable for others. We worked around it, but it wasted time, and was really only okay because of small attendance.
 * Tyler: We also had a bug in OpenThread with libtock-sync that we didn't catch until we were using the network for quite some time. I'll be patching those soon, sending was hanging.
 * Tyler: We also had small attendance: only four people. So thinking for what venues to approach and how to advertise would be helpful for future tutorials.


## TockWorld Registration
 * https://world.tockos.org/tockworld7/agenda/
 * Amit: We're ready for people to register. We have an agenda for all days, although some possible changes still. The agenda is pretty exciting actually.
 * Amit: So, we should encourage people to come. So it's time to reach out to people and propose that they come. For undergraduate students, we'll be able to do some travel grants.
 * Amit: Keynote is Florian Gilcher from Ferrous Systems talking about Ferrocene. But also Bobby Reynolds talking about x86 Tock port for Pluton. And Irena Nita talking about WebAssembly TockOS. And Lawrence Esswood from Google about porting Tock to a CHERI-based system. And Amalia Simion on multiplexing serial messages.
 * Alex: I'm hoping that some of that stuff gets open-sourced. Very excited about it. There's a lot of duplication of effort going on. We'd also benefit for the x86 stuff for education.
 * Tyler: Amit mentioned student travel grants. We also have a growing group at UCSD who are working on Tock. How should we direct them to registration?
 * Amit: Pat and I will figure out how best to handle it and advertise broadly.
 * Alex: Do we need to register for a ticket as well?
 * Pat: Yes, if you're coming buy a ticket


## New Kernel Release
 * Amit: We should do a new kernel release. Mostly because it's been a while and it's good to keep it as some pace. There are changes and fixes to important systems. So one question there is do folks agree?
 * Amit: And then are there particular features that are around the corner which we would want to include in a release? For example, IF the peripheral stuff was close to being done we'd possibly wait for it. But for something several months away, we should just move ahead now.
 * Amit: We should also use this as an early testing ground for some automated hardware testing on what we have set up
 * Hudson: Is there anything that people feel needs to get in?
 * Branden: Maybe some of the timer stuff? I didn't follow what the state of that is
 * Leon: That's close
 * Amit: Okay, those are clearly in then.
 * Pat: What about 15.4 interface stuff and known bugs?
 * Amit: I do think getting 15.4 stuff in before a release would be nice. But I don't know if it should be blocking. It's not central for donwstream users.
 * Hudson: I think the stuff with known bugs is likely in libtock-c instead of the kernel
 * Tyler: That's correct
 * Tyler: I don't know if we want to block on it, but Brad is working on splitting up the 15.4 stack.
 * Brad: No, let's not wait on it. That's going to have lots of little future changes, so we don't need to force it right now.
 * Hudson: Last week we also discussed not tying together libtock-c and kernel releases. So we could tag a release and start testing soon. I'll create a tracking issue
 * Brad: already made it https://github.com/tock/tock/issues/3197
 * Hudson: I'll add some stuff to that and update it.


## Reframing Core Call and Meetings
 * Amit: Background here is that the core calls over the years have had multiple purposes. One is to catch up on PRs and core WG business like Tockworld. The other has been outreach like having people jump in with questions. It's been invite-only, but pretty open to pulling people in. In practice, it's really just been the core folks anyways.
 * Amit: So, we should be more explicit on the role of these meetings so they can be most useful for that. Probably making decisions on PRs that can't happen async over the PR.
 * Amit: Other background is that as I've made more effort to talk to Tock users out there, I'm hearing a lot of similar things: complaints/feedback, questions they didn't know who to ask, and lots of good-to-know stuff like custom features or use cases that aren't going to upstream but could direct our efforts. So, instead of me doing one-on-one meetings, it would be good to have a forum to give those voices some space. I do NOT think the core calls would be right for that. Some of it happens on Slack, but that's usually more academic people. And Slack isn't good for long-term things.
 * Amit: So some suggestions. One would be to keep the core calls an actually make them more directed towards voting/deciding on stuff. Those can be more boring and efficient, and could maybe even move based on schedules and timing.
 * Amit: In addition, we might consider a community call or Tock office hours, or something. That only a few people would regularly be on, but people know they can join. The ESP Rust people do something like this. So it would be oriented towards communicating from the core working group to others, such as about big features like Tock peripheral changes, and could be a Q&A feedback thing from users.
 * Amit: Another suggestion would be other communication tools? We don't use the mailing list much, but we could use it more and advertise it more. Some people didn't know they existed, so we should advertise better. Maybe other tools like Discorse or Zulip or something to avoid email but still avoid live chat.
 * Hudson: What do you think that other medium should be?
 * Amit: I don't know. I hate all these tools, but hate some less than others. I found that when Rust moved to Zulip that it was a bit tough to deal with, but it does seem effective. Similarly, Discorse is basically a glorified mailing list.
 * Brad: I think we can just do our best for whatever people use. I'm more concerned about the effort to lead all this stuff. Without staff support, we could do something really well for a month or two, but it'll fall off whenever someone gets busy. Everything you're saying sounds great, but how do you make it work without staff support?
 * Amit: I agree with that. We are working on it, but we're not sure who that staff person would be. Could be one of us, but that's not a long-term solution. For how we can pay them, we do have funds now, but it's a tad tricky to pay people who should clearly be remote. Presumably this would be one of the perceived benefits to financially support the Tock foundation, but that's a chicken-and-egg problem because it doesn't exist yet.
 * Hudson: Okay, so we could adopt a new tool and maybe that would help somewhat. But it's also not clear to me to what extent that would do anything. Zulip works well for Rust because it's ONLY Zulip and Github.
 * Amit: There is a Discord actually, and migration to Zulip has happened slowly and organically.
 * Amit: Actually, I'm not really suggesting switching tools unless someone has a magic bullet. I think more important is to focus the Tock core calls, and set up some Tock office hours time. I'd be happy to lead the office hours for the next few months.
 * Hudson: I am pro this community call idea.
 * Branden: Needs some specific guidelines and suggestions for what topics would be handled in the community call. And advertising to people so they're aware it exists
 * Brad: Would risc-v stuff make sense here? That call never quite hit critical mass.
 * Amit: Maybe? Discussions yes, but decisions/promises no.
 * Hudson: Maybe still small decisions, just not project-wide stuff.
 * Amit: The goal being that not attending the community call doesn't hurt your ability to influence Tock for things you care about. And the Core call would still be relatively open, but would be transparent about being boring.
 * Alyssa: Could do a Slack post about it, and emote react to say you are thinking about coming (and cancel if no one does).
 * Hudson: Especially useful for students playing around with Tock who just want to ask a couple questions
 * Amit: We'll keep chatting and plan to start doing something like this on the sooner side

