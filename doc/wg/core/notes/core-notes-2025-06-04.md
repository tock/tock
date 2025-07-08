# Tock Meeting Notes 2025-06-06

## Attendees
 - Pat Pannuto
 - Johnathan Van Why
 - Leon Schuermann
 - Brad Campbell
 - Amit Levy
 - Branden Ghena
 - Alexandru Radovici
 - Vishwajith


# Updates
 - Alexandru: Tock Euro is on!
 - Draft invitation here: https://docs.google.com/document/d/1vxHUgIKYkQyxTPI7pNu_S8eE8U84GbisYMa9uxpigk4/edit?tab=t.0 , please review, etc
 - Action item: Amit to help Alexandru set up euro.tockworld or similar


# `ProcessArray` with interior mutability

## Context

PR: https://github.com/tock/tock/pull/4447

Uses the Tock-standard method for creating a shared mutable static array to
hold process references. This allows us to remove a couple `addr_of!` per
board. There are then a few improvements that can be made to kernel.rs with
this new type.

This is a variant of https://github.com/tock/tock/pull/4373 that makes the
process array type more explicit, does not include the optimizations for
terminated processes, and does not change how entries in the array are
reserved before creating a ProcessStandard. The intent is not to explicitly
*not* do those things, but just to make the change smaller and hopefully
easier to reason about.

## Notes
 - Brad: (summary of context)
 - Brad: Didn't do the mass-update for all the boards yet; would like consensus on design before grunt work
 - Amit: This is a big step towards removing the potential for multiple mutable aliases
 - Amit: I believe it's the case that if the board doesn't want to print process state in panic handler, it's now possible to avoid all mutabale aliasing, which was not possible before
 - Amit: So, does adding ProcessArray seem like the right way forward?
 - Amit: Is this a meaningful improvement on its own without the rest of the other PR?
 - Branden: Skimmed PR, looks reasonable; changes are ultimately minimal, it's a big-looking change that doesn't really change much
 - Leon: This is mostly anecdotal, but in the multi-core Tock effort, we came up with a pretty similar design, just not going quite as far; but evidence that this is likely a step in the right direction
 - Johnathan: Small code nit, but the high-level looks good
 - Pat: Looks good to me
 - Brad: One change I didn't bubble over was a shift to how slots are allocated when a process is being created.
 - Brad: Right now, we find where in the process array we can store a new process reference, then we go create the process standard object
 - Brad: In the first version, that's flipped; the process is created, and then it hands it to the kernel "store this for me", which finds an open slot and puts it there
 - Brad: I wasn't sure if that was intentional or incidental; is that important?
 - Amit: Details are fuzzy, but it was not incidental it was intentional; came from kernel owns process slice, why would loaders even know internal structure? I think there was also something that was made easier downstream
 - Amit: I think this can be an orthogonal question, we can switch to process array, and then change how loaders interact with the process array separately
 - Brad: Another nice thing, with this model, don't ask for an index number, but rather a reference to a process slot, which is probably closer to the other abstraction too

## Decisions

1. Does adding ProcessArray seem like the right way forward? YES.
2. Is this a meaningful change without the other improvements from #4373? YES.

## Actions

- Sign off on the kernel changes for ProcessArray and then I can update all
of the boards. SIGNED OFF.




# x86 working group

## Context

Two independent groups are working on x86, at least one in production, with moderately different, though hopefully compatible goals. There is at least one PR that is likely to be merged that will break current downstream support (though likely not in a hard way to adjust to and fix, so this one is probably fine).

- https://github.com/tock/tock/pull/4452#issuecomment-2940559059

Vote to establish an x86 working group that will be responsible for the x86-specific crates in the kernel and "modules" in libtock-rs and libtock-c, meaning they will have approval/rejection/merge power for PRs related specifically to those. Proposed membership

- Amit (lead, since I'm somewhat neutral but involved)
- Alexandru (or someone else more in the weeds at OxidOS)
- Microsoft Pluton representative

## Notes
 - Amit: There are at least two big groups working on x86: Microsoft and OxidOS
 - Amit: Microsoft's port is shipped in production, and working on upstream, but limited bandwidth; OxidOS has been able to upstream some changes a bit faster
 - Amit: These are not purely mechanical things, e.g. the recent syscall ABI PR
 - Amit: While the changes don't seem controversial, they will need consensus
 - Amit: This WG can be a forum that ensures that at least one person from each of the stakeholder groups has eyes on things
 - Amit: Q1: Does this make sense?
 - Alexandru: Think this is a good idea.
 - Alexandru: The OxidOS interest isn't as much x86 as it is seamless operation across MPU/MMU
 - Alexandru: We have an ARM64 workig with MMU on actual hardware showing a UI at 60fps; that's not quite in a postition to upstream for several reasons
 - Alexandru: But in order to do MMU/MPU integration seamlessly, we're piggybacking on the x86, since that arch is upstream and the MMU/MPU bits are basically the same
 - Alexandru: Our goal is to keep our downstream ARM64 in sync as much as possible while it has to stay downstream
 - Alexandru: The other principle motivation is to have x86 available for teaching
 - Leon: A WG with one stakeholder doesn't make sense; sounds like we have commitment from one, but need to make sure that someone from Pluton group can commit
 - Amit: Reached out to Pluton team this morning
 - Brad: It's good to have working groups interested in particular subsystems, but how much is there to be done? On some level, for architecture support it's kind of cut and dry
 - Amit: Yeah, x86 really does come in flavors; we see some of that in RISC-V, though that's slowed
 - Amit: I don't think this is the kind of WG that would meet regularly, but rather it's a group for things that are focused on x86, can get signoff on critical pieces
 - Amit: And it's a forum to get folks together when there is something substantial that needs to be decided, or a larger effort to update kernel/userspace together, etc
 - Leon: It sounds a bit like we're conflating two definitions here... we have a semi-rigorous definition of what a working group is and does, but this is closer to 'code owners' maybe?
 - Amit: We do have a WG doc: https://github.com/tock/tock/tree/master/doc/wg
 - Amit: I think both roles are in scope
 - Leon: Agreed; there is the converse question, should we have an ARM and a RISC-V working group? But we can punt on that
 - Amit: Yeah, e.g., there is a long-lingering change in libtock-rs trying to const-ify ARM syscalls that also ends up affecting RISC-V; and there's some specific RISC-V assembly with `nomem` that Clippy was unhappy about that was beyond what I could follow for details—would be nice to have a set of stakeholders we can ping
 - Amit: All to say, it would be good to have for other architectures as well, and they can be lightweight
 - Leon: This was relevant in the (e)PMP changes, which dragged out for a long time; and having a group responsible for signoff
 - Amit: Consensus to create WG?
 - {yes}
 - Amit: Membership: I nominated myself to lead as I think it's important to have someone a bit neutral / central from the project. Alexandru could be core team rep, but probably better to have someone not mixed with other affiliations.
 - Amit: Also suggesting one member from OxidOS and one from Pluton
 - {verdict}: Amit will lead, and will take responsibility for recruiting additional members.

## Decisions

1. Establish a working group?
2. Nominate initial members

## Actions

- Invite initial members
- PR to establish working group
