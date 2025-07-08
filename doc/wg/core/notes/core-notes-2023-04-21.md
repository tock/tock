# Tock Core Notes 2023-04-21

## Attendees
- Hudson Ayers
- Amit Levy
- Leon Schuermann
- Alexandru Radovici
- Branden Ghena
- Tyler Potyondy
- Brad Campbell
- Johnathan Van Why

## Updates
- Hudson: Alistair asked for folks to look at PR 3360 -- he made some updates
  on the PR so now it does not duplicate capsule code and now it just adds new
  system call driver numbers. Alistair would appreciate folks taking another
  look at that, especially now that it is not a very big change
- Leon: I am still not a huge fan of adding new driver numbers for existing
  peripherals if they do not add any new functionality. I still think in the
  long-term that we want to have something like a driver registry where we can
  actually have semantic meanings for two instances of a driver on a given
  board and then a way to request a combination of drivers under a certain
  label. I am not opposed to this in the short term though.
- Hudson: Yeah, I think Alistair probably agrees with that but that seems like
  something which might still be a good ways off.
- Hudson: We were supposed to continue Alyssa's discussion from last week, but
  she is not here today so we will have to defer that agenda item.

## External Dependencies
- Hudson: We talked a while back about moving forward with external
  dependencies in the Tock kernel by starting to split up capsules. Leon got
  around to implementing the first split between core and extra, which is
  great, but we need to figure out next steps on this. My understanding based
  on PR 3346, which is where we initially had some of this discussion on this
  issue, is that we want to have an external subcrate outside of the other
  capsule crates, and that is where something like Alistair's external
  dependency would first live.
- Leon: I had the impression that when we split up capsules our intention was
  to sort of gradually start splitting out subsystems for which it is natural
  for them to rely on external dependencies, and limit the impact of that by
  having boards only rely on crates they require. I think LoRa was maybe an
  example where would just have a LoRa crate in the capsules folder.
- Hudson: Yeah, I think the original motivation for this was Alistair wanting
  to use ghash for AES-GCM in PR 3092.
- Leon: Yeah I'm recalling this. We had talked about the networking stack being
  its own crate and relying on smoltcp potentially.
- Hudson: I think that, recalling some more now, we had a couple options when
  we initially discussed this. One was breaking capsules up into a ton of
  crates, which people did not like so we did not go that way. This core /
  contrib split is more just a nice clean division for now and a roadmap to
  splitting up the capsules directory at all. I think we had talked about doing
  something like what you described, where every capsule that would require a
  unique external dependency would like in its own crate -- that is for
  Alistair's example we would have a crate for aes_gcm in the capsules
  directory, and only that crate would have an external dependency.
- Leon: I don't think this needs to be a strictly per-capsule basis, but if it
  does not make sense for a certain subset of structs in Rust capsules to live
  without an external dependency, we should probably draw some fuzzy boundaries
  about the precise subsystem that they encapsulate, and make that an external
  crate to separate that from the rest of the crates which we maintain.
- Hudson: Ok, yeah.
- Leon: I am not fundamentally opposed to a big external crate, but we don't
  want to make it so projects needing one dependency to have to pull in all
  dependencies.
- Hudson: I think we should tell Alistair to make a single crate for aes_gcm,
  in the capsules directory, which can have a dependency on ghash, and then
  opentitan can depend on that capsules.
- Hudson: We can figure out how to broaden the scope of this crate or create
  new ones as other external dependencies get added.
- Leon: Yeah I think that sounds like what we had talked about and what I had
  envisioned. We need to document this in the external_depenednecies.md file in
  this PR. I could give a shot at creating a new version of it which reflects
  this design.
- Hudson: Great, I am happy to look through that.
- Brad: Where would this new crate live?
- Hudson: In the capsules directory
- Brad: Is capsules a crate?
- Hudson: No, it is just a directory, core and extra are the two crates within
  that.
- Hudson: For external crates, we might want another directory under the
  capsules directory, to delineate that all crates within that directory are
  separate because they have external dependencies.
- Brad: So how much logic should these new capsules have?
- Hudson: I think basically the minimum amount of logic to get things working
  that does not require other capsules to depend on this one.
