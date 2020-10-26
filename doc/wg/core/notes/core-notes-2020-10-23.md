# Tock Core Notes 2020-10-23

## Attending

* Hudson Ayers
* Brad Campbell
* Branden Ghena
* Philip Levis
* Amit Levy
* Pat Pannuto
* Leon Schuermann
* Vadim Sukhomlinov
* Jonathan Van Why
* Alistair

## Updates

### Tock 2.0 system call interface

* Leon: Started implementation work on the 2.0 system call interface
  now that the general design is settled. Still room for some
  variation, but enough established to start writing code.

* Jonathan: Can that be rebased on [the fix for the OpenTitan boot rom
  download](https://github.com/tock/tock/pull/2170)?

* Phil: Yes. There has been some implementation work in the `tock-2.0`
  branch, but the goal is to do a clean rewrite based on the
  experiences of that. The rewrite will be based on HEAD.

  Leon is starting with `syscall.rs` and we are going to discuss the
  results on Sunday. When that is done, going to implement the
  architecture support where he is leading the cortex-m side and I
  take over the lead for RISC-V.

  When all the infrastructure has been established and the
  Driver-trait is changed we might start asking for help porting the
  capsules.

## Tock 1.6

* Brad: Merged every PR outstanding bugfix PR related to 1.6, one of
  which was [the MPU fix](https://github.com/tock/tock/pull/2146). The
  issue was that when a process restarted, it was getting its old MPU
  configuration from before it crashed, although the memory regions
  might have changed. As a result it might have access to regions it
  is not supposed to.

  Three ways to fix

  - cache the original MPU configuration that was set when the process
    was first created and reuse when the process restarts
  - recompute the MPU configuration when the process restarts
  - update the MPU trait to make it easier to decouple calculating the
    addresses from configuring the hardware

  Conclusion: memory overhead to cache not worth it, Hudson found a
  reasonable way to recompute the MPU configuration.

  Processes should now have the correct MPU configuration when they
  restart.

  Currently working on a [fix for the arty-e21
  PMP](https://github.com/tock/tock/pull/2173) which currently allows
  reading memory that it should not allow.

* Amit: What's the plan regarding 1.6? Wait for the fix of the PMP,
  tag a release candidate and test again?

* Brad: Test again - broadly or for the PMP?

  Feeling that not much more testing is going to happen, unless there
  are specific things to look at.

* Hudson: All of the fixes have been platform specific, except for the
  MPU. It does not seem worth it to do all of the release testing.

* Amit: We are comfortable with including the PMP fix and then tagging
  a release?

* Leon: Maybe Alistair can check whether the [64-bit wide alarm
  fix](https://github.com/tock/tock/pull/2164) solves the issue he
  also saw?

* Alistair: Yes, it fixed the issue.

* Brad: Submitted the PMP fix PR.

* Amit: Assuming everything works, 1.6 should then be released today?

* Brad: Yes.

* Leon: Can we integrate Zenodo prior to the release and merge the
  [papers PR](https://github.com/tock/tock/pull/2150) to be able to
  enable this? If that is merged I think there were no more concerns
  regarding the integration.

* Hudson: What is the action item for integrating Zenodo?

* Pat: It's in the settings of the GitHub repository. We should do
  that before we tag the release on GitHub: if it is enabled on
  GitHub, when you do create a release it syncs this action to Zenodo
  in the backend.

* Amit: I approved the PR, we can wait for the test to pass or if
  someone wants to disapprove.

## Tock 2.0

* Amit: Already previewed this in the updates, is there anything to
  augment?

* Phil: Leon just sent me an email with a question regarding a 64-bit
  system call ABI. People are starting to look into running Tock on
  64-bit platforms, but that would mean that the ABI changes.

  Question is whether it should be put in a single TRD or in two.


## Persistent Storage Identifiers

* Phil: Discussion of the OpenTitan call. Alistair has been looking
  into it.

* Alistair: Primary issue is that for a persistent key-value storage
  in nonvolatile memory, the idea is that an app wants to save data
  into a specific key into flash.  After potential kernel- and
  application-updates, the single app should still be able to access
  this data, but no one else.

  Original idea: introduce an app id, with which the kernel could
  track that a specific app id stored some data, which could be saved
  in flash as part of a header. Question: how to generate those ids,
  how to store them?

  Other idea: introduce permissions, where the TBF header would list
  permissions of capsules it wants access to, along with options. In
  the case of flash, the option could then include an identifier,
  which is used to persistently allocate storage regions to the
  app. It can also restrict access to other capsules, such as I2C.

* Leon: These ideas seem orthogonal to each other. Both an application
  id and permissions to access specific capsules are good ideas. There
  was a long [discussion regarding application ids over the mailing
  list](https://groups.google.com/forum/#!msg/tock-dev/aduN7fHWXdI/Lk3bmMVxBQAJ)
  and multiple times in the call.

  Your [PR on elf2tab](https://github.com/tock/elf2tab/pull/27) seems
  particularly important, as you there attempt to introduce the notion
  of an app id.

* Alistair: The problem with application ids is how we make them
  unique. Brad said tockloader could potentially generate them.

* Jonathan: In the previous discussions we did not conclude whether
  application ids must necessarily be unique. Are you referring to
  processes' binaries having different application ids and if so, what
  is the justification for needing them to be unique?

* Brad: What does unique mean?

* Leon: With Tock there is already the notion of an app id in the
  process. It is not an application id but much rather a process
  instance id as it changes when the process restarts. Those need to
  be unique in the kernel. However when we are referring to
  application ids for applications in flash, those ids do not
  necessarily need to be unique.

* Phil: The uniqueness should first of all protect against cases where
  an app is loaded to the kernel which happens to have the id of
  another app, which can then get access to its resources.  Once you
  have a namespace doing access control, you need some way to manage
  that namespace.

  Also, it does not make sense to tie this to an app. Much rather
  there should be a storage identifier, as there is no reason why two
  apps should not be able to do this.

  Having a 1-to-1 mapping between applications and storage resources
  is also problematic, for instance as it makes migration of storage
  from one app to another hard. In the migration case, a new app
  should be able to access the resources of both the old and new
  app. Limiting an app to a vertical silo of storage might be too
  restrictive.

* Leon: When we start to introduce ids that can be repeated multiple
  times, the term "identifier" is misleading. A better comparison
  might be Linux namespaces, controlling the resources you have access
  to, which would be associated by a namespace identifier, which
  multiple apps can be grouped under.

* Vadim: That is the intent of this. We want to introduce access
  control for some syscalls, such that one app can access resources,
  while another app can't, and to make that generic.

  I would like to avoid hardcoding ids to filter apps for syscalls. It
  might be better to have a manifest stating what kind of resources an
  app may use.

* Jonathan: Leon, if I remember correctly - given that this is a
  continuation of our discussion - an application id is that namespace
  concept. It may consist of multiple processes with multiple
  binaries, and that is why we differentiate between an application id
  and process (and it does not match the process struct currently).

* Leon: Yes, that sums up the discussion. There were multiple
  approaches each having a different set of arguments. The primary
  takeaway for me is that we need to differentiate between the notion
  of _verifiable ids_, containing cryptographic information and
  thereby identify an app on a security critical system, enabling
  features such as secure boot. This is different from the current
  discussion about application ids for resource allocation. If we
  limit ourselves to those ids, the summary is accurate.

* Vadim: So far we make the assumption that there is only one
  statically linked app and no loading of processes. If we load
  processes, this becomes more important because now we need to verify
  that a particular app is actually the app it is claiming to be.

  There should be an application id, it should be cryptographically
  signed and verified at some stage while loading the process. It can
  then be used to apply security policies to the application.

* Leon: We already had this discussion once. As far as I can remember
  we concluded that this is generally preferable and should be a
  supported mode of operation, it is not relevant for every board,
  especially not for the default ones in the Tock repository.

  Hence each developer should be given the option to load apps
  themselves (as they already can, by not using the default
  `load_processes` function) and perform cryptographic verification if
  they so desire. In the default case we should use simple ids which
  are persistent across reboots, small in size that they can be
  efficiently compared at runtime. You are free to extend these simple
  ids by generating them using hashes or other cryptographic measures.

* Jonathan: That is why in my proposal, the exact format of an
  application id was left for boards to define. Having a default that
  boards use is reasonable.

* Leon: If we introduce the notion of an application id for resource
  allocation, it would be good if every board, whether it's a default
  one or using custom verification mechanisms supports a common
  denominator of this id. This way capsules and peripherals can be
  written in a way to support resource allocation by means of these
  ids.

  I would therefore use a two-step process. Boards that require it use
  their cryptographic ids to derive a common id. This common id - not
  necessarily based on cryptographic processes - is then further used
  in capsules for persistent resource allocation.

* Vadim: This also means that one would need some hooks in the process
  loading mechanism such that boards can implement custom verification
  steps while loading processes.

* Leon: That already works by not using the default `load_processes`
  function.


* Phil: This discussion illustrates the challenge of the previous
  discussion on the OpenTitan call, that starting from the narrow case
  of Alistair's issue, it is very easy to expand into a general
  discussion about access control and security.

  On the one hand, coming down with a general, sound and expressive
  approach is important. However, the basic case should not be
  blocked, being that an app should persistently be able to get access
  to a nonvolatile storage region.

* Leon: I agree, which is why I am pushing for a common, simple app id
  in the kernel. Alistair has done a great PR for the default case. If
  there are more complex requirements, extending the `load_processes`
  function has no limits.

* Phil: Yes. I think it would be a big mistake to create a 1-to-1
  mapping between an application and a storage region.

* Leon: Does having the notion of resource groups help, where
  applications could be associated to multiple resource groups?

* Phil: Just having resources under a namespace (group) with apps
  being in a namespace would solve these issues.

* Alistair: An app could always add itself to the group, requesting
  resources from that namespace.

* Phil: It is stored in the TBF, so an app cannot modify the group
  allocations. Is your concern that someone can get apps on the board
  that are allocated to this group?

* Alistair: Yes. App ids are better, since as they are unique, a
  second app having the same id can not be added. It is a general
  problem.

* Jonathan: This discussion is about verified vs. unverified ids.

* Vadim: Isn't it sufficient to check whether the app id is unique
  during `load_processes` and if it is not, refuse to load that app.

* Phil: This then causes a denial of service. If the kernel is being
  safe, no app instance of a duplicate app id should run.

* Jonathan: It also provides no security. To prevent impersonation of
  apps, cryptographic verification is required.

* Phil: To ensure security, wherever the app id or storage namespace
  assignment is stored would need to be signed and verified, for
  instance the TBF headers.

* Alistair: The threat model says that TBF headers are not trusted.

* Phil: The kernel does not, but the applications might.

* Jonathan: If the kernel is allowed to cryptographically verify that
  the headers are correct, it might trust the TBF headers, but it
  would have to do that on each boot.


* Brad: Phil, decoupling application identifiers from storage
  associations, that would still require some application identifier?

* Phil: For different security reasons an app identifier makes
  sense. Coupling these things may not be a wise decision in the long
  run.

* Alistair: That is why I like the permissions-based approach more,
  where a TBF header would list the permissions an app would be
  granted.

* Phil: How would that limit the storage regions available to an app?

* Alistair: For the storage permission, the content would be "allowed
  to do the flash syscall and this is my group id".

* Leon: But that is very dynamic for each specific capsule. One might
  want a group number, the next might want to have two values, another
  might want none.

* Alistair: Yes, it would be up to each capsule to define what
  argument it wants.

* Phil: I think we want to both limit the syscalls of an app and limit
  storage groups, but not couple them for the reasons Leon
  mentioned. Otherwise it will get very complex.

* Leon: Integrating general purpose namespace groups might be
  easier. Each app is part of multiple namespaces, not bound to a
  specific capsule. It may then allocate resources under such a
  namespace.

* Alistair: But my approach does not exclusively apply to storage. For
  instance, access to UART could be limited as well.

* Phil: We have a mechanism for doing syscall filtering already. There
  have been discussions on how to specify which app is able to define
  permissions on a device granularity. One of Amits' student built a
  system for defining this.

* Leon: Still think having the notion of a generic resource group
  which applies to all capsules equally is conceptually simpler. Every
  process can be part of `n` resource groups, and can allocate
  resources to a specific group.

* Jonathan: If those should be cryptographically verified, that is
  expensive as different stakeholders "own" resource groups and
  therefore each resource group needs a different signature. Central
  permission lists (part of the kernel) are easier to verify.

* Phil: A good idea is to separate the mechanism from the
  verification.

* Jonathan: Separating the mechanism (giving a process access to a
  resource) from the verification is in line with my arguments.

* Vadim: In my use case the entire image would be verified as part of
  the boot process, so the app ids can be trusted.

* Leon: Just looking at the generic case for boards in the Tock
  repository, those load processes by use of `load_processes` which uses
  the TBF headers. A board such as OpenTitan might have its firmware
  delivered as a single binary that might not use TBF headers. The
  boards have the possibility to stop using TBFs and load processes
  using some other method. Hence those boards might bake the trusted
  information in the kernel.

  In the simple case for the boards in the Tock repository, those
  could use an array of resource groups in the header, which is simply
  not verified.

* Phil: Talking about loading and verifying is an implementation
  detail. Thinking about the interface raises questions such as

  - what would this mechanism mean for the userspace APIs / how would
    it change them
  - how would the kernel check that a process is allowed to do what it
    tries to do


* Amit: This conversation might be getting away from many. Maybe it
  would be good to continue and summarize the discussion on a future
  call?

* Brad: What came up in the OpenTitan call but not here is the
  distinction between a _global_ identifier vs. a _local_ identifier,
  where _global_ would be an identifier that is namespaced under an
  organisation and would not collide, whereas a locally unique
  identifier would be unique across a board.

  My proposal is to add a TLV, not claiming it solves every issue, and
  is inserted by tockloader which ensures it is locally unique. When
  it is updated, tockloader ensures that it does not change its
  identifier.

* Leon: What should be then to about the app id already in the process
  struct? Rename it to be a process identifier instead?

  Also, did we now come to a conclusion about the locally duplicate
  ids? In Alistair's current proposal, those would still exist.

* Brad: First question is just variable naming.

  Second question: the identifier would be unique across a board,
  which the loading tool (tockloader) would enforce.

* Phil: Why does it need to be unique?

* Brad: For our scheduling case it must be unique to be useful.

  Our use case is scheduling over multiple power cycles. Every process
  must need to be identified persistent across reboots.

* Leon: Could you just use the position of the process in the process
  array? That is already contained in `AppId` and would remain
  constant, assuming that the order of apps does not change.

* Brad: It would be possible to implement it this way, but we prefer
  not to rely on that.

* Phil: It's tricky to introduce an id which we are trying to use for
  many things simultaneously, where the actual requirements might be
  different.

  A simple proposal to go with the TLV that Alistair proposed,
  separating the TLV for the app identifier and the TLV for granting
  access to capsules. For now, apps only have a single storage access
  TLV. In the future it is possible that an app could have multiple
  storage TLVs.

* Brad: The notion of a single TLV with dynamic size already exists,
  so this could be used.

* Hudson: Calling out one of Jonathan's quote from the discussion in
  May: "I'm concerned that developing an application specifically for
  persistent storage will result in an application id design that is
  poorly suited for the other use cases, that would either hurt those
  use cases or result in us having multiple types of application id
  which would be confusing, which hurts security and would waste flash
  space."

* Jonathan: I'm happy to reiterate on my proposal. I see a few
  concerns, for instance that Leon wants a fixed size for the
  application id such that we can statically allocate storage
  locations for it. Not opposed to that, but given our cryptographic
  requirements, those ids might then be 512 bits.

* Leon: I would want a two-stage process where the size of the first
  id would be irrelevant and the second id derived from that would be
  small as to allow for efficient comparison and storage during
  runtime.

* Brad: I feel like we are jumping between issues. Phil is proposing
  not an app id, but a storage access id.

* Phil: Correct. And Hudson points out the dangers with this
  identified by Jonathan in the long thread. I would therefore
  withdraw my proposal and reread that thread first.

* Jonathan: I came up with this proposal after a core team call and
  though it would represent the consensus of that call. Leon had some
  concerns which appear to be valid. If we are going down that road
  again, I am willing to iterate upon that design, given that it is
  not far off from a workable solution. It is however specifically
  targeting application ids, not storage ids.

* Amit: With that we should probably end this discussion here and
  continue it on the mailing list or elsewhere.

* Leon: Sounds great. I will also think about my arguments in the
  thread and iterate over them.

