# Tock Core Notes 2023-06-16

Attendees:
- Alexandru Radovici
- Alyssa Haroldsen
- Brad Campbell
- Hudson Ayers
- Johnathan Van Why

# Updates
* Alyssa: I'm taking another look at the MapCell safety PR (#3325)
* Brad: I have TicKV support in `tockloader`. Also we can now add TBF headers to
  compiled binaries, which is useful if you want to add permissions or
  something. I also have a HMAC-SHA256 implementation, but that is pending on
  potential changes to the Digest HIL, which is why I put it on the agenda.
* Hudson: I guess Leon's update is that he has switched us to merge queues.
  Tracked in #3428, and it was #3483 that got this in. It's been tested on a
  couple PRs. Interface much nicer than Bors. There are a couple things still
  outstanding, such as the Netlify integration not working. If docs break in a
  scenario that only happens in the merge queue. Fixing that would require
  deploying on pushes to all branches, which may cost more money. It is no
  longer a required check. I can't remember any cases where Netlify broke but
  normal docs worked. Leon proposed a workaround to build docs using GitHub
  Actions rather than Netlify, proposed in a PR, already looked at by Brad and
  Johnathan.

# PR #3482 -- source of XML files (adding headers)
* Johnathan: There was a question about where files added by @valexandru came
  from. Want an answer so we can add the correct headers.
* Alexandru: Looks like garbage added by MCU tools. I'll ask Alex. Can probably
  remove them.
* Alexandru [later, in chat]: I got an answer from Alex, the redlink tool used
  to flash the device uses this file. This can be seen in the Makefile. This is
  similar to openocd.cfg.
* Hudson [in chat]: Sounds like we do need those then. Do you know how
  the files were originally generated?
* Alexandru [in chat]: I think that mcuexpresso generated them

# PR #3479 -- Add `set_client()` for `ClientDataHash` and `ClientDataVerify`
* Brad: I'm not sure where this stands. Originally there were different traits
  for doing 2 of the 3 operations. There are traits for different client types
  for different operation combinations. However, there isn't a way to use that.
  I'm trying to add functionality to call `set_client` for each client type.
  However, the Rust client types don't work for that -- can't save a supertrait
  somewhere when you only need a fine-grained trait.
* Hudson: To make in concrete. You want to save which supertrait where?
* Brad: Say you have something that implements `Digest`. You call
  `my_hmac.set_client()` and give it a `DataHashClient`. But there's also a
  `set_client` that just takes a `DataClient`. What if you call both and set
  both clients? Does that overwrite the other one? Does it not do anything
  because you already set the client? Right now it doesn't do anything, we only
  use the super traits, not the fine-grained one.
* Hudson: I think the answer is runtime checks. It should always be the case
  that all of the clients either have the same object for all 3 or you have 1.
  The first time you can call `set_client` for `DataHashClient`, which will set
  both, but if there's a later call then a runtime client should refuse to
  accept a different client for `DataClient`. If you want to change the client,
  you should have to set them all to `None` first. Or do we want to support
  having different clients?
* Brad: I think the intent was to have that finer control.
* Hudson: Okay. I was thinking the intention behind the design was to handle
  hardware that only supports some operations.
* Alyssa: Won't this require a bunch of extra vtables to be generated?
* Hudson: It'll require 2 more vtables than a single shared client. I don't know
  how many instances.
* Alyssa: Minimum of 4 words each.
* Hudson: If you have an application that has multiple clients of this HMAC
  hardware, that's a decent amount of complexity, you would expect to have some
  size.
* Alyssa: Yeah, I'm thinking if there's a way to sidestep the problem and have a
  single trait, and have that trait describe whether a feature is supported or
  not.
* Hudson: Do we have an example of somebody setting a different client for these
  operations?
* Brad: I think the answer is no. It's really hard to reason about what that
  would look like. Maybe `Digest` isn't a good example -- you want to be able to
  add data, and verify it's not something else, but when would adding data know
  to call verify? The straightforward case is "I need verify, so I specify that
  trait, and if it doesn't implement it I get a compile error". I think our path
  forward right now is we can do the 3-clients-or-1-client, and by using a
  nightly feature
* Alyssa: Keep in mind the size impact.
* Brad: Or we can keep following the use-the-most-super-trait approach.
* Hudson: It's hard because we don't have a use case, but it seems clear it
  could be useful.
* Alyssa: Why not just return a not-supported error if it tries to do an
  operation it can't.
* Brad: We want compile-time checks.
* Brad: You could imagine something that has two fine-grained traits, where you
  only want to use one.
