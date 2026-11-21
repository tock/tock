# Tock Network WG Meeting Notes

- **Date:** October 26, 2026
- **Participants:**
    - Branden Ghena
    - Leon Schuermann
    - Tyler Potyondy
- **Agenda:**
    1. Updates
    2. IPC Planning
    3. Flash HIL
- **References:**
    - [Flash HIL Update](https://github.com/tock/tock/pull/5310)


## Updates
- None


## IPC Planning
- Branden: I have some students around who will be working on Tock IPC, mostly in the Winter. My plan: first create the new registry (with board-specified service names) and then async mailbox using StreamingProcessSlice.
- Branden: Then/in-parallel work on memory-sharing. Not generically, but just special-purpose proof-of-concepts right now so we can see what does or doesn't work well
- Leon: I did previously recommend that we go the route of arch-specific MPU implementations
- Branden: I'm still hoping to be generic in the end, but hopefully these will inform that design. Worst-case, we'll have these if we do have to have chip-specific designs.
- Branden: Asking for advice on difficulty and how this will go.
- Leon: My calibration for difficulty is thrown off by LLMs. Thinking through the architecture and ramifications will be important, whereas making something that works will be relatively easy
- Leon: Generally, assume LLM involvement in the generated artifact, so you have to probe solution for deficiencies deeply. Creating stuff is quick, but understanding and reasoning comes slowly.
- Branden: What about working with the ARMv7 MPU?
- Tyler: ARMv8 MPU is easier to reason about. ARMv7 is rougher. Hard to keep all the context in your mind as you work on it.
- Branden: Do you have a board for the ARMv8?
- Tyler: No, the nRF54 is ARMv8 though.
- Branden: Ugh. I don't want to be the first to work with it though.
- Leon: nRF53 was recently proposed in a PR in Tock. Different set of tradeoffs in the space than the nRF54s make. For Tock, multi-core systems are fairly interesting.
- Tyler: Cortex-M group from embedded working group has an abstraction for the MPU, but it's just a wrapper around the registers. So it's up to you to implement stuff. Using the Tock MPU is much better actually
- Branden: Okay, thanks for the advice. I'll likely invite them to this meeting, probably starting in the Winter, maybe late Fall.


## Flash HIL
- https://github.com/tock/tock/pull/5310
- Leon: Looking into building gadgets with Tock, and smart home stuff. OpenThread wireless network
- Leon: This requires applications to store wireless credentials. Initially used TickKV. Switched to isolated non-volatile storage. Turns out with have 8+ implementations of flash controllers in Tock, and three users: TickKV, Isolated non-volatile, and a Log. And they all make different assumptions about how this should work.
- Leon: For example, TickKV assumes a write to a page flips bits from 1 to 0. And issues an explicit erase when it needs to flip a 0 to 1. This happens to work on the nRF52 as it erases all of flash, writes to a RAM buffer, then writes to flash later.
- Leon: Another chip, doesn't do an internal erase. Which then broke those assumptions
- Leon: Talked to Brad about this. Flash has been in a mostly-broken state for a long time. I've been focusing on specific issues without making big architecture changes so far
- Branden: Not exactly surprising. Disappointing that HILs have hidden assumptions within them.
- Leon: Not even clear what those assumptions are, and every user has different assumptions. Plus, there's a LOT of heterogeneity in Flash chip implementations. And people come around, fix their board-specific problem, and then things get worse and not better
- Leon: I've got some updates, but this is going to need a lot of eyes on it and deep thought to make it work cross-platform. Ultimately I just want to bring attention to the problem.
- Branden: And this is super relevant to network stacks. OpenThread needs this. LoRa needs it too, and I believe Amit ran into similar problems recently.
- Leon: Some ways to approach. We can get everyone to look into subsystem independently. You can look at the issue. Or I can do a three-minute summary on proposed solution.
- Branden: Let's start with that last one
- Leon: First change is easy. We have an EPROM chip that supports byte-reads and writes without any erases. I think that's just not a flash chip. We should define Flash HIL as something that has erase/write semantics. So the EPROM would be under the non-volatile HIL directly, and not under Flash.
- Leon: Block-device interactions over Flash seem reasonable. You could reserve a couple of pages in the Flash for house-keeping and then you could do wear-leveling. There's an adapter block-device-to-flash, which provides the abstraction on top of Flash
- Leon: An EPROM which doesn't need an erase before a write, isn't a Flash. It should just be a block device directly.
- Leon: Second issue: we have a bunch of Flashes which erase pages, then you can write to that erased page at least once. nRF52 does this. After you erase a page you can write any byte in that page at least two times (toward more zero bits).
- Leon: Other Flashes let you write any number of times, as long as we only flip more bits to zeroes.
- Leon: Other chips would let you write multiple times, but because they use ECC you can only write once in actuality, because the checksum would need more 1 bits.
- Branden: So one write per erase is the abstraction that works for all of those.
- Leon: Flash HIL should be modified to only allow one write per erase, and separately have erase and write functionality which is explicit. No implicit erases. Writes without a preceding erase should throw an error
- Branden: You could keep a bool to track whether there's been a write since erase
- Leon: Third problem, erases are really expensive in terms of endurance. Some chips only allow 1000 erases, which is a crazy low number. For many chips, you can determine whether a page needs to be erased. So there should be a HIL function that lets you check if a page needs to be erased before writing it.
- Leon: In my current proposal, the driver doesn't track things internally. Instead the user of the Flash HIL can ignore the check right after doing an erase.
- Leon: The optimization I'm thinking of, is if you do erase-write-erase-write, bookkeeping in Flash doesn't know if user always does that pattern.
- Branden: What's the other option?
- Leon: Application could know that it never does two writes in a row. Then the flash driver doesn't need to track it
- Leon: Some flashes do support unlimited writes as long they just flip ones to zeros. And for some users, it's really useful to have that interface. So I added a marker trait for those systems which makes it clear that it's permitted to issue multiple writes as long as they only flip ones to zeros.
- Branden: That's important to have?
- Leon: Yes. Many systems rely on this to avoid too many erases. TickKV relies on this.
- Branden: I'm questioning whether bit-clearable should be a separate HIL. Or partial HIL.
- Leon: I think it's a strict superset.
- Branden: Annoying that the "can you write it" function returns "maybe" for this case
- Leon: I do think of that "can you write" function as only an optimization. Function just allows you to occasionally skip an erase
- Branden: You could make "can you write" take in a block to check with. But that's pretty inefficient
- Leon: Crazy inefficient actually
- Leon: In the end, I'm proposing to slightly change the HIL. A new function plus some comments about invariants.

