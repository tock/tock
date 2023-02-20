# Tock Core Notes 2023-01-13

Attendees:
- Leon Schuermann
- Hudson Ayers
- Chris Frantz
- Philip Levis
- Johnathan Van Why
- Branden Ghena
- Brad Campbell
- Alexandru Radovici
- Alyssa Haroldsen

## Updates

* None.

## Open / Merged PRs

* Hudson: Merged 3 minor PRs

  - Fix Imix to not require process credential checking by default
     ([#3376](https://github.com/tock/tock/pull/3376))
  - Enable virtual function elimination by default for OpenTitan
    ([#3358](https://github.com/tock/tock/pull/3358))
  - boards/opentitan: bump to latest rtl sha
    ([#3359](https://github.com/tock/tock/pull/3359))

  Opened PRs:

  - `debug_writer_component`: make debug buffer size configurable; double
    default debug buffer size ([#3369](https://github.com/tock/tock/pull/3369))
  - `boards/qemu_rv32_virt`: set minimum reqd QEMU version to 7.2.0, fix
    Makefile rules, add documentation
    ([#3370](https://github.com/tock/tock/pull/3370))
  - Implementation of PWM functionality for RP2040 ([#3372](https://github.com/tock/tock/pull/3372))
  - Make the set_clocks functions of the RP2040 drivers public to crates only
    ([#3373](https://github.com/tock/tock/pull/3373))

* Branden: Why VFE for OpenTitan but not for others.

* Hudson: Generally very useful, but still has soundness issues. OpenTitan
  really cares about the code size, and the way they are using Tock seems to not
  be affected by the soundness bugs.

  One of the bugs is that, if you use it, but you don't rebuild the standard
  library, that can lead to soundness issues. There was another issue where it
  could be too eager in removing functions and removed some which were meant to
  be called.

* Johnathan: The people working on VFE are working together with the OpenTitan
  team, so it's kind of using what they are building.

## Approach to Propose Changes to Tock

* Alyssa: What is the best format to propose changes to or ideas for Tock?
  Presentation, Issue?

* Johnathan: If it's more concrete a GH Issue might be a good place, otherwise
  perhaps a presentation is better suited.

* Phil: Lean towards presentation, mostly to put the emphasis on discussion.

# License PR (#3318)

* Leon: The PR which initially sparked this discussion (implementation
  of the CAN infrastructure,
  [#3301](https://github.com/tock/tock/pull/3301)) has been waiting
  for a long time now.  Perhaps we can try to get this in, even before
  we finalize the last few formalities on the licensing TRD PR
  ([#3318](https://github.com/tock/tock/pull/3301)).

* Hudson: That seems like the main motivator for us to get the TRD in.  Not
  opposed to merging the CAN PR, if there is general agreement that what's in
  the license PR is what we are going to end up at.

* Pat (chat message): License PR kept falling off my list. Will try to work on
  it as soon as possible. Don't feel like you need my approval at this point,
  just get things in.

* Hudson: Thanks for taking this on in the first place!

  Does everyone feel comfortable with merging the CAN PR before formally merging
  the TRD?

* Johnathan: We were considering having "Copyright Tock Contributors" as a
  standard line, which this PR does not do yet.

* Hudson: Merging this PR even without this statement seems fine. Whatever we
  arrive at is not going to forbid the use of that. To be on the safe side would
  mean including this copyright assignment.

* Johnathan: Is there any other open discussion item? We might be able to
  resolve this today.

*(people going through the PR)*

* Hudson: it seems like this is the only unresolved issue. Main people involved
  in the discussion seem to be Leon, Johnathan and myself. We're missing Amit,
  but could try to come to a conclusion now. Leon, do you want to summarize your
  standing on that.

* Leon: Have some concerns with requiring that line. Even if not relevant from a
  legal perspective, from an open-source community aspect, it would be weird for
  us to assign copyright to an e.g., vendored file, to the general "Tock
  Contributors".

  This is especially problematic for vendored files where adding this line would
  be our only modification. However, even when it's a file written by a Tock
  contributor, it seems inappropriate to force-assign copyright also to the
  wider "Tock Contributors". Having this file seems like appropriation of the
  author's work.

  It seems more elegant to me to have that line be standard, but if a developer
  chooses to not include it, or we happen to vendor an outside file, that's okay
  too. If someone makes substantial contributions to a file, and this person
  happens to be also contributing to Tock, they are in their right to add that
  line back.

* Hudson: My argument is that the history of Tock has shown that this mostly
  won't happen. Basically every file for which "Copyright Tock Contributors" is
  not added initially, people will make edits to it but won't add further
  copyright lines.

* Johnathan: I agree. I think we should have it be in the examples in the TRD
  and ususally add it, but not require it.

* Hudson: I agree with Leon that we should not require this line on e.g.,
  vendored files. Is there an allow-list in the license checker? Then only those
  files would not be required to have this line.

* Johnathan: It is a bit tricky, but we already have a list of files which the
  checker does not look at, which we could reuse.

  I am leaning towards not enforcing this in software.

* Leon: I am not opposed to enforcing this through software in practice. I am
  just opposed to a formal document (TRD) stating this requirement. It's much
  easier to change the enforcement software to adhere to the reference, than the
  other way around.

* Chris: Generally agree with the philosophy on vendored files. If we aren't
  changing it, we shouldn't touch it.

* Hudson: I think that is how the document is currently written.

* Leon: Two lines which worry me: "All textual files that allow comments must
  include license and copyright notices."

  This is further ambiguous as to whether it implicitly enforces these files to
  follow our formatting guidelines.

* Johnathan: Yes, this sentence seems a little too strong.

* Phil: Can we just qualify this with "When possible, "? We can also provide
  particular examples of the exceptions we are thinking of.

* Branden: Impression of this dicussion is that everyone agrees and there is no
  real point of contention, except that we do not want to bind ourselves
  accidentally.

* Hudson: The one concrete change we'd need to make is on line 76-77, loosening
  the statement about where license and copyright headers are required.

* Johnathan: I will type up a commit after the call, to address both the
  exceptions around vendored files and to add "Copyright Tock Contributors" to
  the example.

* Hudson: Going back to the CAN PR. Alex, I think it is safer to add the
  "Copyright Tock Contributors" line, although from this discussion it seems
  like we are not going to require it. Would you be okay with adding that?

* Alex: No problem with that.

* Leon: The CAN PR has been open for a few months, had multiple approving
  reviews and a `last-call` label attached. Would we comfortable merging it
  immediately once the change has been made?

* Hudson: I think so. If anyone wants to speak out against it they can do so
  now.

* Johnathan: License checker PR can also be merged before the TRD, as it is
  effectively not turned on (except for the files added in its PR).

