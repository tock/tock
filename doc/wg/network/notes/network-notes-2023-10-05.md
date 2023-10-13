# Tock Network WG Meeting Notes

- **Date:** October 5, 2023
- **Participants:**
    - Alex Radovici
    - Felix Mada
    - Tyler Potyondy
    - Branden Ghena
    - Leon Schuermann
- **Agenda**
    1. Updates
    2. Ethernet Planning
    3. Buffer Management
- **References:**
    - [#3683](https://github.com/tock/tock/pull/3683)
    - [#3695](https://github.com/tock/tock/pull/3695)


## Updates
- Tyler: Sticking around for a PhD 🎉, so I should be around for a bit
- Tyler: PR for Thread Child (https://github.com/tock/tock/pull/3683) is moving along. Should be merged soon. It's time to take a look at it if you want to. Not really any open questions here, just need comments.
- Tyler: Also, draft of radio capabilities is coming soon
- Leon: Ethernet support for STM32 needs a review too: (https://github.com/tock/tock/pull/3695). It's includes the clock changes Ionut has made and is a follow-up on a PR that had too many changes. So there have been eyes on it (more than it appears). Also the PR goes to tock-ethernet instead of main.
- Alex: Teaching a class on Rust!


## Ethernet Planning
- https://github.com/tock/tock/pull/3695 (and more generally the tock-ethernet branch)
- Leon: The Tock-Ethernet branch emerged from dev plans on Ethernet starting almost two years ago. There have been local versions of userspace Ethernet stacks since 2019, but the code has always been rough and not ready for release. Thinking about constraints on supporting Ethernet, back then it wasn't clear we even wanted to support it, figured a branch that had some infrastructure would be good. Goal was to use the branch to iterate on a HIL that could support various Ethernet chips.
- Leon: Since then, the branch has grown to pretty stable, though preliminary, structure for network access on Tock through Ethernet. Contains multiple network cards. Contains a HIL for sending/receiving Ethernet packets. Contains a capsule that can transfer packets from the HIL to userspace with an internal ring buffer. That's enough to run an HTTP server on Tock it turns out!
- Leon: Some high-level questions: Should we keep operating in a branch? When do we decide it's "stable enough" to merge into Tock main?
- Branden: Isn't maintaining a separate branch a pain? Does it track main?
- Leon: Yes. We've been doing merges, only ever a month behind at worst. Not been too bad. The one big issue was that Github doesn't deal with PRs against branches that merge in significant parts of another upstream branch. The way Github presents the diff is different from your local client. So PR authors need to reset their branches to something rebased on this branch, or the Github diff looks like a ton of meaningless changes. That might be a good reason to merge.
- Branden: Second, what are we waiting on? Since it really only works with itself, it's okay to have even crappy code in Tock main. Experimental system that won't break other things.
- Leon: I think that makes sense
- Leon: Also, the TAP driver for userspace that moves packets from the HIL to userspace is pretty rough. Could definitely use a rewrite.
- Branden: Could happen after we merge with main though.
- Tyler: I think sooner rather than later makes sense. Especially since it only really affects others using Ethernet.
- Tyler: For the 15.4 stack, I've found that really really explicit documentation about what is missing still would be really useful. Made the 15.4 stack more difficult to learn because I assumed some things existed. So I recommend comments, docs, headers in files, anything to show people what's "known missing".
- Leon: Yes, definitely. There are new people joining Tock all the time who don't have all the history. I think this will be especially important for the HIL, which is pretty minimal right now.
- Leon: Do we have any official classification for the states of HILs?
- Branden: No, I don't think so. Mostly legacy knowledge in the group about which things are or aren't stable.
- Leon: So again, not demonstrated well for new/outside contributors
- Leon: I think moving forward with PRs to upstream, after the STM32 stuff lands, would be good
- Branden: One challenge is just the stability of Tock at this point. We used to just push stuff all the time in the old days. But now we have PRs like this that sit around, even though they are for an unstable system and should just move fast
- Tyler: Where would someone start looking into things?
- Leon: I think this PR is actually relatively self-contained. It's been pared down to the minimal stuff. In short, I think the kind of work you've done on the Thread stack thinking about vision isn't where we are here. We're just trying to implement drivers to send packets at all. It's been a very big effort.
- Tyler: Is this the board that Alex brought to Tockworld for Leon? Is it using an FPGA?
- Leon: Right board. But not an FPGA, that was a different demo I did. That's a RISC-V board I have been working on with libtock-c. And Ionut has been working on this STM32 ARM board with libtock-rs. So we're really in a state where we can mix-and-match and things work.
- Leon: Also, Amit has the ethernet-over-USB working. So any USB-capable Tock board can present itself as a ethernet port when plugged into a computer. The profiles are USB EEM and USB ECM. Used for USB ethernet adapters.
- Alex: When I plug in the board, what do I see? A new ethernet card on your computer? (Yes)
- Leon: So anything the computer sends on that interface appear on the device. It's a "virtual cable". Like you had an ethernet port with a cable, plugged into the Tock board


## Buffer Management
- Alex: I'm interested in more discussion about the Buffer stuff we were working on last time
- Alex: So a question from last time. I was under the impression that if you have a constant parameter, that a smaller one would still work. That's wrong though. It has to match exactly (yes)
- Alex: So, you can reduce headroom. But how do you get it back?
- Leon: Yes. One of the ideas is that `reduce_type_headroom` doesn't necessarily give you a brand new type, but it has a lifetime. That doesn't work great with static lifetimes though.
- Leon: Or you can potentially have `reduce_type_headroom` return a new type and also a marker type that holds the information about how much the headroom was reduced. You can plug those back together in some way to combine them. So you split the info on your buffer and then can recombine them.
- Alex: How do you prevent the user from mixing two pieces that weren't originally split?
- Leon: Rust aliasing rules prevent buffers from overlapping. So the pointer of a buffer is enough to determine this.
- Leon: I think these might be overly complicated. I think the internal information in the struct has enough information to know where the headroom was.
- Branden: That's what I expected.
- Alex: Problem is what if you reduce headroom several times. You need a stack of info?
- Branden: I don't think you need a pop. As long as you know min and max, you can just reset to any number between them
- Leon: Yes, each thing would reset to the new headroom, which must have enough space for that by initial construction
- Alex: So each driver would have to manually determine the size it wants
- Branden: Yes. That seems reasonable to me as a network driver choice though
- Leon: Need to separate the ideas of things that are inherently memory unsafe versus semantically tricky. Changing buffer size, between the limits, is safe from a memory perspective.
- Alex: Could we have a PR with just the buffer stuff to start?
- Leon: Could, for sure
- Branden: We will need to display real-world use, hopefully in two different use cases
- Leon: We should definitely chat about the precise requirements of the display driver
- Alex: I actually have two display drivers. One can do separate SPI transfers, one must do a single transfer, so it really really needs to append headers to the front of something
- Branden: SDCard does this too. With lots of copies right now
- Alex: Another use case is for WiFi. We have the Arduino Nano WiFi which has exactly the same problem
- Leon: I have been trying to keep in mind the complexity of the network stack when engineering this idea. Things get pretty messy when thinking about examples there. So a simpler interface to start with will be very useful
- Alex: The "bus driver" has the same issue. It's a shared bus for displays that can use either I2C or SPI.
- Branden: Didn't know that existed. Neat
- Branden: So to start, I think a really really minimal implementation of this idea would be super useful. No bells or whistles. Just the basics. Then we can implement something with it and start asking for features
- Leon: Pretty clear the direction I should move here. Just need the time to do it. A good plan would be for Alex and I to meet up sometime next week to talk about this
