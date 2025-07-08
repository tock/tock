# Tock Network WG Meeting Notes

- **Date:** October 19, 2023
- **Participants:**
    - Tyler Potyondy
    - Branden Ghena
    - Leon Schuermann
    - Felix Mada
    - Alex Radovici
- **Agenda**
    1. Updates
    2. Reserving Ports
    3. Buffer Management
- **References:**
    - [#3683](https://github.com/tock/tock/pull/3683)
    - https://github.com/lschuermann/packetbuffer/blob/12bf7e3959b96354089f8250edf5357c3c1afb72/src/lib.rs


## Updates
- Tyler: Thinking about capabilities for radio control. PR is still coming soon.


## Reserving Ports
- Tyler: Thread sends Mesh-Link Establishment (MLE) messages that keep the mesh attached and determines parent nodes and stuff. Those messages are sent over UDP, with port 19788. So for Thread to work at all, it MUST have control over that Port. So I was thinking about whether Thread should exclusively reserve this port, or if anyone can take it and we just bind to it.
- Tyler: So the general question is, if a capsule requires a certain port, should it reserve it in some way or just request it and fail?
- Branden: Ports being reserved for applications is normal. HTTP, SSH, etc.
- Leon: Does this only affect things if Thread is loaded, or would it be a change to all UDP stack usage even without Thread?
- Tyler: Tricky. Right now the network isn't created until you request from userland to create one, so it binds to the port at that point. There's a chance that someone else has bound the port by then. It's annoying to debug.
- Tyler: I'm leaning towards just letting thread fail at that point, but wanted to discuss it.
- Leon: In other systems, for well-established protocols when your application chooses to open a raw socket and conflict with something in the kernel, is that the kernel silently wins. Not sure this is right, but it's reasonable to say "the kernel always wins". Failing instead of silently not working would be even better.
- Tyler: So you think it's okay for thread to fail if another application takes the port before it?
- Leon: Maybe, we are worried about denial of service where on application just blocks others. The other option is that the kernel can just yank ports from applications. I don't like unconditionally reserving though, as there are all kinds of weird protocols which need various source ports.
- Tyler: So you're proposing that we'd let a UDP application bind to it if thread hasn't yet. But then if we want to create a thread network, the thread network could take the port and make the other application fail.
- Leon: I'd be fine with that. I'm really just saying that other OSes do this
- Branden: Could you claim 19788 at initialization time? Initialization-time failures are generally preferred.
- Tyler: That would be what Leon is against.
- Branden: No, only when you actually load Thread. When you don't include Thread in your board's main.rs file, the port is free to be used by applications.
- Leon: FWIW, I prefer this solution.
- Tyler: Do you think this should be a panic, Branden?
- Branden: If you can conclude immediately that something's wrong, a panic seems reasonable.
- Leon: Doesn't this conflict with dynamic binding of applications to ports?
- Branden: Those applications would be too late then. Should be second-class to things that are known at configuration time. The scenario we're talking about here is a board which has both $protocol and Thread, and both claim it. Then we should panic. Similar to if the kernel registers two HTTP servers on the same port.
- Tyler: Just to clarify, `main.rs` initializes everything fully?
- Leon: In practice yes, but not necessarily
- Tyler: Does that cause an issue too? That would be a weird runtime-versus-static conflict. UDP is always initialized before Thread, but hopefully processes are after that?
- Leon: Yes, processes are after that. But conceptually you could instantiate a kernel peripheral later, and that case would have two reasonable APIs: the bind-to-port API should return an error and there should be a force-bind API which replaces a currently bound port. Ultimately, these nitty-gritty details of exact error behavior and semantics can be determined by the dev based on how they wire things up. We do always complain about how complicated `main.rs` is, but as a benefit, you do have lots of flexibility.
- Tyler: So moving forward, bind to 19788 in the component. And panic if it's already taken.
- Branden: If you can make it that the API from Thread returns an error, but then the component panics, that'd be a great design.
- Tyler: Yeah, the component would check for failure and panic.
- Branden: That's ideal
- Leon: Agreed. We have the components as a default, but they're not at all required. So others who disagree can instantiate peripherals themselves.


## Buffer Management
- Leon: https://github.com/lschuermann/packetbuffer/blob/master/src/lib.rs
- Leon: Specifically, https://github.com/lschuermann/packetbuffer/blob/12bf7e3959b96354089f8250edf5357c3c1afb72/src/lib.rs
- Leon: Some work in progress here. Doesn't compile, might not be around forever. Types are maybe not beautiful right now, but I tried to pull our discussions into this. So this is a set of type abstractions which could set up the buffers we talked about.
- Leon: PacketBuffer is the main type here. Generic over whether it's contiguous and how much header space it has. Header isn't necessarily the true space, but at least the minimum available space.
- Leon: We'll add methods here. We might have methods to give us a raw pointer for DMA use for example.
- Leon: This is a trait as it could be backed by Rust slice or Rust struct.
- Leon: PacketBufferEnd holds no data, always contiguous, no header room, it's a dummy end element for linked lists
- Leon: PacketSlice is the sliced instance of a single element from a PacketBuffer. So it's got generics for itself and generics for the _next_ PacketBuffer in the linked list
- Leon: Similarly is PacketArray, which has contiguous and a fixed length generic, plus again stuff for the next one.
- Branden: So PacketArray and PacketSlice are implementations of PacketBuffer (yes)
- Branden: Why linked lists?
- Leon: Most generic implementation. This lets you combine things in arbitrary arrays, like header followed by data, followed by more data, etc. Even for hardware without linked buffer support, you end up with a two item list, where the second is a PacketBufferEnd, which is a zero-sized type.
- Leon: Going into the example here, we can make a slice, make a PacketSlice over it, and that uses `from_slice_mut_end()`, which implicitly creats the PacketBufferEnd for you to stop the list
- Leon: We say here that the header has 32 bytes, so that 32 is taken away from the full size of the slice so it's reserved.
- Leon: To pass this to something that has a headroom requirement, we can move headroom around as needed. This should take no runtime overhead, but fail at compile time if headroom is too small. 
- Branden: Can you ever get back to the bigger size?
- Leon: Working on that. There was a compiler bug with which trait it was calling. I'll fix that at some point. Totally doable.
- Leon: Back to example, we can make a PacketArray around our PacketSlice. Since it's two buffers now, it's got to be non-contiguous. If you put True there you'll get another compile-time error.
- Leon: Slices do know their original sizes and can be restored, but that's a runtime check. That's a destructive operation. It might eat some of your data if you're not careful.
- Leon: Making smaller headroom is non-destructive as it should check at compile time.
- Alex: So the array next packet buffer would be prepended?
- Leon: Yes, but non-contiguous
- Alex: So could you add space for footers? Maybe non-contiguously appending buffers?
- Leon: Not in its current form. I'll have to think about that. It's a lot of linear type composition right now, like linear list operations. We're only really modifying what's in front. So the magic here is that we still have the old type hidden deep in the type somewhere. We overwrite by pre-pending the new type.
- Alex: If you can pre-pend like this, do you still need the headroom? For contiguous things
- Leon: I think footers are totally doable
- Alex: We need to abstract this away somehow
- Leon: I'm hoping type inference saves us. Hoping hard
- Branden: Maybe a macro to create this for us?
- Leon: Yes
- Leon: I will have to think about footer changes though
- Alex: There's no footer for ethernet or IP? (no)
- Leon: There is a 32-bit CRC, but that's usually done by the MAC without allocated space
- Branden: I think footers are going to have to exist for a general solution
- Leon: Okay, adding one more confusion. So, how does this work for structs and clients passing stuff.
- Leon: I have an example mimic-ing that. Dispatching a buffer and handing it back. This example is rough, but perhaps understandable
- Leon: We have a ImANetworkLayer which is a lower layer like IP. Has minimum requirements imposed by the lower layers beneath it. It accepts a PacketBuffer generic over those same constants.
- Leon: We can't directly take clients, but need adapters that do the type conversions.
- Leon: I have a higher-level adapter. This does the conversion from one layer to another, taking BOTH generic sizes. Contiguous must match between the two layers, but headroom doesn't.
- Leon: We have a reference to the type we actually want to call methods on.
- Leon: When we actually want to dispatch a buffer, it gets passed a higher-layer buffer type and converts it to the lower-layer buffer type seamlessly with a shrink call
- Leon: I haven't written the upcall part yet, but that would do the same adaption but with restore call
- Leon: All of this conversion should be almost zero cost, except for the restore which needs a size comparison.
- Leon: Still working on moving some things between traits to make code compile, but promising
- Leon: I'm really hopeful that most things inline and vanish totally. We'll have to do a measurement though
- Branden: So after types and Rust fighting, we'll definitely need to focus on ergonomics
- Leon: Two aspects there, 1) understanding and juggling types and 2) making it nice
- Leon: I think the understanding isn't so bad with documentation. Writing it and fixing compiler annoyances is hard, but the WHY isn't so bad
- Leon: What I'm really worried about is that it might look really ugly
- Branden: Again, I'm really hopeful for macros helping us here. The entire adapter looks like stuff that's autogenerated. Could turn into something that's a single `generate_adapter!` macro
- Leon: Optimistic. 
- Branden: I do want to try this out before ergonomics. We want the lower stuff solid before thinking about exactly how people are going to use it
- Leon: Still need to develop larger, do some measurements. Then we need to actually apply it and see if it works. Finally, even if all of that works, ergonomics will be key

