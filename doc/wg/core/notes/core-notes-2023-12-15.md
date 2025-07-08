# Tock Meeting Notes 11/17/23
=============================

## Attendees
- Branden Ghena
- Amit Levy
- Johnathan Van Why
- Leon Schuermann
- Alexandru Radovici
- Tyler Potyondy
- Hudson Ayers
- Alyssa Haroldsen
- George Cosma
- Brad Campbell
- Andrew Imwalle


## Updates
 - Alex: Introducing George Cosma who is my student for several years, working on tockloader-rs
 - Alex: From networking group: Leon created a stable version of the PacketBuffer library. We're hoping to be close to have a buffer management solution that doesn't use any unstable features. We'll be starting proof-of-concept with it
 - Alyssa: Two issues about ReadableProcessBuffer we could add to agenda
 - Leon: Not fully working yet, but we have a CW310 board that OpenTitan runs on that's not generally accessible. So our thoughts were making it available over the internet. Could be an infrastructure for flexible access to boards for CI or for letting other developers play with them.
 - Tyler: Once that's working, and maybe if it's more concrete, I'd love to touch base. We have a test network at UCSD that would be neat to let people work with.
 - Amit: There were some sensor networks testbeds back in the day, right?
 - Brad: Yes, multiple
 - Tyler: It would be great to be able to run CI jobs with a working network
 - Leon: This project is hacked together right now. Linux container that has access to only a single board. Definitely work in progress
 - Tyler: Is there a way to only run CI if it touches certain files?
 - Alex: Running conditional jobs is a bit of a nightmare on github, especially with merge queues
 - Branden: Maybe on command? Something like a bors command that could run stuff
 - Alex: I do have some stuff I can share. I can send a workflow file if anyone needs it


## Tockworld Scheduling
 - Amit: Tockworld is an annual in-person two-three day workshop/mini-conference for Tock developers. The hope in particular this year is to make it a bit broader. It's previously been the active developers getting together in person to discuss development questions and plans. But this time around we're also hoping to have a broader appeal and include users and other curious folks.
 - Amit: Based on the availability we collected with a survey, it seems like the two most appropriate times for Tockworld would be August 14-16 or June 26-28. Held in San Diego.
 - Alyssa: Preference for August, all things being equal
 - Alex: Might not be able to attend in August
 - Amit: It's not going to totally work for everyone. Some people didn't get or fill out the survey. It would be great to have people from industry attend if possible. So, one question is whether one of these time frames makes more or less sense.
 - Jonathan: Either could work for me
 - Hudson: I could make either. My broad observation is that my coworkers tend to take vacations in August. That's anecdotal though
 - Amit: Alex's unavailability in August might be representative of people in Europe
 - Alex: It's actually a summer school thing. We've been doing for a very long time. I might be able to push it around if decided very soon.
 - Amit: Okay, so it does seem like June 26-28 is the most favorable time unless people voice big concerns
 - Branden: Not to put Andrew on the spot, but would someone from your team be interested if they are available?
 - Andrew: I'll reach out to the team. For the dates, I have no particular preferences.
 - Amit: Okay, tentatively June 26 then


## Tockloader-RS Discussion
 - https://github.com/tock/tockloader-rs
 - George: Goal is to port Tockloader to Rust. What I'm here for today is to ask for feedback. I've been working on this for the last 6 months. Some feedback I need is that we need to have a structure in place so development can accelerate. As of right now in the main repo there's basically nothing, just some command-line parsing. The PRs have added a lot more stuff
 - George: So, I want to figure out how to get new developers able to work on the project. And I want more bite-sized PRs. Right now it would be a huge lift to start, as there's basically no starting point.
 - George: So I want to request aid in this area
 - Amit: One useful data point is that I don't think many people are actively looking at tockloader-rs. So it's great to point out that someone is working on it and that we should pay attention. One useful thing to frame would be given that the python version of Tockloader is pretty full-featured, what seems like the minimal viable product for tockloader-rs that would make a transition seem possible or worthwhile?
 - Brad: Hard to actually answer that. Some thoughts are the install, listen, and list commands, which people use the most. Those commands are mostly "stabilized" right now, although I don't think we made that official anywhere. Then there's a bunch of extra commands, but those are extra.
 - Branden: How many boards would it need to work for?
 - Brad: I think realistically I think we'd want to support at least the serial connection, and jlink or openocd or both. Once you do one doing both is easy. Once you do that, supporting additional boards is pretty trivial.
 - Alex: Is there some way to get feedback on this PR: https://github.com/tock/tockloader-rs/pull/8
 - Amit: Yes, definitely. We should get George on the mailing list and slack too, for better communication. In the future, since this repo is lower on the critical path for most people, it would be good to have the option to nudge people for feedback
 - Branden: So George, to give you the help you want, staring with comments on PR 8 would be best?
 - George: Yes. PR 8 would be best to start. The other PRs are for other features that will be based on PR 8
 - Alex: Where do we bug people on Slack?
 - Amit: Maybe even the devel mailing list. A problem here is that we literally won't see PRs here, unless you bring them up
 - Brad: Core channel in slack too


