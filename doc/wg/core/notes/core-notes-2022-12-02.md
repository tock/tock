# Tock Core Notes 2022-12-02

Attendees:
- Hudson Ayers
- Brad Campbell
- Chris Frantz
- Philip Levis
- Amit Levy
- Pat Pannuto
- Alexandru Radovici
- Jett Rink
- Leon Schuermann
- Vadim Sukhomlinov
- Johnathan Van Why

## Updates

* Phil: In touch with another group of Google. They have some upcoming
  PRs w.r.t. MapCell unsafety and some new use-cases which might
  motivate changes to grants and Tock processes. We are spinning up a
  detailed technical discussion about it. Hopefully we have something
  up by end of month.

* Johnathan: Started on the license header checking tool.

* Leon: Pushed along [the PR](https://github.com/tock/tock/pull/3110)
  to make VirtIO devices work on the QEMU board. It's an old PR, so
  it's no longer on the front page. But if people are able to take a
  look, I'd appreciate that.

  This is one of the key components towards establishing Ethernet
  support on Tock. This establishes support for a virtual RNG which we
  can use to test this part of Tock's infrastructure in CI. It also
  adds a virtual network card, similar to how Linux in QEMU
  communicates over the network. One of the few devices we'd use to
  define an Ethernet HIL ultimately.

* Hudson: Opened [a PR](https://github.com/tock/tock/pull/3336) that
  fixes that `tock-registers`' `matches_any` does not work for
  multi-bit fields. The existing interface does not actually make it
  possible to implement it that way, as it works by adding up many
  individual field values, thus losing information of which fields
  were combined to create this value. Created a new function with a
  slightly different interface, and renamed the old function to
  `any_matching_bits_set` (as it's more efficient for the case where
  we match onto a series of bits).

* Brad: Working on an `elf2tab` update basically rewriting its
  logic. The last change tries to get rid of all the custom logic we
  were doing, and instead parse an ELF properly.

* Phil: Poke
  [on Alistair's PR](https://github.com/tock/tock/pull/3312) on
  external dependencies.

  *added to call agenda*

* Hudson: Wanted to mention the copyright PR. Came to some good
  conclusions, just need to incorporate the feedback.

## Policy on External Dependencies (PR #3312)

* Hudson: Let's talk about the external dependencies [PR
  #3312](https://github.com/tock/tock/pull/3312).

* Brad: Talked about this on the OpenTitan call. Want to have external
  dependencies in some controlled way. Where the PR currently stands
  is to start with a narrow use-case. Essentially limit to
  cryptography dependencies at first, and explain the process for
  review, as well as precisely specify which kind of cryptography
  libraries we'd like to support at first.

  The goal is not to over-specify things, but rather clearly indicate
  to readers our intentions.

  Another thing to specify is how we'd like external dependencies to
  be included in Tock. My proposal is: we have a single global crate
  in the Tock repository. That crate is the only crate which is
  allowed to have external dependencies (with the exception of board
  crates). Each crate which would like to use external dependencies
  would depend on this crate. The reason for this is that it provides
  us an explicit namespace indicating where things are coming.

  This is motivated by changes in the new Rust edition, which no
  longer requires the `extern crate` specifiers, so it may not
  necessarily be clear where a module, e.g. `ghash`, originates
  from. With an explicit crate, we could have our external imports be
  called e.g., `tock_extern::ghash`.

* Hudson: Sounds good. One outstanding idea: may capsules depend on
  directly on external dependencies?

  Alistair's PR adds `ghash` not for app signing, but for the AES
  capsule. Because its a dependency of the capsules crate, every board
  has to fetch and compile all of the `ghash` crate and all of its
  transitive dependencies.

  Curious whether users of Tock may have a problem with that in terms
  of having to audit that code, because it gets compiled along with
  the capsules crate, but not necessarily used? Specifically
  concerning procedural macros or `build.rs` scripts.

* Amit: The task of auditing is made more complex. If there aren't any
  external dependencies, then it is pretty clear that none need to be
  audited. If some dependencies are pulled in but not used, that makes
  it significantly harder to identify whether a dependency requires
  auditing: for example, a capsule could not use that dependency, but
  rely on another capsule which uses it, or re-export it under a less
  obvious name.

* Leon: Is this relating back to the discussion of whether capsules
  should be a single crate? I believe in previous iterations on this
  we didn't come to a conclusion given we didn't have any obvious
  benefits to it, but now there clearly would be some?

* Hudson: Had been advocating for a new external dependencies capsules
  folder, and any capsules that wanted to use external dependencies
  would live in that folder, which would be a separate crate.

  Leon, you're right, having different capsules be separate crates
  would also solve this issue. And anybody wanting to use only a given
  capsule would only need to include it and its explicit dependencies
  in their boards.

* Hudson: Can you think of any reason that would make it challenging
  to have every capsule be in a separate crate?

* Leon: Potentially recursive dependencies. Also, it's hard to define
  what a single capsule is. I think it'd be fair to do a bit of
  grouping.

* Amit: Yes, the network stuff should probably be one crate, or the
  core capsules including console and time. We could then also apply
  different standards for different crates (e.g., allow external
  dependencies only for some, never for the core crates).

* Leon: Related, is the current state of the document still specifying
  that the core kernel can only utilize external dependencies through
  traits which we define, or can it now directly use external
  dependencies.

* Hudson: Document has moved away from that, can now directly
  depend. There are some software-engineering challenges related to
  that.

  For example, tried to do this for just the `ghash` crate. You can
  define a trait which is exactly the interface of this crate. Also
  kind of tricky, as this crate uses its custom types, which you then
  need to pass back in to other functions. Calling macros is also not
  really possible.

* Amit: This doesn't really seem like the right approach. We don't
  want to expose `ghash` for `ghash`'s sake, but to use it for some
  purpose, e.g. validating app header signatures. It seems like that
  should be the functionality we should abstract over.

  Maybe there's some boards which can do that in hardware, other don't
  want to do it, and some want to use a software implementation, which
  internally uses `ghash`.

  We'd be asking Alistair to rewrite a lot of his code then. In
  general, there is an intended way to use these dependencies, and
  we'd be creating this whole additional layer to be able to use
  them. There's a non-zero cost here.

* Amit: Also a non-zero cost with including dependencies in the kernel
  crate.

* Brad: Voicing support for not having traits, which is essentially
  doing HILs for external dependencies. It's not the right place to
  put an abstraction.

* Amit: Not HILs for external dependencies, but for functionality that
  may or may not be exposed by hardware on different platforms. Re
  AES: we probably only ever want to use an AES implementation through
  a common abstraction.

* Brad: That's correct, but that's specific to dependencies which
  implement algorithms such as AES, can't generalize this to all
  motivations to use external dependencies.

* Amit: Question is -- do we have a use-case for which this model does
  not fit the model of Alistair's PR?

* Brad: Perhaps app signing?

* Leon: Alistair's PR seems very close to how we'd like to integrate
  external dependencies, by having it back an interface we already
  defined. Totally different question when app signing is blocked on
  calling out to specific external dependencies or just having some
  external crypto library supported, on boards which don't implement
  this in hardware.

* Hudson: Including `ghash` and putting it behind our current `digest`
  HIL seems tricky. Would likely require us to change that HIL
  substantially.

* Leon: Right, but this still seems pretty close to ideal. We're
  providing some interface to other parts of the kernel and userspace
  which is backed in part by our custom code and in part by
  `ghash`. From the user's perspective it doesn't matter what
  implementation we use underneath, as long as our provided interface
  remains compatible. In Alistair's PR, we can very well replace the
  usage of `ghash` by implementing AES-GCM by simply using another
  library or writing the code ourselves, there's no hard in-kernel
  dependence on `ghash`.

* Hudson: Agreed, if it's only used for AES-GCM. But say I had a board
  with a hardware hash implementation, there is no way to build an AES
  GCM on top of that hardware hash implementation.

* Leon: There is, we'd just need to rewrite large parts of the AES-GCM
  implementation, specifically the parts which use `ghash` right
  now. The fact is that we aren't going to be in the situation where
  we can't remove `ghash` without rewriting parts of the kernel crate,
  as it doesn't directly depend on it. It's not easy or elegant, but
  doable.

* Brad: But the document in the current form would allow the kernel to
  directly depend on a namespaced external crate.

* Leon: Yes, makes me uneasy. The PR I'm seeing right doesn't motivate
  this, and so I don't think we have sufficient proof that this is
  indeed necessary.

* Brad: Which crate then is allowed to use external dependencies?

* Amit: If we have one capsules crate, then only boards and chips. If
  we have many capsules crates, then I'd be fine with relaxing this
  for some crates, e.g., crypto-related capsules as they require
  implementations of algorithms that are not prevalent in
  hardware. Never depend on external crates for core capsules, which
  are things that virtually every board has to depend on, and
  virtualizer capsules.

* Hudson: I'm in favor of what Amit described. This seems like the
  best route forward here and resolves the majority of my concerns
  still on the document.

* Amit: This is morally equivalent to external dependencies in
  specific board crates, except that these components may be useful to
  multiple boards and are hence located in a shared crate.

* Leon: Really like the division of core capsules (without which no
  Tock board can really be useful) along with virtualizers, and then
  other crates which we can partition in arbitrary ways and which may
  well pull in external dependencies.

* Hudson: Seems as good a reason as any to split up capsules.

* Brad: Proposal seems to be that we add a list of crates which are
  allowed to have external dependencies?

* Amit: Rather a list of crates which aren't allowed. There's still a
  case-by-case evaluation per external dependencies, where some would
  probably have more push-back than others. If there's some external
  crate useful for debugging added to a debugging capsules crate, only
  used on boards which need it while debugging, that seems totally
  fine.

* Brad: Are these crates then directly including external
  dependencies?

* Amit: When we're breaking up capsules into multiple crates, it's
  better to have them be depended on directly. There's less reasons
  for this additional indirection, and there's a difference between
  pulling in _an_ external dependency compared to many.

* Brad: Makes sense. By having the core kernel crate not be allowed to
  add any external dependencies, one of the major needs for
  namespacing is no longer there.

* *all* list of crates disallowed to have any external dependencies
  (at least initially):
  - `kernel`
  - core capsules (including virtualizers)
  - libraries
  - arch

* Hudson: refactoring of capsules crate should go in before this
  document is added. Possibly could take a stab at this.

  Reasonable to have a standalone crate for every capsule except for
  core, and when there's direct dependencies between capsules?

* Brad: That seems not great. That's a lot of extra crates to add to
  each board and a bunch of extra files.

* Amit: This would be one additional directory hierarchy. My intuition
  is to have ~10 categories of capsules, such as `core`, `net`,
  `crypto`, `com`, etc.

  We can also start with pulling out the one capsule that wants an
  external dependency.

* Leon: Minimum structure we'd want to have is probably the `core`
  capsules crate, and then a large `contrib` capsules for everything
  else which does not introduce external dependencies, and then start
  breaking up gradually per dependency added.

* Hudson: Yes, would probably try to implement this structure
  first. This seems to let us move forward on the external
  dependencies issue the quickest.
