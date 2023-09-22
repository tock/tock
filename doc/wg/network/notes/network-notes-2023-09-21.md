# Tock Network WG Meeting Notes

- **Date:** September 21st, 2023
- **Participants:**
    - Alex Radovici
    - Felix Mada
    - Branden Ghena
    - Tyler Potyondy
    - Leon Schuermann
    - Cristan Rusu
- **Agenda**
    1. Updates
    2. Continue discussion about PacketBuffers
- **References:**
    - [Thread Network Child Device](https://github.com/tock/tock/pull/3683)

## Updates
- Tyler: The Thread Child joining PR (https://github.com/tock/tock/pull/3683) is shipped, hopefully gets merge during the next week, Pats needs to look at it, I still have a quick question
- Branden: Is this the last PR for Thread? This gets you to a full working Thread Child implementation? You had some earlier PRs to get this work?
- Tyler: It does not have UDP yet, it just shows up in the Open Thread network. I rewrote the whole driver, after 4 months of working on it. I decided to hold of to UDP and sending heartbeats. There will be another PR, that will be the milestone

## Packet Buffers
- Leon: stopped last time due to out of time, happy to continue. I planned to have more examples for this meeting, but the quarter started :) Would this discussion be productive? Not sure that this is the best way forward.
- Branden: Fair question. Do you have any questions that the group should talk about? 
    - You might have a prototype?
    - You are still collecting questions?
- Leon: It's a hybrid approach, we converged during the last meeting to some ideas. The hardware/network constrains we talk about last time confirmed that we need to use something like an `skbuf` or `mbuf` structure.
- Leon: There are two aspects:
    1. What does the Rust compiler permit us to write? - exploration approach, probably not parallel; (Branden: not many people are Rust experts)
    2. How do we bridge these concepts into the actual semantics exposed and expected by different protocols? This is a much more difficult questions. (Branden: it needs at least a prototype interface)
- Leon: Do we need a working prototype or a sketch?
- Branden: A sketch
- Leon: Two ways:
    - A sketch in the next 5 mins
    - I con just release something in the next two days
- Alex: can we do both? start sketching something here and Leon take it forward?
- Leon: typing rust...
- Leon: The concept is to have a type of buffer that support contiguous and non contiguous allocation, supports the concept of headroom to prepend headers with no allocation.
- Leon: impl the `PacketBuffer` trait on several types of buffers
- Branden: can you specify `when HEADROOM >= NEW_HEADROOM`
- Leon: There's a hack you can use. Use impl the right traits and impl associated const and numerics in traits. You can assign an associated type that is a const unit? And you can have an assertion that the compiler executes to panic during compilation. (TODO: Leon double-check this couple of sentences)
- Leon: Example of this is: https://github.com/tock/tock/blob/8c440023feeb9fe4923ee330b1cd76ab93acf005/arch/rv32i/src/pmp.rs#L281
- Branden: The idea here is when you create a PacketBuffer from scratch, it has the max headroom, and then reduce it?
- Leon: example below the rust code
- Branden: How does IP know that it's a max of 32 and 14?
- Leon: the current solution needs the developer to determine this, if the dev chooses a value to small, the instantiation would cause a compile time error
- Tyler: Are you saying that the higher layers use larger buffers?
- Leon: No, the buffer size itself remains constant across layer
- Tyler: The buffer is wrapped at each layer and grows in size?
- Leon: That is true if there is support for non contiguous allocation, but this is not compatible with all the different hardware that we want to support
- Leon: What we can do is, have a constraint so that the if anything says the buffer must be contiguous, then nothing can be a linked list
- Tyler: All with type checking?
- Leon: Yes, that would be the idea
- Alex: Question here: If you have a device that supports non-contiguous buffer, contiguous still works right? Would you have to wrap it to make the types work?
- Leon: You can freely convert one way but not the other. You can always shrink the headroom and you can always turn a contiguous into a non-contiguous, but not the other way around. The one really inelegant thing is that we can't encode these rules to automatically calculate the type for a given layer. The IP layer needs to look at lower-layer implementations to decide what it's requirements are. The benefit is that it fails at compile time if the IP layer guesses wrong. Specifying too much works, but too little should fail to compile
- Branden: This would lead to a scenario where your code doesn't compiler, and you have to enlarge the headroom. This goes back to the initial `static_init!`, which made us edit the board main.rs file whenever we guessed the wrong size for some thing.
- Alex: What is the meaning of `current_head`?
- Leon: Each member of a linked list would be linked into another buffer. Each would have a head and tail element pointer. All data between head and tail would be the "actual data". That mechanism allows us to have fixed buffers that can still grow in head or tail. Head allows us to pre-pend header info without reallocating. Tail allows us to add more data.
- Leon: Let's say it has 4 bytes of headroom and we want to change the min headroom, that doesn't change the Head pointer, it just changes the type signature to make things match.
- Alex: I thought reduce_headroom would actually move the head pointer
- Leon: Oh, it should really be called "reduce_type_headroom" or something. It only changes the type. It doesn't move the actual head pointer
- Alex: Why would you need this?
- Leon: You need to transmute types to convince the compiler that things are compatible.
- Branden: It's more like "guarantee_minimum_headroom"
- Leon: And there is some _other_ function "prepend" which allows you to move the head and get more space. It could fail at runtime if there isn't enough space left in the buffer
- Tyler: One quick question and a couple of comments. Would this replace the subslice type?
- Leon: good question, I do not have an answer to it yet
- Tyler: This seems the way it works for UDP, you slice it and the lower layer holds on to the full buffer allocation, but passes a chunk of it up.
- Leon: Technically what we do here is a strict superset of the underlying impl, the semantics exposed in the API are the inverse
- Tyler: Comment - I think you mentioned earlier, I don't think this is huge holdup, we do this on a net stack, and these stacks have standards with specific buffer sizes.
- Leon: True, good point, one caveat, these sizes that we specify (numbers) are propagated from the hardware impl upwards. The headroom will change based on the hardware.
- Branden: If you have VirtIO ethernet, you might have another headroom requirement that is internal to the hardware and doesn't go to the outside world
- Leon: All these constants depend on the composition of the types that you use.
- Tyler: Really exciting, this seems that it will "just work", does not seem to be a huge change in what people are using now
- Leon: Trades a lot of buffer complexity with really ugly type signatures
- Alex: this can be used for other hardware, like SD cards and displays
- Branden: This types seem impossible :)