- Brad: Ah, but this new crate could use say the core capsules?
- Hudson: Yes.
- Leon: I think it is very important that we are not too strict in limiting
  what can go in these new crates, that was the motivation for splitting this
  up in the first place, since we concluded it is impossible to extract every
  crate into its own trait, especially because of issues with macros
- Brad: So these will be full-fledged capsules?
- Hudson: Yes
- Brad: But they are in their own namespace, so the dependency does not
  propgate
- Hudson: Yep, exactly, only boards that needs these deps will pay the cost of
  them
- Brad: I think this sounds like a nice solution
- Branden: I am sold
- Hudson: I like that we have this one example from Alistair that we can use as
  a trial. It is possible there is something we have not considered, so we
  should not be married to the initial text of this document, but we can always
  make changes based on our experiences.
- Branden: It helps that we do not expect to have a ton of these
- Hudson: Yeah, any maybe we should add some langauge to the document that
  makes it clear that we will not just accept any dependency.
- Brad: Do we have in this document anything that outlines the issues with
  other approaches, that describe why we chose this one?
- Hudson: No, I don't think so.
- Brad: I will try to draft that up and add it so we keep that knowledge
  around.
- Leon: Great, and then we can reconvene on this next week before we ask
  Alistair to make changes
- Branden: Put out a call for when you think its ready, I have been ignoring it
  for awhile since it has been in limbo
- Leon: I think this is pretty exciting for long standing projects like a
  proper BLE stack, for isolated subsystems I think this could be a really
  elegant solution.
- Hudson: I agree. I do think we got kinda lucky that Rubble did not end up
  being the first dependency that we used this for since it is in maintenance
  mode.
- Brad: Sure, but it definitely still informed this notion of board specific
  dependencies.
- Leon: I got a prototype working using smoltcp so that could be another thing
  to try this with
- Hudson: Yeah, Tyler, you should chat with Leon about Tock networking since
  his ethernet work probably has some overlap with what you want to do,
  especially regarding TCP and IPv6

## Tock World Planning
- Hudson: Brad, thoughts on hosting at UVA?
- Brad: Sure
- Amit: Yeah, the late July time period on the east coast worked best for most
  people, and we have not done UVA yet
- Hudson: I am always looking for an excuse to go back to UVA
- Brad: What did we do last year?
- Branden: 1.5 days, but the half day went long
- Amit: I think we can afford a bit longer this time. There might be some
  recent industry developments to discuss, and funding stuff. Might be good to
  do a training.
- Alex: I can volunteer to do a training
- Amit: That would be great
- Brad: It sounds like 2 days would be a good starting point.
- Hudson: I personally would request that it butt up to a weekend on one end

## Other Stuff
- Brad: No progress on stable compiler stuff??
- Hudson: asm\_const is blocked on inline\_asm, which at least seems to be
  making some progress
- Brad: I see, just added a link to that to our tracking issue
- Hudson: naked functions unfortunately seems to be totally stalled, and I am
  not sure there is a clear path forward there, which is a bummer since it was
  looking like it was going to be stabilized just a few months ago.
- Brad: Yeah, I guess let's just keep an eye on it.
- Brad: I have also been thinking about bluetooth. It seems like it is really
  hard to add, there is not good support in Rust. I would love any ideas on
  what we could do short of writing our own stack (which is not something I can
  do).
- Branden: Unfortunately I do not have any ideas
- Hudson: Yeah, there don't seem to be any great Rust libraries out there, the
  few that do exist do not inspire confidence. Most projects just seem to link
  against C libraries if anything.
- Brad: Has anyone tried just running the nordic softdevice on the network core
  of the nrf5340?
- Hudson: Not that I know of, I think even that approach has some memory safety
  concerns because both cores can access the same memory I think
- Amit: I think that fundamentally the right way to think of this is as not
  that different than a peripheral implementing logic like this in hardware.
- Hudson: Sure, anyway, maybe that is something worth looking into.
- Alexandru: I have students working on ambient light and ADC for libtock-rs.
  It looks like these are only implemented for Hail and Imix.Can anyone test?
- Hudson: I am happy to. I also can look into getting you some Imixes
  eventually.
- Alexandru: What is a realistic timeline for deciding on TockWorld dates?
- Amit: I am hoping within the next couple of weeks.
