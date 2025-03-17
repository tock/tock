# Tock Meeting Notes 2025-01-03

## Attendees

- Hudson Ayers
- Amit Levy
- Leon Schuermann
- Johnathan van Why
- Brad Campbell
- Tyler Potyondy
- Kat Fox

## Updates
 * None

## Tock 2.2 Release Plans
* Leon: We tagged 2.2-rc1 a couple weeks ago
* Leon: It is clear we are going to go forward with less testing of all the
  boards we have in tree because it has been discouraging us from making
  releases, and we want more frequent releases
* Leon: Instead, we are mostly relying on the automated testing available via
  the hardware CI infrastructure. The supported boards at least cover all of
  the chips that are used across all of our their 1 boards. I know folks felt
  uneasy about this not including any RISC-V testing, so I have also been
  trying to run a full suite of Litex testing. It seems that we already have
  pretty complete testing in CI for the LiteX emulated platform at least.
* Leon: One major issue is the status of the 15.4 subsystem — I have not really
  gotten it to work, so would like some info from Tyler before stamping the
  current state of things as ok for release.
* Leon: We are almost 2 years from our last release
* Amit: I recall that 15.4 required some magic incantations for stuff to work,
  has anyone done anything else?
* Leon: I know Tyler submitted https://github.com/tock/libtock-c/pull/485 which
  fixes one bug we have seen
* Amit: I think the main issue is we have multiple 15.4 stacks now, one with
  Thread in user space and one with ipv6 in the kernel, and I do not know what
  is enabled by default and how that affects the example applications.
* Leon: I did test openThread and it worked on the nrf boards.
* Brad: There is a shared subset of 15.4 that is exposed to user space, and
  then the OT driver exposes the same set of things plus some additional stuff
* Leon: Tyler, do you know if UDP works with your libtock-c PR?
* Tyler: It should, but I have not touched this since Spring of 2023. The
  problem is the standard 15.4 driver that includes MAC/framing has issues
  because the new 15.4 driver for the radio defaults to being off, but we do
  not have a way to expose turning on the radio to the MAC layer driver used by
  the UDP stack. Yesterday I was working on a fix to add that back and writing
  some treadmill tests for it. The main problem was 15.4 could not transmit on
  the base stack so the UDP tests were failing.
* Leon: What I am hearing is we should probably wait for end of today and see
  if we can get it working, and then release regardless with a note in the
  release notes if needed.
* Tyler: Sounds good to me
* Tyler: Where should I document the stuff that is working now that I have
  tested?
* Leon: The only tests are the UDP ones right?
* Hudson: There are also some raw 15.4 user space tests
* Leon: Can we summarize all this in a comment?
* Tyler: Sure, we can sync up offline on this
* Leon: I just sent a PR that allows us to trigger a treadmill workflow on an
  arbitrary revision. We can’t really test this until it’s merged but it
  already is approved, I think we should just merge and then test it.
* Leon: It is very easy to make mistakes in these GitHub actions PR, but if it
  does not work I will send a fixup PR
* Leon: We also have https://github.com/tock/tock/pull/4275 to specify a
  minimum stable rustc version.
* Brad: This PR originated because I failed to build hail when wanting to test
  it for this release. The issue was my local stable toolchain was too old. And
  it seems there is no way to just specify a minimum version in cargo without
  forcing a specific version, and the error messages are confusing.
* Brad: This PR adds this infrastructure to our make build system instead, it
  automatically updates your toolchain after giving you a warning.
* Leon: The alternative suggested by Johnathan is to specify an exact version
  in rust-toolchain.toml so that we always have users build with the exact
  version. This means more downloaded versions but has some upsides described
  in the PR.
* Leon: To me, it feels like a step backwards to force users to use an older
  compiler version when newer ones work.
* Johnathan: Clippy will disable lints if your MSRV does not support certain
  features, it would be nice for us to take advantage of that. We might
  eventually run into something similar with compiler warnings, since we treat
  warnings as errors.
* Hudson: We only treat warnings as errors in CI, so this would not actually
  break user builds who are using a newer compiler.
* Johnathan: Lints affect everything in dependency trees but this seems like
  not an issue since we only treat warnings as errors in CI.
* Brad: I think it is important that we accept any stable version, I agree with
  Leon. That is kind of the point of Hail using stable, as an example.
* Leon: Next PR that needs to be merged is
  https://github.com/tock/tock/pull/4280, this targets the release branch, I
  will also add one to backport it to master. It adds the version bump and
  release notes. I have another version of this in a hackMD that has more
  detail (https://hackmd.io/TNqklrfYQlqFSJLI5VSYYg ), but I will just use that
  in the release notes and keep the compressed version in the changelog in the
  repo. If there are issues with UDP, they will get added to this PR before
  Leon merges it (as well as to the long version).
* Leon: Last thing before release. Our risc-v tests are mostly just the LiteX
  CI tests, though I will run them on a board as well. I have a dummy PR that
  bumps the libtock-c version pinned for the litex-ci so I can get the CI tests
  to run against the latest libtock-c: https://github.com/tock/tock/pull/4285 .
* Hudson: Tyler do you have PRs up for the 15.4 changes?
* Tyler: Not yet, I am trying to do some testing because I do not have the
  physical boards with me. They should be up in the next few hours.
* Leon: I will wait until tomorrow so those should have a chance to go in.
* Hudson: Tyler can you post on slack when those go up?
* Tyler: Sure, and this is the fix we talked about in the network wg in early
  December, to add a syscall for turning on the radio to that stack.
* Brad: So Hail does not work, but I think that is because of user space. Amit,
  did you get a chance to look at that?
* Amit: Let me do that right now
* Brad: Do you think that it is solvable?
* Amit: yes

