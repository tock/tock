# Tock Meeting Notes 03/01/2024

## Attendees

- Amit Levy
- Phil Levis
- Hudson Ayers
- Leon Schuermann
- Branden Ghena
- Pat Pannuto
- Alexandru Radovici
- Johnathan Van Why
- Brad Campbell
- Andrew Imwalle
- Tyler Potyondy



## Updates

Amit: Assignment of PRs.

Leon: We have a new bot in our organization, to deal with lingering pull requests that are ignored or not getting reviews. It's currently limited. Assigns reviewers to all PRs that are not Draft and aren't marked as Blocked and don't have reviews. I hope this captures our existing policies.

Phil: Maybe 3 days instead of 7, so a WG could meet and assign reviewers?

Leon: Thought about it, if we are flexible and re-assigning reviewers, then that could take off the brunt of the load to discuss in the call.

Amit: Assignment is to triage. Which may mean reviewing. May mean assigning to someone else. In other cases maybe means at least re-visiting that pull request somewhat frequently. 

Phil: So it's administrative responsibility, not code review. That's fine. 3 days is OK.

Leon: Assigned person can delegate, assigning a label.

## Kernel Testing

Brad: We have these kernel tests. They're just commented out. If they fall out of correctness, nobody notices, they just fall out. I went back to the notes from years ago and looked at it, we haven't miraculously arrived at a good approach. I approach brute force. We have a board defined with the tests enabled all the time. They're always checked. Doesn't require manual effort to comment in tests. This would also let us test different schedulers, test different credential checkers. This also lets us reflect all of the options somewhere.

Brad: Downside is it's the brute force method. We have more boards we have to maintain.

AMit: If instead we had some kind of system, a macro, the boards are generated in some methodical way, would that ameliorate the concern of having 100 boards?

Brad: It would to me.

Hudson: Well we won't have a 100 -- do we think that's where it will end up. The numbers were helpful in your PR. If it's 1% or 2% of PRs, that's not so bad. I had a skewed perspective. 

Phil: There's a tension -- we think boards define the system, but we are worrying about having a lot of them.

Amit: What level of test are you thinking of here? The Example Test board you have in the PR is not at all NRF specific. Is that representative. Or conversely, do we think that most tests that would fi this model are board specific, chip specific?

Brad: There will be a roughly equal measure of both. I have another board that does kernel tests for the NRF. There are other issues there, so I didn't include that one.

Amit: Will they reply on processes?

Brad: Yes. Some of them.

Amit: Is that in the board configuration? In tree? A reference?

Brad: I was imagining this would just go in the README. "Here are apps you can run with this board." But we would later define a way to run a specific test, with CI.

Amit: I'm thinking through thoughts and alternatives. Another approach: every board has a test version. That exercises a bunch of functionality. Integration. Another is unit tests, not tests you can just run on the host. They rely on hardware. "Am I correctly getting an interrupt for the AES peripheral on this chip?" Another is more like big integration tests, that would run in CI, that involve applications as well, fully automated, maybe this should just be in a different repo.

Brad: You're talking about a lot of very relevant things. I agree with all of them, we need to move towards doing all of them. I don't know what your concrete proposal is, though. What I like about just getting over this idea that we should have few boards, we just get over this. We can have boards for whatever you want to do. It might become unwieldy, we might figure out ways to make it more straightforward, if we were going to arrive at the perfect solution, we would have done so in the next 3 years, we need to get further along in testing.

Amit: I propose a small amendment to the RFC that either boards/configuration would be more informative, separate top-level called board-tests.

Brad: I hear you, I think there is a benefit to not using the word test, it has meaning in Cargo. If we come up with a word like capsule, we know what we mean.

Amit: My take away is the potential clutter is worth it, if and when we can come up with a name or separation. 

Phil: I think this is a good idea, it might force us to make it easier to maintain and make boards.

Leon: I think this resolves some other longstanding tension, would clearly define which are alternatives, versus which are demonstration boards.

Brad: Move further discussion to the pull request.

Amit: How do we go from here? Does this RFC become a real PR?

Brad: Yes, change the title.

## Updating Nightly

Brad: We can't updated to nightly due to static muts. Leon has an example of rust code that will get rid of the warning. Are we comfortable with using this that gets rid of the warnings in the 50 or so spots? Or do we want a Tock abstraction?

Amit: My understanding of Leon's suggestion, LocalCell, is not to take the code exactly, it deals with reentrancy that doesn't matter. 

Brad: I was talking about "One workaround are diffs that look like". 

Leon: We've been talking about this on the OpenTitan call a way back. Why is this popping up now? What does Rust want us to do with this? We do need them, but there is this dangerous part, of converting them to references or mutable references. One of the dangers we are exposed to, is passing this around. We are still risking violating Rust's aliasing rules around this variability. LocalCell would reduce us to the actions of core pointer read, and core pointer write. No intermediate reference. No leaking of references.

