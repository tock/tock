# Tock Meeting Notes 2024-09-20

## Attendees

- Alexandru Radovici
- Alyssa Haroldsen
- Amit Levy
- Ben Prevor
- Brad Campbell
- Branden Ghena
- Chris Frantz
- Hudson Ayers
- Johnathan Van Why
- Kat Watson
- Lawrence Esswood
- Leon Schuermann
- Pat Pannuto


## Updates
- Leon: Glueing together the GitHub integration to treadmill. I think we can get
  an initial workflow merged that can run a job on every PR. Things will break
  so I wouldn't make it a required check but we can go from there.
- Leon: I was at the open source firmware conference, and I spoke to lowRISC,
  and they're interested in treadmill for internal testing as well.
- Amit: There is a nasty libtock-c timer bug in the unwrapping logic that
  resulted in alarms getting lost. Sent a PR (libtock-c 466) with a fix. Would
  be good to create a test.

## VeeR EL2 PR (#4118) check-in
- Branden: Been here for a little bit. It's changing the obsolete Swerv into the
  new verilator simulation target veer el2. For the most part it's a rename plus
  there are lines of code changing in chip-specific places.
- Brad: Did they ever document it? They linked to a README but they delete their
  READMEs so it doesn't mean much.
- Branden: Yes, there is now documentation on how to run it and such.
- Amit: It hadn't been clear to us what the relationship betwene swerv and veer
  is. We were waiting to understand whether veer should be separate or replace
  it, and now it's been answered.
- Branden: There are chip-specific files like `io.rs`. That probably none of us are able to truly review since we don't know this chip. Are we okay with that as
  long as they're self-contained to the chip?
- Amit: I think so.
- Branden: That's what I think too, but I wanted to make sure everyone is
  on-board with it.

## External x86 crate dependency in PR #4171
- Amit: Would be good to have the authors on the call, but I discussed it with
  the authors and I think I can reflect their motivations reasonably.
- Brad: The context is the PR that adds x86 as an arch crate. The current
  implementation depends on an external x86 crate. It seemed to me it would be
  helpful to have this group generate a recommendation on what the right thing
  to do with this external dependency is. I haven't looked into how it is being
  used, it seems like there are some useful data structures and snippets they're
  pulling in.
- Amit: When I discussed this with the authors, this was one of their main
  concerns. They're aware they are frowned upon with us, but I encouraged them
  to put the PR up so we can have the discussion rather than delaying the PR.
  They included it for more-or-less expediency downstream. They're using it
  mainly for data structures — the crate contains data structures modelling x86
  peripherals. They're using relatively few now, but anticipate that if they do
  a 64-bit version -- that they may -- they'll end up using more (more recent
  x86 architectures use more fixed structures). It seems to me the options are
  that we can be okay with this particular crate. If it stays an external crate,
  it should probably remain in the architecture crate and be re-exported, to
  abstract away that it's an external crate. Then we'd consider it a trusted
  external crate. Another option is we could vendor the crate. Another option is
  we could reimplement the data structures. None of these is particularly
  complicated. Licensing works out, it is MIT-only but could probably ask the
  authors to license as Apache 2.0.
- Johnathan: I don't think we need to care about the exact license of vendored
  code as long as it's roughly similar to Apache-2/MIT.
- Alex: I could see this as opening a pandora's box. I could see many similar
  crates for RISC-V and ARM. Also the x86 crate has four or five external
  dependencies, which we would need to vendor too. My take is we can copy code
  into the architecture crate.
- Brad: I feel pretty strongly we shouldn't have the dependency.
- Amit: I want to remind people the historical reason we've been
  anti-external-dependencies is that as an upstream project, we are trying to
  keep our codebase easily-auditable. Even a vendored dependency is easier than
  an external dependency.
- Alex: It would be much easier to certify without the external dependencies.
- Hudson: In the past, we've limited external dependencies to boards so they can
  be excluded by out-of-tree boards. We can kindof do this with the arch crate.
- Leon: There is an assumption that users can use our code without modification.
- Pat: It's a smaller surface area and not currently painful to vendor, but the
  surface area will grow over time. In the longer term, we can develop tooling
  to help with auditing our dependencies.
- Amit: Lets compare to ghash which we previously accepted. For ghash, we
  created a separate capsule crate so people could exclude. It's also crypto,
  written by trustworthy people, and crypto code is very subtle and hard to fix.
- Amit: [Referring to chat] we maybe can't just copy code into the arch crate
  due to a license issue, so we may have to put it into its own crate or ask for
  it to be relicensed.
- Brad: What is our feedback for the PR?
- Amit: Copy the data structures we need into our repository.
- Alex: Just copy in, because if we re-implement them they'll end up being the
  same thing.
- Amit: My preference is to ask the maintainers for permission. Historically,
  there has not been pushback on it.
- Alex: I'll open an issue and ask.
- Brad: It's still a little unclear what we're doing.
- Amit: There's a couple versions. One is to copy in the crate but delete a lot
  of stuff. Another is to copy the data structures into the x86 arch crate that
  this PR introduces.
