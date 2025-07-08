# Tock Network WG Meeting Notes

- **Date:** November 16, 2023
- **Participants:**
    - Felix Mada
    - Tyler Potyondy
    - Branden Ghena
    - Leon Schuermann
    - Alex Radovici
- **Agenda**
    1. Updates
    2. Tock Networking Tutorial Proposal
    3. Buffer Management
- **References:**
    - [Packet Buffer Approach 1](https://github.com/lschuermann/packetbuffer/blob/d51ef6b922a18d6a7a64c8e0585237845892dba1/src/lib.rs)
    - [Packet Buffer Approach 2](https://github.com/lschuermann/packetbuffer/blob/cfd38a50ead0f7608dd053fe850d2e3d6bd91e85/src/lib.rs)


## Updates
- Leon: Merged STM Ethernet support. Thanks to OxidOS folks for development. Very high quality. On todo list is to rebase Tock-Ethernet on Tock master and make a PR to merge everything.
- Tyler: Doing a master's thesis writeup on Tock/Thread stuff.


## Tock Networking Tutorial - CPS Week
- Tyler: Pat and I have planned a Tock/Thread networking tutorial at some conference, focusing on CPS-IoT Week in Hong Kong this year, May 13-16 2024. Larger academic audience. Tutorial would be demonstrating Tock as a platform you could use for IoT research, rather than developing Tock. Tutorial proposal is due tomorrow.
- Leon: Interesting. I'll see if I can get involved in this, need to check with Amit
- Tyler: I'll circle back with Pat to think about this
- Branden: Concern is that Tyler might not have any support here. Maybe he'll get support on the demo, but if no one else goes to Hong Kong, then everything falls on Tyler in the end
- Leon: What do you expect the audience to be? Number of attendees and familiarity with embedded/Rust
- Branden: It's a big mix of people. Lots of folks from embedded/networks, but also just as many folks from Real-Time or CPS Theory. You'll get people who don't even know command line.
- Branden: Bonus concern is whether Tock is ready for IoT work yet. A little early since some of it doesn't exist.
- Leon: We've really focused on soundness and security, rather than networking so far. We would need to invest more time into development for a proper IoT tutorial.
- Alexandru: Could likely send someone to CPS Week in May.
- Tyler: Hope is that the OpenThread port could be pretty developed by then.


## Buffer Management
- Leon: Last time we talked about this, I gave a rundown of the type infrastructure I tried to create to capture our requirements. We concluded that this seems generally promising but still had some issues. Plan today is to page in some of these issues and highlight some architectural changes.
- Leon: Approach 1 - https://github.com/lschuermann/packetbuffer/blob/d51ef6b922a18d6a7a64c8e0585237845892dba1/src/lib.rs
- Leon: To go over this, we have a PacketBuffer trait. Generic over whether it's a contiguous buffer (array) or non-contiguous (linked list), and that amount of headroom we have (buffer space available at the front). We have the ability to shrink headroom, to reduce the headroom, claiming some of it for writing data into.
- Leon: This commit also has an example Network layer which has some requirements that it's contiguous and has some amount of headroom. Then we had a concept of a higher-layer adapter which takes a type that has a larger headroom and reduces the amount to the exact amount required by our network layer and passes it downward. The adapter handles both passing down and up the stack, is capable of converting a buffer back to its original type.
- Leon: So the network layer stores a number of higher-layer adapters, which take on the role of a "client".
- Leon: This solution has an issue. Does not compile due to nasty issue. In theory all of this makes sense because we can statically determine if headroom is strictly larger, which allows a type conversion. That works. The issue though is that by downsizing the high-level headroom of the PacketBuffer, we get a type back that's generic over the right types, but we don't actually know anything else about the type except that it implements the interface. We don't know if it's the same type as type NPB here. Just because a type implements the same interface in Rust, that doesn't mean it's an identical type. We can create a type from a higher-level, but we can't prove it's the same.
- Leon: Approach 2 - https://github.com/lschuermann/packetbuffer/commit/cfd38a50ead0f7608dd053fe850d2e3d6bd91e85
- Leon: So, we can switch to `dyn` types. Downsides: needs vtables which is more flash and slower speed. Also can't have features like associated trait methods, or methods that don't take a self parameter. Might make some of the features tricky to implement.
- Branden: Why does that matter?
- Leon: You can no longer have associated constants, types, or static methods. I believed those to be useful for some of our types, like iteration. It may be that this is not an issue, but it's a change from my prior implementation attempt.
- Leon: Now we need to make new functions, not associated with anything, that convert from one `dyn` type to another. That does seem to work.
- Leon: For the example from before, we can have a method in the higher-level adapter that takes in a PacketBuffer, which is a concrete type from a higher-level layer. And we can pass in a static reference to this type. Then we can convert that reference into a `dyn` reference and pass it to the lower layer. But, if we have a static mutable reference, and take a static mutable `dyn` reference, we lose the information of the original reference, and we can't reconstruct the original type.
- Leon: So what doesn't work when we use `dyn`, is that we can't have a "pass buffer back" which goes from `dyn` back to the original type. So using trait objects works generally for our scenario. And it works for changing headroom and passing down. What doesn't work with trait objects right now is taking the buffer and converting it to its original types. Especially with static references to these buffers.
- Tyler: When you're saying the original types, what do you mean by that? Can you give an example?
- Leon: Yes. So an original type on line 416 here, we take a chunk of bytes and turn them into a slice and makes a PacketSlice type which holds the buffer and sets aside some amount of headroom. Then we want to shrink the headroom when passing it down. So we pass in a mutable reference and get back a `dyn` reference. But we eventually do need to get back to the original type at some point to release the slice.
- Tyler: There are also two lifetimes in the PacketSlice? Why are those underscores?
- Leon: Yes. I'm not yet at the point where non-static lifetimes work for these conversions. Work in progress.
- Leon: So in summary, we solved the first half of the issue and did shrinking, but we can't yet unshrink.
- Leon: There is a different proposal where in each layer when we convert a buffer, we store a "shadow copy" of some information about this buffer. So when we pass a buffer back up, we can use the information in this copy plus the reference, to be able to reclaim our original type. That's what I'm working on now.
- Leon: So we have a `capture()` function which can claim a buffer and give out a `dyn` reference. And a horribly unsafe `restore()` function which takes a reference and gives back a buffer. What we're trying to do is that whenever we do a cast from type to trait object, we need to save the original type somewhere.
- Alex: Two questions here. There's some kind of downcast in Rust if you know the type right? Second, doesn't every type have an identifier? Could we store that?
- Leon: Yes. Two lines of thoughts here, but I think you're on the right track. First, Rust's references, especially static references, have guarantees about memory not being reused. So we can maybe store a pointer to that struct and get some guarantees due to it being a static lifetime. Second, we can hopefully use Rust's type IDs, plus that trait objects have a reference to a type's vtable, so we can uniquely identify the type.
- Alex: So you could check, and then panic
- Leon: Yeah. We return an option right now. There are checks, which I haven't figure out what are yet, but we could do the checks and then perform the operation.
- Alex: That seems sound to me
- Leon: If we can guarantee the uniqueness of the type ID. And we respect inheritance rules
- Alex: Doesn't Rust have a downcast function which should do this? The Any Trait. I _think_ it should do what you're trying.
- Leon: I have tried to use this in my previous projects unsuccessfully. But it does sound like what you want. Skimming some documentation, it's achieving what I wanted, but safely
- Branden: So in summary, we can create a real thing in a higher level. Then we can pass around trait objects everywhere and do conversions on them. Finally, when it gets to the very top, we can convert back.
- Leon: And we still have static compile-time analysis here for almost everything.
- Leon: One runtime failure. If your network card has multiple outstanding requests and swaps buffers by accident and passes the wrong type back up, the upcall conversion path will fail at runtime.
- Leon: Another thing I haven't seen yet is what the
- Branden: Do we still need the adapter layer? If translations are a standalone function
- Leon: Still needs the capture idea somewhere, could exist in the higher layer maybe? Could have a method that just takes a dyn refernece and a method that returns a dyn reference. If we pass the dyn reference into the client, the client knows the type it needs to recreate. The type is implicitly encoded by using this particular client method.
- Alex: Yeah, it would still call the "restore" method, but it would be generic and Rust would figure it out at compile time
- Leon: Hmmm. Hard to think about live.
- Alex: Is there any way we can do the capture/restore with the from/into or tryFrom? Does the assert break this?
- Leon: The assert doesn't matter. The function could be in an into trait, and we'd panic if it's not true, but the assert only relies on values known at compile time, so the rust compiler will evaluate at compile time. So I think we can put this in any trait. The only ugly thing about this right now, and Rust is actively discussing this, this fails when you compile but not with a cargo check. So it'll only panic when actually producing a binary. The reason for this is that the Rust compiler front-end doesn't do the monomorphization to expand these constants. That happens later, so the check doesn't notice it. Tock CI would still catch it though.
- Leon: Thanks for this insight Alex. New todo for me is to try the Any stuff and see if it breaks.
- Leon: The current code is terribly unsafe, but it does compile and is facially sound.
- Alex: A question is where this will be placed. In a library? Maybe question for next time.
- Alex: I will talk to someone about trying this on the display driver.

