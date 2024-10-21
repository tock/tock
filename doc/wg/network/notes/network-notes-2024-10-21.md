# Tock Network WG Meeting Notes

- **Date:** October 21, 2024
- **Participants:**
    - Alex Radovici
    - Branden Ghena
    - Leon Schuermann
    - Felix Mada
- **Agenda**
    1. Updates
    2. PacketBuffer
    3. StreamingProcessSlice
- **References:**
    - [PacketBuffer](https://github.com/tock/tock/blob/60c1e250856a75bbe57b71a41707e2f1de881653/kernel/src/utilities/packet_buffer.rs)
    - [4208](https://github.com/tock/tock/pull/4208)


## Updates
- None today


## PacketBuffer Review
 * Branden: I made these notes after reviewing the PacketBuffer implementation and provided to Leon. Posting here for the future
 * General notes:
    * What is the "true" external PacketBuffer interface? I guess it's the `PacketBufferMut` impl?? It would be really nice if this file made clear what the externally-usable calls are and what the internal crap is. I guess that's the `pub` keyword? But the impl of `PacketSliceMut` is `pub` too...
    * This file will make clear what the internal implementation of PacketBuffer is and how that works. It doesn't, however, contain justification for _why_ it's like that. I think a lot of comments for functions and structs that explain the "why" would be valuable.
    * This file also doesn't have documentation on how to use it practically, especially with the generic parameters. Maybe a separate file with toy example that uses it would be valuable there so people can see the generics in action but without all the other stuff that comes with, say the Console redesign. Something minimal would go down through at least two layers and then talk to some chip driver at the bottom. Then go back up through those same two layers.

 * Starting in `PacketBufferDyn`:
    * For `reclaim_headroom`, the comment could be more clear here. I think it's taking space from the buffer and giving it to the headroom? Maybe a quick diagram. So false would mean that there wasn't enough buffer space?
    * For `reclaim_tailroom`, it will presumably not move past the headroom marker.
    * For `reset`, what does it do to tailroom? It's unclear to me how reset is different from `reclaim_headroom`
    * For `copy_from_slice_or_err` and `append_from_slice_max`, those apply to the buffer data itself, right? They could use comments. I don't really understand what they do from the names alone. I'm also not sure why these two functions exist? What makes them the "proper" interface?
    * Why do we even need a `prepend_unchecked` operation? I guess the idea is that the checks are already happening at compile time before calling this?

 * In `impl PacketBufferMut`:
    * For `reset`, that's a runtime assert, right? Is that necessary?

 * For `PacketSliceMut`:
    * A diagram here would be really nice. What values are prepended, and how do headroom, payload, and tailroom work?
    * Also a comment that they're stored with native endianness, whatever that may be
    * I don't at all understand why `_inner` is a `u8` here. And the comment above it does nothing to help that. It seems to never be used because we just refer to `self` instead. I kind of thought it would be `[u8]` since it's a transmuted array of data...?
    * In `new`, the comments don't seem to match the code about starting with zero headroom. I think it starts with payload length, headroom bytes of headroom, and length - headroom bytes of tailroom.
    * Have you looked at the assembly for `get_inner_slice_length()`? It seems like it should be pretty optimized, but I'm wondering if it actually happens in practice. Also that the panic isn't there.
    * I don't like that `data_slice` and `data_slice_mut` have slightly different constructions

 * In `impl PacketBufferDyn for PacketSliceMut`:
    * Is the implementation of `len()` correct? The comment there looks plausible, but the implementation looks wrong.
    * Why does `copy_from_slice_or_err` eat up tailroom? I assumed it would write into the payload and error if it didn't fit in the payload (slice.len() - headroom - tailroom).


## PacketBuffer
 * Leon: Looked through Branden's PacketBuffer feedback. Generally I'm pleased that the concept makes sense. The points you raised seemed correct.
 * Branden: Any of these worth having a discussion about?
 * Leon: Went through writing some docs and tests. There was a bug stopping things from being compile-time evaluated previously, which I had to fix. Added some internal complexity.
 * Leon: Another issue worth bringing up. There's a tradeoff that we are unable to provide information in the compile-time errors as to which call site caused the mismatch. The only information you get is where the assertion is located. You get file and line, but it's basically always the same file and line where the assertion is.
 * Leon: Let's give an example of this.
 * Leon: Explains what's going on in this example. It's a trick to force the compiler to monomorphize the const generics and then run an assertion on that, all at compile-time.

   ```rust
   #[inline(always)]
   pub fn reduce_headroom<const NEW_HEAD: usize>(self) -> PacketBufferMut<NEW_HEAD, TAIL> {
       // Const items are global by default and do not have access to generic
       // paramters of the other item (this `reduce_headroom` function). Hence,
       // the following simple const-assertion doesn't work:
       //
       // ```
       // const _: () = assert!(NEW_HEAD <= HEAD);
       // ```
       //
       // We instead trick the compiler into generating a monomorphized
       // instance of a variant-less enum (which ensures that it will not
       // generate code by not being instantiable) that itself contains a
       // const-assertion, using type parameters from the monomorphized
       // instance instead. This monomorphized instance is still "global", but
       // specialized to this particular monomorphized call site.
       //
       // Inspired by https://users.rust-lang.org/t/cant-use-type-parameters-of-outer-function-but-only-in-constant-expression/96023/2
       enum AssertionHelper<const HEAD: usize, const NEW_HEAD: usize> {}
       impl<const HEAD: usize, const NEW_HEAD: usize> AssertionHelper<HEAD, NEW_HEAD> {
           const ASSERTION: () = assert!(
               NEW_HEAD <= HEAD,
               "{}", // This is special-cased to work in const expressions
               concat!(
                   "reduce_headroom cannot increase the headroom of a PacketBufferMut<",
                   // TODO: insert actual `HEAD` value if / when `const_format_args!` is stabilized
                   "HEAD",
                   ", _> to create a PacketBufferMut<",
                   // TODO: insert actual `NEW_HEAD` value if / when `const_format_args!` is stabilized
                   "NEW_HEAD",
                   ", _>",
               )
           );
       }

       // Force monomorphization of the const assertion:
       let _: () = AssertionHelper::<HEAD, NEW_HEAD>::ASSERTION;

       PacketBufferMut { inner: self.inner }
   }
   ```

 * Leon: Here's the error message that's generated from that.
   ```
      ---- kernel/src/utilities/packet_buffer.rs - utilities::packet_buffer::PacketBufferMut<HEAD,TAIL>::reduce_headroom (line 185) stdout ----
   error[E0080]: evaluation of `kernel::utilities::packet_buffer::PacketBufferMut::<HEAD, TAIL>::reduce_headroom::AssertionHelper::<4, 6>::ASSERTION` failed
      --> /home/leons/proj/tock/kernel/kernel/src/utilities/packet_buffer.rs:236:28
       |
   236 |           const ASSERTION: () = assert!(
       |  _______________________________^
   237 | |         NEW_HEAD <= HEAD,
   238 | |         "{}", // This is special-cased to work in const expressions
   239 | |         concat!(
   ...   |
   247 | |         )
   248 | |         );
       | |_________^ the evaluated program panicked at 'reduce_headroom cannot increase the headroom of a PacketBufferMut<HEAD, _> to create a PacketBufferMut<NEW_HEAD, _>', /home/leons/proj/tock/kernel/kernel/src/utilities/packet_buffer.rs:236:31
       |
       = note: this error originates in the macro `$crate::panic::panic_2021` which comes from the expansion of the macro `assert` (in Nightly builds, run with -Z macro-backtrace for more info)
   
   note: the above error was encountered while instantiating `fn kernel::utilities::packet_buffer::PacketBufferMut::<4, 0>::reduce_headroom::<6>`
     --> kernel/src/utilities/packet_buffer.rs:197:51
      |
   15 | let larger_headroom_pb: PacketBufferMut::<6, 0> = pb.reduce_headroom();
      |                                                   ^^^^^^^^^^^^^^^^^^^^
   
   error: aborting due to 1 previous error
   ```
 * Leon: So, line 120 actually _does_ show us the exact callsite where this fails. Which is nice and sufficient to figure out what was going wrong.


- Branden: Should we split this up into two files, one "interface" and one "backend"?
- Leon: The only reason PacketBufferMut is sound, is if it compiles with the invariants stated on PacketBufferDyn. If for instance, PacketBufferDyn didn't always return a true full value for headroom, then you'd be able to create a PacketBufferMut that advertises a const generic headroom that is larger than the underlying headroom. Which would let you call unchecked methods which are incorrect.
- Leon: So the biggest thing that confused me when I came back to this, is that PacketBufferDyn needs to be always implemented correctly, which is why it's an unsafe trait. So PacketBufferDyn doesn't break Rust soundness, but the outer wrapper of PacketBufferMut removes many runtime checks by relying on the external API contract.
- Branden: My most important part from the review here, is that we really need some clean example of how to use this. Something for consumers of PacketBufferMut, not developers. The console implementation is great, but it's too hard to disentangle the details of console from the details of PacketBuffer.
- Branden: `reset()` is a good example of this. I can understand what it does internally (with some help), but I'm missing the context of why it exists and what you'd use it for
- Leon: Good point. Quick example is releasing a buffer once you're done with it and passing it back to the upper layer so it can reuse it. You don't care about the data anymore and just want to make the types work out.


## StreamingProcessSlice
 * https://github.com/tock/tock/pull/4208
 * Leon: Made some changes in addition to cleanup/rebase
 * Alex: This looks great. I'll review soon
 * Leon: First, flag for noting if a StreamingProcessSlice has been exceeded by the kernel. So the userspace could know if a problem has occurred and some data wasn't sent up.
 * Leon: Second, a flag for the userspace to set that signals to the kernel to stop writing to this buffer if it would be exceeded. This makes sure that, for example, if a large buffer doesn't fit that later small packets won't be written.
 * Leon: So, if halt is set, and the userspace sees exceeded is set, the last packet in the buffer is the one before it failed (the failed packet wasn't recorded)
 * Leon: The goal here is that if you want to know if something was silently dropped, you could detect that.
 * Branden: My perspective here is that this is like the overrun bit in a UART. Nobody pays attention to it in practice, but it doesn't hurt that it's there
 * Leon: This PR also breaks CAN right now. So I'm worried about that. It would be nice if Alex could check it out
 * Alex: I will take a look at that too

