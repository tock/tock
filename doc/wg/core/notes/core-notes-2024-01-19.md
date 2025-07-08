# Tock Meeting Notes 1/19/24

## Attendees
- Alyssa Haroldsen
- Alex Radovici
- Amit Levy
- Andrew Imwalle
- Brad Campbell
- Hudson Ayers
- Johnathan Van Why
- Leon Schuermann
- Pat Pannuto
- Tyler Potyondy

## Updates:

- Alex: We started working on the multiplexing of the console. It looks promising.

## [https://github.com/tock/tock/pull/3772] - Signature Credential Checking
- Brad: This adds a structure for checking with signatures for app credentials. Implementing the logic needed for this to check signatures using the signature HIL interface.
- Brad: Once we can validate the cryptographic signature, then all that code has to do is implement the HIL. This will do all the app checking and is just a few lines in `main.rs`.
- Hudson: This is the more generic version of what you submitted a while ago for the RSA.
- Brad: The RSA would plug into this (would not require RSA support).
- Hudson: This would be great for Phil to weigh in on. It connects with his other app signing infrastructure. 
- Hudson: Anything in particular you want people to pay attention to?
- Brad: My hope is to bring this to people's attention since there are no comments on the PR.
- Hudson: Call to everyone, please take a look at this.

## [https://github.com/tock/tock/pull/3782] - Shared `build.rs`
- Hudson: This has request changes from Brad that were later updated to an approve. 
- Brad: I marked this as significant. The actual code changes are minor, but the conceptual changes are significant. It affects how platforms are setup and how they are built. We only have two core team approvals.
- Leon: Having read through this, I think this is a good change. We need to make changes to the board crates if you want to have an out of tree board. This is a relative minor change and I think is a good starting place. One drawback is this does change something for downstream users.
- Brad: I don't think this changes much for downstream users. You can just your own `build.rs`.
- Leon: This changes something for when you want to take a board out of tree. This is one more thing that needs to be changed.
- Hudson: Anyone with an out of tree board already has their own `build.rs` file so this would not create issues unless they link to this which I think is unlikely.

## [https://github.com/tock/tock/pull/3785] - Key Value Syscall Documentation  
- Hudson: Brad went through and added documentation for syscalls. Alistair signed off [check spelling].
- Brad: This is part of my larger series of updates for updating documentation.

## [https://github.com/tock/tock/pull/3791] - Fix Calculation for Subslices 
- Brad: I was trying to use subslices. Currently, if you use subslices and adjust the start of the slice but keep the end of the slice unspecified (go until the end), it only works the first time. You get an invalid slice and it appears your code is broken when in actuality, the subslice is not doing the right thing. 
- Alyssa: Are there unit tests we could copy from the standard library to exhaustively check for correctness?
- Hudson: I imagine there is something somewhere we can adapt.
- Alyssa: More unit testing would catch this and be useful for edge cases.
- Brad: What does that mean for this PR?
- Hudson: I think we should merge.
- Alyssa: Are there unit tests for this PR? If not, please add some.
- Hudson: Currently there are not.
- Alyssa: We should add this.
- Hudson: Should we block the PR over this? 
- Alyssa: I would not say we need to block this, but this would be an important followup.
- Alyssa: Unchecked pointer manipulation math should always be heavily tested in my opinion.
- Hudson: For what it's worth, there is not an `unsafe`in this file. We are only manipulating within an already allocated slice.
- Alyssa: So does this mean it could not go out of bounds? What if we added too much and that is unchecked?
- Amit: I think it will result in a panic.
- Hudson: Errors in this file on its own could not cause something unsound, just an out of bounds panic.
- Alyssa: True, but it is reasonable for code to assume that the implementation of subslice is unchecked.
- Leon: This is particularly scary for code that interacts with DMA peripherals with subslice.

## [https://github.com/tock/tock/pull/3793] - TBF Footer Return Error
- Brad: This is a mistake in the TBF parsing library. Where we put internal error and it really should have been bad TLV. 
- Brad: If you do hit this error, it seems like there is a bug in the code when in actuality it is possible to trigger this error because you created a bad TBF. 
- Brad: I think internal errors should only be for mistakes in Tock.

## [https://github.com/tock/tock/pull/3795] - Enabling Capsules to get Short ID
- Brad: We discussed this during the app id discussion. As far as I can tell, there is no way for a capsule to actually check a process's corresponding app id via the short id. This makes you unable to do anything with identifying applications.
- Brad: I think that process ID is our handle for capsules to access internal process specific items like this so I added the function there.
- Brad: This way you can get the identifier for tracking which application is doing what. 
- Hudson: This makes sense.

