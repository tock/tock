# Tock Core Team Call 2021-07-16

Attending:
- Alexandru Radovici
- Hudson Ayers
- Brad Campbell
- Branden Ghena
- Johnathan Van Why
- Anthony Quiroga
- Pat Pannuto
- Vadim Sukhomlinov


## Compatibility TBF proposal

* Alexandru: Submitted two proposals:

  - https://github.com/tock/tock/pull/2669

    First is with a static simple header, its only purpose is to add
    kernel and ABI version requirements. This would indicate the fixed
    version which an application needs to run. Easy to parse and
    should not increase code size. However its fixed, the header can't
    be changed again. Kernel major and minor version are 8 bits each.

  - https://github.com/tock/tock/pull/2670

    Second PR is a more advanced version of the first one. Adds header
    with compatibility elements. Has a list of compatibilities
    required, one of them being kernel version and one ABI
    version. Has advantage of being extensible. Parser will increase
    in size.

* Johnathan: Is there a reason why the version of the ABI and the
  major version can be different? As far as I know, we follow SemVer,
  which means that a breaking change of the kernel ABI would manifest
  itself in a kernel major version bump.

* Alexandru: Okay, that makes things simpler. Then a second header
  would be simply the kernel minor version.

* Johnathan: About the kernel minor version: maybe that is useful,
  however apps are already supposed to run on multiple boards. Thus
  specifying major and minor version is not sufficient to guarantee an
  app will work.

  The use case is not immediately obvious.

* Alexandru: I agree with that. I am not sure whether we should
  specify a minor version.

* Hudson: It seems to make sense to just have ABI version.

  Also 16 bit for the ABI and 8 bit for the kernel major version seems
  weird. We should only have one of those fields.

* Alexandru: The kernel major version should suffice.

* Brad: What compatibility checks do we want specifically? For
  instance, Tock 2.0 is going to be released, however we're going to
  want to add a more flexible allow API later on. I do not think this
  would be a major version change.

* Johnathan: Yes, this an example for a minor version change.

* Alexandru: If we would add it after the exit system call, it
  wouldn't be breaking, because an app could protect itself.

* Johnathan: Not sure whether apps can protect themselves. Not all of
  our system calls return errors in the same way. If we made a rule
  specifying that all new system calls must return the "normal" system
  call return type used by `subscribe` and `allow`, then yes. If we
  need another system call like `yield` or `exit`, where we are not
  expecting a return value, an app cannot protect itself against a
  missing system call implementation.

* Alexandru: I agree. Thus adding system calls would change the ABI
  version, in my opinion.

* Johnathan: This sounds like an argument in favor of major and minor
  number. It's a backwards compatible in that an older app will run on
  a newer kernel.

* Brad: I see the argument for adding the major and minor version. We
  already have that information and would not need to create another
  number to keep track of.

* Johnathan: We need the major number in the TBF to protect
  it. However, for minor the version, we could add a new system call
  to get information about the kernel version. Disadvantage: with the
  header, it gets bigger for every app.

* Brad: How does that affect this?

* Johnathan: One option is to put both the major and minor version in
  the TBF header and have the kernel check both of them. Another
  option is to just include the major version in the TBF header and
  leave comparing minor versions as a to-be-designed mechanism.

  Downside of including the minor version in the TBF header is, the
  more information we put there, the larger the header gets for every
  app on the board. Probably few apps are going to check the kernel
  minor version.

* Hudson: In this TBF header, I assume the length field describes the
  length of this struct?

* Alexandru: It describes the length of the header (TLV header plus
  the data).

* Hudson: Shouldn't the length be fixed, at least for v1?

* Alexandru: It is fixed for v1, the length is `4`. The TLV header
  requires a length.

  Due to alignment constraints we still have 4 bytes to be used for
  versions. So we could use 16 bits each for the major and minor
  kernel versions and not include an ABI version?

* Brad: I like that.


* Leon: Question for v2. As far as I understand, this proposes are
  more flexible and extensible structure for adding version
  constraints. Wasn't being able to add new fields the idea behind
  TLVs in the first place, and couldn't we just add an additional TBF
  header in the future, if there are additional version constraints to
  consider?

* Brad: Yes.

* Hudson: Also seems reasonable to have another optional TBF header
  for just the minor version, which apps can set if this is relevant
  to them. This could allow us to decrease the size further.

* Brad: For V1, I don't think it's going to get any smaller. If kernel
  major- and minor-version are each 16 bit, we end up with 32 bits
  which matches our padding constraints.

  In the scheme of things, adding a few extra bytes solves a lot of
  problems. I like the major- and minor-version approach, each having
  16 bits.

  The entire header would be entirely optional. Apps do not have to
  specify their version constraints.

[continuation of this discussion later on in the call]

* Brad: We want to get this in before Tock 2.0. What does that mean?
  Does this mean having the kernel check?

* Alexandru: That was my suggestion. Checking the header in the kernel
  and skipping the app if it is not present.

* Leon: Do we want to block on this for Tock 2.0? If I remember
  correctly, last week we agreed this might be possible to integrate
  potentially post-2.0? Or at least have us start release-testing
  without this?