Hudson: So you are advocating LocalCell over Brad's alternative.

Leon: Or CorePtr read and CorePtr write. Single read, single write.

Amit: That only works if we are storing things that we can read and write in that way.

Hudson: Like objects.

Amit: How much does this come up?

Leon: We aren't concerned with atomicity of writes and reads.

Amit: But it would be copied.

Leon: How often do we take a reference without reading it? Which is the same as doing a copy.

Amit: We allocate components statically, in static muts effectively. Those can't be copied out and back in. That's not meaningful. I claim that with components, we are good. They are all locally scoped static muts, in the stack. Where does this show up?

Leon: Shows up in Cortex-M arch crate. Global state shared between system call and trap handler. Boolean uisze.

Amit: In Cortex-M it's possibly there because an atomic doesn't work in Cortex-M0.

Leon: Yeah and we don't need atomic, we just need volatile.

Amit: Which is what they compile to on M4.

Brad: What are you proposing, Amit?

Amit: If it's like 10 cases, and they are all special cases, then just the diff in the comment seems fine.

Brad: It's hard to know, because the kernel crate has to compile before the rest. There's a lot of these. Processes array trips this. Drivers trip this. DeferredCell is the first one.

Hudson: I linked to 3 static muts in every board. ProcessArray, chip structure, and the printer.

Leon: I do believe these instances are masking something we're doing that's unsafe or unsound. For instance, the processes array is a complex beast. How do we ensure that we are sound?

Amit: So, LocalCell, there we would want something like the scope function. Which imposes a lifetime constraint on the reference.

Hudson: I suspect the right answer, is this should unfortunately be handled case-by-case. Some are fine with LocalCell, others might not. There isn't one true way forward.

Leon: Something we are still doing, taking references pointing into process flash. Even the storage driver can violate aliasing rules there. As much as this is terrible and hard work and take a long time, we have to resolve all these use cases.

Hudson: Let's wait for Alyssa to comment on this PR, spend the next 20 minutes on WG reorganization PR?

Brad: We should talk about what the worst case scenario is. We don't want to be able to never update nightly again.

Amit: If I agree to tackle this, is this something we can follow through. 

Brad: If we had a plan within a month to resolve this, that would be great. We need a concrete end to this. 

Amit: We should resolve this within a month. After we've had other people who have really good insight into this, like Alyssa. Leon and I shouldn't go into a dark basement and solve it without talking to people.

Alexandru: I'll add Alex. This is boiling for us. We can talk on What'sApp.

## WG Re-Org

Hudson: I'll share your post to the mailing list.

Amit: My proposed proposal is modest, but the implications would be less modest over time. Even over a short period of time. Very high level...

Phil: How about we just read it?

*reading*

Amit: What do people think of the high-level proposal? Convert to a PR to change the README for the core WG, and the framework for the working groups to break up to/establish/convert.

Phil: Who owns the HILs? And let's not use the name "Communication", it has a meaning in EE.

Amit: OK, I was just trying to capture USB.

Amit: I was trying to move HILs outside of the kernel crate, into places more for their use cases. But the sensors HILs should be managed by the people doing sensors. 15.4 should be networking.

Branden: We are a small group, there is overlap, a core member is in every WG.

Leon: I'm wary of how groups will implement this, the idea that each group sets its own standards. Many contributions do overlap. What do we do with those contributions?

Amit: I think part of the dance that we should have is, both figuring out division of working groups along with potentially adjusting what is where in the various source trees. So there is less overlap. HILs are one example of this. If someone adds a sensor implementation that motivates modest change into the HIL for temperature sensors. That would currently touch the kernel crate and raise alarms. But perhaps we shouldn't care, that should be a trait that's alongside the sensor related stuff.

Leon: Makes total sense. We need to define this exact escalation procedure. We might have a divergence in architecture that's not reflected in CI status going red. E.g., some HIL use some Cell type, others use another.

Phil: It's OK if there are traits outside the kernel. E.g., a specialized 15.4 trait, or a bunch of specialized sensors.

Amit: Maybe this gets worked out through directories. It lets people do stuff, then upstream it later. 

Brad: Separate between Core capsules and extra capsules.

Phil: Or POSIX and ioctl.

Brad: My comments on the initial review of this is, seems like a lot of formalism to end up where we are now. The version of the proposed working groups in the email, would be a bigger umbrella, and diffuse responsibility more, would require standing up those working groups. 

Alexandru: A lot of WGs, a lot of overlap.

Amit: I did limit myself to 8 non-core WGs, which happens to be the number on the core WG, minus Pat.

Phil: It's not that every WG has to meet every week for an hour. Working groups of 2 could be fine.

Amit: Yeah, Crypto isn't going to be extremely involved. We can empower people with responsibility over fractional merging decisions. It's not now, you are part of core, we trust you. There can be more local trust.

