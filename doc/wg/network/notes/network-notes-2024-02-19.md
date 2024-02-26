# Tock Network WG Meeting Notes

- **Date:** February 19, 2024
- **Participants:**
    - Branden Ghena
    - Tyler Potyondy
    - Leon Schuermann
    - Felix Mada
    - Alex Radovici
- **Agenda**
    1. Updates
    2. Thread PRs
    3. Tutorial Logistics
- **References:**
    - [3851](https://github.com/tock/tock/pull/3851)
    - [3859](https://github.com/tock/tock/pull/3859)
    - [Tock Book 26](https://github.com/tock/book/pull/26)


## Updates
- Tyler: We should talk logistics for CSP-IoT tutorial with Alex
- Leon: I made some pretty substantial progress on the encapsulated process framework. We had to call C functions with the right ABI/signature and translate this though our framework. At this point, I'm reasonably confident that I have a generic solution to that which can be automated elegantly. I'm still missing the last 10% or so, but it's increasingly promising now. Good fallback for Thread if we have libtock-C timing issues.


## Thread Status & PRs
- Tyler: State of the world, good progress on the libtock-c front. Everything compiles. Build system is pretty under control at this point. The platform abstraction layer is the last thing to tackle: radio, flash, entropy, and alarm. Alarm is mostly finished and works. Entropy is working. Radio is the most tricky and I've made some PRs for it. Flash has two UCSD undergrads working on it. I have a simple representation of Flash in RAM for now. Turns out that without the Flash working, OpenThread does some weird stuff.
- Branden: Is it weird without the Flash stuff saved in RAM?
- Tyler: It works fine with a RAM representation. The goal is to remove all "asterisk, it works except" disclaimers for now. I think that's a good goal that's achievable. Background: OpenThread needs to save channel and other parameters for it to work, and those really ought to be non-volatile so it stays in the network on reboot.
- Leon: A warning, app-signing plus writing to flash is a bad combo. It turns out writing to your own flash breaks your signature. That's using the AppFlash driver. We could avoid it altogether, or we could try to fix it. But we can't have both right now.
- Tyler: The radio stuff is getting close. From libtock-c sending is working. OpenThread's internal libraries are working too. The function calls roughly set up configuration for the network, then thread start. That starts sending parent requests. All of those are correctly encrypted. They send when they should with correct backoff. So sending is working (once the PRs get merged). Receiving is still in progress.
- Tyler: One reception concern, OpenThread wants a timestamp for when a packet was on the radio. That might require some heavy rework. But I've been looking at how the nRF boards implement this abstractions. They essentially query the time function when the receive callback is called. So we could just do that too. I think that since the ACK happens right away, other stuff is pretty lax on timing concerns. Actually, OpenThread lets you choose how much you're handling timing versus how much the board implementation is.
- Leon: This sounds very similar to the work I needed to do to get IEEE1588 working with timestamps for Ethernet. I did this as a research thing, but one of those versions just had me read the timestamp register of the chip in the radio driver and pass that up to userspace. That gave me a fairly accurate view. That's probably something we could loop through the interfaces for Thread too if necessary. The nice thing about doing it in the Ethernet MAC is that you can read the timer register without going through layers of abstraction.
- Tyler: That's useful. That could be a good abstraction.
- Branden: Or the easy take is to just plop on a timestamp in userspace when you get it.
- Tyler: Yeah, that'll be step one. I think the OpenThread library isn't as time sensitive since it's pushing stuff off to the board.
- Felix: A question, how do you push the timestamp up the stack for Ethernet?
- Leon: In the upcall path, we have an argument, a 128-bit integer that's wrapped in some type about where it was gathered. That's even too complicated for this case, maybe. We might literally just include an integer in the upcall in the interface.
- Felix: Linux or FreeBSD does something similar. But the push the timestamp through the error stack in a weird way.
- Leon: I did look at the Linux implementation. I couldn't figure out what they were doing even after a close look. They have error handlers on file descriptors to push stuff into userspace because of API restrictions, I think. Tock is much simpler, since we can just change interfaces.
- Tyler: Another thing, OpenThread can do encryption in software in userspace (mbedtls), or it can push it down to the board hardware to do. This is a question for everyone, but my gut feeling is that although I've used our crypto implementation for AES128-CCM and I'm pretty confident it's working, I'm even more confident that the mbedtls crypto will be better than ours. So I'm in favor of having OpenThread do all of the packet creation and encryption, and then just sending the fully-formed packet down to userspace.
- Leon: My thoughts, I just spent a lot of time on system call overheads. We do have high overheads for the round-trips for using crypto. Even in the kernel we do pretty expensive dispatch to drivers/hardware. And we have to copy userspace buffers into kernel buffers for DMA. So the question is a tradeoff of the code size for userland apps and the timing overhead of encrypting the payload. Because, I think, Thread only encrypts management traffic which is pretty small. I think it's probably better to just stay in userspace.
- Tyler: Encryption is called as part of sending. So it's just one syscall, but the capsules would connect to encryption mechanisms. So when you send you tell it what encryption you want.
- Leon: I didn't realize that. So encryption is on the path to the radio
- Tyler: The packet will be fully encrypted. Code size is the biggest tradeoff?
- Branden: Probably a timing tradeoff as well. It might not matter in the end. If we just make a decision and the timing works, then great!
- Tyler: The fewer things Tock needs to do, the better (in terms of framing, forming packet, etc.). OpenThread does expect to form and frame the packets. I don't know if there's a way to disable the MLE encryption, I think it always does that itself. It's the link-layer encryption that you could handle yourself or have it do.
- Tyler: https://github.com/tock/tock/pull/3851
- Tyler: current way you send 15.4 packet is specifying:
    - payload
    - dest addr
    - security configuration
  Capsule takes this, kicks off sending. If we want to implement encryption we need to decompose the constructed packet, pull the security config out, encrypting, etc. So what this PR does is provides a way for there to be a "raw/direct" send functionality. So there's another path through the 15.4 capsule that just sends the packet as-is without changing anything.
- Tyler: It would be useful for people to take a look at the PR and give any comments.
- Leon: I think this is great. It's always useful to keep these raw interfaces around, even after better things exist, for testing and whatnot. Ethernet has a raw interface like this.
- Tyler: Other PRs (libtock-c and tock getProcessBuffer API documentation) related to this.
- Branden: Thank you for opening the latter PR. Even if it's not incorrect, there's still a misconception and we should improve on the documentation.
- Tyler: Yeah, I'm always a little hesitant to open things like that to see if I just misunderstand. I think it would make sense to add some comments about the guarantees here.
- Leon: Yeah, you're spot on. We should definitely add this to the documentation.
- Tyler: Seemed weird that you could get a buffer that you have never shared.
- Branden: Are these the only PRs that we need to pay attention to right now? Brad's PR removing `dyn` is open too https://github.com/tock/tock/pull/3859
- Leon: It really shouldn't change anything about composition of capsules. As long as you don't care about swapping out interfaces at runtime, this is a net improvement.
- Tyler: Other major TODO: switching channels. We don't have this implemented. Wrote a draft in September, still need to finish it. What I'm thinking current is that the current method for setting channel is a constant in the main.rs file for Channel Number. That's passed in as the 15.4 driver is created and can't be changed. Adding the ability to change channels isn't that hard, what's harder is controlling who can change the channel and when. For example, if you're listening for 15.4 packets you sure expect the channel not to change.
- Tyler: My idea: trivial kind of "lock" on the channel (e.g., through TakeCell). You'd say "I now have control over changing channels", and then other apps are prevented from that.
- Branden: Channel control would be a mechanism that you grab in userspace, at init-time.
- Tyler: Not necessarily. I think I would have Thread grab at start and just never release for now. I think there's a way you could grab and release when you want. 
- Leon: This is exactly what we do for current non-virtualized capsules. Be careful to make sure this still works if an app has died. The "lock" needs to be freed
- Tyler: Can you share an example of this with me so I can follow it?
- Leon: Yes
- Leon: Example of a non-virtualized capsule that allows a process to take ownership: https://github.com/tock/tock/blob/906bb4fb237531d3eb21b86857bf30e5d5340743/capsules/extra/src/lps25hb.rs#L346
- Branden: I think your initial design sounds right for now. Long-term a virtualizer would base the decision about channel changing on what the state of the radio is (no changing during reception). And you could have multiple transmitters changing channels, as long as they have some way of specifying which channel, with the radio changing back and forth.
- Tyler: We'll have to think about the actual radio driver implementation. Even with no one requesting it, it stays in the receiving state.
- Branden: You'd need to implement a channel search mechanism to find a network on startup.
- Tyler: OpenTread does that -- just need to give it the ability to set the channel. Been pleasantly surprised with how little it assumes you'd do yourself. For example, the alarm never sets multiple alarms at once and virtualises multiple alarms for you. It really does expect to have bare on-metal functions that set registers and have no functionality.
- Branden: Believe OpenThread to be a fairly high-quality artifact, and this confirms that.


## Tutorial Logistics
- Leon: How much time will we have for the tutorial?
- Tyler: Half-day tutorial session. Unclear how long that is (3-5 hours?)
- Leon: Okay, that'll have a big impact on whether we do two things or one
- Tyler: We should definitely consider what our tutorial plan is and make a vague plan.
- Tyler: For Alex, we wanted to touch base on students coming to the tutorial session, and see if things are coming along.
- Alex: Still working through paperwork. TockWorld is easy and has five people coming. The tutorial has more work in progress.
- Leon: One of Amit's other students is starting to use Tock and hacking on kernel things. He may also join the tutorial session in Hong Kong.
- Alex: So the next step is having a meeting of everyone to start working on Thread. We'll need to get some boards on hand to play with it.
- Tyler: Need the nRF52840DK for that. Several of them to make a network
- Alex: Send me information on what to buy
- Tyler: Right now, my energy is just on the libtock-c port of OpenThread. Don't want to do too much of the tutorial until something works. The next step is to come up with a draft of the tutorial plan. I'll try to have a rough sketch of it for the next Network WG call.
- Tyler: In the Tock Book repo, there's an open PR with some discussion https://github.com/tock/book/pull/26 to look through for now
- Tyler: Once we have a plan, we can divide up tasks on writing and testing and whatnot. Ball needs to start rolling on that soon, which requires some starting on my end
- Tyler: It'd be good to get materials together early on, such that participants can download ahead of time.
- Leon: Small update on my side. I heard from a undergrad working on the existing book tutorials and was going to discuss with them some issues they had. They'll be around to test out Thread stuff too
- Branden: I'm also teaching a wireless class this spring and will have people on-hand who know how Thread works but not how Tock works. So they could test too if needed.
- Leon: Will ask Amit to buy some more nRFs. We should make sure that even if something happens, we should be able to support at least most of the people with boards.
- Alex: Any import restrictions in Hong Kong? Concerns about how long that could take if they're concerned. You could be stuck there indefinitely if they get concerned.
- Branden: Should be an action item: talk to people who have done something like this? -> Tyler, Leon(?)
- Alex: We should try to ship them there or buy them there directly. Maybe with the conference organizers.
- Tyler: Reach out to organizers?
- Branden: Yes! If they know of solutions, problem solved!
