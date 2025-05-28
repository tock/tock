# Tock Meeting Notes 2025-05-28

## Attendees
 - Pat Pannuto
 - Hudson Ayers
 - Alexandru Radovici
 - Johnathan Van Why
 - Amit Levy
 - Tyler Potyondy


# Updates
 - Johnathan: No longer at Google, now volunteering in free time


# Main Agenda

## Security Advisory

Redacted from agenda

### Action Items

Core team will follow policy in #4401 to move this along.


## Security Policy Updates
 - https://github.com/tock/tock/pull/4401 has been stalled
 - Pat: Can we unstick this on the call?
 - Amit: Can we come back to this after the next agenda item? Not sure this can be resolved on the call.
 - [paused; returned to at end of call]
 - Amit: Let's take the remaining time to re-read, and apply feedback directly the to PR?
 - Amit: And let's fill out the rest of the contacts table together now
 - Amit: Out of time, but this looks pretty ready, can we just merge?
 - Pat: Can you get the rest of the pending changes sorted and integrated into PR after call?

### Action Items
 - Pat: Action item: Everyone on call should approve (or not) asap; Pat will merge at noon Pacific if no objections.


## Tock Europe

### Context

We will organise the IP Workshop Summer School [1] between 10th - 20th August, in Transylvania [2]. It is an event with about 90 students (high school and university) and professors. One of the tracks that we will teach is embedded Rust, with possibly will be Tock.

We were thinking of using IP Workshop for a two day Tock Europe event. We have talked to some of the European industry partners that use Tock, like Infineon and zeroRISC (semi european), that are not able to travel to the US, but they, might be able to travel to Romania.

We have designed a possible schedule for the event [3].

- [1] https://www.ipworkshop.ro
- [2] https://maps.app.goo.gl/emUaYwcrNmHaCa3U9
- [3] https://docs.google.com/document/d/1ncEhN6y6nQuEGVXaJ2Nrzz-YAH_MNiV1E-SgROPJ

### Notes

 - Alexandru: Cost reductions make travel to US prohibitive for us and related orgs for the moment.
 - Alexandru: However, in-EU travel more acceptable, good opportunity for a "Tock Europe" event.
 - Alexandru: Can't provide funding for travel for others, but accommodations should be very cheap.
 - Amit: Perhaps the Tock Foundation can help with this; have some mechanisms to sponsor events.
 - Amit: Entities we'd imagine... Infineon, ZeroRisc, OxidOS
 - Alexandru: Summer school is Aug 10-20, ~90 high-school and university students take part. Very likely will teach Tock.
 - Alexandru: Will reach out to NXP (they sponsor the school already); will reach out to colleagues at universities; OxidOS has some talks we would like to share [some OxidOS employees cannot enter US at the moment]
 - Alexandru: OpenSK perhaps, as well
 - Amit: High level this seems great
 - Amit: It sounds like this doesn't cannibalize on TockWorld in the fall shortly after because there's a bunch of folks who won't be able to come to the US anyway
 - Amit: Can we think about the focus, tutorial, scope etc that this would target to make it complimentary to TockWorld?
 - Alexandru: Depends a bit on how we want to target things; we have ten days...
 - Alexandru: Infineon has WiFi working on thier chip; we have ARM64 port with MMU and address translation, going to start seeing some upstreaming there; these are examples of talks we could have there
 - Amit: Maybe there is a slightly different application focus; e.g. TockWorld attendees may center on HW RoT and TockEuro more on 'other applications', e.g. embassy, field-update, wifi, etc
 - Alexandru: Tock in industrial general-purpose use? [Ariel OS](https://github.com/ariel-os/ariel-os) project, an OS on embassy; integrate some of their async drivers? Break some boundaries and get Tock better integrated with broader embedded Rust
 - Pat: Just to clarify/understand, how does the proposed Tock event related to the 10-day school event; it would be 2-3 days at the beginning or end, or this is like the whole school event is branded TockEuro?
 - Alexandru: A co-location event, a 2-3 day TockEuro that's co-located with the school, within the summer school; students would join organically; and lots of synergies in already having space, cost reduction, etc
 - Alexandru: If this takes off, could consider a more regular, annual TockEuro
 - Alexandru: Ran into lots of folks at Rust week who mentioned they cannot go to the US
 - Amit: Action items from the rest of us?
 - Alexandru: Reach out to partners (e.g. Microsoft) who have offices in Europe, to see if they want to join?
 - Alexandru: Want to see if some of the US core team can come as well; can't support travel, but housing and food can be order $500USD for the 10 days

### Decisions

 - General support for TockEuro to move forward

### Actions

 - Core Team to identify ability to join
    - Especially if there are any timing constraints
 - Alexandru: Share rough agenda for 2-day event part
 - Prepare reach-out to partners who have EU presence
