# Tock Meeting Notes 2024-05-10

## Attendees
- Branden Ghena
- Brad Campbell
- Leon Schuermann
- Amit Levy
- Andrew Imwalle
- Hudson Ayers
- Viswajith
- Johnathan Van Why
- Alyssa Haroldsen


## Tutorial Updates
 * Leon: We're doing a final run-through of the tutorial. All the code's in place, although there's still a PR or two that won't be controversial. We're still breaking down the code into milestones and making sure things are consistent with the libtock-sync API.
 * Leon: We're also doing a cleanup of the writing and publishing a new VM image.
 * Leon: There will be one more semi-large libtock-c PR, which only adds the tutorial folder with a bunch of apps. So that shouldn't be too hard to merge
 * Brad: Are there other things in the branch that need to be merged upstream, other than what's in the PRs right now?
 * Leon: There's an open PR for the entire branch, I think. We're pushing directly to the branch and everything goes there, so the PR should reflect the changes. That's a libtock-c PR
 * Leon: The changes on the PR, as far as I see, migrate openthread to libtock-c-sync, and add a bunch of applications in the examples / tutorials folder
 * Kernel PR - https://github.com/tock/tock/pull/3979
 * (There is no PR in libtock-c at this time, it'll be here soon)
 * Leon: For the next few days, anything non-controversial but time-sensitive we'll send out a notice in the tutorial slack channel.
 * Brad: That sounds good
 * Leon: We think everything should be in by tomorrow
 * Brad: The getting started guide is also in progress: https://github.com/tock/book/pull/38
 * Branden: Can you explain this?
 * Brad: Backstory from Pat was that they did a tutorial runthrough at UCSD, and the getting started guide was a bit much. I agree that it's got a TON of information, and it's not always obvious which parts are important. So the thought was that breaking it down into parts could help guide you to what's most important.
 * Brad: So I did a first pass in this PR. The goal was for Pat or Tyler to review and make additional updates for problems they saw from the runthrough
 * Brad: As far as the tutorial text goes, is there anything you need there?
 * Leon: We have a draft but we're going through it right now. Tyler is still producing text for the openthread parts. That won't be as bad as it sounds though, as people will only be doing small additions to the openthread starter code.
 * Leon: I'll keep people in the loop as the text changes and when we want reviews
 * Leon: For the entire tutorial we have roughly 3-4 hours, counting some setup. So we are tuning down the tutorial a little to make it fit in time. We have roughly 10 people signed up now. So it's possible that people will finish early, and for those we intend to do ad-hoc walkthroughs of app signing and things like that.
 * Brad: I think you don't have to worry about adding more stuff. People will either just want to stop when finished, or will want to dive deeper and will be able to be guided to documentation
 * Branden: Hands-on stuff like this always takes longer than you expect too. So I wouldn't worry about getting done too early
 * Leon: We do have a getting-started guide, but people aren't necessarily going to go through it in advance, even if we tell them to. So that will take some time too
 * Brad: I do wonder whether you could get people to compilation _while_ you're doing the introduction stuff. Particularly first-round compilation on some of the libraries
 * Leon: I have the VM on some flash drives that I can just do first-round compilation on for them before handing out.
 * Leon: We also have a tock-assets website that uses a CDN, so it should work better than you might expect, even in Hong Kong


## Rust-NL
 * https://2024.rustnl.org/
 * Leon: We met with a bunch of people from the Rust Embedded working group, that work on the HAL crates and things like that
 * Leon: They run into many of the exact same issues we do. Static muts, and hardware CI, and stuff. Really nice to have conversations with them.
 * Amit: Even if sharing code isn't going to happen at a large scale, I think there are many overlaps in challenges and sharing of solutions.
 * Leon: We'll present a more organized version on a subsequent call
 * Brad: It would be pretty cool if we could share testing hardware, for overlapping boards
 * Hudson: Was there any mention of the recent libtock-rs PR to using some of their stuff?
 * Leon: I mentioned it and they approved of it. They thought that this is what their interfaces are for and that it makes sense.


## Libtock-C
 * Brad: Libtock-c rewrite is in, with new interfaces and a new style guide. It's hopefully MUCH more consistent for all drivers. So there's some consistency and most drivers work the same
 * Brad: Other benefits are the split between libtock and libtock-sync. Libtock does asynchronous code only. There are no uses of yield and you'll only get an asynchronous API. There's almost no internal state too, so almost everything is exposed back to you. The services folder is the exception to the state thing.
 * Brad: If you want a synchronous version, which needs some small internal state, that's the libtock-sync code. So hopefully this makes it clearer what to use depending on your desires
 * Hudson: I think it's an enormous improvement
 * Brad: I will add, that from porting some of the examples that if you're using the sync API it's mostly the same as before with some renaming. The asynchronous versions, which have callbacks customized for the use case, take a little learning but even with that for the majority porting should be straightforward. Not a lot of gotchas.
 * Hudson: I noticed in the now-merged PR had a description that some BLE, IPC, and I2C stuff isn't ported over yet. Is that still true and is it tracked in an issue somewhere?
 * Brad: That's correct. We do need a tracking issue for that
 * Hudson: We also move some stuff to a folder outside of CI. Should we include that in the issue too?
 * Brad: Some of those, but not all. Old courses don't really matter. But some of it is SPI stuff that needs to be updated
 * Brad: Amit is putting work into IPC and maybe has something that's close anyways
 * Hudson: Okay, I'll make a tracking issue for this


