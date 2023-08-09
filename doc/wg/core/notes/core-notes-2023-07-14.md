# Tock Core Notes 2023-07-14

Attendees:
- Branden Ghena
- Amit Levy
- Phil Levis
- Leon Schuermann
- Tyler Potyondy
- Saurav K
- Johnathan Van Why
- Alyssa Haroldson
- Alexandru Radovici
- Hudson Ayers


## Updates
 * Tyler: I've been working on Thread networking support as a kernel capsule. I've been able to make good progress and can send/receive UDP messages and I have the encryption library working. All of that came together yesterday. Our goal has been to join an OpenThread network and have a Tock device join as a sleepy end-device. We got all of the handshakes working such that the OpenThread board recognized the Tock board as a child. Going to be polishing up that code and then sending a PR soon.
 * Phil: That's a great accomplishment. We've had big struggles with IP stuff in the past.
 * Amit: When you say you have a UDP packet working, is this before joining a network? I don't remember the Thread stack
 * Tyler: Actually the Thread network stack and our prior work in Tock align well. It's 15.4 on the bottom and IPv6 above that and UDP for the payloads. It uses UDP for some of the fields to make requests to other Thread devices. So the UDP messages are used for joining the network and making parent requests.
 * Branden: Thread gives devices a TON of different addresses. What have you implemented?
 * Tyler: For anyone not familiar, depending on the status of the device, there are different classes of devices and they can be addresses different ways. Currently, it just has a hard-coded IP address based on the MAC address. For the end device that's all I need. There will be a lot of figuring to move from hard-coded values to more generalized things.
 
 * Leon: I've been looking at the PMP implementation for RISC-V and it looks like we haven't been revoking previously allocated regions since 2020. I made a PR that's a hotfix to solve that. It's kind of a major issue. I browsed the rest of the code and it's kind of a mess right now. I've been working on a rewrite to make the code cleaner, more maintainable, and more efficient.
 * Amit: What was the bug you're addressing?
 * Leon: The failure mode right now is that if you configure a process, then reconfigure the PMP to give no access to memory in userspace, the process keeps running just fine. The PR I put out yesterday does resolve this and invalidates regions as necessary. Generally, the different modes are hard to reason about now, which is why I'm doing a rewrite.

 * Alyssa: PR for fixing MapCell safety. It would be good to get more eyes on it.
 * Amit: Yes! That's on my to-do list. Although others should look as well.
 * Hudson: I've been working my way through the PR backlog I've got and will be there soon


## Tock Cells
 * Alyssa: I was hoping to re-architect Tock Cells so there's one module that re-exports everything and makes the individual parts private.
 * Amit: I'm happy with whatever is common.
 * Alyssa: The common choice is to have a flat API rather than having the API mirror the file structure. With modules for structure.
 * Amit: I have no objections.
 * Leon: We should be careful with things on crates.io, but I think we're not on crates.io for Tock Cells. Since we didn't publish this, I think it's fine
 * Alyssa: We have made some breaking changes already.
 * Hudson: Yeah, few guarantees for internal Tock kernel APIs. After the next set of changes to MapCell and everything, it might make sense to version and release Tock Cells. Not urgent though.
 * Alyssa: MapCell is particularly nice. I haven't seen other things that do what it does as well as it does


## TockWorld Planning
 * Amit: To catch others up, on the tutorial front, we're planning to discuss and meet on Monday. Should have more updates then. One thing to talk about is what we want to focus on for other days.
 * Leon: I did like the split sessions based on specific issues. Then presented results to the entire group.
 * Alyssa: FYI, I won't be able to make it this year
 * Amit: For smaller conversations, if we set up a remote option for you, would that make sense?
 * Alyssa: Yes.
 * Hudson: Looking at prior agenda: https://tockos.org/tockworld5/agenda Anyone can throw out ideas for talks they want. Definitely want some room for specific talks and some for group breakouts.
 * Amit: Agreed on high-level structure. I know Leon has some thoughts on OpenTitan work. Maybe Alex's student on networking stuff.
 * Amit: For specific breakouts: community management/engagement (NSF POSE funding), networking (multiple fronts in work here Thread from Tyler, Ethernet from Leon and Johnny), testing smaller/larger scale (integration tests and unit tests), certification and cryptography (external crypto libraries as part of this)
 * Branden: Lets be sure to have lots of breakout sessions, separating community from networking for instance so people can be involved in both.
 * Leon: Some could be large enough that the whole group should be involved
 * Hudson: Breakouts also depend on total attendance, which we're not sure of and Brad is out today. It would be good to have a firm headcount in advance
 * Poll of attendance includes: Amit, Branden, Leon, Tyler, Pat, Brad, Alex + 3 students, Hudson, maybe Phil. Likely Arun and another
 * Phil: Is anyone from Google or Ti50 coming if Johnathan and Alyssa are out?
 * Alyssa: Unsure. I'll follow up
 * Amit: Back to sessions: outreach, structure/organization, and funding
 * Branden: I think two, or _maybe_ three, topics max per breakout given the number of people
 * Leon: I want to give a talk on what I've done in the last few weeks. No firm commitments from Arun or other about a talk, although I'm nudging
 * Hudson: This does seem like a reasonable approach so far. One thing we didn't do last time, but I'd be interested in is thinking about hardware for Tock and what people use primarily. There's a pretty fractured hardware split for main developers these days
 * Branden: From last time, I do want a "state of Tock" discussion again. Also thinking about where to spend effort for future development has been good
 * Hudson: Also definitely thinking about where the spend money if we have it, for sure
 * Leon: This may be reaching into Monday's discussion, but a concern I have is that there isn't a ton of time to get a tutorial done at this point. I'm curious whether it would also make sense to do more of a hackathon or something instead. Not clear what level of interactivity we're expecting there.
 * Amit: I think when we discussed this originally, our hope was to have a specific project to walk people through. The hope was to do an HOTP device. We should finalize this on Monday. I think we're actually quite close to having that working.


