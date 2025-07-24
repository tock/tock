# Tock Meeting Notes 2025-03-26

## Attendees
 - Branden Ghena
 - Leon Schuermann
 - Johnathan Van Why
 - Brad Campbell
 - Alexandru Radovici
 - Kat Fox
 - Hudson Ayers
 - Viswajith Govinda Rajan


## Updates
### Personal Update
 * Johnathan: Moving to new opportunity, but planning to contribute on Tock still as a volunteer.
### x86 Support
 * Alex: Submitted a PR on the x86 port which passes clippy and doesn't depend on the x86 library. It seems to boot, although we haven't loaded apps yet (not quite sure how to). It's a branch off of what Microsoft did
 * Brad: Really exciting
 * Branden: Testing with QEMU?
 * Alex: Yes. Q35-486 with QEMU. It boots the kernel and starts the process console. I'll be pinging Bobby about app loading.
 * Alex: But zero external dependencies right now. Using tock-registers now, with local copy
### Network Working Group
 * Leon: We have all the necessary changes in the tock-ethernet-staging branch. We're going to make a PR soon that merges all of them into master. They've all previously been reviewed, so hopefully this is uncontroversial
 * Leon: Also big thanks to Pat for making the userspace PR work via edits to the Make system
 * Branden: We also discussed two items for broader Tock opinions.
 * Branden: The first, which I believe we'll discuss at the Strategy Workshop, is IPC. As much of networking implementations has been pushed to userspace, inter-process communication is becoming a big issue for Networking support in Tock.
 * Branden: The second, is a question as to how to direct our attention within the kernel. We want to move interfaces into the kernel from userspace where possible for performance, composability, and just because that's how Tock tends to work. However, there are multiple approaches. One is to rewrite in Rust, which tools like PacketBuffer exist to support. That's hard because rewriting is a tremendous effort. Another direction is encapsulated C code in the kernel. This works in a research experiment setting, which is to say it's not at all ready for immediate use in Tock. It would be a large engineering effort to get it working. So given limited effort, figuring out a direction for kernel networking seems like an item the rest of the group could have opinions on.


## CI Failures
 * Leon: Discussed with Brad offline before the call. Failures are not blocking merging right now. I'm investigating what's going on with LiteX


## Software-Based ECDSA Support
 * https://github.com/tock/tock/pull/4372
 * Brad: Implements signatures checking with a particular ECDSA variant
 * Brad: Thanks to Alistair, it turns out we had the infrastructure for this already. It just adds support for boards without hardware support
 * Branden: How is this implemented? A dependency?
 * Brad: A capsule crate, which has an external dependency. So any board that wants to use it can pull in the capsule crate.
 * Leon: This is cool. I think exactly how we intended this to work
 * Brad: There's been no movement on the PR. So I wanted to bring it up
 * Hudson: For the external dependency, I see we pinned a version, 0.13.0. In theory that could change, right? They could point it at a different hash?
 * Leon: I think this would also include any version above 0.13.0. So it could handle 0.13.1
 * Hudson: Is that something we talked about as part of the dependency discussion?
 * Leon: I think so. It's part of the lack of stability guarantees for capsules which have external dependencies. Probably worth checking on that
 * Hudson: Seems like it wouldn't be crazy to require a git hash instead as part of avoiding compromises in the software chain. That would also make sure that people would always get the version we actually tested. I'll look into this
 * Leon: I'm pretty sure this implies semver compatible versions could be pulled in. So if you had 1.0.0, then 1.0.1 would also work. Is that a problem?
 * Leon: The way this usually works in other projects is that you would pin these using cargo.lock, so it would require a cargo lock file update to change versions. So this is more of an issue for us because we don't use lock files
 * Brad: The question I would have is, if a board maintainer wanted to be sure of an exact dependency, does that require anything more than cargo.lock? Or do we need to change something here? I think the goal is that _if_ they want to be sure, they can be.
 * Leon: Cargo.lock is enough. In fact, even if you point at a git hash, that itself will have a cargo.toml which could point at semver compatible things. So it wouldn't work transitively. A cargo.lock is the only way to have a transitive guarantee
 * Hudson: Good point. So it's not necessary for us to require git hashes, since it doesn't fix secondary dependencies.
 * Branden: So the proposal is not for us to commit a cargo.lock, it's for developers to use it if they want to
 * Leon: I'm not proposing either way
 * Hudson: Is it possible to have a cargo.lock just for one dependency without having them for the whole build?
 * Johnathan: No
 * Branden: So that would be a big sweeping change that we probably don't want
 * Leon: Definitely would require revisiting policies
 * Branden: If the concern is making sure others can replicate a setup which worked for us, maybe it's sufficient to have a README with documentation on which version was used when testing for all transitive dependencies
 * Hudson: This PR actually has that already
 * Brad: Confirmed
 * Hudson: Amit also added a comment with a cargo check to show that there's no unsafe in this. It would be cool to run that in CI in case the transitive dependency changed in the future. Not necessary right now though
 * Branden: So the path forward is to get reviews on this.


## Dynamic Process Loading
 * Brad: Where we left off was having an implementation and a TRD. Pat wanted something else from the TRD, but the request is unclear. So I don't know what's wrong with the TRD on dynamic process loading right now.
 * Brad: In the meantime, we also discussed that we could move forward with the implementation. But that has just sat. So my question is what the process is going forward.
 * Branden: I think this just needs developer time to get eyes on it. It is sometimes hard to to tell when PRs like this are good to go or being worked on.
 * Brad: Understandable, but the discussion last time made it seem like this was going to move. And then it didn't
 * Viswajith: We also need this for the tutorial, so it's becoming higher priority
 * Leon: Tested on an nRF?
 * Viswajith: nRF DK and Microbit.
 * Leon: So RISC-V is not supported right now. And that's deliberate, right?
 * Viswajith: Yes. Wanted to get one version of this out there to start. RISC-V has alignment rules that complicate things
 * Leon: Makes sense. Just trying to understand current feature set, not proposing adding things
 * Brad: Related, can someone approve and merge https://github.com/tock/tock/pull/4258
 * Brad: I wanted to push for things while we're here
 * Leon: I think we're good to merge that one. Shouldn't have an issue with the CI problem we discussed previously

