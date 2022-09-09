# Tock Core Notes 2022-07-29

Attendees:
* Alexandru Radovici
* Alyssa Haroldsen
* Arun Thomas
* Brad Campbell
* Branden Ghena
* Chris Frantz
* Hudson Ayers
* Johnathan Van Why
* Leon Schuermann
* Pat Pannuto
* Philip Levis

## Updates
 * Leon: Working on pushing Ethernet support upstream. Have a sort-of-finished
   PR for the QEMU(?) board. Have a pretty-polished PR that's still in draft
   because there are some things with respect to memory safety in VirtIO network
   support. An issue open for supporting physical ethernet chips. I picked up
   work on an SPI-based chip we can attach to any Tock chip with a SPI master
   controller. Looking pretty good so far.
 * Phil: Not an update, but as mentioned at TockWorld I want to talk about
   assigning PRs to people so they don't languish and reviewing outstanding PRs
   during calls.
 * Hudson: Yeah, that's a good point, I remember both of things came up. For the
   first one, I think the ideal thing would be if we could find something like
   the Bors bot that runs on the upstream Rust repository that will make a guess
   at who is the best reviewer for a given PR and assign them. If they're not
   the best reviewer, they can assign someone else. Ideally that would evenly
   distribute the load. For doing PRs during the core call, we could use the
   time at the end of the call for that, or we could schedule a few -- or 20 --
   minutes at the beginning.
 * Phil: Amit used to have a script that would go through all the pull requests
   and autogenerate the list and stuff like that.
 * Leon: If you find a PR that you would be suited to review, you can assign
   yourself. For those with no reviewer assigned it should be easy to go through
   those and have them done in one or two minutes. Not sure if automated efforts
   to assign people would work out.
 * Phil: That's a good idea Leon -- spending the beginning of each call
   assigning them. Maybe we can start doing that next week.
 * Pat: Maybe have the unassigned ones in the agenda, then people can grab a few
   and start before the meeting.
 * Hudson: That's a great idea. If people can assign themselves to a few PRs
   this week, especially any people have basically been owning, then next week I
   can get a list of all that have not been assigned and we can assign them.
 * Branden: One more item that's not an update but doesn't fit anywhere else.
   It's a request. Alyssa, can your push your example unit test code that you
   started, maybe as a draft PR? I'd love to start playing around with that.
 * Alyssa: Yes
 * Brad: On the over-the-air updates side, we're trying to figure out what it
   looks like to restrict that API to only certain applications. Trying
   different options to see how they work -- that'll be the next changes to the
   pull request.
 * Hudson: Has starting to work on the app ID stuff given you thoughts on how
   they might interact?
 * Brad: Conceptually, yes
 * Hudson: Do you expect it to work okay?
 * Brad: From a code point of view, I don't know. Conceptually, yes. Don't know
   if the code will be seamless. We're on the right path.
 * Phil: I'm going to guess it won't be very difficult.
 * Brad: Hope so, it comes down to Rust type stuff.
 * Hudson: Since Phil's stuff makes process loading asynchronous, that
   simplifies stuff anyway.
 * Leon: I've been working with Johnathan and Chris Frantz on `tock-registers`'
   unsoundness with respect to having MMIO memory exposed no Rust references.
   Johnathan has been looking into options there, I've been trying to reconcile
   this to a generics-based approach to also allow testing of register
   peripherals. I was going to ask if on the next call we could present
   preliminary results and have a more focused discussion on it.
 * Hudson: I'm excited to see what you come up with.

