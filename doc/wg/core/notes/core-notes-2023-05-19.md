# Tock Core Notes 2023-05-19

 - Pat Pannuto
 - Amit Levy
 - Alexandru Radovici
 - Leon Schuermann
 - Brad Campbell
 - Branden Ghena
 - Caleb Stanford
 - Hudson Ayers
 - Johnathan Van Why
 - Tyler Potyondy

## Updates
 - (crickets)

## Agenda

### External Dependencies & Cargo Scan

 - Caleb: Off the bat, it's likely that cargo scan can't solve the current problem, but interesting to see if we can't solve in the future
 - Caleb: Working on supply chain security for Rust
 - Caleb: cargo-vet is a Mozilla tool that is currently usable / near-released. It lets you audit supply chain dependencies, but very manual. Line-by-line declaration of what's safe. Not a lot of enforcement that you're actually auditing anything, results in a 'looks good to me effect'.
 - Caleb: Trying to bring some static analysis into workflow to improve automation. i.e., in a big codebase, likely only need to look at a few critical sections, think syscall effects, unsafe, other edge case things
 - Caleb: Current state: We have a tool, it's usable, but it's focused on unsafety and side-effects. Rust, despite being safe, does allow side-effects.
 - Caleb: For your use case, it looks like interest in crypto libraries e.g. ghash. Where you'd need functional correctness. But we filter out that expected code.
 - Amit: That's a slight mischaracterization. Tock relies on Rust's memory/type safety for isolation within the kernel. Many not-fully-trusted things (capsules, ~= driver in linux) are compiled into the kernel. The promise we want to make is not that the driver is functionally correct, but rather that if they are buggy they won't corrupt the rest of the system.
 - Amit: If ghash is buggy or cryptographically leaky, that is not necessarily the kernel's concern, as long as there's no way for the library to say "leak user memory somewhere else" or "read HW stored secret (it doesn't have access to)"
 - Amit: We generally rely on Rust type safety for this. What's made us wary about third-party libraries in the past is that we thus have to check out all uses of unsafe rust
 - Caleb: That's interesting! Focus is not cryptographic assumptions?
 - Amit: It's sort of subtle, but yes, the core kernel doesn't rely on crypto as a core library or for security or correctness. Builds of the kernel, certain products, etc, may have that requirement. But that's for the board/platform owner; and they can already include things that include unsafe Rust etc. We want to be able to make strong guarantees about memory safety in the kernel
 - Amit: In particular, if there's a bunch of drivers useful across many boards, those are semi-trusted; they probably do the right thing; but you're not going to audit the complete driver behavior when you want to add a complex sensor. That's what we want the strong compile-time guarantees for.
 - Caleb: Compile-time means you're okay with auditing?
 - Amit: We're fine to audit. Current approach is the 'forbid unsafe' trait for these crates, plus some restrictions on available interfaces. What we don't have to date is ways of enforcing this on third-party / dependent crates.
 - Caleb: And that's because of how cargo works?
 - Amit: Yes.
 - Caleb: And the only thing you're looking at right now is ghash?
 - Hudson: Because of this limit on auditability / enforcement, we have been very resistant to any external crates. We've been using ghash as our motivating example to figure it out. In practice, that's manifest as our new [External Dependencies](https://github.com/tock/tock/blob/master/doc/ExternalDependencies.md) policy
 - Hudson: Goal in the long term would be able to do this for more down the line. E.g., we've thought about Rubble Bluetooth crate (though that's now dead?).
 - Caleb: Interesting... I think our tool may be more applicable than I thought. We are at early stages and are looking for real-world case studies. Once the tool is a little be more mature, we can try running it on Tock and seeing if we get useful results..
 - Caleb: I did a quick run on ghash. Ghash doesn't have any unsafety or side effects in the whole thing.
 - Caleb: I guess in that case are we worried about the board that's using ghash?
 - Amit: I believe that ghash that some of its optional dependencies may use unsafe. So really what we care about is 'for a particular instantiations that there is no unsafe'. I suppose if there's more precision in 'whether any of the unsafe is reachable' that's also interesting; but first-order is a grep of unsafe in the set of all dependencies actually instantiated.
 - Amit: The more interesting perhaps is the counterfactual. Ghash is being pulled in because it meets the criteria of it would likely be worse for us to reimplement or even vendor due to the subtlety of crypto. In the past, however, Hudson spent a huge amount of time reimplementing 6LowPAN and UDP and such in a way that was Tock-specific because we don't have a way of reasoning about whether a third-party dependency is using unsafe, or if a version bump starts to, etc. So ultimately, what _could_ we pull in and how could we allow developers to rely on third-party libraries in a way that we've forgone with great expense to date.
 - Amit: This may be less satisfying in the short term as running on Tock today will say no dependencies :)
 - Caleb: Clarification—are you interesting in auditing your own code or just dependencies
 - Amit: Our own code base is trusted.
 - Amit: I don't want to dismiss that if there are tools that could help us verify that our trusted code is actually trustworthy is valuable. But conceptually, that's the boundary
 - Amit: For dependencies, we don't really have any (except crates in the same repo)
 - Amit: The second thing, unclear if the tool does this or could, but it seems like it'd be nice to allow people to pull in a dependency that's a big useful crate that maybe uses unsafe, but the Tock code uses the dependency in such a way that the unsafe is never exercised. That seems useful. If the unsafe part doesn't affect the compiled artifact, that seems useful and cool
 - Caleb: We're close to having that tool. We're not quite there, but that's the plan and soon on the roadmap (to filter out parts of deps you don't use)
 - Hudson: This applies to transitive dependencies?
 - Caleb: Yes... we're also in the process of writing that :)
 - Caleb: Currently, it will work on a single crate a time, but we're working on transitive
 - Hudson: Is it an issue that Tock runs on embedded platforms / binaries? There are large parts of the Tock kernel you can't run on a Linux machine?
 - Caleb: Great question. Platform-specific code and build.rs and build-time / compile-time flags?
 - Hudson: Any of these... I might imagine runtime profiling of all the code that could be called as a way of seeing all code paths. Or it could be some symbolic execution thing, where you could ignore platform-specific stuff (e.g. just executing llvm ir, etc)
 - Caleb: It gets more complicated because of that, particularly because arbitrary stuff can happen at build time. I need determinism of the build. 
 - Hudson: Is this only a problem if people use build.rs?
 - Caleb: build.rs is the biggest problem, but also macros and side-effects at build time
 - Hudson: We don't use procedural macros at all
 - Hudson: And minimal use of build.rs
 - Pat: We're actually stronger here, we intentionally target deterministic, reproducible builds
 - Caleb: This is actually the best-case scenario for an auditing tool; you forbid a lot of what makes our life difficult. Also worried about platform-defines / conditional code
 - Pat: You're going to love us again. We generally forbid `cfg`
 - Caleb: Okay, this is sounding great. We can run on ghash and its transitive dependencies, but that will likely be a trivial audit... it sounds like more interested in the future case?
 - Hudson: I would say we are more interested in the future crate. But, it would be really nice to have confirmation that we don't touch any of ghash's unsafe
 - Branden: What would this look like one day.. a CI thing?
 - Hudson: In the medium-term, any time we update a dependency to a new version, this is a tool that we run. That can be CI-enforced.
 - Amit: Also depends on how expensive it is. In the Safe Haskell world, which is more rudimentary, it's just part of the compile process.
 - Caleb: We've thought about CI integration.. on the roadmap.. if that would be the most useful use case we can triage that up
 - Amit: On our side, it looks like there's essentially a command that you run. Seems pretty easy for us to just add a target to our Makefile that the CI calls. If it fails it fails, etc
 - Branden: We update dependencies rarely enough that manual is fine for now.
 - Amit: Could imagine cool ways to go from here.. if it's expensive that you really only want to run when-needed, could do some pass for a hash of the dependency tree, etc
 - Caleb: It's pretty lightweight. We're not doing any heavyweight symbolic execution or something that takes hours to run.
 - Caleb: But it does require human-in-the-loop to make decisions about what it finds and what you want to do with them
 - Amit: Is the invariant it finds nothing / or don't admit
 - Pat: Is it deterministic in what it reports? Can we mark something as verified / allowlisted / audited?
 - Caleb: That's exactly what we're working on right now. Goal is to be able to prove that existing audit is still valid for an updated crate.
 - Caleb: How can I be helpful from here?
 - Amit: That's a good question. One of us should find a student :)
 - Caleb: I'm doing a lot of development on cargo-scan, but finding a student would be good
 - Amit: It would be good to have someone more familiar with Tock working on this
 - Alexandru: I have some students who might want to this
 - Caleb: Happy to set up a more focused meeting with a student, have them reach out
 - Caleb: I don't think cargo-scan is mature enough to run on all your dep's at once
 - Caleb: Maybe we follow-up in a month, and we can establish whether it's useful
 - Alexandru: That works great here—we have a program that sponsors student open source over the summer, starts in one month
 - Caleb: That sounds great; happy to tailor development towards your use cases; looking for users
 - Amit: Is this something we fits the eventual POSE security role?
 - Pat: Absolutely, it's right in the heart of the 'infrastructure' we defined
 - Pat: Also, we have TockWorld July 26-28, it would be great if you can come
 - Caleb: Maybe... what is it?
 - Amit: Annual, in-person meetups / workshops with folks involved in Tock development and use. This time around we're looking to kick off bootstrapping of a sustainable open source ecosystem. Goal is gathering wants and desires from stakeholders; get buy-in from OSE commitments; in general more formal processes for forming, auditing, maintaining security guarantees very in scope
 - Caleb: Sounds good. Let's stay in touch. Especially if we can get a student involved, good opportunity for focused push and uptake.

## Outro?
 - None.
