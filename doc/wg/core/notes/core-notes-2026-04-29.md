# Tock Meeting Notes 2026-04-29

## Attendees
 - Branden Ghena
 - Leon Schuermann
 - Johnathan Van Why
 - Amit Levy
 - Pat Pannuto
 - Alexandru Radovici
 - Guests
    - Emily Zhang
    - Mae Milano


## Updates
### DMASlice
 * Leon: DMASlice was merged today, which is huge. Talking with Johnathan about a potential Tock registers integration, which wouldn't be in the first implementation, but could be done with a macro wrapping registers. Should ideally allow us to tackle DMA issues across our drivers.
 * Johnathan: DMA is still on my to-do list, but at the bottom. So it's not totally out for the first implementation, but unlikely
### Tockworld Europe
 * Amit: Tockworld Europe is happening in Bucharest on June 25/26. More centered on safety-critical applications. Automotive and other non-secure-root-of-trust participants. Some good talks. Alex and Amit will be there, potentially Brad as well. More details to come soon
 * Alex: That date was the best for us and for companies. I'll send details in coming weeks!
### svd2regs Lock File
 * Branden: Dependabot PR that bumps a dependency of svd2regs. https://github.com/tock/tock/pull/4806
 * Branden: But actually what I wanted to check on is that this PR adds a lock file to svd2regs. So there are two lock files in Tock now. I wanted to see if anyone had thoughts or opinions on this.
 * Leon: It didn't actually add the Lock file, just update it. So the update was uncontroversial at least
 * Branden: Oh, I was confused, I see. Mostly I wanted to see if anyone cared about the lock file existing. Let me know if you do. (no comments)


## Eliminating Grant Reentrancy Bugs
 * Leon: Emily is an undergrad at Princeton looking into an issue in Tock. We move lots of checks to compile time, but one issue has been reentrancy in grants. It used to be unsound to double-enter them, so we added a runtime panic. But now double-entering a grant crashes the kernel. So Emily has been working on finding potential issues at compile time instead.
 * Emily: Advised by Professor Mae Milano and Leon.
 * Presentation is given. Link here: https://drive.google.com/file/d/1Xmv6C1nhPuSY5KqkBjpismUQGFacPXqJ/view?usp=sharing
 * Some high-level notes on what Emily discussed:
     * Dynamic runtime checks crash the kernel which is problematic
     * Reentrancy in grants is one such issue, as it would provide two aliasing mutable references, but instead has a runtime panic.
     * Deferred calls are the way to do this instead, which resets the call stack and removes reentrancy
     * Goal: statically detect reentrancy bugs
     * Approach, use Rust's affine type system. Use a &mut Token to mutably borrow, returning access to the thing. Now if reentrancy were to occur, it would need a second token, which it can't have, so that's a compile-time issue.
     * Implementation uses MIR dump to instrument source by adding affine token references
     * Preliminary results: generating Control Flow Graphs from MIR works so far. Proof-of-concept testing
     * Tested with four examples: buggy, correct, compile-time unreachable false positive, runtime unreachable false positive
     * Future steps: automate full pipeline of MIR -> CFG -> Paths -> Instrumentation. Also handle false positives
     * Q&A
        * Branden: is the vision here to add instrumentation just for a test? Or would the instrumentation go in the code-base long-term so we'd notice issues?
        * Emily: We think it would be self-contained. So something like a CI pass would notice it
        * Johnathan: Did you look at the Cargo call-path tool? It's similar, but kind-of unmaintained
        * Emily: We did look into that. But first it can't generate line number provenance for instrumentation. It also said it didn't have completely correct handling of dynamic dispatch or function pointers. And deferred calls use those. Plus it's stuck on the 2021 nightly.
        * Johnathan: Can MIR do dynamic dispatch?
        * Emily: For our tool, we only need to draw edges for each potential execution path. We sort-of brute force it.


