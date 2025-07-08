# Tock Core Notes 2023-09-08

Attendees
---------

 - Pat Pannuto
 - Amit Levy
 - Alexandru Radovici
 - Hudson Ayers
 - Viswajith Govinda Rajan
 - Branden Ghena
 - Leon Schuermann
 - Philip Levis
 - Brad Campbell
 - Chris Frantz
 - Johnathan Van Why
 - Tyler Potyondy

Updates
-------
 - Brad: I bought this, https://www.makerfabs.com/makepython-nrf52840.html, nrf52840 with a display; possibly a way to test the screen stack
 - Brad: There is a bug in their schematic... so they may not have tested this much
 - Phil: What's the bug?
 - Brad: Using a drop-in module, and they swapped two pins; LED is on 1.10 in schematic but on 1.11 in practice
 - Hudson: How did you find that out...?
 - Brad: To their credit, there's a github repo which includes their hardware file. Tried to figure out which nrf module they used. Used an image search of all things to find the module, and realized the pin order was different
 - Hudson: What's the bootloader situation? Same as the Clue boards?
 - Brad: Maybe? There are two USB headers, so you can have a UART if you want or USB direct to to NRF; I used UART so tockloader can do the baud rate trick
 - Brad: Bootstrapping however for first load of bootloader will require JTAG
 - Hudson: And there's still the panic issue with CDC...
 - Alexandru: No debugger, just USB connection?
 - Brad: Correct.

 - Tyler: Quick Thread update: PR went out middle of the week, #3683
 - Tyler: Tock now consistently joins open thread network, child can join a parent; no heartbeat yet, but joins work reliably now
 - Hudson: That's very exciting :)


