# Tock Network WG Meeting Notes

- **Date:** December 14, 2023
- **Participants:**
    - Tyler Potyondy
    - Branden Ghena
    - Leon Schuermann
    - Felix Mada
    - Alex Radovici
- **Agenda**
    1. Updates
    2. Buffer Management
    3. Next Meeting Plan
- **References:**
    - [PacketBuffer Mockup](https://github.com/lschuermann/packetbuffer/commit/acea95a5b28ebc5343289e1b62144bd89b153715)
    - [Trait Casting Playground](https://play.rust-lang.org/?version=nightly&mode=debug&edition=2021&gist=2c7fb2971338e478d5d9efadd821372f)


## Updates
- Tyler: CPS-IoT tutorial for Tock is accepted! Any advice is appreciated.
- Leon: Have some undergrads here that could help with it.
- Branden: Sooner is better. Especially if you have a design, you can pull in others to help build stuff.
- Alex: Tyler should reach out to me for planning on students.


## Buffer Management
- Leon: https://github.com/lschuermann/packetbuffer/commit/acea95a5b28ebc5343289e1b62144bd89b153715
- Leon: To review, my original proposal used PacketBuffer types and changed the generic arguments with regards to headroom when passing around. The problem is that Rust treats the types as entirely different for monomorpization. So we can't convert types, even if they implement the same trait, because the compiler could be doing entirely different things.
- Leon: The solution to this is `dyn` traits. But when we pass around static trait objects, we lose access to the underlying type and there's no way to reconstruct it.
- Leon: As Alex mentioned, the `any` trait in Rust can get the TypeID for a trait, which can let us convert a reference for a unknown type back into a proper type if those IDs match. That has a guarantee that things will actually match.
- Leon: I implemented that, which wasn't super straightforward. There was actually a PR in December that enabled this in Rust. So current stable breaks, as trait upcasting is "experimental". But for nightly this moved all the way to stable, with no feature flag. It'll be in the next rust release. Marked for stabilization.
- Leon: Playground with example: https://play.rust-lang.org/?version=nightly&mode=debug&edition=2021&gist=2c7fb2971338e478d5d9efadd821372f
- Leon: So now we have some trait and a struct that implements it. In the main method, we create a static mutable reference to that struct. We can even create a `dyn` trait object from that reference. To convert back from the trait object to the original type, I tried to use the typecasting infrastructure by getting the type ID and doing a comparison for specific reference types. If they are equal, we can transmute, right? That was my first attempt in `safe_upcast()`.
- Leon: However, it turns out that the two IDs never match. That was weird to me because they _are_ actually the same type. But it turns out taking the typeID of a trait object won't match the ID of the proper type. It encodes that it is a trait object in the typeID. So the only way to see if TypeIDs line up in Rust is with a proper `Any` object, which overloads the `.type_id()` field to reflect the proper underlying type.
- Leon: The new feature just merged lets us do trait upcasting, which is if the trait has a strict superset trait requirement that they must implement `Any`, then we can convert from trait object into an `Any` trait object. Then we can use the `downcast_ref` method to convert back into a struct.
- Branden: `downcast_ref()` panics?
- Leon: It returns an option actually!
- Tyler: So bigger picture, what's the reason for the upcast downcast?
- Leon: For our real use-case, we have a PacketBuffer with requirements encoded in the generics of the type. We want to convert implementations to traits that can be passed into a shared network layer. That works by just converting types into trait objects. On the way back in the upcall though, we don't have a way right now to convert the trait objects back into real structs. And we need the real struct to access the underlying buffers again.
- Leon: So what I've done is implemented this particular way of converting Any trait objects into proper types. One caveat is that Any is only implemented for `static` types. So we can't use lifetimes everywhere and all buffers have to be static. This would be a bummer, but it's the Tock reality anyways.
- Leon: So now we can shrink buffer headroom when passing down. The layer gets the trait object, and then right now just passes it back as a mock-up. Then our higher layer we convert into a dyn Any reference, then `downcast_mut` to get the original type. Or if it's wrong it panics right now.
- Tyler: That makes sense now
- Leon: There is one tiny bit of unsafety, and maybe unsoundness. `shrink_packet_buffer_headroom()` checks that the parameters for the buffer are valid such that it _could_ shrink in size. Then it does an unsafe transmute. While that does make sense conceptually, technically we're still converting between different types. So it's important for us to be sure that we can do this conversion without breaking any Rust invariants. One issue: Rust could choose different layouts or methods based on the generic parameters. We can prevent that by making a wrapper around our type. These parameters are markers, which shouldn't affect layout. So we should be able to have the PacketBuffer wrap a type which isn't generic over the parameters. So the method can pull out of one wrapper and put into another wrapper.
- Alex: So you'd do a move? Construct a new type and move the contents?
- Leon: Yes. One additional thing though, we'd have to pass back the exact same memory space. The cool thing in Rust though is that if you have a mutable reference you have full control of the memory. So we can swap in a dummy type, then change the thing, then swap back.
- Leon: So we have to receive and pass back the exact same pointer. But mem::swap will let us take a type out of a memory location, extract the buffer by destructing it, make a new type, and then swap back in to the location. So we just have to guarantee that the types have the same layout.
- Alex: Does this work here?
- Leon: I believe so. We'll only swap the internal type.
- Branden: And that can replace the transmute.
- Leon: Yes. It _probably_ creates valid binaries as-is. But probably isn't a good enough guarantee
- Leon: So testing is what's next. Testing on the screen interface would be useful.
- Alex: This is all on github?
- Leon: Yes, you can use this right now
- Leon: And this should be stable very soon. Unless something is broken with its implementation.
- Branden: Why is the function called `downcast_mut` anyways?
- Leon: Any is a super type. So we're going to something totally generic, then "down" to something specific
- Branden: So this requires static? Is that an issue?
- Leon: I think the possible use case is "what if we just wanted to add a small header to something". That could be local allocation. Is that what you're talking about?
- Branden: There were some case where passing slices made sense instead of everything
- Leon: So here we can still pass the whole thing but do operations on it that feel like slicing. The only reason I can think of why we want to pass a slice is to dynamically pull chunks of memory from a pool. But _that_ pool would be static, so we could make those static probably
- Alex: We might need some kind of dynamic allocation for network stacks
- Leon: Agreed. Avoid if possible, but we'll deal with that when we get there
- Branden: Something really great here is that the transformations are very minimal magic. Like three lines of code and it's all casts. Having the PacketBuffer not feel very magic will help get people to use it. It's encouraging
- Leon: Plan from here is to make the wrapper so I can get rid of the transmute

## Next Meeting Plan
- Branden: We'll meet on January 11th for our next meeting. Same time
- Branden: After discussion, it sounds like we're good through January. Might reschedule time for February onward if needed.

