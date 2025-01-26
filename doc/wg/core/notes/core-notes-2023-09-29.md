# Tock Core Notes 2023-09-29

Attendees:
------------

- Tyler Potyondy
- Johnathan Van Why
- Hudson Ayers
- Alexandru Radovici 
- Brad Campbell
- Leon Schuermann
- Pat Pannuto

Updates:
-------------
- Brad: I have been working with the nrf52840 board I bought with a display.
- Brad: I am mainly working on the screen HIL and understanding the state of that. 

Tock Registers PR
-----------------
- Hudson: We should discuss the release/PR for Tock registers (https://github.com/tock/tock/pull/3693).
- Pat: I can make the case we can just merge this.
- Hudson: I agree. 
- Pat: Tock registers have been used by downstream users off and on.
- Pat: As Tock registers gains more traction, we have an  obligation to downstream users to make it more stable. Most of the recent updates have been rather trivial. There have only been four commits in the past year, two of which are rust updates and a clippy update. The only changes to code are the matches all API changes Hudson made 10 months ago. I know we have a long standing mission to update some safety issues, but I propose we roll those changes up in 2.0. 
- Johnathan Van Why: I agree. What I am working on would be 2.0.
- Hudson: The fact that there might be a major transition seems to be a time to move to 2.0 to communicate breaking changes. We may annoy people if they didn't realize there are likely to be a breaking change.
- Pat: This doesn't seem contentious.
- Pat: More contentious, I propose we pull Tock registers into a separate repository and have it be
a dependency of Tock.
- Leon: Back to Pat's original point, I agree with everything said; it concerns me that we will be releasing a stable version that may have some soundness issues. We should communicate this for internal tracking and external users.
- Pat: Do we have that written cleanly anywhere?
- Leon: I don't think so, we should. 
- Johnathan: I can write that up.
- Pat: Reasonable to document known issues.
- Pat: How do people feel about me creating a Tock registers repository under the Tock organization?
- Brad: We need to be careful we are not breaking backwards compatibility.
- Brad: Downside, we technically cannot do this and be in compliance with the external library policy.
- Leon: Because the Kernel crate pulls it in?
- Brad: I don't know if the kernel does, but chips does.
- Pat: Does it count as an external dependency if it is under the Tock umbrella?
- Hudson: I think we can easily explain/justify anything under Tock umbrella is not an external dependency.
- Hudson: This may be confusing though if developers believe Tock does not use external dependencies, but then see cargo.io downloading.
- Brad: If this is an internal dependency and is controlled by Tock, why bother with adding a whole new repository and make seem to be new project when it is not?
- Leon: One argument is that having these workspace projects and cargo is unwieldy. How cargo searches within workspace project is unclear. It would be easier and more clear for this dependency to be a separate repository.
- Hudson: We could in theory have Tock registers as its own workspace still in a top level Tock repository the same way we have some of the tools, right?
- Leon: You cannot specify subpaths in cargo. The semantics are super weird.
- Brad: I make a proposal that we could move it to an external repository, only include in kernel crate and therefore be very explicit about how we manage this.
- Hudson: It should never be the case that kernel crate vs chip crate pulls in different versions.
- Hudson: As a prerequisite, we need to only pull into one kernel crate; this shouldn't be too difficult.
- Brad: I am in support of this. 
- Pat: Action items (1) Release 0.9 on cargo after merge; (2) Create Tock registers repository; (3) Create PR in Tock removing references to registers and update documentation to explain.
- Hudson: Another downside to this is that  being a separate repository means fewer people will review/see changes. 
- Pat: I think that this may be good in the sense only people interested in Tock registers will see changes. 
- Brad: Why not include the history?
- Leon: I think this is helpful.
- Pat: Okay, I will look into this.
- Brad: Having thorough CI testing across different repositories is becoming more confusing.This is something we should think about.
- Hudson: This is an interesting point, we may merge something to Tock registers and then later find out that it is unusable from Tock's perspective.
- Leon: We could integrate a CI workflow (similar to userspace libraries). 
- Leon: We should certainly codify this and test prior to releases.
- Leon: One nice takeaway, we are no longer in a state with Tock using an unreleased version of Tock registers. 
- Pat: We should be better about pushing out these changes and improvements. Hudson's fix in Tock registers has sat for around a year.
- Hudson: This all sounds good to me.