* Hudson: Is ADC and ADCHighSpeed an example?
* Brad: It's not. Maybe that's the problem -- there isn't a good example where
  they don't nest in a more usefully-separate way. For Digest, who will have a
  use case where you only want to add data. Say we didn't have the data one,
  just hash and verify. It may be very feasible to implement both but have a use
  case where you don't need both. When you call `set_client` it doesn't do
  anything and you don't get a callback, so you have to support the verify
  callback.
* Hudson: If you don't want it, then drop the callback.
* Hudson: I have to drop for another meeting, I'll take a look later today.

# HMAC and SHA virtualizers don't virtualize
* Brad: I was trying to implement `set_client` for that code, but didn't realize
  the code doesn't do what it's supposed to and couldn't get the types to work.
  Johnathan?
* Johnathan: I haven't really looked at it, kinda avoiding the crypto stuff. The
  OT project likely won't use it -- we'll do our own bespoke stuff -- except
  maybe to interface with app ID verification.
* Brad: We kinda merged a bunch of code we shouldn't have because it doesn't do
  anything, now we're in an awkward spot.

# MapCell Safety PR (#3325)
* Alyssa: How did you test the size impact?
* Brad: I think Hudson and others ran the same tool as the size CI check.
* Alyssa: Can I run that on my machine?
* Brad: Yes. I'm sure it's a `make ci-<something>`.
* Alyssa: Johnathan, do you know something about it?
* Johnathan: If it runs in GitHub Actions, then you can find the workflow file
  and find the command from there.
* Alyssa: I had some ideas for how to decrease the size, mostly by reducing
  panics. Could get rid of `try_borrow_mut` to get rid of the reentrance, can
  fix soundness and save space.
* Alyssa [conversation resumed after next conversation]: What is the
  relationship between MapCell and TakeCell?
* Brad: We had issues using TakeCell with large types, as large copies would
  happen. Created MapCell to address that.
* Alyssa: Looking at the source code, the description of MapCell looks like the
  opposite of reality.
* Brad: I think we're above my pay grade. I think you talking to Amit would
  resolve this in a few minutes.
* Alyssa: If I sent in a PR with some suggested rewordings, would people like
  that? I'd love to know the original motivation for this before I do it.
* Brad: I've told you what I know about it. I use `tock-cells` but don't
  implement it. It seems that clarification would be great.
* Alyssa: I guess the first step is for me to reproduce the case that caused
  MapCell to be created.
* Brad: IIRC, the high-speed multi-buffer ADC is where we noticed issues.

# TockWorld Tutorial
* Alexandru: I would like to start preparing. Is there a document somewhere for
  planning? Any audience you know about? I see you've chosen a Nordic board.
* Brad: There seems to be a limited audience, we need to invite people. We have
  buttons and LEDs. To answer your question, no, I have nothing to drop in
  that's tested.
* Alexandru: Is the intent to have pre-set-up breadboards/extensions/etc or have
  the attendees wire it themselves.
* Brad: One option is to use the bare board without anything.
* Alexandru: The purpose being "this is how you upload Tock, this is how you
  sign apps, etc".
* Brad: We can talk about signing apps, we can write capsules, we can have
  malicious apps.
* Alexandru: That was my question -- what can we do without additional sensors
  attached?
* Brad: Nothing has been decided yet, listing options. I kind of like the idea
  of having options so if at attendee wants to connect sensors they can. If
  someone is more interested in Rust or security or whatever we can have a
  version for that.
* Johnathan: zeroRISC has interest in a tutorial on writing capsules and on
  using `libtock-rs` apps. May have representation on this call in the future.
* Alexandru: Should we assume applicants know Rust?
* Brad: Not sure.
* Alexandru: If they've never written a state machine and don't know Rust it'll
  become difficult.
* Brad: Yeah, that's true. My impression is that if capsule writing is too hard,
  just getting a start at it is fine. I don't think we should have something
  where you need to complete part of it to do another part, that could be
  problematic. I think it's okay if someone just can't complete it. It's just
  one part of it.
* Alexandru: I've seen an interesting approach in the UX. They had several
  milestones, but if you miss one, they have the code for the next milestone.
* Brad: That could work, yeah.
* Alexandru: Do you want me to sketch a proposal for next week?
* Brad: I think we should work towards convergence, absolutely. I've been
  writing guides in the Tock book. They're not finished but there are pieces. My
  approach has been to have the different guides and we can pick and choose
  which to use. Then people not at the tutorial could use them in the future.
  They're in a branch in the book repository.
