# Tock Meeting Notes 2025-09-10

## Attendees
 - Brad Campbell
 - Johnathan Van Why
 - Pat Pannuto
 - Amit Levy
 - Alexandru Radovici
 - Leon Schuermann
 - Hudson Ayers

# Agenda
 - Updates
 - LLMs and Tock Contributions
 - Tock registers & External Dependencies Policy
 - SingleThreadValue Blog Post

## Updates
 - Amit: What's our view on updating Rust for libtock-rs, in particular to grab naked function support, would make some things easier
 - Johnathan: Should document, but policy for libtock-rs MSRV is update as-needed when useful, and fix anything you break along the way
 - Johnathan: For naked functions, my stance is use them where they are ergnomic; if global or standalone asm is better for some reason, that's fine too
 - Amit: Great; any other updates?
 - Johnathan: Trying to review https://github.com/tock/libtock-rs/pull/582 by end of week. Making benchmarks to inform design at https://github.com/tock/design-explorations/pull/5

## LLMs and Tock Contributions

### Context

People are using LLMs to write/refactor/debug/etc code. There is one now
experimental such suggested contribution and one non-experimental one,
as well a draft PR to add an `AGENTS.md` file to better support
LLM-based tools:

- LLM-based refactor: https://urldefense.com/v3/__https://github.com/tock/tock/pull/4583__;!!Mih3wA!DFKMcnwAF-kWa6NJtt1bZHQm2a9enefAaQcbCSjj4mx2ATZ1_-0Yswmx-2rNQIcLxjQvRDYkFERSSqZCIQ$
- LLM-spotted alarm bug: https://urldefense.com/v3/__https://github.com/tock/tock/pull/4590__;!!Mih3wA!DFKMcnwAF-kWa6NJtt1bZHQm2a9enefAaQcbCSjj4mx2ATZ1_-0Yswmx-2rNQIcLxjQvRDYkFER4ASvjKA$
- `AGENTS.md`: https://urldefense.com/v3/__https://github.com/tock/tock/pull/4582__;!!Mih3wA!DFKMcnwAF-kWa6NJtt1bZHQm2a9enefAaQcbCSjj4mx2ATZ1_-0Yswmx-2rNQIcLxjQvRDYkFEST68sL3w$

Given the inherent potential and risks of these tools, I think we should
decide how we treat their use in the Tock project.

- What can we gain from embracing these tools, or making them easier to
  work with for Tock?

- What risks do using these tools pose, esp. around introducing bugs
  in critical subsystems, potential copyright concerns, or applying Tock
  in safety critical fields (i.e., do they interact with
  certification?).

- Should we be encouraging, discouraging, or restricting the tools
  used to make upstream contributions?

