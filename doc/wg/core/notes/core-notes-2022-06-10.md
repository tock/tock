# Tock Core Notes 2022-06-10

Attendees:
- Hudson Ayers
- Alyssa Haroldsen
- Amit Levy
- Alexandru Radovici
- Vadim Sukhomlinov
- Johnathan Van Why

# Updates

- Leon: Been in contact with a researcher of Tampere University in Finland. [The
  group](https://sochub.fi/) is building a custom ASIC with various subsystems
  on there (e.g. DSP, AI accelerator, Ethernet) and have a few RISC-V cores in
  there. They would like to use Tock as an orchestration and general purpose OS
  on there. Use Rust excessively, so Tock is really attractive. Their CPU is the
  Ibex in the rv32e variant (16 registers).

  They have an initial tapeout and perform the hardware bringup now. Tock's
  requirements may influence the second hardware revision, for instance to
  include an MPU with the processor, etc. I'm chatting with them regarding
  Tock's requirements and how we can help with the bringup.

- Alexandru: Started unifying the text screen and graphics screen into a single
  HIL in response to discussions with dcz. Still a draft, feedback is welcome!
  Used SPI TRD as an inspiration.

- Leon: Have the Jazda devboard, can test potentially.

# VolatileCell Licensing Issue

- Johnathan: [posts into chat]

  ```
  Commit that added VolatileCell to Tock:
  https://github.com/tock/tock/commit/852b757ba5e3617c39204bcca0db26ae617e9186

  Source that VolatileCell was copied from:
  https://github.com/hackndev/zinc/blob/master/volatile_cell/lib.rs

  Alternative VolatileCell that Amit shared with me:
  https://github.com/japaric/vcell
  ```

  The source code of `volatile_cell.rs` includes a link to the original source,
  which is Apache-2.0, but not also MIT-licensed. We copied it without the
  license header.

  Amit found another `VolatileCell`, which is not stemming from the same
  codebase, dual-licensed under Apache-2.0 and MIT. We could swap it out to be
  in the clear.

- Leon: Our `VolatileCell` does not have many LOC, the implementation is
  trivial. Doubt that licensing would be a proper legal issue. Arguably there is
  no other way to implement such a construct at all.

- Alyssa: There are other ways to implement volatile accesses generally, but for
  the specific `VolatileCell` there is no other way the basic `set`/`get`
  functionality could be implemented. We at most copied these methods, the
  documentation etc. are not copied.

- Amit: The old repo is archived, a reasonable approach might be to reach out to
  the original author, explain the situation and get a written exception that
  this is acceptable use.

- Johnathan: Seems to be more than one author. Would need to get everyone's
  approval. Some of them touch irrelevant parts / whitespace, so need to count
  exactly how many to contact.

- Amit: Need to look at the code from when we've copied it.

- Johnathan: Copied after the last modification to the original.

  There seems to be only one change which affects the type after the initial
  commit and we've undone that change within our codebase.

- Alyssa: What would change when we'd use japaric's variant?

- Leon: seems almost identical, I don't think it'd require any significant
  changes.

  We wanted to eliminate usage of `VolatileCell` within tock-registers
  anyways. Might just push forward on that issue and then remove `VolatileCell`?

- Amit: I will email the author. I am nearly certain that this code was not
  copied, but I just reimplemented this very small block of code. I don't know
  how meaningful it is to copy it over from some new repository. The more
  pressing issue seems to legal certainty for when the code is used in actual
  products.

- Alyssa: Would be shocked if these lines are legally enforceable.

- Leon: Agree with the sentiment that we should give proper attribution wherever
  possible. Contacting the author might be an elegant solution.

- Hudson: The history of the `VolatileCell` file in the Zinc repository is not
  complete because it was moved several times. The pre-move unfortunately goes
  back to the initial commit of the entire repository.

## TockWorld 5 Remote Presentation & Recording

- Johnathan: Chris Frantz might want to give a remote presentation at TockWorld
  but knows only in early July.

- {Amit,Hudson}: That seems fine.

- Johnathan: Question from Luis Marques as to whether there are going to be
  recording or transcripts of the presentations at TockWorld.

- Leon: We did agree on live-streaming presentations already, would be easy to
  just record using the tool we use to stream if the presenter is okay with
  that.

- Hudson: Yes. Probably would not want to record the discussion.

- Amit: Branden also seemed to believe the rooms are set up with AV equipment
  anyways.

## HMAC and Digest HIL - Mutable/Immutable Buffer Approach

- Hudson: Phil wanted to mention that he has published an updated HMAC HIL and
  within that PR a new TRD for Digest. This represents Phil's approach to
  calculating digests over that located in flash, memory, or partially in both
  regions. This is important to check the integrity of processes which have
  their code stored in flash, to avoid copying the entire process.

  This might be at odds with the approach Leon and I have been pursuing. Phil's
  reasoning is likely that agreeing on his approach would unblock the Digest and
  HMAC HILs. But perhaps Leon's codebase is in a state where we can also talk
  about using that instead.

- Leon: Been working on Miri tests on the new approach, testing it in a way
  which resembles typical usage in Tock. Have not noticed any unsoundness
  yet. Needs more work.

  In a previous call, at the very end, we talked about a good compromise: Phil's
  approach is really good at resolving this single issue in the Digest / HMAC
  scenario. Hudson's and my approach seeks to solve more issues, such as the
  current DMA buffer unsoundness, LeasableBuffer integration, etc., all in a
  single solution and type infrastructure.

  Our approach still seems viable and nice, but it needs more time. It seems
  trivial to migrate Phil's approach to ours at the given time. I'd be happy
  with merging Phil's approach first and then migrating later.

- Hudson: I agree. I still prefer our approach, but I am sympathetic that Phil
  is blocked on this. It does not seem like merging his version now is going to
  cause trouble when we want to migrate it to our approach.

- Leon: Perfect is the enemy of good. Once Phil's approach is in, the first step
  to supporting mutable & immutable buffers in a single API is done. Our
  approach will be an improvement on that interface, if it works. Just having
  Phil's approach in, though, I going to provide justification to continue
  working on this, and extending to existing subsystems.

- Alyssa: Also comes with an example on how the existing API is used in
  practice, which is helpful.

## Proposed UART HIL TRD Changes (draft 5)

- Leon: Proposed some changes to the just merged draft 4 of the new UART HIL
  TRD. Noticed a few oddities while implementing the new HIL's interface for the
  `sifive` chip. I realize many of my proposed changes are subjective in nature,
  however for further chips it's important to have a common ground to design
  compliant implementations.

- Hudson: I'm fine with these changes. Likely Phil just needs to do a pass over
  this.

- Leon: Will shoot him an email, thanks!

## Ti50 Team Tock Survey

- Alyssa: was thinking about sending out a survey to the Ti50 team before
  TockWorld about the painpoints and things they like about Tock. (There is one
  major painpoint w.r.t. to size and fixed size increase per app added.)

  Would that survey be worth discussing a TockWorld? What do people think?

- {Amit,Hudson,Leon}: Sounds great!

- Alyssa: More in the form of presentation or discussion?

- Amit: Likely depends on the results of that survey, and how much time that
  would take to present. I'd be very happy about a presentation focusing on the
  painpoints of the Ti50 team.
