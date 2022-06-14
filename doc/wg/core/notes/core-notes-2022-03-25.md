# Attendees
- Amit Levy
- Hudson Ayers
- Philip Levis
- Alexandru Radovici
- Johnathan Van Why
- Pat Pannuto
- Alyssa Haroldsen
- Branden Ghena
- Vadim Sukhomlinov


## Updates
- Phil: I am continuing my work on Process loading and signing. One thing that is tricky is that
  there is not the same hardware crypto available on different platforms, and
  we don't have software implementations. I am implementing a software SHA256 so
  we can test across platforms. I want to avoid incorporating licenses if
  possible.

## Uninit in static buffers
- Alyssa: we have storage::read() syscall which reads from storage
  if you have a 1k buffer you want to read into, we find we have to 0 it out
  in userspace before we write to it, because Rust expects a [u8] slice and
  not a `[MaybeUninit<u8>]`. This wastes cycles.
- Alyssa: The in-progress switch of Readable and Writable ProcessBuffers to use raw
  pointers should make this not an issue on the kernel side.
- Alyssa: Problem with working with uninit data is that it is not guaranteed to be
  fixed, this is a unique property of uninitialized memory. However it seems
  that on all our platforms integer data should be frozen once passed across the
  system call boundary.
- Alyssa: Another complicating factor is that rules around uninitialized byte
  buffers are unclear -- is it UB to create
  an uninit integer, or just to read from it?
- Phil: Why do we want this buffer to be uninitialized?
- Alyssa: It is wasteful to zero large structures which we are going to
  immediately write over.
- Amit: So to clarify. I am in rust userspace, I want to read 1k from flash. I
  don't want to waste 1000 cycles initializing a large vector in the process
  stack because that would be slow since we have to 0 it out and doing so is a
  waste since once we pass the buffer to the kernel this is going to be
  overwritten. The problem is that the system call interface takes an
  initialized slice, and so there would need to be some casting, and the
  semantics of that cast are undefined.
- Johnathan: So we are talking about using ReadWriteAllow, which takes `&mut
  [u8]`, to pass these uninit buffers
- Alyssa: yes
- Johnathan: I was going to suggest just adding an API to libtock-rs that
  takes MaybeUninit instead, but one threat model concern is that we
  might be sharing secrets that were previously in RAM with a capsule
- Alyssa: Yes, and for this reason I think that it might be smart to have a
  write-only allow.
- Amit: That doesn't sound like a bad idea on its face
- Phil: How would you make a type capable of this in the kernel?
- Alyssa: MaybeUninit is an example of a type that does this -- it takes
  unsafe to read from it
- Phil: How does that work for an array where you might only write to part of
  it?
- Alyssa: Functions that take in a WriteOnlySlice and output
  ReadableWritableSlice
- Hudson: How does that cover the "write only part" portion of Phil's Q?
- Alyssa: It depends how important that is. Rust std library does have a
  nightly-only structure that provides some support for this by tracking
  which portions of a buffer have been initialized. Alternatively we
  can just give it an API where you have to write an entire buffer to write
  this write-only slice
- Amit: A slice of MaybeUninits might cover this
- Alyssa: Could have a partial write function that returns a WriteOnlySlice
  and a ReadableWritableSlice split across the portion that you wrote
- Phil: Can we talk about the use case a bit? This sounds like 256 wasted
  cycles? Is this worth rearchitecting chunks of the kernel for this?
- Alyssa: This is multiplied across every flash read. And there is code for
  these mem clears.
- Alyssa: Also we are using the zerocopy library, where we take a structure created
  using `new_zeroed()` and then pass an `as_mut()` slice to the kernel. so we have
  monomorphization based on the size we are loading in memory.
- Johnathan: It does seem that these kernel changes could be larger, size wise
  than the size in userspace from all of this.
- Alyssa: It just in general seems like this is something that should be
  possible
- Hudson: One note, we do this in libtock-c all the time
- Alyssa: Yeah, that's allowed
- Johnathan: Well when we do that in libtock-c that is a threat model concern.
- Phil: If you cared about it in your app you would 0 it
- Alyssa: Our storage controller being able to read uninitialized data is not
  part of our (Ti50)'s threat model
- Johnathan: I think Ti50 has a different threat model.
- Amit: The high level goal is achievable by zeroing out the memory, so this
  is just a question of performance trade-offs (between cycles and size it
  seems)
- Alyssa: another option could be a libtock-rs option for sharing uninit
  memory without using a new syscall. That would require an agreement with
  the capsule that it will only write to the memory or that you don't care if
  it reads from uninit memory. There is also a fourth option which is uninit
  data in the u8 directly, that passes in miri now but it is unclear if it is
  unsound.
- Amit: Even the last option requires the same security concept with capsules
- Johnathan: I think it would be reasonable to change the threat model to say
  that a process can choose to trust a capsule with arbitrary data, and then
  add an API to libtock-rs that does this.
- Phil: I agree with Amit's characterization here about this being an
  empirical question of performance trade-offs. My intuition is that the kernel
  costs would be greater, but the only way to know is to do this empirically
