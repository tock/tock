# Tock Core Notes 05/29/2020

Attending
 - Brad Campbell
 - Alistair
 - Leon Schurmann
 - Pat Pannuto
 - Samuel Jero
 - Johnathan Van Why
 - Philip Levis
 - Vadim  Sukhomlinov
 - Branden Ghena
 - Hudson Ayers
 - Garret Kelly
 - Vadim Sukhomlinov
 - Guillaume
 - Andrey Pronin

## Updates:
 * Pat: CI Revamp is almost done
 * Hudson: 15.4 interop works now, also an undergrad at Stanford has LED
   blinking on an nrf53
 * Phil/others: This is interesting bc it is a cortex-m33, it has
   trustzone-m support, and has a second co-processor for networking. We are
   interested in hardware interlocks only controllable from secure mode for a
   grant at stanford
 * Vadim: At Google we have been working on having multiple apps compiled
   with the kernel, and what I have discovered was that I was able to build
   the kernel as a relocatable objects so I am able to link apps with it but I ran
   into an interesting issue on the toolchain side which is that when I compile
   the kernel the compiler built-ins are linked in, so we expose symbols which
   conflict with those from apps written in Rust, so I was next trying to figure
   out how to be able to build kernel as library so that it would not contain all
   the compiler built-ins and std parts, but instead the std parts would be
   available as shared libraries for use by the kernel or by apps. I don't have a
   good solution so far, all my solutions are kinda ugly, I can rename symbols in
   the kernel, so they will not conflict, but not perfect.
 * Brad: I have been working on compiling libtock-c apps for risc-v by
   default, and working on tockloader support for TABS that have binaries
   inside that were compiled for a fixed address. That is a work in progress

## Component Interface  Last Call
 * Pat: Homework was to take a look and approve the new approach or suggest
   another
 * Pat: Holler if you have additional thoughts otherwise we are gonna move
   forward
 * Phil: I approve the design, think it is a step in the right direction,
   think we need to remember that the original goal of components was to
   allow for much easier configuration of boards and think there are still steps
   remaining to get there