## External Dependencies in Tock Registers
 * https://github.com/tock/tock/pull/4814
 * Johnathan: Tock registers has been a long-time in progress. I was trying to do things without Proc macros, but I found things were getting complex enough that proc macros are needed. And making things sound and testable is just not feasible with normal macros. They're nontrivial complex proc macros.
 * Johnathan: Unfortunately in Rust, the only way to write nontrivial proc macros requires some external crates: syn, quote, proc-macro2, and unicode-ident. They're not even maintained by Rust proper, they're an individual author. But they're very commonly used and I don't see a way around them.
 * Johnathan: So this PR proposes that Tock registers and only Tock registers can rely on those.
 * Johnathan: I expect some pushback, and there is a theoretical world where we don't use those crates, but it's a huge development and maintenance effort.
 * Amit: These crates are highly standard to use. One bummer, an artifact of how cargo is structured, is that even though these crates are only used during compilation, they need to be declared as regular dependencies. Unlike test dependencies, which are only available when compiling and running tests. So that means they really do taint the perceived surface area of runtime dependencies, even though they're only used to generate code at compile time. Which could in principle be audited separately
 * Amit: To me, the question is how would this impact, potentially negatively, cases where we might care about certification?
 * Alex: Twofold issue. Dependency is scoped, and I think we can argue that's necessary. And it's not used throughout the code at runtime, which is great. The challenge is certifying the code generated by the proc macro. Can we expand it with cargo expand?
 * Johnathan: Yes
 * Alex: We would need to access that generated code and certify that.
 * Johnathan: These is an asterisk on that. Cargo expand output doesn't compile on a stable compiler.
 * Amit: That's unrelated to the dependencies though.
 * Alex: It is easier to certify code without proc macros. But I understand the challenge
 * Leon: If we could have a build of Tock where the register macros are replaced with expanded versions, would that help?
 * Alex: Yes, we'd certify that. We certify out of context, and tock registers is mostly used in low-level crates. But we could certify the expanded code for sure
 * Pat: Does it make sense to consider something of a two-pass compilation? Tock generates register definitions first, the compiles itself without the dependencies? It could be a two-stage thing.
 * Alex: That would help. Still complicated because I expect it to be a huge amount of code. But showing samples of what this does would help. We could show that it can be inspected.
 * Johnathan:
 * Alex: Grain of salt: no one has ever done this before
 * Pat: Other people do svd to registers. Do they certify that?
 * Alex: Usually you buy certified registers from someone else. They might have dumped it before certifying.
 * Pat: So do we need tock registers to be built in one unified build with the rest of the compilation? Or can it be a tool for transforming code separately then build later
 * Amit: Is that practical Johnathan?
 * Johnathan: We'd add a third crate which is a library crate which the macro crate invokes that does the parsing and code generation. Then that tool could input register definitions and output Rust code.
 * Leon: That doesn't have to be the default workflow in Tock. The spirit of the external dependencies policy is to not dig ourselves into a hole where we rely on something and can't remove it. Validating that this workflow works, we could have convenience but mechanically remove those crates as needed.
 * Johnathan: By default you'd use Tock registers as a proc macro. Other users could use it as a tools instead. Some feature flag flipped off.
 * Branden: We'd still be locking ourselves into these external crates by using tock registers though
 * Amit: Yes. This is acceptable if we can convince ourselves that these specific crates are endemic enough to the Rust ecosystem that they may as well be core. In terms of stability of relying on them.
 * Pat: Johnathan mentioned that Rust core should adopt this. Does anyone know why they haven't?
 * Amit: I think there's a general line of thought to not crowd out ecosystem projects.
 * Branden: I think a lot of people don't care about dependencies like we do either
 * Leon: I do think this is the only reasonable option. But I also do think these crates OUGHT to be part of the core part of Rust. Rust really ought to be including these.
 * Alex: Good luck. It would be a lot of work
 * Leon: Yeah, unlikely to change.
 * Amit: For what it's worth, the author of these, David Tolnay, is part of the Rust library team. It's not a totally random person.
 * Amit: So, do we think the addition to the external dependency policies in this PR is acceptable?
 * Leon: I say yes. I would feel significantly more confident if we explored the option of having ahead-of-time expansion. So if these don't exist we could still have saved hard-coded registers previously generated.
 * Johnathan: We'd have to change this writeup though.
 * Amit: We could merge this now and then update the writeup again later.
 * Amit: Thoughts on this PR
 * Pat: Yes
 * Alex: Yes
 * Branden: I won't stand in the way. I'm undecided
 * Johnathan: Unhappy yes from me
 * Amit: Need feedback from other Core members too. Also probably raise with other stakeholders downstream


## Pluggable Virtualizer Device
 * https://github.com/tock/tock/pull/4802
 * Alex: PR we sent. Issue is that virtualizers, especially bus virtualizers, tend to implement list from first client to last. So if the first client has data, that's the only client ever served. So for example, if the kernel calls debug enough, then the capsule printouts never print.
 * Alex: We wanted to change this to Round Robin. But we thought this still might not be enough. What if we want to prioritize some clients. For example, a high-priority brake system for a car. So instead we thought to make this a pluggable policy.
 * Alex: The trait implementation is left to the board. We provided two policies. The default now is still client-ordered like today. We also added Round Robin.
 * Alex: We could change Round Robin to be the default for today. But now if someone wants to customize they could add a trait.
 * Alex: This is only for UART for now, but after we agree we'll add to the others.
 * Branden: Awesome. I support Round Robin for sure as default. I think our security policy says that's what we should do
 * Amit: Yeah. I also think making it pluggable is a good idea, just like we did with the scheduler.
 * Amit: So the decision here is whether to do Round Robin by default. We could do that without iterating on exactly what the trait looks like.
 * Amit: My opinion here is that Round Robin is uncontroversial and good. So it makes sense to do that and get it out of the way. I am also in favor of a more pluggable system, but separating the two PRs is probably the fastest way to get an improvement.
 * Alex: Alright. So we can send another PR changing all bus virtualizers to do Round Robin. Then a separate PR with a pluggable policy.
 * Alex: Sending the PR is easy. Where do we discuss what the trait looks like?
 * Amit: On this PR is fine.
 * Alex: One note: downstream users will see a change with Round Robin. If someone relied on this, it will break. Print output or SPI messages could be reordered if there are multiple clients.




