# Tock Core Notes 2022-02-25

Attendees:
 - Branden Ghena
 - Amit Levy
 - Leon Schuermann
 - Johnathan Van Why
 - Alexandru Radovici
 - Pat Pannuto
 - Vadim Sukhomlinov
 - Jett Rink
 - Brad Campbell
 - Alyssa Haroldsen


## Updates

### Stable Tock
 * Johnathan: Rust update means that some assembly features are now stable
 * Alyssa: What are the blockers for stable Tock?
 * Brad: There are still some issues. There's an issue with them. https://github.com/tock/tock/issues/1654 Several have good signs, but unlikely to have changed in just a week or two.
 * Brad: I did submit a PR to update to a new nightly. We should be able to move ahead and remove the ASM feature from Tock.

### App Completion Codes
 * Alyssa: Draft TRD for app completion codes. Should I apply the suggestions there and commit?
 * Leon: Yes, I think that it's a good summary of the points we had in mind.

### Process Slices
 * Leon: Got the PR for transitioning process slices to raw pointers. https://github.com/tock/tock/pull/2977 Thanks to Alyssa for recognizing that we had unsound code.
 * Leon: This PR introduces many unsafe blocks into sensitive areas of the kernel, and will require a lot of manual checking. Hopefully some automatic checking as well if I can get that working.
 * Leon: I transitioned from square brackets to unwrap calls, so I want to get rid of the panics, where possible.
 * Alyssa: That shouldn't change the panic behavior right?
 * Leon: Yes. But because we needed to rework the interface anyways, that it might be a good idea to remove implicit panics, since it's a friction point in Tock. Indexing into untrusted data could make the kernel panic, so we want to avoid that altogether.
 * Alyssa: Providing panicking functions and see the .unwrap() is best. But providing some kind of unwrap_or_else() would be good too.
 * Leon: Currently we provide only non-panicking functions and use unwrap() in capsule code.
 * Alyssa: I think that's the right thing to do.
 * Leon: The other thing this PR does, and I couldn't not make this change, was that we had some safety invariants to adhere to, like maximum buffer sizes and non-null pointers. There's careful reading to make sure it's not messed up.
 * Alyssa: Do you mention this in the PR?
 * Leon: Yes, there should be info in the PR.
 * Alyssa: Okay, I'll take a look and add comments.
 * Alyssa: One thing I just noticed was that there are lots of explicit lifetimes, that I think aren't needed.
 * Leon: We do want to not leak a buffer that outlives the memory. I'm aware that this is a choice, but I think that being explicit about lifetimes with annotations is useful. People may disagree and I'd be open to changes.
 * Alyssa: I see. It can be subtle. Overall bounding lifetimes is good.
 * Leon: So summary - take a look if you have the time. This interface really needs to be sound and correct
 * Brad: When I looked at the console code in the PR, you think about what capsules are likely to do when sharing a buffer, it's mostly just copying bytes from userspace to a kernel buffer. Could we add helper functions that copy data from one buffer to another in a way that doesn't panic?
 * Leon: We have that. We have the usual copy-from-slice and a copy-to-slice. The functions haven't been used very much, and have even been re-implemented with for loops, but the functions are there.
 * Brad: Good to know. We should think about which cases should be changed over to that, which could get rid of the unwrap in places.
 * Leon: Yes. I have a local commit with some of those changes.
 * Brad: Cool
 * Alyssa: So the question is whether we want to add more unsafe operations to remove the .unwraps, or hope that optimization will elide many of them?
 * Leon: I would like to think that panics are avoidable, for instance on ARM they can be turned into conditional execution. We definitely want to remove panics and return proper errors instead. So we don't want to add more panics.
 * Alyssa: Sorry, I wasn't proposing more panics. I was asking if an unchecked version should exist for cases where it's easy to prove that it won't panic.
 * Leon: Well, an unchecked version would have to be unsafe, and unsafe isn't allowed in capsules. So I think practically, that would not be a useful feature.
 * Alyssa: I do definitely support returning useful error codes instead of panicking.
 * Leon: What we do now is return an option, which is converted into a result, which makes you think about what type of error should be returned and what the API is.
 * Alyssa: I mean the functions calling .get, not .get itself.
 * Leon: That makes sense, sure.

