# Tock Meeting Notes 2025-11-20

## Attendees
 - Alexandru Radovici
 - Brad Campbell
 - Branden Ghena
 - Johnathan Van Why
 - Leon Schuermann
 - Tyler Potyondy

## Updates

*Crickets*

### x86 cdecl to C ABI PR (#4662)
 - Leon: Brad, can you give some background on this PR?
 - Brad: Updating Rust gave a new warning because Rust doesn't understand cross
   compiling well. The easy fix is to change `cdecl` to `C`, because `C` is
   valid on x86_64, whereas `cdecl` is not. There is a bit of a question whether
   it is better to specify `cdecl` specifically rather than use `C` like we do
   everywhere else in Tock.
 - Leon: Right, so that comment comes mostly from Bobby who is obviously a major
   stakeholder in this part of the codebase. To restate his argument, the `C`
   ABI string is equivalent to `cdecl`, but only when targeting this particular
   platform. He cautions against this change because for most platforms they are
   substantially different, and `cdecl` is the canonical name on the platform
   this code targets. The way I understand it, we have one calling convention
   that would technically work and get rid of the warnings, with no tangible
   change in generated code, but it's sending the wrong signal in the codebase
   as to this being a different calling convention than intended for these
   targets. Is that a fair characterization?
 - Brad: Mostly. It's strange that any of our code is sending a signal about
   other platforms than what it is designed for. We're forced into it, so sorry,
   but we're not claiming it works or will be meaningful, just that it doesn't
   give warnings.
 - Branden: You could build this on ARM and it's totally irrelevant.
 - Branden: So `cdecl` is what is actually happening when compiling for
   `x86-32`. Now it's complaining because `cdecl` is not supported on our host
   platforms. What's the argument against just changing it?
 - Brad: If you haven't spent as much time with this particular annoyance of
   Cargo, it feels strange to do something that is less straightforward for the
   32-bit platform.
 - Johnathan: Three options: Switch to extern `C`, not compile the code on `x86`
   + rearchitect Tock build system, use a macro like the `fn_abi` crate.
 - Leon: Initially in favor of leaving the more correct convention (`cdecl`) in
   place.
 - Branden: I'd love to leave it, but if that gives us errors. And `C` is the
   same thing on other platforms. Could leave a comment saying "this is really
   `cdecl`".
 - Leon: I thought this warranted a short discussion, because these are not
   fundamentally new issues, is that in contrast to other cases (like
   target-specific inline ASM), is that we could always swap out the body
   without changing the type signature. But the ABI is part of the type
   signature, and while we could add conditional compilation attributes, so it's
   irrelevant at the call site, if you want to reexport it like in a trait or
   constant, then those uses have to change it. That seems super quirky to me.
 - Johnathan: Could supply type aliases for callers to use... wait can we? I'm
   not sure.
 - Leon: I think so. But that's a solution to a problem that should not exist.
 - Leon: We could decide that conditional compilation is the way to go, could
   change `cdecl` to `C` perhaps with a comment as Branden described. Or, and
   I'm not convinced of this yet, we could raise our frustrations to the Rust
   project upstream.
 - Brad: I might propose that we merge the PR now, maybe with the comment. Then
   send it to the x86 working group, asking them to propose something else if
   they want.
 - Branden: I think that's fairly reasonable. I think there's a bigger problem
   that maybe the x86 working group should raise with Rust. I haven't looked at
   this macro, but the implementation is like 100 lines so we can vendor it if
   we want.
 - Johnathan: I agree with Brad's proposal.
 - Brad: What also is confusing is "what in the world is happening right now?".
   It's not clear. At least this makes it clear what's happening.
 - Leon: That's a fair point. I don't feel strongly about the proposal. Given
   that Bobby voiced a strong opinion here, do we want to give the `x86` working
   group time to respond to that first?
 - Johnathan: We could `allow` the lint and not make the ABI change. That puts
   less impetus on the x86 WG to fix fast, but they would have to change it
   before it becomes a hard error.
 - Leon: Yeah emphasize that we will merge the ABI change if this becomes a hard
   error.
 - Johnathan: I would be very surprised if this becomes a hard error before 2027
   because that would be breaking Rust's stability promise.
 - Brad: How hard would it be to check that this will fix the warning?
 - Leon: Should be easy, just add an allow statement. Is that a resolution that
   we can agree on? If it does fix the warning.
 - Brad: I'm skeptical that anything will get done but sure, it's no worse than
   we have now.
 - Johnathan: I agree with Leon's resolution.
 - Branden: No super strong opinion, I think it's fine either way. We do have to
   come back to it later, but there's only a few instances of this so who cares.
 - Leon: I will test that this actually resolves this warning, summarize this
   discussion on the PR, and suggest the x86 working group follow up with a
   better solution.
 - Johnathan: How about I summarize this discussion on the PR instead?
 - Leon: I don't like forcing their hand because this is x86 WG code.
 - Brad: That's why I think the allow is a great compromise. In my mind,
   updating Rust trumps the x86 working group.
 - Leon: Yeah that makes sense. Any other comments?
 - Branden: Nope

### WiFi PR (#4529)
 - Alex: Is there any other feedback on the WiFi PR?
 - Branden: I have not looked in several days. What's the status?
 - Alex: Irina addressed the comments, and has time to address further feedback
   now but will busy in a week so it would be nice to finish it soon.
 - Branden: I will make sure to scan through it again.
 - Branden: Can also discuss at the network WG next week. Does Irina have any
   questions for us?
 - Irina (on Alex's mic): No.
 - Leon: Is it reasonable for us to set a Monday deadline and transition this
   into a proper or pseudo last-call?
 - Branden: It has no approvals now. It could have approvals by Monday.
 - Leon: Could set Monday as a virtual deadline for the network WG to review by.
 - Alex: That would be great. No need to approve, just look + give feedback.
