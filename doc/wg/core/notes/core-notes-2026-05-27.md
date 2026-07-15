# Tock Meeting Notes 2026-05-27

## Attendees
 - Johnathan Van Why
 - Brad Campbell
 - Amit Levy
 - Hudson Ayers


## Updates
### Tock-Registers
 * Johnathan: we have settled on register\_map as a name and on keeping multi-layout support, but not emphasizing it in the documentation


## Tock Registers Merge Plan
 * Johnathan: On this call I am looking for consensus that we want to move forward with the design currently in the giant PR. Once we have that, I will look into splitting up into multiple PRs
 * Brad: What does splitting the PR get us? Does that help you?
 * Johnathan: I think there will be different reviewers for different parts of the changes, and I think giant PRs rarely get properly reviewed
 * Johnathan: I am planning a tree of PRs about 6 nodes deep, with some of those layers being multiple PRs wide.
 * Hudson: I recommend not splitting as many of the width PRs if you think they will have a single reviewer — it can become a lot of overhead to track that many separate PRs
 * Johnathan: Sure, I will let reviewers decide if they want to request additional splitting.
 * Brad: The main thing I am concerned about is versioning numbers (0/1/2) — we are currently on v0, need to indicate a split. Also, we need a concrete plan for update tock to the new registers format
 * Johnathan: Some people proposed jumping from v0 straight to 2, I am considering that.
 * Johnathan: I was planning to work out the remaining things on matrix (e.g. dev branch vs. master, etc.)

## PR 4479
 * https://github.com/tock/tock/pull/4479
 * Brad: This changes some debug functions in the kernel to use a new capability rather than unsafe
 * Amit: This is tricky — this sort of tests what is reasonably in the domain of capabilities vs. unsafe. I think the question to ask is whether this is only protecting privileged kernel functionality or is this also protecting Rust soundness? Can misuse of this capability to call panic\_print() from a non-panicking context break Rust?
 * Hudson: It seems relevant that we call disable\_app\_mpu in panic\_print — could that be misused to create unsoundness?
 * Brad: Leon would say that if you called this and then ran an app then you could touch arbitrary memory owned by the kernel from that app, which would be unsound, and you could do that without calling unsafe
 * Amit: I think that is fixable by re-enabling the app\_mpu later in that function
 * Brad: Lets take the hypothetical that we could fix all potential soundness issues, would we still prefer to have this be unsafe? Is it worth continuing with the capability idea?
 * Amit: I would definitely prefer capability if there is no soundness implication
 * Brad: I think we need to fix our safety docs for everything called in this so we can reason about this better, and then decide

## PR 4682
 * https://github.com/tock/tock/pull/4682
 * Amit: I approved this awhile ago
 * Johnathan: Do we need something to prevent init from being called multiple times?
 * Amit: I don’t think so, for the same reason we did not prevent the things init is now calling from being called multiple times before

## PR 4793
 * https://github.com/tock/tock/pull/4793
 * Brad: This is an alternative to #4716 to fix a soundness issue in process_create
 * Brad: Basically, use MaybeUninit in the RingBuffer rather than pre-initializing everything in the RingBuffer
 * Amit: Didn’t this leave off with you trying to verify this using Flux?
 * Brad: Yeah there were some issues with the indexing brackets, I resolved that by using .get(), so the Flux issue is handled
 * Brad: I think I fixed Leon’s comment, 
 * Amit: Is that not what was going to be proven using Flux?
 * Brad: I don’t know. I am not sure if Flux is going to help us with the correct assume_init API. Lines 106 and 128 are the lines in question here.
 * Amit: The important variant is that if is_valid returns true, line 159 executed with self.tail being equivalent to ….
 * Amit: Ok, so are we waiting on anything? Did something change in your latest force push such that Leon’s comment is now outdated? I guess the other comment on this is that this is hard to reason about 
 * Amit: Since changes are requested on this, I am approving the design as seeming sound, and that way you can merge once you work through those changes

## PR 4794
 * https://github.com/tock/tock/pull/4794
 * Brad: This has my name on it, but was pulled from a new board contribution
 * Brad: Pat had a few concerns with this, I think maybe we should just close it
 * Amit: Yeah, lets close it

## PR 4817
 * https://github.com/tock/tock/pull/4817
 * Brad: If you have a public key you trust for services, which is separate from other keys you have, we have capsules for key switching etc. and that works in our tutorial. However, when adding that functionality for mobisys we do not have a way to propagate information about which key worked, other than its order. We want to be able to use the kernel attributes (at the end of the kernel binary) and put keys there and tag them as trusted for different things. This PR adds an API to propagate that metadata. The keys could actually be anywhere, this PR just adds a way for services to verify that whatever thing was verified with whatever key.
 * Amit: I approved
 * Hudson: This seems reasonable
 * Brad: Hopefully there will be followon PRs, and I think those will just be capsules