* Johnathan: If, when we introduce this feature, we make the kernel
  skip apps without this header, that is a breaking change. I think it
  is an app choosing to protect itself, so a kernel should load an app
  blindly if it lacks the header.

* Alexandru: But this will not protect v1.x apps from being
  loaded. Those will then still fault.

* Hudson: We can make this a configurable option from the board. As
  such developers could choose whether this header is checked or
  ignored, if one does not want to include it for whatever reason.

* Johnathan: I would expect some users at Google to utilize this
  feature and remove that code.

* Leon: Would be convenient and simple to implement, just pass an
  additional parameter to `load_processes`, which is only a helper
  anyways.

  For all of the upstream boards, we would probably leave the checks
  enabled. However, it'd be easy enough for people to opt out then.

* Alexandru: Sounds good. I'll have a PR ready by next week. Still not
  clear to me whether `tockloader` passes unknown headers.

* Hudson: Probably something we can start release testing without.

* Brad: What does the option do?

* Hudson: Passing some flag to load processes
  (e.g. `check_kernel_version`). If that is true, the kernel will
  refuse to load an app which is missing this TBF header. By default,
  this means that running a 2.0 kernel on an upstream board will cause
  the kernel to print a warning or fault. Setting it to false will
  disable these checks. This means that having many small apps on a
  board does not require the header to be present in each app.

* Brad: What would that be set to, for example on `hail`?

* Hudson: For upstream boards it'd be set to `true`.

* Brad: If I'm a developer, I  build an app with libtock-c and install
  it, and now my kernel panics?

* Alexandru: Not panicing, just skipping.

* Brad: But then my app won't run.

* Alexandru: But if we enable `debug_load_processes` the user will be
  informed.

* Hudson: I do not think that users making this mistake will know
  about kernel config options.

* Alexandru: So then print a message. It's the same as "cannot
  allocate memory".

* Brad: It is not. The same app will go from working to not working,
  but not for any technical reason.

* Alexandru: But it will print that it is not compatible to the
  current kernel.

* Brad: It is, the users might just have an older version of
  `elf2tab`.

* Alexandru: You mean when the header is missing? That's why you have
  the flag: either enforce it or don't. We could print that the user
  should switch the flag to `false`.

  Proposed message: _This app does not provide compatibility
  information. If you want to load it, switch this flag to false._

* Brad: Trying to watch out for people unfamiliar to Tock. Everytime
  we make the user do something, we should ask ourselves from the
  user's perspective: why do I have to do something, why don't you
  just ship me working code?

* Alexandru: On the other hand, if you load a v1 app, it faults.

* Brad: We cannot look at only one side of the coin.

* Hudson: I agree that panicing or printing a message just because
  someone has an older version of elf2tab is pretty bad.

* Leon: I am in favor of having a breaking change now, if there is a
  clear message shown to the user on the precise steps to do, as for
  instance: update your `elf2tab` version. We can communicate that,
  for Tock 2.0 apps to be run on a 2.0 kernel, with the version check
  flag enabled, an update of `elf2tab` is required.

  If there is a clearly communicated error message, I'd be fine with
  refusing to work until the issue has been resolved on the user's
  side.

* Alexandru: Agree with Leon. My students had a lot of problems with
  loading 1.x apps.

* Branden: Would the kernel printing "you need to update elf2tab"
  would resolve some of the tension you see?

* Brad: Its better, but I would be annoyed.

* Branden: This is only an issue if you're an old user now switching
  to 2.0. If you're a new user, you're starting out with an up-to-date
  version of `elf2tab`. If you're an old user, you'd be more annoyed
  if an app silently failed, rather than the kernel telling you to
  update your toolchain.

* Hudson: You would also be annoyed if you are loading a 2.0 app on a
  2.0 kernel, and the kernel still telling you to update `el2tab`.

* Alexandru: How difficult would it be to change `make` to update
  `elf2tab`? Is that possible? What's the blocker with doing that?

* Brad: We do that with `rustup`.

* Alexandru: The Makefile could issue an error if the `elf2tab`
  version is too old.

* Hudson: I'm in favor of, as part of the 2.0 version, bumping the
  minimum required `elf2tab` version, and introducing checks as to
  whether their `elf2tab` version is not recent enough.

* Brad: I would rather have the tooling which gets this app on the
  board to indicate this.

* Hudson: But that is not possible, except if we're using
  tockloader. Proprietary loaders do not support this.

* Leon: Isn't that complementary to this? I suppose it's still useful
  to have metadata in the kernel to allow determining information
  about the kernel on a board and have tooling check this?

* Hudson: Not what I was trying to say. Trying to say that
  `tockloader` can check the minimum required `elf2tab` version.

* Leon: But only for 2.0 apps. Older versions should still work for
  1.0 apps.

* Hudson: We could just bump the minimum supported version of
  `elf2tab` regardless.

* Alexandru: Might not use `tockloader` with `elf2tab`. Might just get
  a binary from the Internet.

