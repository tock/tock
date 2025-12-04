# Tock Meeting Notes 2025-12-04

## Attendees
 - Branden Ghena
 - Hudson Ayers
 - Johnathan Van Why
 - Leon Schuermann
 - Amit Levy


## Updates
### IPC
 * Branden: https://github.com/tock/tock/pull/4680
 * Branden: This is an RFC documenting IPC efforts. It mostly aligns with our discussions from September at Tockworld, actually recording them in writing. It also includes goals/non-goals and an example of how mechanisms could be used for a Thread service.
 * Branden: This documentation does NOT include any shared memory mechanism yet. We wanted to get this out for eyes first, then work on an implementation for allow-based messaging, and then circle back to shared memory which is going to be more complicated.
 * Branden: For now, our ask is just eyes on this when you have time. If you spot anything that seems problematic or that you disagree with, let us know.
### Removing Static Mut
 * Amit: Working on abstraction for panic UART with single-thread value. In the works still
 * Amit: Works for one board, but has conflicts and needs to be added to other boards
 * Leon: It would be good to get reviews on a draft for a single board first


## Single Thread Value
 * Leon: New panic resource for debug. A few calls ago we had a discussion about removing static muts. We established a mini taskforce of Brad, Amit, and Leon to figure out how to fix it. Brad has put in a lot of effort on this. One long-standing issue is how invasive and non-mechanical these efforts are.
 * Leon: So to move the efforts forward, what I think Brad intends is to get more eyes on this new PR https://github.com/tock/tock/pull/4676
 * Leon: This PR take the non-mechanical static muts for resources shared between board and panic routines in io.rs and stuffs them into a struct, which is itself within a Single-Thread Value. This PR ports all boards to use this struct-based approach. It doesn't remove all static muts, notably UARTs for printing and some for tests still exist. But if this was merged, it would be a massive step forward to getting rid of the static muts.
 * Leon: The PR is ready. It's been a bunch of work and touches many boards. So we really want to review and merge this ASAP if there are no sticking points.
 * Hudson: It seems like this PR does the same changes for every board. I assume it's been tested on some main ones. People should just review debug.rs in the kernel and then a few boards for io.rs and main.rs and that'll cover it.
 * Leon: I don't feel comfortable pulling the trigger given that I contributed to this, but I did a full review pass and it's a pretty mechanical change, except in that io.rs somewhat varies among boards.
 * Leon: For testing, this does pass LiteX, which does exercise panic handling. Treadmill also tests panic handling on an nRF52840DK. So there is some evidence that this will work. Once we remove static mut for good we'll tag a new release which will do more rigorous testing.
 * Amit: Leon and Brad did the implementation, but I was a participant in the decision for what to do. I'll approve
 * Branden: We'll take reviewing this PR as the action item.
 * Leon: Good. There will also be follow-ups which are more board-specific. Hoping to get these all worked out soonish


## Single Thread Value Blog Post
 * Branden: https://github.com/tock/tock-www/pull/122
 * Branden: This is a long-standing blog post about Single-Thread Value that Brad wrote up. He's looking for a review from Leon or Amit to verify that what he wrote is accurate before it gets posted.
 * Leon: I do think this writeup is perfectly good and fine. This blog post is useful for explaining and sharing with the Rust ecosystem more broadly. We want to explain why we needed this and why we think it's sound. I want to add sections to this post to add that information. I've been swamped and haven't had time to do it.
 * Leon: If we want to merge this now, we could do that and come back to it later. But I do think the post is a good place to do this.
 * Amit: We could always update the blog post.
 * Leon: That removes the push to update it somewhat.
 * Branden: Have Leon or Amit read it? At least some approval would be useful that the current text is valid.
 * Amit: Could Leon state now what things he wants to add?
 * Leon: No, it would take longer to look into everything and pull thoughts together. The information is in conversations throughout matrix and emails.
 * Amit: How about big, high-level bullet points. Maybe as a comment on the PR. Doesn't have to be fleshed out yet if we can be clear about what needs to get added.
 * Leon: Sounds good, I'll leave a comment

