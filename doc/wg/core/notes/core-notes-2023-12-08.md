# Tock Core Notes 2023-12-08

## Attendees
- Tyler Potyondy
- Phil Levis
- Andrew Imwalle
- Alex Radovici
- Alyssa Haroldson
- Leon Schuermann
- Johnathan Van Why
- Hudson Ayers

## Updates
# Binary Working Group
- Hudson: Rust binary size working group. Some people reached out over github. Johnathan discussed this last week.
- Alyssa: Working to help make the documentation more understandable. 
- Alyssa: Some interesting topics have come up already, in particular regarding string formatting.
- Alyssa: I want to put together the argument that printf style strings are the smallest way to do this.
- Hudson: I am interested in if the core library gets compiled with your optimizations even when you are not using build standard. It seems people have claimed this is the case, but I remember seeing some pretty different / worse codegen when I looked into this a year ago. This may not be up to date, but I would be curious what the current standing is. 
- Alyssa: It makes sense to me that there would be a prebuilt copy optimized for size and a prebuilt copy optimized for space.
- Alyssa: One other question, would inlining mirror require rebuilding?
- Hudson: I do not think so. 
- Alyssa: One other thought: unpacking large result objects often results in poor codegen even with inlining. Perhaps mirror inlining may help with this.
- Hudson: I cannot remember if we turned this on in Tock.

## TickV Discussion (https://github.com/tock/tock/discussions/3709)
- Andrew: Just as a heads up, some of the agenda emails have been going to junk.
- Andrew: I have been focusing on TickV and key value for my team.
- Andrew: The TickV spec states that a limitation is fragmentation when something is written to a region in flash memory and is never removed.
- Andrew: In such cases, even if the rest of the region is cleared, the flash memory might still indicate that the region is full, despite having only one valid entry. The current implementation of garbage collect is unable to solve this problem, which leads to the flash filling up while only having a few valid entries.
- Andrew: I have been looking for a way to handle this and would love feedback.
- Hudson: This could be helpful to place in the issues section on the main Tock repo.
- Hudson: The two main people who have done development on this are Brad and Alistair.
- Andrew: Maybe we can postpone this discussion to include them.
- Andrew: There was a previous discussion regarding the zeroize function in tick-v. 
- Andrew: Effectively zeroize does not have any downsides. Primarily a security improvement. I do not want to make a PR for things people have not signed off upon.
- Hudson: I am not entirely familiar with TickV, but I assume there is a performance tradeoff for doing this.
- Andrew: This is true. There is a slight performance hit. Alistair's biggest concern was it may cause additional wear. There may be an increased code size, but this should be minimal since it would replace the functionality of invalidate.  
- Hudson: It seems Alistair signed off on some parts of your previous discussion.
- Hudson: Alistair is the most familiar with this. People are more likely to look at PRs than issues.
- Andrew: The conversation seemed to have stalled.
- Andrew: One other question, do we have plans for LMS verification? If not, my team would be interested in working on this.
- Johnathan: Are you talking about adding a crypto API for signature verification or integrating it into app id?
- Andrew: I believe integrating it in. This is not my specific project, but I believe Tock currently only supports RSA. There are not any Rust embedded optimized versions of LMS.
- Johnathan: Adding a new crypto API would likely just be adding a new syscall driver. App id was designed to be extensible. 
- Johnathan: Neither should be controversial.
- Alyssa: I would want to know what cryptographic review the implementation has undergone. This should be included in the PR.
- Andrew: I believe it should be the standard replacing RSA as RSA becomes deprecated.
- Alyssa: This review should be regarding the implementation.
- Hudson: This would be a neat feature. I doubt their will be reluctance.

## TockWorld 
- Hudson: Latest status, Brad sent out a survey. 
- Alex: I was interested if these poll results / potential dates are available. This would be helpful for planning.
- Alex: If we can settle on dates in January, that would be very helpful for us.
- Hudson: I will reach out to Brad for an update.
