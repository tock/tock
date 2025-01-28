# Tock Meeting Notes 2025-01-22

---

Agenda
- Updates
- WG Report ups (Network WG)
- TRD104: Explicit type summary for syscalls
- CapabilityPtr vs. MachinePtr

---

## Attendees
 - Branden Ghena
 - Amit Levy
 - Leon Schuermann
 - Pat Pannuto
 - Johnathan Van Why
 - Hudson Ayers
 - Lawrence Esswood
 - Brad Campbell
 - Kat Fox
 - Alexandru Radovici
 - Tyler Potyondy
 - Chris Frantz

## Updates

 - (full agenda; tight meeting; nothing substantial)
 - Branden: Scratch network WG update; bi-weekly meetings and this was off week.

## TRD104: Explicit type summary for syscalls ( https://github.com/tock/tock/pull/4228 )

### Agenda Notes

A proposal to add consistent and explicit type information to syscall definitions in TRD104.

#### Context

https://github.com/tock/tock/pull/4228

#### Decisions

1. Do we to include explicit type information for each system call in TRD104? (up or down)

2. If yes, does PR #4228 have exactly the right types? If not, what would needs to change? If unclear, what questions to be resolved?

#### Actions

- Close PR if we (1) is no.

- Merge PR if (2) is yes

### Meeting Notes

 - Pat: History...
 - Amit: Essentially what the PR does is add a type column for tables in TRD104
   which document system calls.
 - Amit: The types currently are the basic integers, etc. Could in the future
 - Johnathan: The motivation is pointers versus integers, which there's agreement about.
 - Johnathan: The bear it pokes, without agreement, is pointers versus `usize`, which gets messier on 64-bit, where I don't think we have agreement yet.
 - Amit: So, first question: Is this something that TRD104 should be explicit about?
 - Amit: On 32-bit something could have been `usize` or `u32` and that's all the same thing, in practice, once we include 64-bit it's not longer the same thing, so should TRD104 be explicit about which one is the right one for syscall arguments?
 - Leon: One argument was that TRD104 is only defined for 32-bit interfaces, so whether we can have a syscall interface that works on 64-bit platforms is a different system call encoding trait; even if we had a document or policy or platform, we'll always need a new 64-bit trait.
 - Amit: There are a few places where the proposed modifications are not just `u32` versus `usize`, for example, search for `*mut`; we pass an array with address and size
 - Johnathan: Right, so the 64-bit issue was discussed and the resolution is that TRD104 restricts itself to only 32-bit platforms; that makes sense.
 - Amit: Back to the first question: do we want TRD104 to be explicit about types?
 - Johnathan/Brad: I think so
 - Leon: The particular motivating example to me is something like the Yield value
 - Amit: We're also restricting to `u32`/`usize` instead of `i32`/`isize`, or larger than `u8`, or etc. There is also a statement about what kind of value.
 - Lawrence: I do wonder, given that we only imagine this to be used on 32-bit platforms, if it would be more convenient to express as `usize` instead of `u32`. Does putting `u32` in places cause unnecessary churn in practice, especially given that things are defined cleanly with the 32-bit limitation?
 - Johnathan: Well, things are already implemented as `u32`...
 - Lawrence: Yes, but if we are writing a document for intent
 - Johnathan: Though Pat has a point that at the ABI-level there is no distinction between the two
 - Lawrence: I agree, no difference at an ABI level, but there is at an API level
 - Amit: **Conclusion 1:** we do want to document strong types.
 - Leon: What would we do if there were a slot for a value that could be a length or an integer, based on the specific command
 - Leon: I think it having an informational note would help alleviate the concern
 - Amit: Let's take `Command` as an example, we have all `u32`'s proposed for their arguments in the PR now
 - Amit: What we're saying is that the space of possible driver numbers is 32 bits (there are no more than 2^32 drivers), but we have fragmentation for spacing, etc; and further that each driver has no more than 2^32 subcommands. That seems reasonable regardless of the size of the register on the platform
 - Amit: Arguments 2 and 3, however, are really specific to what the specific command means. The language used in Yield was "opaque, register-sized value".
 - Lawrence: Is opaque register right, given that there are machines with multiple sizes/types/etc of register?
 - Pat: The document is really referring to registers designed for syscalls, which restricts that in practice
 - Leon: I agree with Amit generally about the expression of opaque types
 - Brad: I agree with the 2/3 cases, but less for 0/1, as it's not about `u32` etc, it's about having at least 32 bits
 - Lawrence: There is a potential pain point here in 64-bit platform compatibility
 - Pat: Following up on a comment from earlier, is the solution possibly to just have two columns in each table, an ABI and an API column? One which expresses what the syscall ABI must be able to handle (pass 32 bits of information) and one which adds semantic meaning (e.g. for command 0/1, TRD104 expresses them as `u32` here, but for command 2/3, it is opaque bits, deferring semantic/API meaning to specific commands)?
 - Brad: re paint points: We can certainly support 64-bit lengths on things that want to support it, might just have more overhead on 32-bit platforms
 - Lawrence: Re ABI/API, this can 'lock out' options in 32/64 compatibility; certainly in TRD105 we'll want to have `usize` to talk about length in places
 - Johnathan: Even on a 64-bit platform, anyone who wants a length of 64 bits is probably going to be unhappy with Tock as-is, since there's a lot of assumptions.
 - Lawrence: I do have this problem in part now; 32-bit platform with access to larger/more memory; largely resolved with `usize` in practice, but a few pain points / places where there's `u32`
 - Amit: Resurfacing brad's point, Want to avoid having capsules that only work on certain platforms but can't work on others. Is that for TRD104 to enforce, or should be up to individual drivers?
 - Amit: It seems like there might be cases where there are highly-platform specific drivers, that will only ever work on one platform, so why restrict that?
 - Leon: In contrast, if we wanted to restrict this at the ABI level; if we in the future defined 64-bit ABI which has multiple flavors of command, one of which is `u32/u64/usize`, but they can all collapse onto the same ABI implementation since they all fit
 - Brad: I propose that we don't have consensus. Can we break this into separate commits, with the non-controversial and the controversial ones? Move discussion online so we can get to the rest of the agenda?
 - Lawrence: re Leon, somewhat wary of additional low-level commands to minimize churn
 - Amit: Second Brad's proposal re question 2, agreement on the majority of the types, but a few specific points of discussion are needed, and we should try to separate those out.
 - Pat: Sounds like a plan; action item on me to execute.