Component PRs
-------------
 - Hudson: PRs from Brad: #3657 and #3681
 - Hudson: Brad, start with the `pub type` PR? (#3657)
 - Brad: Originally motivated by the key/value stack; many layers and it doesn't use `dyn`; types become very redundant and long
 - Brad: There's been discussion on the OT call that asks whether there is a nice way to do this? How bad are macros? How much of this should be integrated into components versus something we just kind of try out in one place?
 - Brad: It's not the most exciting PR, so want to raise attention and get some thoughts
 - Phil: I look at the change in code and I'm excited....
 - Brad: That's why I opened the temperature stack change; without that changes look like swap of one line of code for another
 - Phil: That's true, but it matters where the code is and what it represents
 - Phil: Having a complex type definition such that one a type is used it's simple, that's like C++ templates -- footnotes, not inline digressions
 - Hudson: Yeah, I think I'm pretty on board. I think we should be trying to avoid `dyn` when possible, and this change makes avoiding `dyn` more palatable
 - Leon: We talked about this on the OT call Wed, but what turned me around is the thinking that these type aliases do for type creation what components to for encapsulation of peripheral creation
 - Brad: Other thoughts?
 - Phil: Ancillary point, but, components and their macros are always challenging and we've evolved with discovery of new subtleties... do we have a doc anywhere of the whole view of the component system, what the macros all do, and how the system works?
 - Brad: There's really two layers. The `static_buf` family of macros are well documented. We cleaned this up, and now there's just some upper-layer macros that effectively call `static_buf` a lot; not sure those are documented well anywhere
 - Hudson: Are you counting your stuff in the Tock Book?
 - Brad: Forgot about that..
 - Phil: But nothing in `doc/`
 - Hudson: Part of the problem is that we keep planning to improve things here, so final docs a bit held back
 - Phil: Yeah; not a blocking thing for this PR, just flagging that when components change, others come back to it and can be lost
 - Brad: The only real documentation is in tutorial format, in the book, and in the porting guide
 - Brad: But there's not doc with the rationale, constraints, etc
 - Brad: If there are any other thoughts/comments please post on PR
 - Brad: The reason I have more tepid enthusiasm is because of the change to the top of `main.rs`; we're going to end up with a huge block of these type definitions. Have to use in the platform struct and where we instantiate the component. It's a bit clumsy I think
 - Leon: Is this part of the change strictly necessary? Can we keep the change contained to the components?
 - Leon: I agree, having the virtual kv types without any context is not very elegant
 - Brad: It doesn't have to be, but my main motivation is the Platform struct; without declaring the types earlier, we'd have to use the existing giant type definition
 - Leon: I don't think that's entirely true; would have just the base types without lifetimes or platform-specific bits
 - Phil: Your point is well taken, but that may just be an artifact of having only one `main.rs` file; where other paradigms would have a dedicated declaration file that hides it away. For all of C's failings, `typedef` is pretty awesome
 - Brad: Leon, I'm trying to see if I can see how that would work; my experience writing this PR is that it's really hard to keep track of all of this. Once it's there it's clear, but creating it is really hard
 - Leon: I think--not sure if true---there isn't any real guidance on how to use components and which layers should plug into what; we have like 10 layers of nesting, but it's not clear what component plugs into what other component; can see the frustration in trying to write that
 - Leon: Now we have two stages; instead of plugging types as they are realized, you plug the outputs of one component into the input of another; and this entire type composition is a bit statement at the top of main. Indifferent to the necessity of that second step.
 - Brad: Right, and I think that's another thing we have wrestle with a bit. Similar to how using `dyn` simplifies things, we've also written components such that you take whatever was in e.g. imix, a first reference impl, and factoring that out. Coming from one implementation leads to unintended structure on how things are to be used in components. Not clear when things should be encapsulating more or less. The way the types actually work out will ultimately depend on how much encapsulation there is, and this encapsulation makes it hard to explain how to compose things
 - Leon: Right, and this is really circling back to the challenges in composition, not type aliases?
 - Brad: There's an expected structure, do we encode that in the component somehow such that there can be less redundancy in the `main.rs` files if you're just doing the 'standard' thing
 - Leon: There's two parts to encoding structure. On the one hand, this is where type aliases fall short; they allow for these template patterns, but don't allow for traits. The compiler can only tell you that the composition of underlying types doesn't work, not that the composition of aliases shouldn't have work. Now this also means that you have to understand the subsystems in order to understand how to compose holistically.
 - Brad: Exactly. The fine-grained components are great when you want to be able to do something different, but it means the whole stack has to be in your working memory in order to be able to compose anything.
 - Brad: What I'm hearing is that this is a step forward; no real objections; worth adding. I would like to play around to see if we can move any of what's in `main.rs` into the component in a way of explaining "this is a valid way of composing components in this file". It's a type, you can use it or not, so not binding, but eases
 - Phil: That sounds good. This is a step forward, but not a final
 - Brad: Yeah, this is actually the second PR exploring this; the first was very macro heavy, did not like that as much

libtock-c newlib updates
------------------------
 - Brad: This kicked off with Alistair updating newlib to 4.3
 - Brad: We've had this weird dependency in that we ship newlib, but also requires that you have newlib installed so that you have headers
 - This PR removes any dependency on local install; so can use toolchains from ubuntu/risc-v/arm/homebrew/whoever
 - This thus builds everything by default now, and provides the libc/c++ libraries for risc-v
 - Leon: This is awesome.
 - Leon: But, didn't we have an issues that newlib headers are GPL licensed?
 - Brad: That's kind of why I brought it up
 - Hudson: Are the headers GPL??
 - Leoon: They seem to be mostly BSD, but there's a smattering of other licenses; couple public domain, couple redhat, couple GPL... this is a mess
 - Hudson: Yeah, that's frustrating
 - Leon: It's always bothered me a little how much we vendor
 - Leon: What a lot of other projects do is keep built things out of the repository, and download / link / etc on build
 - Pat: That works? That we have a built thing that we host that people download and that's okay?
 - Leon: I'm not a lawyer... but that seems to be state of the art
 - Johnathan: I suspect if we distribute binaries we have to be able to distribute sources
 - Hudson: Can't just link to sources?
 - Johnathan: Would need a backup, if that link disappears, we have a problem
 - Philip: The variety of these header files makes me skittish
 - Brad: About merging them? About using them?
 - Philip: Umm... both? all?
 - Johnathan: We have an existing issue with libtock-c that's a licensing concern
 - Johnathan: It hasn't been address yet, but we had been considering moving to picolib-c or other implementation
 - Leon: That open PR looking at that only compiles one application that does no allocation; picolib-c isn't compatible with Tock's `sbrk` implementation
 - Pat: Could we just add a `memop` to make this work with Tock for whatever their `sbrk` needs?
 - Leon: picolib-c doesn't support any kind of hook for a custom `sbrk`; i.e., can't perform `memop` in piclob-c; it expects only a flat address space
 - Phil: So it expects no MPU
 - Leon: Yeah, only bare metal
 - Alexandru: Could we commit something to picolib-c?
 - Leon: Yeah, probably; their build/link/etc ecosystem is just complex
 - Leon: The summary from the open PR / first experiment is that updating to picolib-c is a significant undertaking
 - Alexandru: huge problem for OxidOS
 - Branden: What does industry use
 - Alexandru: Use vendor libraries from Vector etc. There are required certifications. These libraries cost $1k+ per-project.
    - NXP uses redlib: https://community.nxp.com/t5/LPCXpresso-IDE-FAQs/What-are-Redlib-and-Newlib/m-p/475288
 - Branden: So even if we wrote our own, we'd have a problem
 - Alexandru: Yeah. We actually hit this problem with Rust too. Ferrous systems certified the complier, but not the core library; and there isn't sufficient tooling to enable certification of the core library currently
 - Brad: Should we just vendor all this crap in one bit zip blob?
 - Leon: I'd be in favor, if only because we're currently mixing the headers from some random Debian image with something we've build
 - Brad: The other thing with this pull request is that it adds multiple many-megabyte libraries
 - Leon: This always surprises me with libtock-c on a clone
 - Brad: Yeah, so this would be the way out
 - Phil: Looking at picolib-c, it has support for things like `mmap`; it really expects only a flat address space?
 - Leon: May be more than than; but if you dig into the `sbrk` path, it's a fixed implementation that just moves the heap until it hits a predefined start symbol. I tried to pull this part out; they have scripts that let you configure parts of the library for platforms, but this does not seem to be a configurable parameter
 - Brad: So, what we can do is create a package that people can download in support of libtock-c apps
 - Brad: And namespace it some what such that if people want to try picolib they can, etc
 - Brad: That would let us disentangle this; right now it's all really messy; it would be a hard switch, one or the other
 - Pat: Yeah.. that seems reasonable
 - Leon: It would be nice to get clarification on whether this actually solves the license problems with newlib
 - Leon: Though at-worst it's the same as-is license-wise, and is a strict usability improvement
 - Brad: ... assuming our mirrors don't go down :)
 - Leon: We can always add more!
 - Brad: But yeah, that's helpful

Last-Minute Update
-----------------
 - Leon: QEMU has new release that fixes PMP, one less blocker for that PR
 - Leon: This, unfortunately, also means that we won't be able to use any packaged QEMU for the foreseeable future, so we'll have to target some release
