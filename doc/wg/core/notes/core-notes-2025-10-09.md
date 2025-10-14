# Tock Meeting Notes 2025-10-09

## Attendees
 - Branden Ghena
 - Pat Pannuto
 - Brad Campbell
 - Hudson Ayers
 - Amit Levy
 - Leon Schuermann
 - Tyler Potyondy

# Agenda
 - Updates
 - Code Goals Document

## Updates
### Flux
 - Pat: Flux PR has substance and is worth looking at. It does rely on the cargo lock updates first
 - Pat: Right now it's just optional testing stuff. But there are some thoughts that requiring it as part of a build could reduce code size, for example it reduced ring buffer code size by 25%!! That's a future discussion at some point for whether we want it. It would be an external dependency
 - Brad: It would be nice of there was some sort of output from Flux. When you run tests, you know they run and they spit out an answer. Right now it's a bunch of warnings and compiles and just doesn't say anything when it works. If we could push them to have output that would be nice
 - Pat: Like if it said "18 out of 18 assertions proven"
 - Brad: Yeah, exactly
 - Tyler: Or a negative test where it fails as expected
 - Brad: For sure. It would also be helpful for people to see how it would work if you violated it
### Tockloader
 - Brad: I pushed a new minor version of Tockloader. Probe-rs support and stuff to make QEMU boards more reliable
 - Brad: It was pretty easy to add probe-rs support to Tockloader. We could build more on top of it if people actually use it
### LLM PRs
 - Hudson: Alex and I are meeting to discuss AI code updates. We'll plan to make some PRs with documentation and recommendations
### Crypto WG
 - Tyler: Still planning times for meetings.

## Code Goals Document
 - https://github.com/tock/tock/pull/4599
 - Brad: Trying to put down in words what we've already been doing. A resource for people to understand how we manage the project and why. Set expectations. Something we could point people at for our rationale for why we ask for things.
 - Brad: It's not too long. Fairly general. I don't want specific rules/regulations as a challenge, but themes
 - Brad: I wrote this a while ago. Seems fine to me. I'm looking for specific wording changes and to move forward on it
 - Amit: This seems good to me. It doesn't have specificity, but is still something that's valuable. Goals are useful and pretty unimpeachable
 - Amit: I'd be happy linking to this from a contribution guideline or the PR template or something
 - Branden: So does this go in the Tock repo or the Book repo? How do you know?
 - Brad: Repo docs are about the repo and how we manage it. The book is about the OS and using it.
 - Amit: You could imagine having a near-identical version of this in the book. But a more specific and detailed version in this repo which could diverge. In the repo it could have guidelines for when different levels of scrutiny would apply for example. More specific language of how PRs should be constructed, and when it's appropriate to block them vs merge them. Current language could probably fit in both.
 - Branden: Let's merge it then

## Neglected PRs
 - https://github.com/tock/tock/pull/4615
 - Brad: Version number update steps
 - https://github.com/tock/tock/pull/4605
 - Brad: Cargo lock stuff is pretty ready to go
 - https://github.com/tock/tock/pull/4583
 - Hudson: I did want to hear back from Microsoft first. Let's wait a week. If we haven't heard from them by then we can just merge