## [https://github.com/tock/tock/pull/3803] - Stable Rust Discussion
- Brad: There has been an open Rust issue for naked functions.
- Brad: Naked functions have been the major blocker for Tock using stable Rust. 
- Brad: A lot of people were exciting in 2022 to stabilize naked functions. There is one person standing in the way of that. It does not look like it is going anywhere. 
- Brad: A lot of people who care about this have switched to `global asm` and is not worried about stable Rust.
- Brad: I think we should get off this bandwagon and just use `global asm`.
- Brad: Doing that would mean we can compile cortex-m with the stable compiler. We would require one other change to get this to work for RISC-V.
- Brad: This does work, now that being said, this is a proof of concept in a way. I imagine we will still compile in nightly, but as part of testing we will confirm that it is possible to compile with stable if people desire that.
- Hudson: And we are going to insure that by configuring Hail to compile using stable by default? 
- Brad: That was my thought. Hail seemed like an okay candidate.
- Leon: From previous discussions, I remember having nightly in the long run being an optional feature and building all boards in both nightly and stable. I believe this shouldn't be too much work to integrate into CI.
- Brad: Currently, we are compiling our own standard library. I do not think we want to stop doing that. For CI, we could do what you propose Leon (compile for stable and nightly).
- Amit: Remind me why we would stick to nightly still?
- Brad: Because we want to compile our own standard library with optimizations which cause code size savings.
- Brad: We also want to use custom tests framework.
- Hudson: And there is also the virtual function elimination option which provides substantial size savings.
- Alyssa: Just a warning with that flag, we've managed to get some miscompiles from it, but only in bazel.
- Alex: While code size is a concern, being able to compile using stable rust is a major advantage for safety critical users. Nightly is not accepted / certified. 
- Brad: Absolutely. I do not know if this is the best way, but I added a `make` flag to turn off all things in the build system that require nightly so it can compile with stable.
- Leon: I am in favor of these changes. I dislike `global asm` with needing external function definitions and separate asm blocks even if they are kept close together in the source files.
- Leon: This would make some of the bugs I have experienced in the hard fault handler in cortex-m more likely to appear. You lose the tight coupling between the function signature and the assembly that stands behind that.
- Leon: I guess this is a necessary evil, but I believe it goes against the direction I think this particular piece of code in Tock should move.
- Brad: You are not alone. I do not know how else to proceed. Either someone needs to get naked functions working in the Rust compiler or more directly say we need this on the github issue.  
- Alyssa: When was the last time we mentioned this is needed on the github issue?
- Brad: I did this, but one person has been causing this to be a standstill.
- Alyssa: What specific engineering needs are there?
- Johnathan: The current implementation make use of some LLVM support for naked functions that is apparently a little hacky and not something the Rust compiler should rely on long term. The proposed refactoring is to change the implementation on the Rust side so that when it mono-morphizes the naked function it outputs a global asm statement instead in the LLVM IR which is apparently a better solution to the problem. 
- Johnathan: What is unclear to me is if changing this refactoring after stabilizing would be a breaking change. That is unknown to me. This may be the reason for the hesitancy.
- Hudson: Brad do you want to mention the target feature situation?
- Brad: Rust added a hidden nightly dependency. There is cargo Rust feature that allows for conditional compilation.
- Brad: This is nice because we could have code that is only for a subset of cortex-m chips in a crate with code for all cortex-m chips. This flag is only set correctly on the nightly compiler. This is another thing we need to revert to be able to use stable.
- Hudson: The failure mode for using stable on the target feature was surprising to me.
- Brad: This is not clear what is supposed to happen.
- Hudson: Your PR relies on a more namespacing approach to separate these things?
- Brad: The traditional solution is to create a new crate for platform specific crates. 
- Hudson: Any other comments on this? We will will wait for a few more reviews on this PR.

## [https://github.com/tock/tock/pull/3796] - Enable Short ID Calculation to use References to Process
- Brad: I think it was generally understood that we should be able to take the process name and create an identifier based on that.
- Brad: In fact, the draft TRD indicates that exact use case. The draft TRD indicates that the function that creates the short ID would be given this process reference.
- Brad: In the actual Tock source code, that function is only given the credential that was used. 
- Brad: This PR adds both: you get the credential that was used and a reference to the process. If you want to make your short ID based on something in the headers, you can.
- Brad: This is particularly important because the credential itself is not in the integrity region. This means you couldn't place an identifier there since you cannot check that.
- Brad: So if you wanted to use something that was covered under a signature, it needs to go somewhere else.
- Brad: It makes sense that the short ID creation function would have access to the entire process object. 
- Hudson: The main discussion on the PR was if the credential should be passed in at all.
- Brad: Yes, I'm not sure how deep to go into this here now.
- Hudson: Your change seems to be a needed fix. Whether the credential should be there could be discussed as a followup.

## Documentation / Tock Book PRs
- [https://github.com/tock/book/pull/23] - Move Kernel Docs to Book
- [https://github.com/tock/book/pull/24] - HOTP Tutorial Update
- [https://github.com/tock/book/pull/25] - Stack Diagram to TickV
- [https://github.com/tock/book/pull/28] - Add Chapter Listing
- [https://github.com/tock/book/pull/29] - Rendered TRDs

## App ID Discussion - Should Errors be Propagated?
- Brad: I intend to convert this to an issue. This warrants another round of discussion. I think Phil will need to be involved.
- Brad: I do not think app IDs are currently usable as we had intended them to be.
