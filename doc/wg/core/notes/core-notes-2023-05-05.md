# Tock Core Notes 2023-05-05

Attendees:
 - Branden Ghena
 - Johnathan Van Why
 - Leon Schuermann
 - Alyssa Haroldson
 - Alexandru Radovici
 - Amit Levy
 - Hudson Ayers


## Updates
 * Alex: Working on ethernet along with some support from Leon. It did work and a packet has been successfully sent!
 * Leon: I was trying to get a hold of the STM evaluation board with Ethernet, but it's not for sale anywhere due to chip shortage. I do have a somewhat _cursed_ NXP board. Complicated, but I think it's close to working. Would be nice to have multiple examples
 * Alex: I might be able to ship you a board! I also have some contacts at NXP

 * Amit: Student working on experimental moving kernel into interrupt-mode finished her thesis. See if she'll still work on it afterwards. I don't have synthesized things to share about it yet, thesis was very prototype. Still seems promising. Stripped down blink without timer worked. Reasonably significant savings for performance for basic system calls, which is where I'd expect the _least_ performance savings. Other than some engineering issues, it actually seems pretty doable, at least on ARM.
 * Amit: I expect to have more to share about that in the future


## Bors Deprecation
 * https://github.com/tock/tock/issues/3428
 * Hudson: We could host our own bors or move to github's merge queue, which I think Leon and I both think is a good idea
 * Hudson: We would need to experiment with the merge queue for a bit to make sure it'll be good, but I think it'll be doable. I'm happy to take a stab at it, maybe I'll try on libtock-c to start
 * Hudson: Alex also wanted it for tockloader-rs, although I don't think it'll be needed if you don't have CI
 * Alex: I would like to experiment with it though, can you give me permissions?
 * Amit: Done
 * Leon: I could also take a look, but I can just assist Hudson if he has time
 * Leon: The only thing I wanted to discuss was starting somewhere else other than the main one
 * Hudson: libtock-c does have some CI actions, so it should be a fine test location
 * Amit: Could we hit a github action limit?
 * Hudson: We're replacing bors, so I think it should be okay still
 * Alex: I think they have unlimited minutes for open-source projects
 * Alex: For enterprise, the Tock CI is HUGELY consuming. So it _would_ be good to only run some tests. It doesn't seem like merge queue supports this.
 * Alex: Merge queue is also very strange, it runs the tests as if they were in the PR, but actually they run in an environment without git somehow, where git commands fail. Then after they merge the start running the tests. It's very strange when I played with it in other repos. Pretty horrible and not well documented
 * Hudson: Okay, this might not be straightforward then. We call git commands in Tock makefiles, I think
 * Branden: If bors is deprecated, what is Rust doing?
 * Hudson: They use their own stuff
 * Johnathan: bors-ng was an open-source tool replicating rust's bors stuff. But then github thought it was a good idea and made merge queue. So now they're deprecating it 
 * Leon: There's a good blog post about why they are deprecating it. Bors couldn't do some things in github, it was impossible. And if githubt has a new tool, they're not going to support bors-ng. https://bors.tech/newsletter/2023/05/01/tmib-76/


## External Dependencies Documentation
 * https://github.com/tock/tock/pull/3312
 * Leon: I don't know if there's a ton to lead here. We've talked about this a bunch and I've tried to update the document to reflect our conclusions and still be coherent. But since it's distributed across so many discussions it's hard to keep track over the status of things.
 * Leon: So my proposal is to do a brief pass over the current state of the document and see its state
 * (pausing to read)
 * Hudson: Seems right to me. Maybe we could move the rationale to the end, so most people can focus on what the rules are
 * Amit: Agreed seems right to me too. But in the top, maybe don't throw shade at cargo so much
 * Branden: Do we need wrapper-traits or not? We say not to use them up top, but say they might be good for board-specific stuff later
 * Leon: Yeah, good catch. I think they make sense where they make sense. Not a requirement, but if they're useful we're good with them
 * Branden: I think it's good to say "do what makes sense", so I'm on board with that
 * Johnathan: I think this policy only applies to thinks in the kernel binary, not tools for building to binary. Is that correct? We might want to clarify
 * Hudson: That is correct
 * Johnathan: For OpenTitan, we're considering the opentitan tool being a dependency, which doesn't appear in the final binary but is useful for testing. So we could add a "scope" section to the PR
 * Hudson: Great, after a few more edits, I think this is ready to merge


## Tockworld Planning
 * Results on survey discussed
 * Most popular dates are July 21, 24, 26-28
 * Johnathan: unavailable in that entire range unfortunately
 * Amit: I wonder if we could find a time that better works for Alex and Alistair. Maybe we could find a time that partially works for each of them?
 * Branden: Two full days makes sense. Last time we did 1.5 days and the half went long
 * Amit: I wonder if it would make sense to do a training session there
 * Alex: I would be interested! But we would have to have boards in the US already
 * Amit: Then we could extend to three days and maybe bring in folks for the training day who wouldn't otherwise be coming. Maybe some lowrisc or zerorisc folks.
 * Amit: So maybe the 24-25 range? Or 26-28th (most likely)
 * Amit: So the next step is to check in with Brad and confirm real dates so we can get flights
 * Hudson/Amit: Notes that transit from DC airport to UVA will be a two-hour drive or possibly an Amtrak train ride

