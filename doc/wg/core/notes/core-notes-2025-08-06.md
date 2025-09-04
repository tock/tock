# Tock Meeting Notes 2025-08-06

## Attendees
 - Branden Ghena
 - Brad Campbell
 - Hudson Ayers
 - Amit Levy
 - Pat Pannuto
 - Alexandru Radovici
 - Leon Schuermann
 - Johnathan Van Why


## Updates
* Notes did not begin being taken until the assembly formatting discussion

## Assembly Formatting Discussion
* Johnathan: Brad opened a PR proposing a style for `asm!()` blocks and switching us to that
* Johnathan: I pointed out that upstream is taking a different style approach
* Johnathan: Neither is supported in rustfmt currently. Pat also opened a PR switching format to the upstream approach.
* Brad: I think the upstream approach is horrible and a step backwards. I would rather have to keep fixing the assembly I would actually prefer that to switching to the upstream approach. If rustfmt formats asm, we can switch then, but I want to hold off until then
* Johnathan: I don’t think rustfmt is likely to ever format assembly, because whitespace matters in some assembly
* Johnathan: I actually disagree with Brad and like the Rust upstream style, but am fine to wait until there is tooling upstream to make the switch
* Pat: I noted in the upstream discussion that none of their examples have comments in them! They proposed leading comments as a fix, which I don’t love — makes it hard to copy over assembly with inline comments. I think this is a good opportunity to influence the Rust style some. Person who created the upstream style indicated they are not necessarily opposed to multi-line asm blocks. Maybe we should try to influence?
* Brad: Maybe we should try...but I am pessimistic.
* Johnathan: The Rust reference and rust by example have comments as Rust comments, and not asm comments
* Brad: Those comments are different purposes! If you want maintainable assembly code you need to document what it is doing and explain line-by-line. I don’t understand.
* Leon: Obviously a lot of different opinions, but seems we have little incentive to converge on current upstream style. Someone proposed including asm from external file. Upside of this is then its just .S, punts on entire issue.
* Hudson: The whole point of inline asm is you want to see how the assembly interacts with the code around it
* Leon: I agree, especially because things like clobbers etc. are tied to the contents of the assembly itself
* Brad: Should we just not do anything? My original PR does change a lot of things
* Leon: I think there is a difference in improving what we have now and defining a style guide. Without a style guide we might end up with code churn though. But we don’t see much refactoring of asm anyway.
* Leon: Having these PRs come in is probably a net benefit
* Pat: Because compilers just call third-party assemblers under the hood, some ancient assemblers do not support // comments, only /* */. So we should not support // comments.
* Brad: But we aren’t using those assemblers
* Pat: I think once we have more x86 tools in place, we will run into them
* Leon: I don’t think Rust is moving away from LLVM, we will only be using that I think
* Brad: /* comments are terrible!
* Hudson: I’m with Brad
* Pat: Ok, we can wait until we hit the problem
* Leon: I think we should propose our style upstream. Minimize bike shedding on Brad’s PR, merge it but not the style guide, and move on.
* Hudson: I like that proposal
* Pat: That seems like a fine path forward to me
* Johnathan: I abstain
* Amit: Alex/Johnathan, can you please join matrix for the static mut channel? Then we can set up a time to sync up about it. 
* Johnathan: I will figure out what matrix is and make an account
* Alex: Same

## PR 4416:
* Alex: Can we unblock this?
