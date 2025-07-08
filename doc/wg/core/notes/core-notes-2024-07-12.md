# Tock Meeting Notes 2024-07-12

## Attendees

- Branden Ghena
- Amit Levy
- Leon Schuermann
- Alexandru Radovici
- Alyssa Haroldsen
- Brad Campbell
- Andrew Imwalle
- Pat Pannuto
- Tyler Potyondy
- Johnathan Van Why
- Hudson Ayers


## Updates
* Alex: Two interns started working on tockloader-rs
* Branden: Some thinking about Integration Testing going on. Feel free to add comments here: https://docs.google.com/presentation/d/1XXDGhU1Qckvjh-0POrclnhw-SklpiOI2OtiswG0Nbrg/edit?usp=sharing
* Alyssa: For Chromebooks, in the System settings, some will say that they're running Ti50, and that means they're running Tock.
* Alyssa: chrome://system -> cr50_version = ti50 means TockOS
* Amit: I'm going through the breakouts from Tockworld, collecting notes and sending out messages with some action items. We're missing notes from the Code Size breakout, if anyone has them or just any action items, that would be great.
* Alex: I'll think about it and come back
* Amit: Brad already made some progress on the non-xip (execute in place) stuff with documentation
* Brad: Can you include Windows support in that effort?
* Amit: Yes. Who should be included in it?
* Brad: At the Thread tutorial, Bobby was the point person who was able to get Tock working on Windows and was helping others.


## Lingering PRs
* Amit: Looked through active PRs and pretty much all of them are either blocked or have recent comments.
* Branden: Should we be assigning people to _every_ PR? PRs that get comments don't have an assignee, which means that they might linger.
* Leon: I did contribute to Cargo recently, and they automatically assign someone who can reassign as needed. Was pretty nice.
* Brad: I thought we did this now?
* Amit: Two differences: 1) we only assign nightly and 2) we only assign if there are no comments. So should we automatically assign?
* Brad: I thought we already were, so that's good
* Amit: We also have a policy on how long last-call PRs should sit. Should that be automated in some way?
* Leon: We could auto-merge on last call after some amount of time?
* Branden: Can only core team members assign last-call label?
* Leon: I'll have to check on that
* Brad: I like merging last-call automatically
* Alyssa: I'd request that these be 24 business hours if possible
* Amit: Okay, so does everyone agree to move to assigning PRs immediately? (all agree or are quiet)


## Libtock-rs 802.15.4 Raw Support
* https://github.com/tock/libtock-rs/pull/551
* Johnathan: I foresee a bad contributor experience here and want to prevent it. The PR here adds 802.15.4 raw support and is generally approved, but isn't passing Miri in CI because of undefined behavior. The contributor isn't clear how to track down the problem.
* Johnathan: I skimmed briefly and found some things that look unsound, but I don't have time to help fix this due to job and vacation.
* Johnathan: So the question is what to do. Are we willing to accept unsound PRs? Or do we have a way to help them fix this? The author seems possibly less experienced with handling unsound Rust issues like this
* Amit: All the unsound stuff does seem to be isolated to the 15.4 buffer management stuff. This contributor reasonably seems to have copied the C version, which unsurprisingly doesn't meet Rust's soundness requirements.
* Amit: This feels to me like something that the Networking WG might have some stake in?
* Tyler: I'm fairly unfamiliar with libtock-rs. Is this possible using entirely safe Rust? If it's refactoring the code, that's easier for us to help with. Is this possible to do in safe rust?
* Johnathan: It should be possible. However, it might require new APIs in libtock-rs to do so. So there should be a way, but it might require some unsafe libtock platform code
* Amit: In the interim, at least two things seem reasonable to me. It's not trivial to make big changes to libtock-rs. The new buffer proposals could help with this maybe? So 1) this seems fine to me if the code lives outside of libtock-rs for experiments with some documentation that it's unsound and 2) we could allow this unsoundness with a big warning sign and todo to warn people not to copy it or use it in practice. I do share the concern that it's not obvious to tell this person how to fix it, and saying "sorry go figure it out" is not a great contributor experience
* Brad: One thing is, I don't think changes to buffers is going to solve this. It's basically already doing the buffer stuff we proposed. We've put a lot of work into cycling through buffers and were concerned it would have issues in libtock-rs, and now it looks like that's true. We don't know though whether it's really really hard, or just hard because experienced people haven't looked into it.
* Amit: We will have to resolve this for libtock-rs at some point, and maybe that'll fix 15.4 when it happens as a plug-in solution? So maybe this could move forward independently of that. This does seem to inform our buffer management plans though
* Alyssa: My policy: module-private unsound concrete code is tolerable so long as it's well documented as such. But it should not be merged as-is with a public unsound API. That should be marked unsafe at least
* Amit: Does that seem reasonable?
* Johnathan: With the correct comments, that seems fine?
* Amit: Like "This is unsound and unsafe, so do not use"
* Johnathan: Ideally, someone should understand the issue from the comments. There are two issues here, actually Miri UB and having the manually call Drop.
* Amit: So the question is, who's going to do this?
* Brad: Could we dedicate next week's meeting to it? Having a bunch of people look into the issue?
* Amit: Johnathan, do you think that would help?
* Johnathan: Maybe
* Amit: What if the Network Working Group looks at it?
* Branden: I'm worried that it's a Rust issue not a Networking issue
* Alex: I could take a look though
* Tyler: If it passes the Miri test does that mean it's good to go?
* Johnathan: It could still have issues that the tests don't exercise.
* Amit: So the question isn't necessarily how to minimally change this example, but rather how to model the buffer management in Rust. Which as Brad is pointing out, is a general question
* Alex: Skimming the code, we did have a similar problem once with CAN, and we never figured out a safe solution.
* Branden: We'll add this to the agenda for Networking WG for Monday. At least we can look at it.
* Pat: Should we invite the PR author too?
* Branden: If we have bigger changes, yes. I would like us to understand what's up before spending someone else's time on it
* Hudson: If we switch to static lifetime buffers, would that help our problem?
* Johnathan: I think it would not fix the issue. It would make a buffer you can share, but never get back to read.
* Hudson: The problem is that if you share a read/write buffer with a static lifetime, then you never get it back because you shared it forever? (yes)
* Amit: Okay, so a good first step for the Network WG would be to articulate the problem and have a plan, which might be that we need someone else to help figure it out