* Leon: Right, and we don't want to force presence of this additional
  header, for instance for people who choose to opt out as Johnathan
  indicated.

* Brad: How are people getting outdated apps?

* Alexandru: For instance, from a forked or out-of-date `libtock-c`.

* Hudson: Or using `libtock-rs` today.

* Branden: But then, adding infrastructure to update `elf2tab` to the
  Makefiles wouldn't help people.

* Alexandru: Yes, hence have runtime checks in the kernel as a last
  resort.

* Leon: To summarize, it seems we now have two options to suppress
  these checks: either properly update `elf2tab` to include the
  version header as discussed, or recompile the kernel with a specific
  flag disabled, both of which are clearly communicated to the user,
  at least when the kernel tries to load them. That sounds sufficient
  to me? I don't think we get around one breaking change in the chain.

* Brad: I struggle with this being an optional header.

* Leon: Okay, here's a question - we have the header with a major and
  minor version. How do we handle forks which have their own
  versioning scheme if this header is mandatory? Or development /
  company internal forks which want to have their own style of checks.

  If I were to ship this as a product, using a customized binary, I
  wouldn't want to include this header. Also it would indicate that I
  am stating some compatibility w.r.t. the then unrelated Tock
  upstream versioning.

* Hudson: I'm confused at how making this optional helps.

* Brad: Makes the rule clear and reduces complexity. If there's an app
  not having this header, it won't run.

* Hudson: Wouldn't be an issue for upstream, only for users building
  their own boards.

* Brad: Still, we have a lot of boards. Confusing if configured
  differently.

* Hudson: All upstream boards should use the same default
  configuration.

* Leon: It's worth nothing that one can build a custom version of
  `load_processes` which does not perform these checks, but I'd be
  bothered to do so if the only thing is that I don't want to have
  these headers checked. Thus vouching for a parameter.

* Brad: I'd be more inclined to not have `load_processes` change, but
  refactoring the core functionality and having one which has all
  options, and one we generally use. In line with the general spirit
  of `load_processes` just being a helper function. We should expose
  one which has all the features.

* Hudson: I agree. I think it's good to be able to avoid performing
  these checks without duplicating the entirety of `load_processes`.

* Alexandru: So the suggestion is to rename the current
  `load_processes` to `load_processes_advanced` and introduce a
  wrapper function that calls this one with some default arguments?

* Brad: Correct.

* Branden: Solves the "how do we add new configuration options to
  `load_processes` without affecting boards" issue for the future,
  which is great. Have we solved the issue of making the kernel fail
  for 1.x apps for inexperienced users, to understand why stuff is
  failing.

* Leon: Specifically for why stuff is failing, I think it might be
  worth writing up a document explaining exactly what to do and why
  this switch was necessary, and printing a link to that resource.

* Branden: For people who are out of tree, this will fail with the
  kernel. For people who are on mainline, it will still fail until we
  add checks to the Makefiles.

* Brad: We have to inform the users about new version
  requirements. `libtock-rs` also needs to be updated, but there it
  should be easier to determine what board we're compiling for, so
  it's easier to opt out of writing the headers there.

# Tock 2.0 testing

* Hudson: Created a plan for Tock 2.0 testing and asked people to sign
  up. Most boards have been signed up for, with a few exceptions.

  https://github.com/tock/tock/pull/2632 is merged. Last major
  externally visible change for Tock 2.0. Other than that, the kernel
  crate exports and smaller stuff.

* Brad: The kernel crate has been changed. Very little is now exported
  directly from the root module. It gives us a lot more structure and
  namespacing makes things easier.

  It allows us to make better grouping. We can break things up and
  export them under the same namespace. We do that for syscall and
  process currently.

  Some of the traits changed as well. In `platform`, we've split up
  the `Platform` trait into individual traits. For instance, the
  mapping of syscall numbers to capsules is its own trait now.

  We now have a `KernelResources` trait. This is the de facto way for
  a board to configure the kernel. Before, things were more
  scattered. Basically now, everything which is a configuration option
  goes to this one trait.

  Also, the `Chip` trait no longer provides the `Watchdog` and the
  `SchedulerTimer`. A chip not necessarily knows this information.

  It is a massive PR with essentially every file touched. At this
  point, most boards have been converted. Once the CI passes, we can
  hopefully merge it.

  There is still one outstanding name, currently called
  `SyscallDispatch`, but as it is really only a mapping we might
  change it to `SyscallDriverLookup`.

* Hudson: Do you know which boards are still remaining?

* Brad: All expect for the nRF and SAM4L. Essentially the newer
  boards.

  Porting is pretty mechanical.

* Leon: Would you say porting is also possible without access to the
  hardware?

* Brad: Yes. Only the `SyscallTimer` is tricky. One has to be careful
  that this is reflected in the board now.

* Leon: Great. If there is errors introduced we'd catch them during
  release testing?

* Brad: Yes, that is what I expect.

* Hudson: We should block starting release testing on this.

* Hudson: Alistair has been pinging OpenTitan folks to release a new
  bitstream. We might have lost access to the site where these
  artifacts are hosted.

* Johnathan: Will look into resolving this issue.