## App ID
 * Phil: I wasn't here for the last call where I think it was discussed. I know
   one of the things that came up is the code size impact. I wanted to get to
   the bottom of where that is coming from -- how much is unavoidable, how much
   is avoidable. In doing so I improved the code size tools a bunch.
 * Phil: Verifying the process credentials are correct is an asynchronous
   operation, because it may depend on a crypto hardware accelerator. Loading
   processes into memory and checking their values can still by synchronous, but
   making the transition from loaded into runnable is asynchronous. This adds
   complexity to the kernel, as we have to have new state machines and we have
   new process states. We have to make sure that something whose credentials
   didn't pass doesn't have a workaround to still get it to run. When I trim
   down stuff and compile it for the CW310, it adds about 1200 bytes of code.
   400 bytes is in the kernel itself -- things like additional process states.
   400 bytes is in the Tock TBF library. About 250 are in the boot sequence --
   parsing processes and their footers. There's another 100 or so in assorted
   functions -- e.g. new methods on processes. This 1200 bytes is if you do not
   do any checking -- load every process and mark them all runnable. There's
   some other data in there like strings which I will look at, but this is the
   basic cost.
 * Hudson: Have you looked at the cost with virtual function elimination
   enabled? That may get rid of some of the unused methods on `Process`.
 * Phil: I suspect that will get less savings than you think because the
   additional methods are like "this thing has been loaded, mark it runnable"
   which you have to do anyway. I can try. I guess it will not give us very much
   -- guess like 50 bytes or something. The basic challenge is the ability to do
   this requires changing the process state machine and therefore the kernel
   state machine for loading processes.
 * Hudson: Alyssa, I know when we talked about this on the last call, this was
   something you had a lot of feedback on. Do you have any comments on this? It
   sounds like the overhead is less than when we last spoke.
 * Alyssa: Are the numbers on GitHub so I can check them out?
 * Phil: Not currently. I will put them on the PR.
 * Alyssa: Did you investigate whether you can control this via a feature?
 * Phil: I put in a method where you don't do any checking, which pulls out a
   lot of the state machine, and that's how you get the 1.2k. To fully excise
   the feature would be a pretty invasive change to a lot of methods on
   `Process`. There are certain things you can do today which you can't do if
   you have credentials -- like marking processes runnable requires a capability
   now. We have been very leery of using features.
 * Alyssa: I'm just considering how for every feature, unless it's well
   controlled, it costs extra space. I'm thinking more like the Linux model
   where at the beginning you tell it what features you want and it does it.
   That's how you build a tiny Linux that can fit on a floppy drop.
 * Phil: Right, so if we wanted to go down this path then the way to do this
   would be to have two different versions of `Process`. One with checkable
   credentials, one which doesn't.
 * Alyssa: I need to see the PR a little better to see why it needs to be so
   invasive and not a little bit more modular.
 * Phil: If the answer is we can never add anything because it adds code space,
   that's a tricky situation to be in.
 * Alyssa: There is a clear answer to that and the clear answer is features.
 * Phil: We've had many discussions about that and that is not an answer we are
   comfortable with.
 * Alyssa: Over the long term, I don't see how we will be able to continue to
   add features if we don't have some way to control the explosion of the kernel
   size.
 * Leon: I think this particular example is bad in the way that this has such
   deep integration in the process loading state machine, and we have to keep
   much more information in the process than other features. I'm having a hard
   time coming up with examples, but I know in the fast we have used generics to
   entirely omit extra information from the binary. So this is an outlier we are
   looking at here.
 * Alyssa: Okay. I still need to take a closer look at this.
 * Phil: Let me put the numbers together and
 * Alyssa: I don't love the code size increase and I don't love that tracking
   upstream Tock means now we have less room to do our things. That's where I'm
   coming from. I have to keep track of upstream but also have to keep track of
   our code size.
 * Phil: Understood. I think Leon's point is this isn't just adding a feature,
   it is a foundational change about the security model the kernel can support
   that has been pending for a very long time. I'm not happy with the 1.2k and
   would like it to get smaller and am happy to keep working on it, but at some
   point we have to figure out how big is too big. If it was 4 bytes we wouldn't
   have an issue here.
 * Leon: With the disclaimer of only having looked at this for an hour or so,
   I'd say there is some unavoidable impact, but I could imagine that if we
   tried hard we could make Tock TBF to be configurable, etc. The question is if
   we got it down to 1k or 800 bytes, would we still be having the same
   discussion.
 * Alyssa: I can't give you an exact number, but something like 500. As long as
   this is not a regular trend where we will be adding 1.2k unavoidable features
   often, I don't think it will be a big problem. If I am able to construct a
   way to modularize that, would you be open to such a PR?
 * Phil: Yes. The way that this would be modularized would be having different
   implementations of `Process`. A lot of the implementation would be shared, so
   how do you factor that out? A `ProcessStandard` and a `ProcessUnchecked`, or
   something like that.
 * Alyssa: I do need to understand the security model for app IDs better to do
   this effectively, but I'm imagining what Linux does -- basically `cfg-if`.
 * Brad: That's a much more difficult conversation and pull request. We've had
   difficult experiences in the past where it becomes very difficult to maintain
   so many different code paths. Easier to maintain if things are expressed
   through the Rust code itself.
 * Alyssa: I think an all-or-nothing approaching is going to hamper the project
   long-term. We will eventually -- without a doubt -- need a way to fence off
   code features of the kernel.
 * Leon: For many things, we have that. I think there is a difficulty in
   terminology. On one hand, you have features of an operating system, and on
   the other hand you have Cargo features which you can use for conditional
   compilation. I don't think that Cargo features are the right tool for us, as
   we have to test many combinations as part of the compilation target. So to
   have good coverage
 * Alyssa: That's entirely normal. Every major kernel I know does it
 * Leon: For instance, for the process fault handler, we use these traits we
   define and we have very lightweight implementations for when you don't want
   the additional complexity added. That would amount to the same overhead as a
   top-level feature tag without the overhead of having top-level cargo
   features, and we keep Rust type safety. We do want to support features in the
   sense of making the kernel configurable, but not through Cargo features if we
   can avoid that.
 * Alyssa: I don't think this is tenable long-term. You're going to have to have
   config flags someday. It doesn't have to be Cargo features specifically, but
   some way to do conditional compilation to enable or disable features.
 * Hudson: Which we do have -- `kernel/src/config.rs` -- and there are multiple
   things we turn on and off that way. I think this discussion is getting
   derailed with conflating configuration with specifically cargo features.
   Alyssa, if you want to take a look at the PR and give concrete suggestions
   for how you would shave some of this overhead off that would be a more useful
   way for us to frame any discussion around this.
 * Alyssa: I will probably upload a version that does it exactly how I see it
   all the time in the Linux kernel. I will also try a mechanism that takes
   advantage of generics. How do you feel about two `Process` implementations
   but a config-controlled type alias?
 * Leon: The code idea of the `Process` trait is you can add either a second
   upstream or a downstream implementation without any issues. Could help test
   whether the API works out.
 * Phil: Just because there are ways people have dealt with struggles in
   particular languages -- we have concerns about cases where the code that is
   running is not also code but also impacted by how you compiled it. There is
   state in its construction that is not embodied in its code. A `#define`
   passed on the command line is an example of that. This led us to the config
   structure we have in the kernel. Sometimes we do want to use features for the
   hardware file, but we are very limited and constrained in what those can do.
   It would be a discussion whether something like a type alias could fit in
   that bin.
 * Phil: It definitely seems like something we should explore and figure out.
   This is a good test of process.
 * Alyssa: I don't think we'll be able to avoid conditional compilation forever.
   I guess we can try.
 * Hudson: We do have conditional compilation in the kernel -- there are
   multiple config flags there now -- they are just contained in a particular
   way.
 * Alyssa: From my memory, don't they only control individual constants, and the
   optimizer would remove unused code based on those constants?
 * Hudson: Correct
 * Alyssa: That's quite a bit different
 * Leon: That still makes the compiler check whether type safety will work out
   for any configuration.
 * Alyssa: I think a config flag on a struct field is perfectly reasonable.
   Perhaps that's just me.
 * Hudson: The biggest problem is that when we went down that path in the past,
   we didn't have a way to use our CI to confirm that every combination of
   config flags would compile. Contiki went this way, and if you've ever tried
   to pick arbitrary configuration like 50% don't compile, and that's something
   we wanted to avoid for Tock. Easier to avoid for Linux where you have a
   billion users.
 * Pat: Linux famously has `make randomconfig` and has farms testing build
   configs
 * Hudson: Maybe something like that would make it more pallatable.
 * Alyssa: This seems like an infrastructure problem
 * Leon: It isn't just going to be one config flag. If we open the gate to
   enabling this, saying something like "oh this won't be used for features that
   can be implemented otherwise", and we'll end up with exponential complexity
   and it will be a real issue. We still can't *test* test -- in the sense of
   runtime tests -- every configuration, but compilation tests have been a major
   contributor to our code's stability and finding bugs. Every with 6, 8, 9
   interoperable features, it will be hard to test.
 * Alyssa: You don't need to test every single combination. If you keep new
   configs controlled -- don't add them very often -- and you have a set of
   configurations that should be tested, then with 7 features you probably only
   need 10 tests.
 * Alexandru: I think the question that Leon tries to put here is "where do you
   draw the line?" How many features do you add?
 * Phil: I think that the fact that Linux has farms testing random
   configurations doesn't make it desirable -- it is a lesser of two evils
   option.
 * Phil: This is the first feature of this kind in Tock. It isn't something that
   happens very option -- it's touching on the security model and relationship
   of processes with the kernel.
 * Alyssa: What do you mean by "of this kind"?
 * Phil: Adding a substantial feature to the kernel, really changing the
   relationship of how things work and increasing kernel size by this much.
 * Hudson: Typically features are added in capsules. E.g. we added the process
   console, and if you don't want it you don't add it to your board. This is
   unique because of the extent to which it has to be integrated into the core
   kernel.
 * Alyssa: Okay
 * Phil: We've had things like the system call ABI, where we decided to redesign
   it, but not where a new feature is added. Can somebody else think of
   something like this?
 * Hudson: You could argue a lot of the stuff that extends TBF headers, but
   that's a much smaller overhead.
 * Leon: And those are things that we can remove without touching core parts of
   the kernel, low-hanging fruits. When we're at that level of trying to
   optimize things, we can say that for some target audience we should have an
   entirely system call interface and buffer sharing, and at that point it is no
   longer Tock. Where do we draw the line?
 * Alexandru: It's a slippery slope here -- as soon as we start adding features,
   we will find arguments for adding new features.
 * Alyssa: The argument will probably be that it is adding code size a ton and
   we can't keep doing that. If it's in a capsule then you don't need to add a
   feature, but for these sorts of things that change the core security model of
   Tock -- that as far as I can tell Ti50 doesn't need -- are the sort of things
   that are appropriate to gate behind features. The things that change
   fundamentals for what security the kernel provides and its associated costs.
 * Brad: On that point, I don't think this changes, I think this is realizing
   what we always thought the kernel would do and haven't implemented. Like
   maybe the restart process and not just having processes crash and panic the
   kernel. These are the changes that we wish were on day one, but of course
   can't be. We're really realizing what we set out to do when we said we wanted
   to have a security-focused kernel.
 * Alyssa: I understand. It's difficult balancing
 * Alexandru: At least for our use case in automotive, this feature is
   paramount. Were it not for Phil, we would've implemented it.
 * Phil: I think Ti50 has a really important counterpoint -- you know, rather
   than having per-process credentials, we will just sign the whole thing
   together and we will verify the processes when we verify the kernel as part
   of the boot process. You can couple than, and check the whole thing.
 * Alyssa: The cost of dynamicism, essentially.
 * Phil: What I hear from this is that right now we're at 1200, and if you're
   not using this then we want to minimize the cost. For Tock TBF, we may be
   able to pull that out so certain types aren't parsed and yeah. But also, your
   approach of having a `ProcessStandard` and a `ProcessUnchecked` would also be
   another approach.
 * Leon: I don't know whether this approach will stay in the long term and be
   worth maintaining, but it is also a goal to have two `Process`
   implementations. May want to have two process types with different security
   models.
 * Phil: I think there will be some shared core functionality because we don't
   want code duplication, and can factor aspects like security models that are
   different.
 * Leon: We'd essentially have a diamond structure -- one trait, two
   implementations, and a common backend.
 * Alyssa: I like the idea. My thought is to take the existing `Process` code
   and the PR for app IDs, consolidate them into a base process, and have them
   delegate as much as possible to share code.
 * Hudson: I don't hate the idea -- if we're going to have multiple process
   types -- of having a base process others rely on. I'd be interested to see
   how clean an implementation of that ends up being.
 * Phil: I think it is necessary -- you don't want to have two lines of code
   that do the same thing.
 * Leon: There are valid reasons to have entirely differently `Process`es that
   work completely differently but this is not one of them.
 * Phil: Agreed
 * Hudson: We're excited to see what you'll propose, and then we can have a
   concrete discussion of the tradeoffs.
 * Alyssa: I also need to understand what your hangups are on configs so I know
   what to avoid
 * Hudson: The primary hangups are the testing concern and concerns about
   fundamentally changing kernel design.
 * [Brad pastes
   https://github.com/tock/tock/blob/master/doc/Design.md#ease-of-use-and-understanding
   into chat]
 * Phil: I think this is coming from the embedded side of things. You can't just
   store the configuration in the kernel because there isn't a way to get it
   out.
 * Hudson: Brad, did you want to take some time to talk about the state of
   things for the changes you've made to `tockloader` and `elf2tab` for app ID?
 * Brad: Phil really kicked this off, I've been playing around with `elf2tab`
   and `tockloader`. I haven't done much testing with boards and deploying the
   kernel + apps and with the kernel PR. The current state is `elf2tab` can add
   signatures and reserve space for future signatures, and `tockloader`
   understands them now. It can parse them, add and remove them, and check they
   match the rest of the TBF application. Hopefully can also flash them onto
   boards. Support is preliminarily robust in what it can do -- there are
   probably edge cases that need to be tested. Provides the same experience as
   tockloader provides to other tasks, just to the new footer. I'm sure there
   are additional features that we'd want, but for the initial implementation
   the features should be pretty good at this point.
 * Hudson: It seems most of the PRs can be merged now. Are you waiting for more
   reviews there, or is the hope that some of that will go through in the next
   couple days?
 * Brad: I would like to have hardware in front of me, and double-check that if
   we do update `elf2tab` and `tockloader` that nothing breaks. Don't expect
   that, but I'd like to confirm. Do have some time because we don't expect
   people to download the latest-and-greatest of the tools. I do think we should
   get them in there so people interested in working on this can use the tools
   without having to manage the PRs. The feature should be backwards-compatible
   -- we want to support people who don't update their kernel right away.
 * Hudson: That's a good point. Where did we end up on that one portion of the
   original PR that was going to require an elf2tab update?
 * Phil: We decided not to do that. There was a difference between documentation
   and implementation on an offset, and we decided to update the documentation
   to match.
 * Hudson: That makes the `elf2tab` stuff easier.
 * Leon: I was going to do a shameless plug -- the QEMU board and flash loader
   support should help test.
 * Hudson: It seems the LiteX CI has found a bug that currently exists in the
   PR.
 * Leon: Not necessarily a bug, just an incompatibility. If there's any issue I
   can look at specifically, please let me know, as I know it is difficult to
   get working on anything but my setup.
 * Hudson: I managed to get LiteX to run and toggled stuff in the kernel. The
   kernel seems to be running fine -- no panic -- but the process is being found
   at the wrong location. Probably related to these changes.
 * Leon: That shouldn't be LiteX-specific, should be RISC-V in general.
 * Hudson: Or maybe something about the `elf2tab` stuff.
 * Phil: I have been able to run in Verilator on Earlgray. Definitely will keep
   on working on this. Have been able to test on Imix, would greatly appreciate
   help testing.
 * Brad: So others are aware, on Cortex-M platforms where we pad to a power of
   two, pre-this-change, we did the padding by adding zeroes to the application
   binary. Now elf2tab tries to opportunistically insert a reserved footer
   credential section. The idea is you can reserve space such that if you want
   to add a new credential later, you don't have to modify a part of the
   application covered by another credential. So on Cortex-M, even if you don't
   use any credential stuff, you will still end up with one, which should be
   fine. An old kernel should ignore it. If you look at `tockloader` you may see
   the credential show up.
 * Hudson: I wanted to ask before Phil had to leave and didn't, but I do wonder
   if we want to consider trying to quickly tag a minor release before we merge
   the app ID PR. It's a major change, and we have over a year of changes since
   Tock 2.0 that are meaningful improvements but not fundamental changes. Would
   be good to have a tested release with all the new post-2.0 stuff.
 * Leon: I think this would put us in the same situation as we were after 2.0,
   where we immediately changed how Allow works then didn't do a release for a
   long time. So we should probably do a release afterwards. I also want to fix
   unsoundness w.r.t. userspace buffers shared with devices, and maybe want to
   push that into the first release as well.
 * Hudson: I think you're right we should also try to do a release relatively
   soon after this. I don't think we should have the mindset of releasing every
   year, and we should try to get back to more frequent releases when we have
   big items. I think that you're right that it's a big issue that we haven't
   done a release since Allow -- that's been blocking `libtock-rs` development,
   which is unfortunate. Also easier to issue a release if you recently issued a
   release, because stuff's more tested and you find fewer bugs.
 * Alexandru: I agree and think a release before app ID would be good. I would
   like to incorporate Leon's Allow soundness work.
 * Brad: This is a bigger discussion: if we did want to do a release, what
   should we include?
 * Hudson: Maybe that should be the first item for next week. There's one minute
   left, and a couple people have had to drop already.
 * Leon: I didn't want to decide on this now, just raise the question, thought
   I'd have direction.
 * Hudson: We can talk more next week.