- Hudson: The crate itself has other dependencies. Are we confident that we
  won't need any of those when we extract what we're using? Will we need to
  vendor in the sub-dependencies.
- Amit: If it turns out they're used — they're definitely not needed — we can
  work around them.
- Hudson: I'm more concerned about the build dependencies.
- Leon: One argument about chopping up the crate is you lose the ability to
  automatically pull in upstream changes. Ultimately, what should factor in is
  how stable the code they're vendoring is. If we expect a lot of churn, that's
  an argument against it.
- Alex: I would say it's stable, the last commit was ten months ago.
- Amit: That is my assessment as well. It's not un-subtle to get right, but the
  likelihood of there being changes we would want to pull in that are nontrivial
  to reimplement ourselves is very low because of the nature of what the crate
  is.
- Leon: It would be super nice to be notified about changes that we want to pull
  in.
- Alex: Subscribing to the repo is enough. If they have one commit every 4-6
  months, that's fine.

## Supporting different compiler versions for CHERI
- Lawrence: The main issue with CHERI is the CHERI LLVM is very behind -- months
  to a year or two. That prevents us from progressing rustc to a recent version.
  Given how new Tock's official rustc version is, it's unlikely we will have
  that a rustc version new enough to build Tock. I've been hacking in unstable
  features that are stable on latest rustc but not on CHERI rustc. There are a
  couple places that need more work. I anticipate this getting worse over time
  as Tock moves to new compiler versions.
- Johnathan: I don't think the delta will grow -- Tock's reason for using a
  recent Rust is to try to run on stable Rust, and the number of unstable
  features we need is going down. I could see us having an non-latest-stable
  MSRV in the future. If CHERI remains 1-2 years behind, I think the gap will
  close.
- Amit: I think this would mean having cfgs in a bunch of places, that add
  unstable feature declarations and maybe a polyfill. Whether we're okay with
  that is the question.
- Brad: It sounds like there's three options. One is to not support CHERI
  upstream. Two is to hold back the Rust version we're using to minimize the
  gap. Three is to have a cfg polyfill thing that minimizes the gaps. I'm most
  supported of the cfg option. This sounds like a case where the cfgs are not
  ambiguous -- it's just "are you using CHERI or are you not". I don't like
  restricting how we can update the Rust compiler. I don't think we should
  refuse to support CHERI -- we should move forward and try new things.
- Lawrence: I also wanted to push a feature to allow board initialization at
  compile time, but it uses an unstable feature (const traits). It will still
  work on CHERI, and I can feature-gate it behind that. Are people happy with me
  pushing it?
- Pat: To clarify, that would end up as a cfg feature gate that lives in the
  feature board?
- Lawrence: That cfg would be in the kernel rust files.
- Pat: That makes it uglier.
- Lawrence: I could move most of it into the components, but some parts need to
  be in the kernel (making some functions const).
- Lawrence: It's not necessary for CHERI but it is the case that the CHERI demo
  board uses it. I'll send the PR and people can reject it, we don't need it.

## Supporting multiple copies of a syscall driver
- Chris: Currently there are a number of syscall drivers defined like
  I2C/console that do not have an instant number. That leaves the question of
  "which I2C peripheral am I talking to"? For example, OT has four UARTs and
  three I2C controllers. My recollection is the capsules themselves don't have
  an instant number, and they go to one peripheral on most other chips rather
  than allowing a multitude.
- Leon: PR #1735
- Chris: I took a fast look through that. The thing that follows on is, in
  libtock-c the driver number is a parameter, while libtock-rs encodes it as a
  struct generic const parameter. I don't know enough about your solution to
  speak one way on another about it, but based on how libtock-rs is set up,
  having dynamic driver numbers seems to be in conflict there.
- Leon: It's very deliberate that capsules are abstracting one single
  peripheral, and we don't unnecessarily virtualize within a capsule. The idea
  is definitely that you have multiple different capsules, and downstream boards
  can allocate different numbers for different capsules. We're lacking a way to
  determine which peripheral is using which driver number. I haven't thought
  about userspace, and I want to decide on the kernel interface first before we
  start modifying userspace.
- Chris: My initial idea is to statically allocate the driver numbers and use
  those static IDs in apps to control what we're talking to.
- Amit: The system call API has mostly been designed around exposing
  higher-level things, like temperature sensors, rather than lower-level things,
  like I2C. The bus interfaces are mostly for debugging or whatever, so changing
  them is not a big deal. Exposing a system call driver that exposes multiple
  UARTs would be reasonable, by e.g. having a different range of command numbers
  for different UARTs.
- Leon: We don't really have anything that does this upstream.
- Amit: GPIO and LED do this.
- Leon: It's hard to use the raw instance number exposed by the driver without
  the context of where it maps to the board.
- Chris: Yes.
- Leon: I think discoverability is important, either statically or at runtime.