## Tock 2.0 Roadmap
 * Pat: Listing major 2.0 changes from issue
 * Pat: syscall interface:
     * coalescing (command/allow/etc) to reduce # of calls
     * changing the arguments (especially returns)
     * revamp of timing interior
     * revamp of grant/process/allow interface
     * syscal:: fancy yeilds
     * syscall:: exit syscall
     * removing &dyn objects
     * removing static mut data
     * towards Rust stable:
         * removing in_band_lifetimes
         * driver registry
         * fully implement threat model
 * Johnathan: polyglot runtime no longer a 2.0 issue
 * Phil: before we go about prioritizing these, are we actually sure these
   are all things we want to do?
 * Phil: For me, in particular I have doubts about removing dyn objects and
   coalescing syscalls
 * Brad: what does getting to rust stable have to do with tock 2.0?
 * Pat: I had in mind that doing a release of Tock on stable would be nice
 * Hudson: Arent there some blockers upstream that we have no idea when they
   will land?
 * Johnathan: I am trying to get libtock-rs on rust stable, that would be a
   good test for if it should be possible for the main kernel.
 * Leon: Is rust stable even really a feature if it doesnt change the
   underlying binary?
 * Johnathan: It is if you are building Tock kernel within a larger build
   system
 * Sam: I think I would be inclined to start with the other set of features,
   especially if we are thinking of the point of Rust 2.0 as because of ABI
   breakage. If we get Rust stuff in there, thats great too, but I could see going
   to Rust stable on 2.1 or whatever, its not a major version change necessarily.
 * Vadim: I think Rust stable is kinda a nice feature which doesnt change
   anything from the application's standpoint but syscall changes is
   something which would be critical and for our use case once we start building
   big applications a working and mostly stable syscall interface on both sides is
   more important
 * Pat: Seems like there is a lot of interest in the syscall stuff, should
   we arrive on some order/mechanism for those changes?
 * Vadim: Also, should we comply with C ABI?
 * Phil: I am of the opinion that it should be in one branch, but we should
   serialize the changes in that branch. I dont think we should let this
   slowly trickle into master
 * Pat: Is there someone willing/able to lead the syscall redesign?
 * Phil: I think the obvious person there would be Amit (who is not on the
   call), so lets check with him
 * Vadim: On my side I am very invested in this but I am of course (newer?)
   to Tock development so my decisions might not be well justified.
 * Phil: If amit is not willing I will volunteer
 * Phil: I think besides the ABI the only one that is must do is fixing the
   time HIL
 * Guillame: Agreed that is important to fix, whether for 1.6 or 2.0, I
   think it is essential to fix this soon
 * Brad: I think we need to figure out exactly what is in scope of required
   for 2.0
 * Phil: I think we need a design process for figuring out what we want
 * Pat: we have 16 open RFCs, I feel like 2.0 is a good opportunity to
   decide yes/no on these things. But maybe we should just be focusing on
   ABI changes? I'm not sure.
 * Phil: we should offer good explanations on all RFCs before we
   accept/close any of them
 * Hudson: I think 2.0 should focus on just the ABI changes, personally
 * Pat: I guess what I really want is a pre-2.0 sweep of these changes that
   touch everything so they do not continue to languish forever
 * Phil: Those things are languishing for a reason sometimes, bc those
   changes are good but not good enough for someone to find a reason to do
   them. I think it is important that we have a clear statement of architectural
   direction and goals for the OS.
 * Phil: looking at the RFCs they really vary from "improving the flash HIL"
   to "stable rustc"
 * Phil: Suggestion: make it a priority for these calls/the mailing list to
   walk through these RFCs
 * Pat: Should part of passing judgement on an RFC require some sort of plan
   for implementation
 * Hudson: I think so
 * Branden: I think it helps but its still no guarantee it gets there
 * Leon: I think a two stage process could be good -- first propose the idea
   and get feedback and then a proof of concept implementation that isnt
   perfect for people to look at
 * Brad: RFCs can become tracking issues
 * Pat: So maybe the thought then is that moving forward we pick one RFC or
   two RFCs a week and start going through the backlog
 * Hudson: I think it would be good to pick them a week in advance to allow
   for some opportunity to discuss these things in advance
 * Brad: Is there anything special about what is tagged as RFC right now or
   do we maybe need to tag more things?
 * Phil: I think that is a bit off topic, might as well start with what we
   have
 * Leon: be careful to also include RFCs submitted as PRs
 * Phil: I will message Amit about 2.0
 * Pat: Lets pick an RFC for next week, I choose age priority
 * Pat: So, lets decide on the ADC HIL
 * Brad: Decide on what it will look like? Or just talk about it?
 * Pat: We should decide by next week whatever needs to be decided about it.
 * Brad: Is it controversial?
 * Pat: Idk, this hasn't been touched since 2019. Maybe we will just decide
   its fine as is and close the issue
 * Brad: I think the answer will obviously be yes lets change it, but we
   need someone to think a lot about it and actually write a new one
 * Hudson: Yeah given that we only have one ADC implementation (sam4l) it
   may be hard to figure out what the interface is missing
 * Brad: IMO a phone call is not a great time to talk about exactly how an
   interface should look
 * Branden: Maybe we should be labeling some of these differently to
   separate out the 2 concerns. It feels like the De ADC isn't really an RFC
   now and that issue is just a marker that we know this is gonna need a redesign.
 * Pat: When you look at Rust RFCs, the RFC itself is creating a document
   which is an authoritative document about what completing this RFC will
   result in. We on the other hand just have issue threads.
 * Brad: That sounds great, but is it gonna happen?
 * *awkward silence*
 * Brad: I agree with branden we need two tags, one which says we want to do
   something but we dont even know what the steps are and we need more
   discussion.
 * Pat: sounds like first the we need to do is categorize better -- maybe a
   RFI target (request for information) or "needsinfo" or "needs decision"
 * Branden: Is there a better issue you all have in mind we can start with
   (instead of ADC)
 * Pat: #1143 - redoing the kernel crate internals. The interface between
   the gargantuan process.rs needs to be rethought.
 * Brad: Doesn't that fit squarely in the other category of we want to do
   this but don't know how?
 * Pat: Well the issue with the ADC is we dont have 2 ADCs. For this we are
   just blocked on thinking.
 * Brad: 1501 is one that we have a trial implementation of but we havent
   decided if we wanna make it more prolific or just leave it as an nrf52
   artifact. multi language binaries is another example, where it is just a
   question of is that a priority.
 * Pat: We need to pick something for next week.
 * Brad: Maybe we can do that over slack
 * Brad: Part of one of the RFCs is future of tock hardware -- but it seems
   like more and more of the compelling boards dont have an FTDI chip which
   means that a USB stack is required for debug output. Does anyone have a good
   idea of where USB stands?
 * Branden: OpenSK does a not trivial amount of USB stuff for the nrf52.
 * Alistair: I also looked at USB for OpenTitan and I am gonna put some time
   into that once I get it setup.
 * Brad: Hard to know what state everything is in and what is essential or
   not. Also the tests fail on both imix and nrf
 * Hudson: We should probably add those tests to the release testing
 * Branden: We should ping guillaume offline and figure out the status on
   the nrf, and narrow down what the HIL for USB should really look like
   because it seems like an important interface that we should crystallize.
 * Brad: I am just concerned about getting CDC to work so we can support
   these boards.
 * Branden: I was reading a lot about that last night, I'll writeup an issue
 * Brad: This is starting to feel like a task force thing, whatever that is.
 * Hudson: I am gonna archive the old etherpad stuff because it seems to get
   really slow periodically
