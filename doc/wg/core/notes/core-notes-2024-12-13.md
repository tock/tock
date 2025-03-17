# Tock Meeting Notes 2024-12-13

## Attendees

- Hudson Ayers
- Amit Levy
- Leon Schuermann
- Alexandru
- Branden Ghena
- Jonathan Van Why
- Benjamin Trevor


## Updates
### Viewing Treadmill CI Runs
* Leon: We are still waiting on the arm-v6m PR that fixes the control register to go in before we can do a release. Alex, there are a couple simple suggested fixes we want applied to fix a broken URL and then it could go in
* Alex: Sorry, missed those, will look now
* Leon: Once that is in we can tag a release candidate, then we will open up a couple week window for people to do as much testing as they like. We set the release bar at all of the automated testing working, so I think if we create the candidate today we can probably do a release next Friday
* Leon: As soon as a release is officially tagged, I will open up an issue for a new release
* Alex: I merged the suggested changes
* Leon: I will nominate anyone else to merge this PR
* Amit: I will take a closer look after the meeting


## PR 4266
* https://github.com/tock/tock/pull/4266
* Amit: This PR modifies the frequency of the ARM systick
* Leon: I proposed talking about this because we discussed this awhile back about making frequencies const generics for other clock traits, and we hit an issue where one chip was dynamically retrieving frequencies from the hardware, which does not mesh with the use of const generics for that. At the time, we decided that all our interfaces assume that frequencies do not change. This PR thus makes me skittish because it assumes that we can change frequencies. On one hand, this PR adds an unsafe method, so we could say it is on the user to use it correctly. But with out current HIL I do not think it is possible to use it correctly without changing everything that depends on it.
* Alex: Gabby is one of my employees from Oxide, we need this because we are using low power modes which changes the frequency of the systick. We need this in order to use low power mode.
* Amit: The PR says the systick is driven by an external clock which is changing
* Amit: I think this is pretty unrelated to the time HIL and stack, the systick is not used for that
* Leon: I see it only gets used as a SchedulerTimer — if this is not used for the Time HIL maybe I started the wrong discussion
* Amit: The systick as written is not usable for the time HIL because the time HIL takes a const generic argument.
* Alex: Yeah for other clocks we change the divider so the frequency is unchanged, we cannot do that for the systick
* Amit: Exactly, the systick is already incompatible with the Time HIL traits
* Leon: We have one chip in tree that uses a custom implementation of the Frequency trait to dynamically change the Frequency at runtime, so technically it could be used
* Leon: It sounds to me like we have an invariant in our time HIL, we assume that frequency does not change at runtime, but this is not truly encoded in the type system. The Time HIL advertises its frequency through an associated type, but it returns it through a runtime method. I think that we should make the time HIL fundamentally incompatible with changing frequencies, then we can go forward with this change
* Amit: We could use a const generic instead of an associated type, or maybe we can do const associated types now?
* Leon: Then we have to do something about that in-tree chip that currently changes things dynamically (the IMX1060).
* Amit: In any case, the systick is already a bad match for the time HIL because it does not interrupt when you are asleep
* Alex: It runs in low power but not deep sleep
* Leon: I now feel good that changing the frequency as this particular PR does sounds fine. But we should add docs that say that this implementation of systick is incompatible with the Time HIL.
* Amit: I disagree, it is compatible, but you have to make the same promises we currently make for the IMX chip.
* Leon: OK, maybe the time HIL documentation should be updated to better state its assumptions
* Amit: Agree, but that should not be in this PR
* Leon: Sure, that makes sense. We should just fix this documentation mismatch
* Alex: I will ask my colleagues to document this in the time HIL
* Amit: Or we could do it, either way
* Amit: Another thing: this PR uses the unsafe marker to warn people to tread lightly bc of system implications, but not to cover any case of Rust unsoundness. In other cases we would have used a capability for this.
* Alex: I vote capability, we have a power management capsule for this
* Amit: Could this capability also then apply to the constructors of the systick?
* Alex: Yes I was gonna suggest that
* Leon: I agree with set_frequency not using unsafe, but I think the SYstick struct can access MMIO memory when you run new()
* Amit: It does not, look at the implementation — it just sets fields
* Leon: I am looking now, it is confusing
* Leon: function hertz() calls into systick_base which ultimately references the static registers in MMIO memory
* Hudson: The unsafe for that is when declaring the static registers
* Leon: Well that means that you implicitly call that unsafe when importing the module
* Hudson: That is the case for all of our chip peripherals
* Leon: I guess that makes sense, we should change all our peripherals to require new() to consume the static registers item
* Leon: that is what we do in most other drivers anyways
* Alex: Should we be cfg’ing this away unless this is compiled for the ARM targets?
* Leon: Certainly yes
* Amit: At least systick base should be
* Alex: We could even add a compiler error there
* Branden: So winding back, should new() take a capability?
* Leon: I think it can
* Alex: Yeah, new() taking the static ref would allow us to do unit testing.
* Amit: I think this could be zero overhead by making hertz() an Either. Hertz and ExternalClock are mutually exclusive in practice. Right now we return a hertz value of 0 to indicate you should look at the TMS register. So this could be an enum. Oh wait — never mind, we need a systickbase regardless.
* Leon: So what capability should set\_hertz() take?
* Amit: we should create one — like the SysTick capability
* Amit: That could also be used for the constructor, these are all basically the same capability, you are asserting you know how to get the frequency.
* Leon: So we could just define it in that module
* Amit: Yes I think so
* Leon: The semantics of this are architecture specific so this makes sense
* Leon: Could this create issues for something as a result of adding generics
* Hudson: Capabilities do not have to be generic parameters
* Leon: Right now this is a method defined not as part of a scheduler timer trait, but the underlying Systick interface. If there are future plans to make this part of a more generic interface, we would need a general capability that could be exported through that generic interface.
* Leon: Alex, will you open source the code that is using this?
* Alex: We will see, I hope to
* Amit: It seems unlikely to me that this would be general…this does not even make sense for all systick instances, so I have a hard time imagining doing this in a generic way.
* Leon: I just want to make sure that upper layers built on this do not have issues
* Amit: Only people calling this interface need to name these types, 
* Leon: What about even people calling it indirectly?
* Amit: The only thing that is required is for the function to take an argument of that type. Only the thing that directly calls those functions needs to have that capability and hold it and even have knowledge of it.
* Leon: I see, there could be a wrapper structure held at the implementation level. That wraps up my concerns, I can share a summary in the PR discussion
* Alex: Thank you,  I will ask Gabby to fix things up on Monday
* Alex: In general, we will be sending some additional PRs without usage examples in the coming days. Sorry about that.
* Amit: It is fine if examples are abstract, but having something is useful
* Alex: I will ask them to document better
* Branden: You mean as a comment on the PR or in the description?
* Amit: Yes