### Call Notes

 - Amit: Summary, since RustConf, we've seen two PRs using AI tools,[
 - Amit: Experience that Leon had was, (a) cool that it fixed a bug, but (b) concerning that it did it following a semantic change that the author probably wasn't expecting
 - Amit: What are the risks here and how do they relate to our code review policies, practices?
 - Amit: Do we want to encourage these tools, block them?
 - Amit: This also, of course, relates to the `AGENTS.md` PR
 - Amit: Open floor for thoughts generally?
 - Alexandru: We had about six summer internship students; they wrote PRs towards a fork. It felt like at least half were probably LLM-written. I don't know which tools they used, but I felt like I had to invest significantly more time in review, and it was also hard to communicate what needed to update or change
 - Alexandru: My fear is that we will be flooded with PRs that are hard to review; though, if the LLM is very good, we might not even know
 - Johnathan: We don't current have any stated policy, and my instinct is that the people who would have flooded us
 - Johnathan: I'll second the potential copyright concerns, but I doubt we will get a good answer at this stage
 - Alexandru: Agreed with copyright; though again hard to even know
 - Alexandru: Ultimately, it'll be hard to even 
 - Hudson: Even in the pre-LLM era, if people contributed code from somewhere else that violated copyright, we would not have known that; it's not necessarily reasonable/feasible for us to gatekeep this; it's not like there's a monolithic "copyrighted code" thing we can query
 - Amit: I agree; and the risk of Tock itself being held legally liable is pretty low
 - Amit: My concern is more around the optics, and how it might impact whether people use or avoid Tock as a project (because of its stance on AI)
 - Amit: E.g., if in 1-2y, there's a backlash because of, say, copyright issues, one might imagine that projects which have a strong stance against AI become more attractive. Similar to how people view AGPL code as radioactive and avoid.
 - Alexandru: I think people will use them regardless of whether we allow it or not; we can't tell
 - Alexandru: Maybe we need something more like a contributer attribution / checkbox, where authors positively assert that contributers own their work, and if they use an LLM that they verified copyright
 - Amit: Right, it's not that we can actually enforce, but it's more about the public stance
 - Johnathan: We don't encourage, but we allow them; as a reminder you are responsible for validating copyright; if LLM-generated PRs is challenging to review, we will close more aggressively
 - Leon: They two PRs that are up right now have shown that for use cases where these tools are nominally best now, e.g. refactors, these tools are dangerous; the only reason that I noticed the code change is because of the diff to the existing file. If I compare this to Hudson's PR, which includes the file move, it's harder to see this. Especially because of how LLMs operate, they don't understand "rename", and it would be easy to introduce changes non-maliciously
 - Alexandru: What you're saying isn't really related to LLMs so much as review policy
 - Leon: I agree, this is independent of LLM policy.
 - Leon: We need more mechanical support to verify that what PRs do match what authors claim
 - Alexandru: So use an LLM in review pipeline :)
 - Amit: This does seem quite related to some of the tooling we had proposed to build with the Safe OSE proposal
 - Amit: It doesn't necessarily have to be LLM, e.g., tooling can validate that it's really all just style changes, etc
 - Leon: There is a clear difference between LLMs authoring code and reviewing code
 - Alexandru: We're actually submitting a grant proposal for this: Use LLM to answer "Does PR text match code"
 - Amit: For now then, do we want to
    - encourage the use of these kinds of tools (e.g., by merging `AGENTS.md` PR)
    - discourage by adding policy/text that tries to restrict it
    - stay kind of neutral and say nothing
 - Johnathan: If we add `AGENTS.md` we need policy too, because having one implicitly encourages
 - Brad: I fall in the "I dont know" camp; I want to encourage... it seems like we've crossed a threshold we can't ignore; developers will be increasingly trained on how to use them
 - Brad: To take a stance that "we want developers to stay in the past" feels a path to fading out a la Tock 2.0
 - Brad: To the point of our policy, is there prior art elsewhere in the open source ecosystem?
 - Brad: It seems like something we should do, but don't necessarily need to do now---need to go learn more
 - Alexandru: Need to separate into two concerns, we need to address the copyright promptly (i.e., checkbox that says "you own the code, if you used LLMs, you verified, etc"); then look at LLM policy more broadly
 - Alexandru: And explicitly assert that "if I used an LLM, I checked"
 - Johnathan: There's a name for this kind of thing, a CLA
 - Amit: But we don't want copyright attribution
 - Pat: CLA doesn't say anything about copyright; it's a general tool for the agreement for a contributor to a project; copyright just happens to be a common term
 - Pat: And GitHub has good tooling to help enforce CLAs
 - Amit/Leon: Yes, but CLA has a bad reputation b/c of copyright
 - Amit: Leaves me reticent to add any formal CLA machinery now
 - Amit: So policy conclusion is "we don't know yet", and I propose we add the checkpoint and establish a task force for policy
 - (consensus and membership for task force)
 - Pat: I think there is real danger to the checkbox, i.e., how are we doing this mechanistically?
 - Amit: Like we review the other ones, make format, doc updates...
 - Pat: But we aren't terribly dilligent about enforcing those; ultimately other things in CI protect us if e.g. the format checkbox is unchecked
 - Pat: For copyright, there is a substantial difference to merging a PR which, even accidentally, leaves that box unchecked---that is Tock directly endorsing a contribution where the author did not assert they own the copyright; the checkbox feels dangerous.
 - Amit: I retract suggestion to do checkbox now, we can discuss in future.

### Decisions

- Encourage, discourage, restrict, or don't know yet?
   - Don't know yet.
   - Establish task force to investigate: Alexandru, Hudson

- Decide on guidelines for use of LLMs for writing or reviewing Tock
  code or documentation.
   - Deferred to task force.

### Actions

Either:

- Write down use guidelines

- Establish a TF to explore and propose an answer to these questions



## Extracting Tock Registers

### Context

