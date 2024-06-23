# Tock Network WG Meeting Notes

- **Date:** April 15, 2024
- **Participants:**
    - Alex Radovici
    - Tyler Potyondy
    - Branden Ghena
    - Leon Schuermann
    - Felix Mada
    - Amalia Simion
- **Agenda**
    1. Updates
    2. PacketBuffer
- **References:**
    - None


## Updates
### 15.4/Thread Status
* Tyler: 15.4 PRs have been merged.
* Tyler: Something I've been working on is setting up a small OpenThread network that will run some automated tests on whether Thread still works whenever I update things. Testing 15.4 would also be helpful. It's easy for that driver to pass CI, but fail in the real-world. Maybe eventually this could even be part of the hardware CI system.
* Branden: Sounds awesome and super valuable. Just don't let it take too much time from the tutorial.
* Tyler: Agreed. Focusing on a minimal version
### Tutorial Documentation
* Leon: Where are we in tutorial docs?
* Tyler: It's fallen a little behind. Working on that this week as a major goal. Outline and application are sketched out, we need to start writing things
* Leon: Working on SOSP deadline now, but next week I'm going to focus on this
* Tyler: Everything does seem to be working for the OpenThread port. So it's definitely time to get the tutorial stuff working
* Leon: I'll be at the tutorial, as well another PhD student from my lab.


## PacketBuffer
* Alex: We tried to integrate the PacketBuffer stuff, but we had to place a LOT of constant generics in the upper layers. We need to somehow pass down all the constants. Without const-generic-expressions, we're stuck doing this. I can't add one layer on top of another without knowing how much it might append
* Leon: I previously had a solution where we break layers into two parts, one to pass down buffers and one to pass up buffers. The goal was to avoid the requirement of knowing the head/tail requirements at all points.
* Alex: Please share. I'm trying to figure this out. I tried doing associated constants, but that didn't work. Macros also didn't fix it. The upper layer needs a buffer with a head/tail. Then the lower layer needs a buffer with a different head/tail.
* Leon: Each layer does need to know the head/tail room of the layers above and below it
* Alex: This isn't enough. The virtual uart device needs to take the mux which needs to take a trait of the underlying layer. So I think they have to be everywhere.
* Leon: Oh, I do think that's an issue. This is a similar issue to composing things in main.rs, which probably need type aliases to solve them, in an ugly way. I don't know that there's another way
* Alex: Const generic expressions would help. But not stabilizing anytime soon
* Alex: I was unsure whether generics are worth it for the buffer
* Leon: Generics make it hard and ugly to instantiate things, but they do allow us to be composable
* Alex: Well, just for the buffer it might not be worth, we can always use a struct with variables.
* Leon: That would be the linux kernel's skbuffer. But it wouldn't be compile-time validated. And some operations can work without run-time checks.
* Alex: I agree. I do think this is worth it. But if we ever get simple const-generic-expressions, that will make this even more valuable
* Alex: It's going to be pretty ugly to instantiate. I hope there are some type aliases or something to clean that up.
* Leon: We could pass the dynamic trait object. We might consider whether the upcall path should avoid arguments with const generics, and just takes the trait object
* Alex: The constants won't be in the type on the upcall path anyways.
* Leon: Okay, we should definitely do trait objects on the way up. And there's a helper function that does that
* Alex: Yeah, better than a string of like 6 constants. It's important for everything using this to take generics instead of trait objects though
* Leon: To summarize: we have a problem that the way we compose layers is with proper generic types, avoiding dynamic trait objects. So something like the Debug writer takes a generic type that implements the uart transmit trait, like the mux. So each layer needs to know how much headroom the layers above and below take. But that seems fine because what we could do is have all of these types represented as type aliases. On the upcall path, we would need to know how much headroom to restore on each additional layer, so the trait object would need ALL of the headroom from all of the layers. These buffers are wrapped in a type that has the generic arguments. So we could unwrap at each layer and pass the non-generic types up the stack instead.
* Alex: I'm still not sure how to deal with this if the lower UART isn't really a UART and needs another layer. With dynamic traits this gets really bad
* Leon: We should avoid those
* Alex: But on the upcall path, we need the dynamic trait, or it would be a circular reference
* Alex: Amalia did all of the work here, but it's good that we're finding issues
* Group: We discussed this problem in more depth with some quick examples. Here's the high-level takeaways:
    * Most layers would need four generic constants: current head/tail room and lower head/tail room
    * Some layers might needs six generic constants: upper, current, and lower
    * Instantiation of these in main.rs would be ROUGH. All constants have to be listed. Would need type aliases and maybe macros to clean this up
    * Overall, going to make code a little harder to read/understand, but for the value of compile-time checks
    * Const generic expressions would reduce this to two (or sometimes four maybe) constants, but we can't wait on it