## Meeting Time
* Leon: I brought this up because we chatted about it before the call, this is an inconvenient time for Europeans
* Alex: I realize I am the only one in Europe though! It is only 7pm in Romania
* Amit: Starting earlier than 9am on the west coast would be hard. Would a different day help?
* Alex: yes because 7pm on a Friday I am often in a car. Different weekday could be a bit better
* Amit: should we do a survey? Should we think about this with the term turning around?
* Branden: Hudson mentioned how we used to reschedule this every quarter, I think we should try to avoid that. But a one-time or rare change is fine
* Hudson: I propose we keep the time but send out a survey for day of the week
* Amit: Should I send that broadly or core team?
* Branden: Core team first. If not satisfiable, reach more broadly.

## Dynamic Trait Object PR
* Leon: https://github.com/tock/tock/pull/4260
* Alex: I realized I cannot use different servo motors
* Leon: Makes sense, that is simple. Are there concerns with relaxing the trait bound?
* Alex: I am planning to update the PR to make the changes discussed on it including moving more subsystems over. Please keep it open until I do so
* Amit: If everything compiles it seems we are safe to make this change
* Leon: This PR suggests the defaults for Sized vs ?Sized are not right for Tock. One reservation for this is if we encounter a place where Sized is needed we should change it back.
* Hudson: What if in the future we want to define a trait that is generic over all GPIOPins? Does this not prevent that?
* Alex: If that happens, we can make another type and wrap the original one, so would be a pretty trivial fix
* Hudson: That sounds fine to me, was not saying I want to block this.
* Leon: I can comment on this PR as well with a summary. We will wait for you to change more subsystems over