## AppIdPolicy
* https://github.com/tock/tock/pull/4028
* Brad: Motivation is subtle, but there's a lot of description. The issue that came up last week is that it would be nice to not just have a usize, but have a generic type. However, that propagates _everywhere_ in the kernel. So one option is to just leave it as usize, and you have that many bits. Another option is that we keep templating and say that it's okay that things spread. Or maybe a magic solution I don't know about, if there is one
* Leon: I'm not personally worried about usize not having enough bits. But it's kind of a bummer we can't just pass an enum or a struct, which is clearer than a magic usize. However, I can see that having a generic type literally everywhere in our code base is a problem. The usability does take a hit.
* Amit: So do you think we should stick with usize?
* Leon: Yes. Or communicate the information out-of-band. Out-of-band you'd have to have your verifier write to some separate array that's the same size as the number of processes you're verifying, which seems frustrating.
* Leon: I don't think usize is elegant, but I think it's better than nothing
* Brad: Imagine a scenario where you want to have signed apps and have two keys. One key allows privileged behavior, and the other key is for signing apps but not privilege. So what you need is a verifier that can distinguish between the two keys, and you need some way to communicate with the rest of the Tock kernel what privilege is allowed. We've previously thought about doing that with an AppID short ID. To do that though, you need the AppID assigner to know which key was used, and there's no good way to do that right now. The verifier just says yes or no.
* Brad: So, it seems like the verifier should keep some information around about which key was used. The verifier and assigner run separately, possibly at separate times, so the information needs to be stored somewhere. That's what this enables.
* Leon: And my reasoning for generics, was that it makes sense to have some generic type T which only the verifier can make, and you could opt-out if you don't care, by specifying a unit type.
* Leon: My concrete proposal: we could change it later to a generic size if we needed. So we can leave it as a usize for now. One thing we could do is wrap the usize in a struct, so it's more clear that it's meaningful.
* Brad: That's fine
* Amit: And that's the main outstanding thing from last time's call? (yes)
* Alyssa: One possibly useful thing. There's https://docs.rs/zerocopy/0.8.0-alpha.16/zerocopy/trait.TryFromBytes.html which can convert bytes into enums. Which is a safe, checked transmute.
* Leon: I can see the appeal, although I'm worried that it buys us a bit, but we're still stuck at some predetermined size underneath. So we might as well just stick with the usize


## Storage Permissions
* https://github.com/tock/tock/pull/4031
* Brad: I wrote the TRD and wrote most of this along the way. We merged the TRD. The only big difference now, is that I've added capabilities so only privileged code can create the storage. This is really just an implementation of the TRD though. So instead of a single way to assign storage permissions to applications, any kernel can decide how to do it
* Amit: So does this just need eyeballs? Or is there a particular issue?
* Brad: Just eyes right now
* Leon: I'll take a look at it


## Core Team Call Timing
Amit: I'd like to do another round of scheduling for this call. To see if there's a time that works better. If there are no objections, I'll circulate a survey (no objections)


