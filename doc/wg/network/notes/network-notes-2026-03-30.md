# Tock Network WG Meeting Notes

- **Date:** March 30, 2026
- **Participants:**
    - Tyler Potyondy
    - Branden Ghena
    - Leon Schuermann
- **Agenda:**
    1. Updates
    2. STM32WLE5xx PR
    3. 15.4 Libtock-rs Tests
    4. IPC
- **References:**
    - [STM32WLE5xx PR](https://github.com/tock/tock/pull/4695)
    - [15.4 Libtock-rs Tests PR](https://github.com/tock/tock-hardware-ci/pull/50)
    - [IPC Design Doc](https://github.com/tock/tock/pull/4680)


## Updates
- None

## STM32WLE5xx
 * https://github.com/tock/tock/pull/4695
 * Branden: Took me forever to get feedback. The only outstanding bit: does it have to be done this way?
 * Tyler: I wanted to do exactly what you describe with a rising-edge trigger instead of a level trigger. But I couldn't get it working for some reason. I think hardware is set up in a way that prevents us from doing it. I'm 95%+ certain on this. I spent a few days reading and looking into it, without finding a way to do it.
 * Branden: It's just a GPIO interrupt, right?
 * Tyler: No. There's an internal SPI bus, and then interrupt lines from the radio go directly into the interrupt hardware in the chip. Directly to the NVIC. So that's why we're doing this weird "virtual GPIO" thing to begin with. Instead of having a GPIO, there are some registers for accessing it.
 * Branden: This seems like the best of a bad situation then. Makes sense that the NVIC can't do edge detection, just level. So you have to mask it.
 * Branden: Presumably other implementations also need to mask it right? While they do the SPI transaction.
 * Tyler: Yeah. I did look at other implementations for this. RadioLib had a long discussion about how weird it is.
 * Branden: Okay, I think this is good to go then.

## 15.4 Libtock-rs Tests
 * https://github.com/tock/tock-hardware-ci/pull/50
 * Branden: Status of this? Is it still waiting?
 * Leon: Waiting on me. There's a bunch of work outstanding on the Treadmill infrastructure, particularly for long-term stability and where to we physically install it. This PR can move forward before that though.
 * Tyler: I kicked off the CI tests to evaulate it, and it passed no issue.
 * Leon: Treadmill 15.4 tests are running nightly fine, right?
 * Tyler: Yup. Ran last night and passed almost every days. Failed three days in the last 6 months
 * Leon: Awesome. No recent failures of the system.
 * Tyler: The changes are just adding the libtock-rs tests. To complement the existing libtock-c stuff

## IPC
 * Branden: Still super interested, still just need engineering work. But I've been swamped and am unlikely to get more time before summer.
 * Leon: Similar. I've been involved in a course taking a lot of time this semester. Moving forward on some IPC research now though.
 * Leon: I just did a pass on the reference document. https://github.com/tock/tock/pull/4680
 * Leon: I found an old conversation about ProcessIDs that we need to continue on. Discovery and Communication are bound by some IPC Handle, gotten from discovery and given to communication. That handle is the ProcessID, which right now is a 32-bit integer. The problem is that 32 bits is technically a value that could overflow.
 * Branden: 4 billion is a pretty big number. A restart every millisecond means ~40 days before an issue.
 * Leon: It's conceivable though. Also it's a `usize`, so semantics change based on platform size.
 * Branden: Couldn't we just have an if-check for that?
 * Leon: Then processes could restart themselves enough to crash the kernel. Which is bad.
 * Branden: Oops. Yes
 * Leon: So we make it 64-bit. And overflow goes out of scope.
 * Branden: Downsides?
 * Leon: Arithmetic could be expensive. But +1 is easy and compilers fix it. Comparisons are fine. It'll increase memory use by 1 machine word wherever a Process ID is used. Worth considering. And it might be that we don't have enough registers to communicate this with userspace for the current IPC mechanism. It could affect register pressure on new mechanisms too.
 * Leon: So, it could be good to reason about this first, before we build the basic mechanisms.
 * Branden: As long as we have one more available, then the change becomes trivial.
 * Leon: Yeah, we could watch that. Should be easy to append.
 * Leon: I'm also watching for other things that think about ProcessIDs and expect them to be unique and meaningful. Maybe Grant entry. Thinking about this for half an hour has me convinced that we want to avoid it altogether with 64-bit IDs.
 * Branden: Yeah. I agree that we don't want to reason about it. Doing 64-bit is fine, just feels unnecessary.
 * Leon: It's the number of restarts all time, not just the number of processes.
 * Branden: So if restarts are often, and kernel runs for a long time, then this is relevant.
 * Leon: So we could work on communication mechanisms now
 * Branden: Yes. If we get async mailbox and sync mailbox done, we could push this all to Tock and have something usable. Shared memory will be a big discussion and design point, but right now the focus is on getting stuff working first.