## The Thread Child joining PR
- Tyler: PR is up, a lot of stuff is just to get it to compile. Not as much complicated state-machine stuff as prior commits, even if there is more "code" here
- Tyler: One question: I have an issue, at the Thread layer, UDP packets have another layer of encryption (MLE encryption). We use the AES-128 CCM crypto engine. Crypto needs a static buffer to be passed to it. In order to save memory, my implementation only uses a buffer for recv and send, and just passes that buffer to the crypto function when recv/sending (as opposed to having a separate crypto buffer as well).
- Branden: It hands off ownership back and forth to the crypto engine
- Tyler: The issue is that you can't just pass a slice, it has to consume the buffer as it needs a static buffer. One of two things have to happen:
    - 1. That buffer has to be replaced when the callback occurs. 
    - 2. You have no way of knowing how long the original packet was and if I pass in a buffer that is 60 bytes, I cannot return 200 bytes buffer back (the original size of the buffer)
- Tyler: The way `SubSlice` (previously `LeasableBuffer`) works:
    1. yous slice it
    2. you pass in the slice
    3. reset on the callback
- Tyler: The compiler does not accept it, it needs a static one.
- Branden: The interface for the crypt expects a static buf instead of a slice? Can we change that or that will break other things?
- Tyler: my workaround - I have another variable as part of the state machine that tracks the size of the buffer passed to crypto.
- Tyler: unless I save how long the buffer was, the crypt engine returns the full 200 bytes buffer and we have no way (without tracking the size) to know how long the packet itself was.
- Branden: You take the whole static buffer, say 200 bytes, and pass it to the crypto, saying it has 60 bytes. Now the engine knows to do crypto on the 60 bytes and later returns the buffer in a callback. But the engine does not return the length in that callback, so instead you have to hang on to it yourself so you remember when the callback occurs.
- Tyler: Curious if someone encountered this?
- Branden: I wanted to suggest `LeasableBuffer`, but it seems that you tried it.
- Branden: I think your solution is a reasonable thing to do
- Branden: Perhaps you should change the callback to include the length?
- Tyler: This seems reasonable.
- Alex: This will be a problem for `PacketBuffer`
- Tyler: The problem is that buffers have to be static, that takes a way a lot of the abstractions
- Branden: For hardware buffers need to be static
- Branden: I think this group is great for these things exactly, as we can discuss these solutions that we come up.


```rust

//              head               tail
//              |                  |
// [0, 0, 0, 0, DATA, 1, 2, 3, 42, 0, 0, 0, 0]  -> [ ... ]
// PacketArray<true, 4>.reduce_headroom::<2>()

trait PacketBuffer<CONTIGUOUS: bool, HEADROOM: usize> {
    fn reduce_type_headroom<NEW_HEADROOM: usize>(self) -> PacketBuffer<CONTIGUOUS, NEW_HEADROOM>
        when HEADROOM >= NEW_HEADROOM
    {
            
    }
    
    fn prepend(data: &[u8]) -> PacketBuffer<CONTIGUOUS, NEW_HEADROOM> {
        self.current_head -= data.len()
    }
    
}

// Borrows a slice to an array
struct PacketSlice<'a> { // implements PacketBuffer
    current_head: usize, // offset into the buffer in some way
    slice: &'a mut [u8],
    next_buffer: &PacketBuffer<_, _>,
}

// Owns an array
struct PacketArray<SIZE: usize> { // implements PacketBuffer
    current_head: usize,
    arr: [u8; SIZE],
    next_buffer: &PacketBuffer<_, _>,
}


impl EthernetHIL for MyEthernetMAC {
    fn transmit_buffer(buf: impl PacketBuffer<true, 32>) {
        
    }
}

// Example of a stackup that needs header room
//  allocate buffer
//   |
//   v
//  UDP
//   |
//   |
//  IP (24 bytes header + max(32, 14) = 56 bytes)
//   |
//   |------------------------------\
//  Ethernet A (32 byte headroom)   Ethernet B (14 byte headroom)


//  allocate buffer
//   |
//   v
//  UDP
//   |
//   |
//  IP (non-contig, 0 byte headroom)
//   |
//   |------------------------------------------\
//  Ethernet A (non-contig, 0 byte headroom)   Ethernet B (non-contig, 0 byte headroom)

```
