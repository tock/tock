# Tock Core Notes 2022-10-14

Attendees:
- Adithya Anand
- Hudson Ayers
- Chris Franzt
- Alyssa Haroldsen
- Philip Levis
- Amit Levy
- Alexandru Radovici
- Leon Schuermann
- Vadim Sukhomlinov
- Johnathan Van Why

## Updates

* Hudson, Phil: App ID PR merged. (*woohoo*)

* Hudson: Brad has started cortex-m exception handling rewrite,
  getting close to component revamp. `finalize` will no longer be an
  unsafe method. After that, revisit making `static_init!` unsafe
  debate and how to make it safe for unit tests.

* Hudson: OpenTitan version bump appears to be close to ready. This
  will also unblock automatically generating peripheral register
  definitions.

* Chris: Yes, although for the automatic register definitions
  generation, some more work is required in addition to that.

  We might generate the definitions once per release through a manual
  invocation of the script. Alternatively, we can integrate this into
  the OpenTitan release pipeline.

### Text and graphic displays: couple of PRs open, varying states.

* Phil: Would like to talk about the various text & graphic display
  PRs.

  - Alex: Planning to work on this over the next weeks. Unify the
    bitmap, text & 7-segment displays. Was hoping to get some more
    input from @dzc-self. Displays vary significantly, blocked at
    finding a reasonable abstraction over these types of displays.

  - Phil: Unifying 7-segment with text / bitmap seems tricky. Perhaps
    unifying the control path.

  - Alex: Need to sketch this out. Will send you an email.

### Packed System Calls

* Alex: Working on the packed system calls. Struggling to find an
  implementation which does not significantly increase kernel text
  size.

### Virtual Alarm Issue - PR #3277

