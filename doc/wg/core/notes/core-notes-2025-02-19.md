# Tock Meeting Notes 2025-02-19

## Attendees
 - Branden Ghena
 - Brad Campbell
 - Pat Pannuto
 - Johnathan Van Why
 - Amit Levy
 - Tyler Potyondy
 - Viswajith


## Updates
 - Brad: Dynamic process loading continues to make progress
 - Brad: Writing the TRD helped to identify things that we need to improve, especially error cases / etc
 - Brad: Both libtock-c and kernel implementations keep getting closer... thinking about how to test this. Added a feature to Tockloader so that Tockloader can now write apps and padding apps exactly as your specify (as opposed to normal behavior of ordering by size / etc). This would allow you to set up the platform with some installed apps, then do dynamic loading, and see what happens
 - Branden: I have a hard time following what's a draft PR versus what we should really be looking at. e.g., the TRD, is that something we should be looking at?
 - Brad: Yes, the TRD is reviewable
 - Brad: 4335, don't know the status there, Vish?
 - Vish: That can be closed.
 - Vish: 3941 is the one that everyone should get their eyes on.
 - Braden: Which to focus on first?
 - Vish: Probably the TRD first; gives best overview. 3941 is bigger / implementation.

## March Tock Workshop
 - Amit: Need to finalize time date. Days/times in question are:
    - Mar 25 at 12:00 US/Eastern
    - Mar 27 at 14:00 US/Eastern
 - Amit: The concern with the 14:00 Eastern is that it's later in Europe, roughly 9pm
 - (various): lots of more minor reasons the 27th is more convenient for many
 - Amit: That isn't too late; let's go for it
 - Pat: And we settled on 4 hours long?
 - Brad: Yeah, that was somewhat arbitrary, but that's the plan for now.
 - Amit: Next action item, I will take on, is remaining logistics, agenda, advertising, etc

## MobiSys
 - Amit: Tyler, can I put you on the spot to remind us what has been accepted, what degrees of freedom, etc?
 - Tyler: The plan/thinking is to target the MobiSys tutorial more at what is a RoT, how does Tock provide this?
 - Tyler: Request from MobiSys organizers to have more industry engagement; sounds like that is in the works?
 - Tyler: Plan is a mixture of lecture and tutorial content.
 - Tyler: For tutorial, have flexibility, but the plan at the moment is kind of the standard (1) this is Tock, then (2) syscall filtering, process loading, etc, and (3) something wireless
 - Pat: The accepted proposal is pretty open-ended, and the organizers seem very flexible; so we are in a do what we want
 - Pat: Some folks from ZeroRisc are confirmed interested in coming and contributing materials and support for RoT
 - Pat: The todo for us really now is to decide what we want to do, who will generate materials, and who will run the event?
 - Pat: So, who would like to be involved in the generation of materials and running of the tutorial?
 - yeses: Tyler, Amit, Brad
 - Brad: Prior tutorials have been building-block-y, each part builds on the other
 - Brad: One thing that is coming out here is a more modular approach; maybe we set things up so that we have modules that you can drop-in/out on and participate in independently; each one is standalone; people can participate in the one they find most interesting, etc
 - Brad: Plus a "module zero" which is a setup kind of thing
 - Brad: This lets us be more adaptive to what people want
 - Brad: Lets us do RoT without committing whole event to it
 - Tyler: Like that idea. We do encounter the people who come in halfway through for various reasons, or can't stay whole day, etc
 - Pat: Something more drop-in friendly likely better fit
 - Brad: To push back on own idea... will definitely be harder if you don't attend the getting started module... maybe the RPi with VNCs which worked really well at EWSN is enough to mitigate that
 - Amit: It's also partially a question of what we do in the module etc
 - Amit: We should think some about hardware platform; we have been using nrf52840dk's for convenience and we have them; but not the best for RoT. Maybe it should be FPGAs running OT or QEMU
 - Pat: Doesn't have to be same platform across whole thing; don't want to throw away nrf52840dk and working things
 - Amit: Yeah, ultimately depends on what we want to do for various modules
 - Brad: Vish and I have talked about showcasing process loading
 - Amit: Though that may be discordant with what ZeroRisc does; they are interested in dynamic loading, but don't want to do that yet
 - Brad: Do want to keep in mind the MobiSys audience; not going to be a room full of people thinking about RoTs
 - Tyler: Doesn't have to be completely coherent across modules, can have an overarching theme, but don't need content building with this view
 - Pat: Summarizing—we will most likely have an event with "3.5" modules, a 0.5 Tock Setup and then three semi-independent modules
    - A "RoT" module likely run by Amit / ZeroRisc
    - A "Virginia" module run by Brad / Vish
    - A "UCSD" module run by Tyler / Pat
 - Pat: Details of the modules can be a bit TBD for the moment, and we can put together an overarching narrative as it solidifies
 - Pat: I will take as action item consolidating people who are interested in working on this to one email list, scheduling a planning meeting, etc.

## Quick PR Nudges
 - Brad: Anything left for Machine Register? (Group: no, merge)
 - Pat: Alistair posted a few PRs that needed looking at in slack? On second look, those are libtock-c things and probably my responsibility; will try to look sometime today hopefully
