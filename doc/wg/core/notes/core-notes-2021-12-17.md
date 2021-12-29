## Attendees
 - Alexandru Radovici
 - Leon Schuermann
 - Philip Levis
 - Hudson Ayers
 - Johnathan Van Why
 - Branden Ghena
 - Alyssa Haroldsen

## Updates

 - Johnathan: Libtock-rs update. I've got enough of it merged that it should be usable. I don't have example apps or documentation or integration testing. So you have to work with me to understand it until I get those written, but it should be usable. Ti50 is moving over to use it.

 - Phil: HURRAY!

 - Hudson: Pretty exciting. Once there's documentation and such you'll be ready for people to port drivers?

 - Johnathan: Yeah, I'll start with console and debug. One document I want to write is "how to port a driver".

 - Leon: Testing: integrating allows into the grant region, IPC breakage, Jett has showed us how CI is so valuable. I'm looking forward to incorporating this into the LiteX tests.

 - Johnathan: Were you running it in Verilator, or an FPGA?

 - Leon: Verilator, also works on FPGAs. That's eoomething we could also talk with Pat and the hardware CI folks. We could talk about an uniform way to run these tests.

 - Alyssa: Comcast seems to not like this meeting on Friday mornings. Calling in.

 - Hudson: We've been doing updates. If anyone else has some more.

 - Alyssa: I can provide an updated on the zero sized type (ZST) for ProcessBuffer. I sent a message to the unsafe code guidelines people about it. I have an example for it. I was talking to unsafe code guidelines. It passes under Miri with default flags, but not with unsafe pointer tags enabled.  This means it *might* be unsound, but it's not clear. So we should convert to raw pointers, since the enum doesn't bring any benefits without big runtime posts. So it sounds like it is not sound now, but might be sound once extern types are standardized.

 - Leon: This is good. Based on this, it sounds like we should move to raw pointers. We've been trying to avoid big breaking changes to capsules.

 - Alyssa: That's why I've been avoiding trying to break the APIs, trying to avoid raw pointers, wanting to keep the API the same.

 - Leon: Yeah, thank you for the investigation. We definitely need to find a solution. What we are doing isn't sound.

 - Alyssa: There isn't any easy solution except raw pointers. I have some other ideas which might make the ZST idea work for the purposes we need, but I won't stand by them until I talk with people who know pointer provenance better than I do.

 - Hudson: Any more updates before the first agenda item?

 - Hudson: Well, in that case, Pat isn't here yet, but Leon are you equipped to talk about the item you added?

 - Leon: Yes, if there are unresolved questions we can answer it over email.

 - Leon: Around when we released 2.0, we talked about splitting out the Tock registers crate as that seemed to have some advantages. With all of the improvements, and more generic trait-based interface we have under the library. It's useful to a wide variety of crates across the Rust ecosystem. So the question is whether it's desirable by us to split it out. It's already on Crates.io. It's a schroedinger-crate. Kind of independent, but not, tracking our Rust toolchain.

 - Leon: Are there any opinions on that issue?

 - Hudson: If it's moved, any time we want to update the Rust version, this library would have to be updated to use the new Rust version.

 - Leon: We have seen some breakages when updated Rust versions. The library is in the state that it requires only one nightly feature. It's very likely to be compatible with stable in the near future. Potentially provide features on the crate that make it compatible with a wide variety of nightly versions. Then try to get it to stable as soon as we can. Then we don't have to worry about compatibility with the Tock Rust toolchain version at any time?

 - Hudson: This would be forked in the Tock organization?

 - Leon: The primary issue isn't splitting it out, but dealing with external dependencies. We've just vendored them in. I'm wondering if long-run that's a stable solution. Or we can pin different revisions. That's the larger issue at hand.

 - Hudson: For the specific case of register, if we can have dependencies within the Tock organization, that seems pretty tame. Broader question, changing our overarching policy that is a tough conversation to have without Amit and Brad here.

 - Leon: I think the idea that it's OK to pin things inside the project is OK. But there are also technical questions, of how we pin, for example versions in Crates.io, vendoring it into submodules. I wanted to bring this up, because there have been comments from Google folks, depending on which method we use, it might break internal build processes.

 - Phil: I like the limited view of dependencies within the Tock organization. Is stable 2 months or 2 years? Tension between good for us and good for the community.
 
 - Leon: Probably not 2 months but also not 2 years.  I think separating this out will help a lot.  I agree about good for us and good for the community. The way people do this now is hard for them.

 - Phil: I just worry about the tension: a lot of work from us for a tiny benefit is not good, but a small amount of work from us for a great benefit to the community is good.

 - Johnathan: I don't think there should be an issue from the Google side.

 - Leon: You mentioned sometjhing about issues with Crates.io, itn would be much harder for you to do downstream?

 - Johnathan: Crates.io is actually easier, I don't think this is the case. Ti50 is already handling crates from Crates.io.

 - Leon: That's great to know. I'd rather use standard was in the Rust ecosystem, rather than building some custom hacks. If we could just pull from Crates.io, like everyone else, that would be a great advantage.

 - Johnathan: That should be fine.

 - Leon: I agree, we should not make any decisions with Brad and Amit, because they have a lot of thoughts on external dependencies.

 - Leon: We want Tock to be reproducible. So when we build things, we want to be able to include a Cargo.toml file. Sorry, Cargo.lock. I wanted to know if this would have any issues. Rust says that for artifacts, binaries, to be reproducible, you should check in Cargo.lock to be reproducible.

 - Hudson: We used to, and it  was a huge pain for regular developers. Lots of conflicts, didn't give much benefit.

 - Alyssa: Are you thinking for all crates, or examples, or the kernel, or what?

 - Leon: I think in terms of workspaces.

 - Alyssa: If we did that, I would change the Ti50 build process to remove the Cargo.lock file. No, wait, we would ignore it since it wouldn't be in our workspace.

 - Leon: If you're using it as a library, it would ignore it.

 - Alyssa: You can always try, and it's causing problems, and if so, take it out.

 - Hudson: We did, and it did have problems, and so we took it out.

 - Leon: I don't suppose you have any specifics on what these were, besides merge conflicts?

 - Hudson: Yeah, any change to any file required also all of the Cargo lock file, and so every change has merge conflicts.

 - Leon: Ah, it locks the individual crates in our workspace.

 - Alyssa: We weren't using a cargo worksspace, that might have made things more broken.

 - Hudson: Yeah, I think upstream wasn't using a workspace either.

 - Leon: Interesting issue, I will look into modern Cargo's behavior as it reflects on workspaces. We want binary reproducibility in the kernel, especially when we pull in external dependencies.

 - Hudson: We have it now, but once we move things into external dependencies, we'll lose it?

 - Leon: Yes, and so if we do, we'll lose it. I was hoping that Cargo.lock files would be  a solution to this issue. If we do include them, we would have binary reproducibility. We would have a build failure if the upstream dependency changed.

 - Hudson: If we pin by a git commit hash, that would give us equivalent to what we have now.

 - Alyssa: Except for external dependencies without a Cargo.lock.

 - Hudson: Right, I am assuming are external dependencies don't have dependencies.

 - Alyssa: If Tock is tied to a specific git commit, but there is no Cargo.lock to reference, the build isn't reproducible. Even if you specify all of yours, and transitive dependencies could have different versions, so it's not sufficient to use a specific git commit to create a reproducible build.

 - Leon: Yeah, I think we were not thinking of external dependencies that have external dependencies, so as long as we do that we won't have this issue.

 - Alex: How do you tie to a git commit using Crates.io?

 - Leon: You can't -- we need to decide whether to use git and commits, or Crates.io and Cargo.lock.

 - Alex: What about including the Cargo.lock only in the commit that would be a release, then remove it inbetween.

 - Hudson: Having a Cargo.lock file in these dependencies, is much less of a pain than in the main Tock repository.

 - Leon: In library crates, you are not expected to check in a Cargo.lock file. We are only concerned about what kinds of headaches checking in this file will have for developers. It seems OK if it behaves with workspaces, if it only updates with external dependencies, not things internal to the repository?

 - Alyssa, Hudson: Yes.

 - Leon: Let's put this as a low priority item for next time, so Amit and Brad can discuss.

 - Hudson: Next time will be 3 weeks from today.

 - Leon: That's fine, Pat's on board with this change. I'll play around with this. We might create a Tock-registers repository within the Tock organization, but it won't be official. It's just for testing. That's everything from my side.

 - Hudson: Alyssa, did you want to talk about your RFC for exit codes? [https://github.com/tock/tock/pull/2914]

 - Alyssa: Yes, what's blocking it?

 - Leon: We want to have implicit behavior of the kernel based on return codes. There might not be an explicit statement, but if the kernel does make an implicit assumption, we want to avoid it in case an application wants to do something different.

 - Alyssa: Primary intention is to allow for the ability to automatically determine if an application exited successfully or not, and based on that take an action. I would like to, within a capsule, take an action based on a completion code. I'm looking for the possibility for the kernel to do something in the future.

 - Leon: I think we want to make sure the kernel doesn't make assumptions.

 - Alyssa: Agreed, but if it's just zero or non-zero, that's a huge error space.

 - Alex: How about adding another parameter?

 - Alyssa: This is why I say that a non-zero error code may not be an error. Except 1-1024, which is TRD104.

 - Alex: What if you just want to know if it's an error or not.

 - Hudson: Why would adding another parameter be any different than specifying some as success and some as failure.

 - Leon: Primary pushback is that the kernel should not... the error code is primarily about other apps.

 - Johnathan: Or capsules?

 - Leon: Right, or capsules?

 - Alyssa: It's a convention. Even for programs like test, where it's part of the expected life cycle.  I wanted to be a little bit more lawyeristic about this. A non-zero SHOULD does not imply that all non-zero are error codes.

 - Phil: What are our requirements?

 - Alyssa: I would like a convnetion, that if an app completed with completion code 0, this means it was success, not an abnormal app completion.

 - Leon: I'm fine with this, we can be like POSIX. But we aren't quite Linux. But say, when an application returns 14, it doesn't say that it succeeds, or it failed, does that bring us any real benefits.

 - Alyssa: It's mostly that it can print out text for the TRD104 error code. Or "App returned 6, likely INVALID." If they're like, why is it returning this, why is this specific error message, then don't use a TRD 104 error code.

 - Leon: So this is for diagnostic purposes. Because we can't use this for any other reasons.

 - Alyssa: Mayne the kernel can use it.

 - Leon: If we do want the kernel to do something, then we want a more rigid specification.

 - Alyssa: I think it is mostly for diagnostic purposes.

 - Alex: This is why I am in favor of an extra parameter. Because then the exit code would be.

 - Alyssa: Always the option of whether the top bit is set. 

 - Alex: That would work too.

 - Hudson: Top bit advantage is we dont want to change the syscalls.

 - Phil: TRD104 is really for function return values, bubbling it up to main you lose location.

 - Alyssa: At that point you need better inspection, use syscall tracing.

 - Alyssa: I have some thoughts on open EMs? I've noticed a few places that we have a new type wrapping an int, where we treat it like an enum, where it can have any value. When I have that kind of pattern, I defined all of the constants as associated constants, not constants within the type. So this way it has the same syntax as enum variants. So it becomes identical in semantics to a C++ scoped enum. I was wondering on the possibility of changing some of the libtock-rs structures to follow that pattern.

 - Johnathan: Sounds great to me! I didn't realize you could this without a trait.

 - Alyssa: They did it a few years ago, it's how uint32::MAX works.

 - Johnathan: Oh, cool! I use that all the time!

 - Alyssa: Could I talk for a couple of minutes on porting Ti50 to Tock 2.0.

 - Alyssa: I see we're supposed to be getting the result type, and manually getting the types out of a return. I've been working on an into that's generic over the success and failure types. And if it's the correct success variant, it's OK. If it's the correct failure, it's Err. If it's the incorrect value, if it's failure it preserves the error code, if it's the wrong success, it turns into Err with error code fail. Interest in upstreaming?

 - Johnathan: Yes, but I want to know what the upstream code implications are. There's probably a design for the command system call that involves generics that avoids code bloat issues.  I think there's a better way to do a design than I did. But it'll be more than 3 minutes we have.

 - Alyssa: Can you explain code bloat?

 - Johnathan: If command returns a 3-type result, then you end up with command has its own internal match, the API implementation has its own match call, those are expensive to compile, unless it's inlined the compiler can't merge them.

 - Alyssa: This only returns Ok or Error.

 - Johnathan: I'm going to want to see code size.

 - Alyssa: I'll send you what I'm working on. Thank you.