## CapabilityPtr vs. MachinePtr ( https://github.com/tock/tock/pull/4250#issuecomment-2606035602 )

### Agenda Notes

#### Decisions

1. Does it matter how they are implemented?

2. If yes to (1), which of the options should we go with?

#### Actions

- If no to (1) just leave it up to Brad/Jonathan to make a choice for the moment
- Otherwise implement the decision in (2) and merge #4250

### Meeting Notes

 - Johnathan: CapabilityPtr is already upstreamed, it happens to be able to represent an arbitrary machine register, which isn't its primary purpose, but we used it as such
 - Johnathan: This PR adds an explicit machine register type, for things where we don't have any additional context beyond this fits in a machine register, to split out the use cases of type-erased value versus typed
 - Johnathan: The implementation has explored a few choices, first was aliased type, but that exposes CapabilityPtr's API to MachineRegister which is undesired
 - Johnathan: Then we looked at whether CapabilityPtr or MachineRegister should be a base type the other derives from
 - Johnathan: Started digging into implementation due to provenance, and it's messier than we want, as one way or the other internal details end up leaking in practice
 - Johnathan: My current proposal/preferred answer is "Option F" in the PR, which keeps CapabilityPtr as the foundation but notes it has this little MachineRegister capacity on the side
 - Lawrence: the advantage of F over the alias is that you don't expose all of CapabilityPtr's API to MachineRegister use cases?
 - Johnathan: Yes, MachineRegister API should be narrower, really just conversions to other types
 - Leon: Slightly confused; don't disagree with building one atop the other; from a user perspective, if they're implemented in the same crate (and can use shared internals) that's orthogonal do the interface exposed to users outside the crate
 - Johnathan: I think that's an argument in favor of what I'm saying today—conceptually, MachinePtr can contain CapabilityPtr or something else, but in practice it always holds a CapabilityPtr (users just don't see that)
 - Amit: Leon's comment meshes with proposal
 - Amit: It doesn't matter how they're implemented, what matters is the API
 - Leon: Where we (rel to Lawrence) diverge is that MachineRegister does not just wrap a `usize`, it wraps the largest thing a register can contain
 - Lawrence: No, we agree, was just talking about platforms without capability hardware
 - Amit: Great, so return to "doesn't matter how they're implemented", just API is sensible; Proposal F
 - Pat: Can't take notes and digest this; will review after meeting; expect that I'm in agreement
 - Johnathan: Re Brad's concerns, I will attempt to split out provenance into separate commits
 - Amit: Sounds like consensus!
