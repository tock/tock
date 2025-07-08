# Tock Core Notes 05-07-2021

## Attending
 - Pat Pannuto
 - Amit Levy
 - Leon Schuermann
 - Branden Ghena
 - Johnathan Van Why
 - Brad Campbell
 - Hudson Ayers
 - Vadim Sukhomlinov
 - Alexandru Radovici
 - Philip Levis
 - Gabe Marcano

## Updates
- Leon: All non-virtualized capsules are ported, so next step is to rebase and
  hopefully merge the callback swapping PR.
- Amit: Any update on the Leon/Hudson AppSlice overhead exploration?
- Leon: Yeah I have had conversations with Amit and Hudson about this, and am
  writing up a document about it to share with everyone.
- Hudson: I am working to rebase the callback PR, most of the way there
- Alexandru: I have been updating the SPI and I2C HILs to better follow the
  guidelines in Phil's HIL how-to TRD.
- Phil: Some undergrad students at Stanford looking to reduce code size have
  not had a lot of luck with some things they thought would. But they did find
  that certain core library functions add a lot more code size than one would
  expect.
- Vadim: One trick I have found is that rather than entering grant in each
  command, move it into a single location. This reduced code size by a good
  bit.
- Phil: That makes a lot of sense.
- Vadim: AppSlice::take() becomes very expensive
- Phil: Because it can panic?
- Vadim: No it returns an option
- Phil: Cool, well seems good to investigate further.
- Vadim: Shared
  https://gist.github.com/vsukhoml/6e215fed2460efb47c6bc897931e92cc which
  greatly reduces code size compared to existing operations on AppSlice.
- Amit: Looks similar to MapCell
- Vadim: It is the take which is different.

## Tock registers update
- Amit: Leon has submitted a PR (https://github.com/tock/tock/pull/2517)
  that makes some large scale changes to
  the tock registers crate. I want to discuss them
- Leon: Basically, until now it has not been easy to re-use the register code
  for things like the RISCV CSRs or in-memory registers. To get there, this PR
  creates some Register traits that are shared by CPU registers and things like
  CSRs, which allows for a shared implementation of all the helper methods and
  lets us abstract all of the differences between the underlying hardware behind
  this trait.
- Leon: The main downside here is that we now have to import a bunch of traits
  into a bunch of files just so we can use the methods defined in those traits.
  Notably, Guillaume and Andre Richter both had similar ideas about how to do
  this, so I think a lot of users of tock registers see the benefit of this
  change.
- Pat: I put this on the agenda, because this changes an interface that is used
  by some people outside of Tock. I also think this represent a good
  opportunity to publish tock-registers as a versioned external crate. But if we
  want to do that I think we may want to seek more public feedback.
- Pat: This would mean we basically "redo" what Andre has already done with
  registers-rs, and would mean we have an external dependency in Tock, but its
  exactly the kind of external dependency we have said we would be OK with (no
  dependencies in that external crate, it is audited and trusted)
- Amit: My sense is that we could still have this be a submodule in Tock, or a
  vendor dependency, or we could pin a particular git version in our
  Cargo.toml. From a safety / auditability perspective this is basically as strong as
  having it in the repository.
- Amit: This is also not code that is used in capsules, which is reassuring.
  Nonetheless it feels like this can open a Pandora's box
- Johnathan: Pinning to a git commit in Cargo.toml is problematic for our
  code-auditing practices.
- Amit: This would not happen in the kernel crate, but in the chips crate
- Leon: Currently we actually reexport all those types in the kernel crate
- Amit: Ah right...that is mostly historical so we didn't have to change so
  many files when the implementation moved to libraries
- Leon: I kinda like this approach though.
- Hudson: One other note is this would probably also be used in arch/ for
  risc-v CSRs
- Leon: Also, potentially for networking code? So maybe in capsules?
- Amit: Not so sure about that because of how this crate uses unsafe.
- Leon: My next step after this PR would be a feature flag to disable the
  register types and only export the interfaces
- Leon: Note that currently capsules already have access to this code because
  of how kernel/ reexports it
- Alexandru: I have wanted to use these types in capsules for sensors that have
  register types
- Amit: What is the downside to just keeping this in the Tock repo?
- Leon: I think these two changes are orthogonal
- Leon: It would probably just be more elegant not to have it coupled to Tock
  versioning and so on
- Amit: So it does not seem as if this PR is blocking on that decision
- Pat: Yeah I expected to merge this PR on the call, but wanted to use it as a
  launching off point for this discussion
- Amit: Let's survey the room

  - Leon - take it out
  - Alex - keep it in
  - Phil - don't care
  - Brad - keep it in
  - Hudson - keep it in
  - Vadim - I'm fine leaving it
  - Branden - I fully support moving it into its own repo. Not strong feelings tho.
    Would signal more people they can use
    this. Fine to kick the can down the road
  - Pat - out, same as Branden, I think
    this is a good launching off point for external deps
  - Johnathan - no opinion as
    long as there is no git commit dependency in a crate OpenTitan depends on
  - Gabe - no strong opinion, kicking the can sounds good

- Amit: What is the specific issue with pinning to a commit?
- Johnathan: We have
  to either vendor in deps or use cargo audit. Cargo audit uses crate hash from
  cargo metadata which is not a git ID. This will make it hard to use automated
  tools for auditing.
- Amit: So a crate from crates.io with a particular version
  would be fine?  
- Johnathan: Correct. It's possible a git repo would be okay but
  I would have to look into that.  
- Amit: Another approach would be we pull in Andre's changes
- Leon: They are the exact same
- Amit: Cool, so we would combine
  forces but Andre's crate would just live in our repo basically
- Leon: Andre has
  said he would be open to just having the crate in our repo obsolete his crate
- Amit: Seems like we are gonna kick the can down the road. But does sound like
  eventually it might make sense to externalize some stuff in a thoughtful way.

## Path to 2.0
- Amit: Posted a link to https://github.com/tock/tock/issues/2429
- Phil: I don't think we should block on rewriting kernel crate internals
- Brad: Updating to 2.0 means doing a lot of changes to imports and such, I
  just want to minimize changes after 2.0
- Brad: Agreed that rewrites of implementations should not block 2.0
- Amit: I am going to strikethrough that on the list
- Amit: Should we talk about the remaining 3 items?
- Amit: Changelog is a no braining, and the AppSlice/Callback stuff has been
  discussed.
- Phil: I think it would be worth talking about reorganizing kernel exports
- Pasted: https://github.com/tock/tock/issues/2429
- Brad: https://github.com/tock/tock/pull/2551 is the most up to date
  discussion
- Brad: Basically, since we started Tock specific types and traits were
  exported directly by the kernel crate. This has gotten confusing and
  sometimes imports make little sense if you can't see the file these traits were
  written in (e.g. use kernel::ReadWrite).
- Brad: So I proposed some more namespaces to make all this more logical.
- Brad: explained the contents of 2551
- Amit: For the most part this looks good to me, and I agree the namespacing is
  currently a mess.
- Phil: Yeah I think this is an awesome step forward. My one question is
  schedulers -- why is that `pub mod`?
- Leon: Because the boards have to instantiate the schedulers and pass them
  into the kernel.
- Amit: That said, it might be good to hide some of that..
- Brad: I think we can leave Platform as Platform.
- Phil: I have some comments in the discussion about how some things are
  composed into a single trait that might wanna be split apart, but that may be
  a discussion for another time
- Amit: So it seems the high bit from this is that #2551 is broadly good and we
  may need to iterate a bit more but its close.
- Brad: Yes, and would love any more comments
- Amit: Doing that would leave the two mechanical things for 2.0, which is also
  good.