- Amit: If I understand Alyssa and Johnathan then you are happy to trust the
  capsule, and this is more about trying to stick to the letter of the current
  threat model?
- Alyssa/Johnathan: We do trust this capsule to do what it says.
  The primary blocker now
  is the lack of a libtock-rs API that takes uninit memory.
- Amit: My thought then is that if you are cool with having unsafe code in the
  app to handle the uninitialized memory in order to avoid this overhead of
  zeroing out the memory, that is fine. If that is not cool or needs to be in
  libtock-rs, there is a question of whether this generally is applicable to
  other applications because of the threat model concerns.
- Amit: Basically it is a reasonable threat model for apps to trust particular
  capsules, but we don't want to generalize that to all applications.
  Basically we don't want a safe operation in libtock-rs that does this if it would
  only work for certain capsules.
- Alyssa: This would be checked at the type level
- Amit: So this is not a general interface we would expose to any application
  using allow for any purpose?
- Alyssa: For that we would need to have a raw pointer allow that is exposed
  (to some degree). The issue here is what is a specialized userspace driver
  going to do without that? Directly talk to the kernel via asm or use
  `libtock_syscalls`?
- Hudson: It seems like we might want an `unsafe` API for this to indicate it
  it not generally okay to use with arbitrary capsules from a security
  perspective.
- Amit: If the only way to actually allow this in the kernel is to convert
  this uninitialized memory to a slice that can be read, doesn't this break
  type safety? If I allocate on the stack some type with private fields and
  then that gets deallocated and I end up sharing it through a safe interface
  and it gets shared in a readable way we have broken the encapsulation that is
  part of type safety
- Alyssa: Well from the uninit memory you can't actually get the original
  type, just the underlying representation
- Amit: Well at some level there is something performing an unsafe operation to
  convert this to a slice
- Alyssa: Not necessarily
- Amit: but it seems like inevitably we are at some point going to make it so
  the kernel can read this uninitialized memory without using unsafe, because
  some unsafe in libtock-rs was encapsulated in a safe function
- Alyssa: what use of unsafe?
- Hudson: I imagine the use of unsafe would be just to call the raw syscall
- Alyssa: Yeah but none of the memory transformation is unsafe. It is sound to
  go from MaybeUninit to raw pointer and length
- Amit: But on the kernel side, we are transforming that pointer and size into
  a slice of u8's that can be read
- Alyssa: Once we convert to raw pointers in the kernel for ProcessBuffers it
  wont actually be a slice. It remains true we are trusting the capsule in
  that we are ok with it reading from uninit memory. This sounds like UB,
  but whether it actually is depends on whether the buffer is frozen when it
  crosses the syscall boundary
- Hudson: Once the kernel receives a buffer via allow, isn't that buffer
  always going to be frozen?
- Alyssa: Yes, though I believe that is hardware specific, there could be
  weird things with virtual memory managers
- Johnathan: That would be entirely under the kernel's control though
- Alyssa: Yeah, so it should be frozen.
- Amit: I still feel the function that enables this in userspace should be
  marked unsafe. It could enable some library to wrap some stack pointer that
  contains data that it did not create to get wrapped and passed to some
  untrusted capsule that could leak that data.
- Alyssa: I don't think that is a memory safety thing, its a security thing.
  That is not part of the ti50 threat model though.
- Amit: I think I would be uncomfortable with libtock-rs as a core library
  exposing this operation as safe because of what it hides and because it
  breaks encapsulation
- Alyssa: Is there anywhere else we use unsafe to mark security concerns
  instead of memory safety?
- Amit: In the kernel, everywhere. Maybe not in libtock-rs
- Alex: If you pass MaybeUninit data to the kernel, and then unallow the data,
  how can you know that the kernel wrote that data?
- Alyssa: Once you pass this across the system call boundary the optimizer
  does not know what could happen
- Johnathan: The asm for system calls informs the rust compiler that the data
  could be written to.
- Hudson: what will unallow() look like for this new API? Will it return an
  initialized slice?
- Alyssa: Well.. it could return a MaybeUninit, and then based on the success
  of the operation it could call `assume_init()`.
- Alex: How do you return success?
- Alyssa: Not sure
- Phil: The low level driver gets an upcall indicating the read completed and
  was successful. Based on that, the userspace library knows whether the data
  was initialized or not
- Amit: This all seems reasonable but all this conversion stuff should be
  happening in capsule-specific drivers not being a low-level system call API.
- Alyssa: I think there should be a well-lit path for sharing uninit data in
  libtock-rs, with security disclaimers.
- Phil: I am still interested in an empirical performance evaluation
- Hudson: I think that evaluation will be very dependent on the particular
  use of uninitialized data being considered so we may not get clear-cut
  results.
- Alyssa: A lot of this is motivated by C firmware engineers being generally
  frustrated with the fact they have to 0 memory to work with it at all
- Phil: Regardless, it seems clear that if we measured this for an example it would shine
  some insight into which approach is better for many use cases. If the kernel
  cost of a new system call is high enough we would simply never consider it.
  Not saying that will happen but we want to know the constant factors here.