* Hudson: Jett submitted a fix for virtual alarm (PR
  [#3277](https://github.com/tock/tock/pull/3277)), such that in
  certain scenarios we avoid missing alarms. There appear to be some
  tradeoffs. Assigned Phil to that.

* Phil: Thought we had a solution to that. We keep on having these
  point solutions to subtle problems, there may be a larger underlying
  issue with our testing methodology. Will take a look today.

## Documentation/implementation mismatch for an unknown driver_num (#3278)

* Johnathan: TRD 104 has a general clause saying that "if a process
  calls a system call and passes a driver number that the kernel
  doesn't have, the kernel should return `ENODEVICE`". If I remember
  correctly, this is behind some error checks, such as the in-bounds
  check for upcall numbers.

  In neither Tock 2.0 nor 2.1 this is done for all system
  calls. `subscribe` 2.0 we returned `NOMEM` instead. In 2.1, `allow`
  and `subscribe` return `NOMEM` instead. We are not aware of any code
  relying on that. `libtock-rs` does not actually interpret the error
  codes, just forwards them. `libtock-rs`'s fake kernel used them.

  How to fix this? Are we just going to change it to return `NODEVICE`
  in every case?

* Hudson: Brad believes that we should consider this a bugfix and do a
  minor version bump.

* Phil: Agreed.

* Hudson: Anyone opposed to that? (*no*)

* Alexandru: PR [#3276](https://github.com/tock/tock/pull/3276) which
  reduces the kernel size already contains a fix for that.

* Hudson: Is it the case that it is difficult to split out this change
  separately from the PR?

* Alex: No, will split it out.

* Leon: It's good for our release notes to be able to link to a
  specific PR for breaking changes, where these changes are isolated
  in that PR.

* Johnathan: Adding a comment on the original issue.

## Guidelines for when external dependencies *can* be allowed

* Phil: postpone to next week with Brad on the call. Can explain what
  he said on the OpenTitan call.

  Generally, using external dependencies seem necessary in some cases,
  but we have to be very careful about the chain of dependencies
  pulled in.

  One thought was to only allow leaf-dependencies.

## Implicit Support of Conditional Compilation in Register Definitions (Issue #3257)

* Leon: People have been using tock-registers with Rust's conditional
  compilation support to include or exclude the generation of certain
  fields in `tock-registers`'s `register_structs!` macro.

  This used to work through some tricks employing zero-sized reserved
  fields to automatically insert padding in these definitons.

  The errors which now prevent this from working probably have been
  errors for some time now, however they would be issued to the user
  as part of Rust `#[test]` tests which would need to be explicitly
  run (through e.g., `cargo test`). In the latest `tock-registers`
  release, these sanity checks have been converted from runtime tests
  to compile time errors.

  On the one hand, it seems to me that these usage patterns would
  still be good to support through `tock-registers`. On the other
  hand, we perhaps should think about whether supporting this is worth
  the effort and whether we want to make guarantees about supporting
  these use cases in the future.

* Phil: Strongly against using this within Tock itself. That being
  said, if external users want to do this, no one is going to stop
  them.

* Johnathan: When working on the unit-testing support and fixing
  `tock-register`'s soundness issues, it became obvious that I needed
  to make backwards-incompatible changes.

  There is going to be a similar discussion: do we want to keep both
  versions around, tagging one as `1.0` and the other as
  `2.0`. Maintain both in parallel? Keep both versions around in the
  repository? This issue is another one which feeds into this issue.

  Conditional compilation is going get really difficult with unit test
  support. Configs are going to get very fine-grained, switching off
  certain parts of traits.

  It seems to me the answer is going to be: merging in my new
  functionality, we release the old version as `v1.0` and then release
  `v2.0` with the new interface.

* Leon: One thing this issue makes really clear: guaranteeing API
  stability is already hard, but doing it for a `macro_rules!` API is
  so much harder.

  It seems very important that we have a solution for users of
  `tock-registers` right now --- which may well be saying that this is
  now unsupported (we're still pre-release). However, this is still an
  important discussion going forward: our new interface will
  presumably still be exporting macros. How are we going to manage
  user expectations and API stability?

  Specifically: we're exposing a macro infrastructure to users. Not
  using procedural macros, by Tock's principles. Because we're
  operating on a syntax-tree abstraction, manipulating tokens, I would
  go as far as to say that any slight changes to this provided macro
  infrastructure has the potential to break external usages in subtle
  and unexpected ways.

* Johnathan: Not entirely convinced that we're not using procedural
  macros.

* Leon: Not necessarily opposed to this, but it'll require an entirely
  new discussion around introducing any kind of procedural macro into
  Tock. We have been refraining from that.

* Alyssa: `syn` is a lot more rigorous in what it would parse compared
  to `macro_rules!`. It would make things more complicated, but also
  it would force us to think about a lot of these weirder scenarios
  up-front.

  If valid input that was previously accepted is now rejected in a
  future version, that is a breaking change. For the next version of
  `tock-registers` to avoid this from happening, we should extremely
  narrow and defensive of the things we support.

* Leon: Remaining question is -- for the currently released version,
  we're still in the initial development phase, version `0.x.y`. From
  a SemVer perspective, it would be fine to make these breaking
  changes. Do we want to take a stance on this and say that this is
  now unsupported, or invest effort and try to fix it?

* Alyssa: Support that mode of usage explicitly, unless we have a
  really good reason not to do that.

* Hudson: A reason not to support it would mean that we'd invest
  effort in fixing this, which would be made completely redundant and
  incompatible with Johnathan's rewrite.

* Alyssa: If it accepts attributes to propagate them, it should do so
  correctly.

* Johnathan: The challenge with this is doc-comments. It interprets
  them as attributes. If you want your registers to be documented such
  that this shows up in Rustdoc, you have to allow passing arguments.

* Alyssa: Don't have to allow all attributes. Can limit to doc
  comments. Really easy in procedural macros. Should be doable in
  `macro_rules!`.

* Hudson: Hadn't promised the library was stable. Wouldn't be breaking
  anything by just not supporting this.

* Alyssa: Issue with all code generation -- have to predict how it's
  going to be used.

* Leon: Circling back to the original issue -- there seem to be three
  possible solutions:

  1. Can't reproduce the issue locally. This may just be a
     misunderstanding, and so perhaps resolving it is fine.

  2. If this turns out to be breaking, we could either take the stance
     that this specific use-case is just no longer supported,

  3. or we could introduce a flag to turn these compile-time checks
     off.

* Alyssa: If we were stable, I'd say revert the breaking changes. On a
  version 0, it's fine.

* Hudson: There doesn't seem to be a lot of motivation to stabilize
  `tock-registers` at this point. We've got some pretty big changes
  coming up, so there does not seem to be a point in making any
  guarantees to downstream users.

* Alyssa: Obviously, we should provide some best-effort stability such
  that things that look like they should work do work. Ultimately,
  we're always going to be encountering really obscure uses of
  provided interfaces that break. Whether these deserve fixing should
  probably be decided on a case-by-case basis, depending on just how
  obscure they are.

* Leon: Takeaway from this: try to get the issue resolved by attemting
  to support this usecase, but if it ultimately does not work with our
  current release, we prefer to keep the static assertions. This is
  not a dealbreaker for downstream code.
