# Tock Core Notes 2022-11-04

Attendees:
- Alexandru Radovici
- Amit Levy
- Brad Campbell
- Hudson Ayers
- Jett Rink
- Johnathan Van Why
- Leon Schuermann
- Pat Pannuto
- Phil Levis
- Vadim Sukhomlinov

## Updates

### App ID PR #3307: Correct Handling of Userspace Binary Version Numbers

* Phil: Created a PR ([#3307](https://github.com/tock/tock/pull/3307)) to update
  AppId. There is an issue with the current version in tree: it did not obey
  version numbers. If there are two binaries which have the same ID, the system
  should boot the app which has the higher version number. Going to optimize
  code size today.

  Gotcha: to make the loading algorithm simple, there are cases in which there
  can be transitive blocking in loading applications: binary A is blocked by B
  (e.g., it has a higher version number), B is in turn blocked by C, but C does
  not block on A, then it is possible that A does not boot. Seems like a
  reasonable limitation, appreciate feedback.

* Hudson: Tried looking at code size impact of this PR, but looks like benchmark
  CI did not run.

* Hudson: Brad is trying to port Apache NimBLE Bluetooth in userspace. Inspired
  partially by the fact that Rubble has been moved into maintenance mode. Also
  looked into Zephyr's bluetooth code, relying to much on Zephyr in general.

### Ethernet on Tock

* Leon: Created a tracking issue for Ethernet support in Tock
  ([#3308](https://github.com/tock/tock/issues/3308)). Rebased a lot of my
  previous downstream work, cleaned that up and pushed it to the `tock-ethernet`
  branches on the `tock` and `libtock-c` repositories respectively. It should be
  in a state where all LiteX boards can run a Webserver (using LwIP) out of the
  box on this brach. Now shifting efforts to get the ENC28J60 SPI-connected
  Ethernet chip, as well as the QEMU VirtIO network card support finalized.

* Amit: How are you doing TCP?

* Leon: Currently all done in userspace through LwIP. The kernel just contains a
  "TAP driver", similar to Linux or BSD TAP devices. Ultimate goal: use this as
  a basis to get applications connected, iteratively move parts of the stack
  into the kernel (perhaps in an intermediate stage calling back-and-forth from
  userspace to kernel to multiple times). It's really hard to design an
  efficient, adaptable, performant, ... network stack from nothing, so a good
  approach seems to approach this step-by-step.

* Alexandru: I have a Bachelor's student who is going to write an Ethernet
  driver for one of the STM boards, to also support a physical Ethernet
  interface.

### Interrrupt & Context Switch Code Investigation / Redesign

* Amit: There is an undergraduate student looking into the scheduling and
  interrupt issues that we have encountered. She is starting to look at how we
  could restructure our interrupt handling to hopefully make it more efficient
  and also address the complexity of the current solution, which makes it hard
  to track down the issues we encounter from time to time.

* Alexandru: Can you connect us with her? We have some people working on similar
  things as well.

* Brad: Does this include tracking down the issue we've experiencing on this one
  build for `imix`?

* Amit: Yes and no. She has looked into that, as part of that also to understand
  the current implementation. It is plausible that the proposal will be
  different enough to make fixing the particular bug we're experiencing now
  largely redundant.

* Brad: It seems like we're always going to wonder what the root cause of this
  bug was. Definitely curious on what she finds out. It seems like what we have
  now is not so bad.

* Amit: At a high level, the task is to look into alternatives (such as
  PENDSV). The outcome is either a proposal or implementation, or a well
  thought-out explanation for why the way we're doing things right now is
  necessary. The flow of how interrupts get handled in the kernel is a solution
  derived from constraints that we have, it's not clear whether what we have is
  the right solution.

* Hudson: If we want to find out what the original bug is, if anyone on the call
  owns a J-Trace, that might be the easiest way to get to the bottom of
  this. It's pretty expensive.

* Leon: It seems not entirely clear whether this would be compatible with the
  SAM4L and/or whether `imix` has all of the required pins available.

* Pat: We can probably get more information with the J-Trace still.

* Leon: From what I've read, SAM4L has a custom mechanism to access serial trace
  data on a pin which may or may not be available.

* Pat: Perhaps we can also use a Saleae Logic for that. We could write a custom
  protocol debugger. The hypothesis is that the trace is output on a single pin,
  producing a serial stream. The Saleae should be able to sample at a
  high-enough frequency.

## Significant Pull Requests

* Hudson: A few new PRs:

  - [#3303](https://github.com/tock/tock/pull/3303): Update Nightly Oct 2022 and
    remove asm_sym feature
  - [#3263](https://github.com/tock/tock/pull/3263): Use Cargo workspace package
    table
  - [#3276](https://github.com/tock/tock/pull/3276): Fixup return values for
    NODEVICE in master

  Open PR:
  - [#3312](https://github.com/tock/tock/pull/3312): doc: Add
    ExternalDependencies.md
  - [#3310](https://github.com/tock/tock/pull/3310): USB implementation for
    Raspberry Pi Pico


## License Headers Discussion

* Hudson: [PR #3301](https://github.com/tock/tock/pull/3301) by Alex's company
  introduces some license headers into the repository.

* Leon: Brought up this issue as I remembered that we had a related discussion a
  few years back. This was around the issue of adding license headers to files
  in the Tock repository as a potential requirement by Google. This PR would add
  a license header and copyright notice. Given that we did not really reach any
  conclusion after the previous discussion, it seems good to at least have a
  consistent line of argument w.r.t. whether we'd allow license headers or
  copyright statements in files and what they must look like, before we
  implicitly commit to something by merging this PR.

  The discussion last week did also not bring any conclusions. We established
  that removing the license text and retaining only copyright statements may be
  more problematic than having both or none.

* Johnathan: The reason why this makes me uncomfortable is that it may seem as
  if these files were then subject to the default copyright (where _default
  copyright_ is not a well-defined term, varies internationally). Someone might
  see these copyright statements and assume it is not licensed under MIT or
  Apache 2.0.

* Leon: Would be fine with adding license texts and copyright notices to files
  (where copyright can be attributed to invidiuals or legal entities, depending
  on the contributor's legislation). Main worry is that it is going to be hard
  to manually enforce that there is a consistent license assignment on all files
  (i.e. making sure the license text is identical). This seems rather
  important. We may want to then commit to using automated tools to ensure that
  all added license texts are consistent.

* Phil: What is the concern with putting licenses in every file?

* Pat: Code churning overhead. Also, if we are having licenses in each file, we
  have to have someone build a tool to enforce that at PR time.

* Johnathan: If you all come up with the license header, I can forward this to
  the OpenTitan legal committee.

* Amit: If we added a single copyright header to all files (something like
  "Copyright <year> Tock Developers"), that seems fine. What about copyright
  assignments to individuals?

* Leon: I presume this is fine. In many countries, it is actually not possible
  to "transfer" copyright. As long as we have each file and each contribution
  consistently licensed (under MIT and Apache-2.0), commiters can retain their
  copyright but give a non-revocable, non-exclusive usage agreement as defined
  by these licenses.

* Johnathan: Alex, would you be okay with a generic copyright statement such as
  "Copyright Tock Contributors"?

* Alex: Needs to be proof of authorship as mandated by investors, prefer to have
  the name included.

* Amit: At some point, who is the author of a file? For a new contribution that
  is clear. What happens if files change over time?

* Pat: This comes down to collaborative copyright. It really is the licensing
  which matters in the end.

* Leon: In many jurisdictions, whether we have explicit copyright statements or
  not is actually irrelevant. What is important that that we are granted the
  same irrevocable and non-exclusive rights from every contribution, which is
  what we are relying on currently through the presence of the LICENSE
  file. Despite not having explicit copyright assignments the files currently
  are still subject to copyright.

  I'm advocating for a consistent license assignment in each file, if we do
  include headers in files.

* Phil: Agreed with the fact that it is the license that matters and not the
  copyright, for a variety of reasons.

* Amit: Perhaps it would help to clarify what Rust does and what we're
  advocating for. It seems like this position entails that generally it is fine
  to add license / copyright headers, but they are not required. Contributors
  can add a header indicating our license and add a copyright assignment along
  with their name and/or organization.

* Hudson: Generally seem to be three options:

  - Don't support having license and copyright headers in files and refuse to
    accept PRs which add them. Does not seem like a good option.

  - By default insert a license header in every file. In the general case it
    assigns copyright to the Tock contributors. Individual contributors can add
    their name to it. We would have a tool to check that the license text is
    identical in every file.

  - We a notice [similar to Rust's in its LICENSE
    file](https://github.com/rust-lang/rust/commit/69b1ccb44e76bc5c3eb815bec852e13becdb96f4),
    indicating that "[c]opyrights in the Rust project are retained by their
    contributors. No copyright assignment is required to contribute to the Rust
    project. [...] Some files include explicit copyright notices and/or license
    notices."

    One issue with this is that we don't explicitly place license headers in
    each file (while copyright notices may well be in these files).

* Pat: Looking at the [OpenTitan license
  header](https://github.com/lowRISC/opentitan/blob/5af4ad37777f38efc31c579efce649dddaa2541b/sw/device/silicon_creator/rom/e2e/empty_test.c),
  we could reduce ourselves to the lower two lines and include it in every
  file. Do not even need to explicitly include "Copyright Tock
  contributors". It's fine for individual contributors to add copyright notices
  if they want.

* Phil: OpenTitan's strategy views files in the context of their
  repository. However, reusable components can be copied to other repositories,
  and then the `LICENSE` file pointer does not apply any more.

* Pat: There's still the SPDX license identifier, and there is even a syntax for
  dual-licensing.

  I feel like I have a good enough understanding of the various constraints to
  draft this up in a TRD.

* Phil: Remaining issue - suppose we have a file contributed by developer A, but
  then developer B makes significant changes and wants to add a copyright line,
  but contributor A does not want that.

* Leon: Fine from a legal standpoint. By developer A granting us the original
  file under MIT and Apache-2.0, the we are in the right to make these changes.

* Phil: This is not a legal argument, just what if a developer asks us not
  to. We wouldn't want to make developer A upset, so this would have to involve
  a conversation.

* Amit: In these cases, we can do some mediation if there is a problem.

* Phil: In TinyOS, the copyrights were held by the original contributors
  (e.g. schools / invidual developers). It was nice to see which people and
  organizations contributed to files over time.

* Amit: To be clear, it seem fine in this context for individuals and
  organizations to still add their individual copyright lines.

* Johnathan: I can offer to write the CI enforcement tool for this. And once Pat
  has a writeup and it has received a significant consensus in the project, I
  can present it to the OpenTitan legal committee.

# PR #3312: Document Requirements for Adding External Crates

* Phil: Concern raised by Brad - if we are pulling in a crate, what is the full
  set of dependencies we are adding. `cargo tree` gives us these insights. This
  seems like a middle-ground to determine a set of dependencies which a given
  Tock release has.

  [First draft shared by
  Alistair](https://github.com/tock/tock/blob/1ed33ef32424bee2c7e4b2a2106a4a7b41f99944/doc/ExternalDependencies.md).
  Not perfectly in line with what we've talked about, but a good start.

* Hudson: We probably do not want to start out as broad and permissive as the
  text suggests. It also does not address the issue of our safety argument for
  capsules: given we're saying that capsules are safe because they cannot use
  `unsafe`, capsules using `unsafe` are in conflict with that. This would be
  expanding the trust boundary for our memory safety argument.

* Phil: Response to that is that we use the `core` library, which is unsafe
  internally.

  Our reason for allowing external dependencies for use-cases such as
  cryptography implementations is that our own implementations of these
  libraries could and would likely reduce the security of Tock.

* Amit: Disagree. It's different security concerns completely. One of them is
  linking against untrusted code, having buffer overflows, accessing raw memory
  and doing an unbounded number of other bad things. The other is related to the
  correctness and security of the actual cryptographic primitive, e.g.,
  protecting against side-channels.

* Phil: Generally, the following statement seems to be below the bar for usage
  of external dependencies we want to maintain: "The external crate must provide
  important functionality that couldn't easily or realistically be provided by
  the Tock developers."

* Hudson: We would most likely want to use these libraries within `capsules`.

* Amit: Does not have to be in `capsules`. For instance, for a cryptographic
  library there could be many different implementations of these primitives
  through a common interface (including in hardware). Capsules are only
  dependent on the definition of this interface. If a board chooses to expand
  its TCB to include trusting some hardware component or an external library,
  that's a reasonable and informed choice.

* Hudson: This adds the upside that external dependencies using `build.rs` or
  procedural macros will not force people to run this code on their host just
  for using capsules.

* Phil: Like the idea of pulling in external dependencies not in `capsules`, but
  only through the board or chip implementation.

* Amit: I think it might be fine for `capsules` to, in principle, depend on
  external crates if we can ensure that these crates abide to the same
  restrictions and rules that capsules otherwise have to.

  For the kernel that is more tricky, as it is part of the TCB. What code does
  there matters more.