## Precompiled Newlib
 - https://github.com/tock/libtock-c/pull/353
 - Brad: Nothing has really changed since the last update on the call. I can repeat that if useful
 - Amit: I was hoping that we could use the time to solicit Brad to remind us what the purpose of doing this is and what challenges we're facing, so we can figure out how/when to merge
 - Brad: 1) we were never compiling newlib binaries for Risc-V, so this does that
 - Brad: 2) we used our own compiled newlib, but implicitly required people to have newlib installed because we used the installed headers
 - Brad: 3) also different versions of GCC can or cannot compile the latest newlib for all systems. Example is Ubuntu can't compile the newest newlib for Risc-V
 - Brad: 4) we haven't had a reproducible environment for building binaries
 - Brad: So now we have system that will pull in pre-compiled binaries from a mirror we host for ARM and Risc-V. The reason this isn't a slam dunk is that the C++ libraries seem to have a strong dependency between the version of GCC and version of headers in the library. So the compiled libraries really need to have the exact same compiler used as they were compiled with. So the build system figures out which GCC you have for ARM and Risc-V, then downloads the built files which match that version. So there's a build system where the build you're doing is dependent on which version you have installed. This is not exactly where we wanted to be. But that does resolve allowing us to update newlib.
 - Brad: That's the major changes summarized
 - Branden: There's also the issue of having pre-compiled binaries downloaded from a random server we host that was discussed and I believe resolved.
 - Brad: Yes, we did decide this was okay. We have hashes committed and now have a reproducible environment that could recreate them.
 - Leon: I also made a system to periodically check that the mirrors are still up
 - Amit: Does this interact in any way with newlib licensing concerns?
 - Brad: I think it's orthogonal
 - Branden: Well Brad has made an entire system for creating and shipping binaries and headers in environments. So that would apply to new libraries too
 - Leon: Also we have a generic license file in the repo and say it applies to everything. Now that the files are out of our repo, we don't have to worry that we're incorrectly claiming license on them.
 - Amit: This answers the question to me so I'm comfortable moving forward
 - Leon: There's a warning that exists with newlib now that we compile it ourselves. That's still an issue?
 - Brad: Yes, thanks, there are a couple. The newlib headers are somehow incompatible, maybe this only shows up for us? The bug should theoretically be upstreamed. There's also an issue that libC++ stuff, we have a warning on that headers match your compiler. But now that we're shipping our own headers and not using the compiler ones, that warning gets triggered, but somehow doesn't cause the build to fail. The third one is that we can't #include assert anymore, and I don't know why. It complains that things are getting double-included and I can't explain why.
 - Alyssa: Different header guards maybe?
 - Brad: Maybe? I found some comment somewhere that assert actually doesn't have header guards at all? I just removed the include in the one app that uses assert. If someone wants to investigate more, be my guest
 - Brad: One last thing. That PR is at a state where it's done in that it should be reviewed. There might be ways to improve it. But it shouldn't be merged yet. Once everyone is approving it, I'd like to re-run the docker stuff and make sure that everything matches. Then we can merge.


## ReadableProcessBuffer
 - https://github.com/tock/tock/issues/3756
 - https://github.com/tock/tock/issues/3757
 - Alyssa: Both related but somewhat orthogonal. Base question: can we make breaking changes to ReadableProcessBuffer and WriteableProcessBuffer?
 - Amit: We absolutely can. The system call interface is where we drew the line for breaking changes that would require a major version update. Of course we should tread lightly in terms of not breaking non-upstreamed builds willy-nilly. But I don't think that either of the changes you're proposing would do that unless maybe someone has a different implementation for a different architecture. Doesn't seem like a big deal
 - Alyssa: WriteableProcessBuffer doesn't have a mutable pointer function. There is a way to get it, but it's not implementation-safe.
 - Alyssa: The other is that both should really be unsafe traits, because anything that takes or returns a raw pointer should be unsafe. If anything is requiring the pointer to be a valid raw pointer, then you need an unsafe guarantee that they programmed a valid pointer there. You could imaging someone returning NULL, which is sound but other people working with it could assume it's valid. The two options for fixing that are to make it unsafe or make it a sealed trait so external users can't implement it. I lean towards the unsafe trait so external users can implement their own
 - Hudson: I lean the same way
 - Leon: One case we can refer to is the integration of the allow system call that lets users change the buffer while allowed. Downstream users might want to implement different calls like this.
 - Amit: And we know there's support out-of-tree for different architectures. So there doesn't seem to be a disadvantage to having it be an unsafe trait. There is a clear advantage that adding an implementation wouldn't require modifying a specific module that would have to be forked or accepted upstream.
 - Amit: I think Alyssa answered this in the issue, but there's virtually no downside to marking the trait unsafe, except that arbitrary untrusted code can't implement it, which is exactly what we want. If you're using some code that implements this, that code needs to be part of your auditing.
 - Alyssa: Yes. Other implementations should be audited for sure
 - Alyssa: One issue is if a downstream users has a forbid on unsafe code, this could cause churn. For people with a "pure safe codebase"
 - Amit: That seems both unlikely and a thing to challenge/avoid. For this code be truly trusted, we'd have to do a bunch of checks that we really can't do
 - Alyssa: Right, there's no safe way to prove a raw pointer is valid.
 - Leon: I would say that we need lots of documentation about WHY it's unsafe
 - Alyssa: I know how I would write the safety documentation
 - Leon: We cared a lot about documenting that particular infrastructure. I'd add that we don't make stability guarantees for this particular infrastructure, although this is tricky for _other_ changes (not these ones you proposed) because this does affect our ABI behavior. But this case is okay.
 - Amit: As for adding `mut_ptr`. First of all it seems like a yes. I don't know why it's a breaking change though.
 - Alyssa: Downstream implementations would have to implement it and I don't feel comfortable providing a default.
 - Alyssa: Thanks. I'll send PRs for this
 - Amit: Yes. My take is that both of these seem good. `mut_ptr` seems less necessary but is good. And I don't think that it's a problem to change this as fixing it downstream is easy. And we really are very unlikely to guarantee stability at this level. It's not even capsule-level.
 - Alyssa: Does anyone know any safe traits that take or return raw pointers?
 - Amit: Not off the top of my head


## Future Meetings
 - Next call is Friday, January 5th. Next two weeks off for holidays.