Long-ago (Sep 2023, according to the half-finished branch on my machine),
we agreed to extract tock-registers from the main tock repo. That project
never got finished, but more interesting use cases of tock-registers (see
tyler's TockWorld8 talk) have motivated making it work more like a "regular
Rust crate" — in particular, the use of a hard-coded path confuses Cargo's
ability to resolve "same or different".

The result is a suite of PRs:
 - https://urldefense.com/v3/__https://github.com/tock/tock/pull/4587__;!!Mih3wA!DFKMcnwAF-kWa6NJtt1bZHQm2a9enefAaQcbCSjj4mx2ATZ1_-0Yswmx-2rNQIcLxjQvRDYkFERa_f6jnw$
 - https://urldefense.com/v3/__https://github.com/tock/tock/pull/4588__;!!Mih3wA!DFKMcnwAF-kWa6NJtt1bZHQm2a9enefAaQcbCSjj4mx2ATZ1_-0Yswmx-2rNQIcLxjQvRDYkFERBfta16w$
 - https://urldefense.com/v3/__https://github.com/tock/tock/pull/4589__;!!Mih3wA!DFKMcnwAF-kWa6NJtt1bZHQm2a9enefAaQcbCSjj4mx2ATZ1_-0Yswmx-2rNQIcLxjQvRDYkFET1JQ-3bA$

We should start discussion with the last one (4589), as that discusses the
policy for "Tock-sponsored external dependencies" and how we would like to
manage them.

### Decisions

1. Determine how we want to express "Tock-sponsored external" dependencies.

### Call Notes

- Pat: Driven by Tyler's safe MMIO work. There is a sitation where the
  safe-mmio crate depends on tock-registers. But when trying to use
  safe-mmio inside of Tock, Rust treats the two paths to the
  `tock-registers` crate (one using a `crates.io` rev in the safe-mmio
  crate, the other using a workspace-internal path) as different
  dependencies.

  So, next to splitting out `tock-registers` into its separate repo,
  there's a question of "what is the right way to eventually refer to
  the external `tock-registers` crate from within Tock?"

  Effectively: `crates.io` PR vs. git revision pinned
  dependencies. Everyone should take a quick look at the PR:
  https://github.com/tock/tock/pull/4589

- Brad: do we have the meeting minutes from the last time we discussed
  this? Why didn't we follow-through with splitting out Tock registers?

- Pat: last time we planned on splitting it out, but it just feel off
  the table because of other things.

- Amit: why don't we override the safe-mmio internal tock-registers
  dependency, when using it with the upstream Tock codebase? Would
  solve the issue.

- Leon: Yes, that would work.

- Alexandru: would prefer not to have external dependencies.

- Brad: two issues: resolving Tyler's dependency conflict, and
  splitting code out. If we can get the former working, that would
  remove pressure from the more general question of splitting
  tock-registers out.

- Leon: tangential -- we also have other external dependencies,
  referenced from `crates.io` that we're not locking today. That is an
  issue, which this would solve.

- Amit: we shouldn't have `Cargo.lock` files for binary dependencies,
  right?

- Leon: lockfiles are per-Workspace. We have at least one non-library
  crate in our workspace (e.g., board crates), so this is why Cargo
  generates a lockfile. If a library dependency is used from a
  workspace that has a lockfile, that lockfile is ignored.

- Amit: wouldn't solve the issue for out of tree boards.

- Leon: yes, those should have their own lockfile.

- Amit: favor Brad's position -- we resolve the short term issue. The
  question before us is not whether to split out the crate, but how
  to. So maybe we should resolve that question first.

- Leon: don't want this to linger again for 2 years. We'll have to
  page in a bunch of context yet again.

- Brad: We did generally agree to split out tock-registers:
  https://github.com/tock/tock/blob/5f606cc9352797b50262b4970ba169d8336aa6c4/doc/wg/core/notes/core-notes-2023-09-29.md?plain=1#L36

- Pat: Agree it shouldn't linger, we should resolve PR 4589 in the
  next week or two.

### Actions

- Core team to review and comment on 4589, attempt to come to consensus online,
  or we can revisit on future call.




## `SingleThreadValue` Blog Post

### Context

We [just merged](https://github.com/tock/tock/pull/4551)
`SingleThreadValue`. This is a hard question the Rust ecosystem is
struggling with, and having a write-up that is both (1) accessible to a
more general audience, and (2) accurate to the important details is
valuable.

I think it's worth spending 5-10m of full core team time for folks to
review and refine the post.

https://github.com/tock/tock-www/pull/122

### Call Notes

 - Amit: Pretty much out of time here, let's end the call a few minutes early, and everyone read and review this async.

### Decisions

N/A

### Actions

- Revise the blog post

- Merge (and post)

