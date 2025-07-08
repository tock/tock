Tock Core Notes 2023-11-10
==========================

## Attendees
 - Branden Ghena
 - Leon Schuermann
 - Alyssa Haroldsen
 - Hudson Ayers
 - Brad Campbell
 - Johnathan Van Why
 - Alexandru Radovici
 - Amit Levy

## Updates

### Newlib
 * Brad: For libtock-c newlib packaging process. Last time we talked about this, there were some questions about where to store the artifacts and how reproducible it is. I now have a docker file that is pretty close to being able to build the artifact both for us and in our CI. It seems that newlib is not as extensively tested as I had hoped it was. 
 * Hudson: The reason you need a docker container is that Ubuntu 22 shipped with GCC 10?
 * Brad: We just wanted a way that even if the artifacts disappear, someone could go back and rebuild newlib for some moment in time. As a side effect, and why I really wanted it, is that it lets me build it at all. There is indeed a conflict with newer and older versions of GCC. For RISC-V newer versions of GCC build a newlib that doesn't work with old versions.

 * Leon: I have a CW310 board, which is a reference for the OpenTitan RISC-V FPGA. Zerorisc provided it. I may or may not be able to provide remote access to that board for testing changes.

### Syscall Tunnel
 * Johnathan: I'm building a proof-of-concept for what I call the "syscall tunnel" for libtock-rs. It forwards system calls over a UART to allow apps to run on a Host computer and forward requests to Tock on a microcontroller.
 * Johnathan: I'm running into a rough bug here, which might be undefined behavior? I can't reproduce in Miri.
 * Branden: What's the purpose behind that?
 * Johnathan: Testing infrastructure. Make calls to a tock board to perform operations. Then run a check on some other board to see that it received it.
 * Brad: So the Tock board runs some userland app that receives the syscalls? (yes)
 * Johnathan: Right now it needs the pin-based allow API
 * Hudson: For the device under test here, how does it communicate over UART back to the host? Does it send additional system calls? Or does userspace have raw control of the underlying UART?
 * Johnathan: Just normal console calls. Works fine. Although it gets really confused if the host makes a console call.
 * Branden: I have some undergrad doing a similar thing: two microcontrollers running Tock which could send syscalls back and forth between them

### Ferrocene Compiler
 * Hudson: Folks might have seen that Ferrocene had its first certified compiler release just a few days ago. Various certifications. Cruise heard about it and was interested now that it's automotive certified. Ferrocene is just the upstream rust compiler, but like 8 months back or so? I think that once we get to running on stable rust, it would be interesting to try to match up Tock's stable rust with Ferrocene's.
 * Amit: I had a chance to talk to Florian at Ferrocene and asked some questions. Some clarifications are that Ferrocene is not "certified" but "qualified" which is essentially an attestation that the process the company went through to choose the compiler and modify it plus the process they have for patching bugs is up to snuff for the automotive agency. To clarify that it's maybe a little less meaningful than I had originally understood, although I agree that tracking Ferrocene releases would be nice.
 * Amit: Certification seems to only happen on actual products.
 * Alex: The problem is still that the core library isn't certified. So I'm not sure how you can use it. Another company, AdaCore worked with Ferrous Systems and also has a "qualified" compiler now, but still no library. We definitely need that
 * Hudson: There may have been a disagreement about open sourcing from what I've seen
 * Amit: The answer I got from Ferrous systems about the core library is that compiler uses the core library and it's not necessary for the core library to be certified to use the compiler. And that you "could" use your own core library for compiling your program.
 * Alex: Yes. I just don't want to write my own library and certify it. That's a lot of stuff. Something like 60K lines of code. Libc is comparably MUCH simpler, like 1K.
 * Alyssa: I think you could have way less Core library and still "compile". Although it would be the most basic thing that works. Way less lines, just no functionality.
 * Amit: So it's worth exploring what subset of the Core library is actually necessary for different parts of Tock.
 * Alyssa: I think any lang item that's mentioned has to exist. Lang items connect the compiler to the core library.
 * Alex: Iterators are required for sure if you want to do for loops...
 * Branden: Panic info too, I guess
 * Amit: https://github.com/rust-lang/rust/blob/master/compiler/rustc_hir/src/lang_items.rs
 * Johnathan: Funny. Does this solve our "core fmt is bloated" issues? If we write our own core we could do better?
 * Alyssa: Not without changing the API. `ufmt` has a different API. You could make an alternative library removing most functionality.
 * Johnathan: Panic utilities might rely on some functionality
 * Alyssa: Maybe a lot of it could call an undefined method, and you could see what is or isn't used
 * Amit: Interesting to consider. To the extent that it's a possible necessity to have an alternate Core library, it would be an opportunity to change some aspects. We could remove things we don't want
 * Alyssa: There is a question about how much you want to break from upstream core. I personally think that certification should happen on the upstream Core, if possible
 * Hudson: I think the cost to certify is somewhat linear with the volume of code. Which is a problem
 * Alyssa: A problem is that upstream core compatible code would end up having almost all of that.
 * Amit: In other scenarios, like security certification, there are other avenues like looking at the binary which maybe don't need a wholly certified toolchain and libraries


## 64-bit Tock Support
 * Amit: Heard from a few people that there is at least one compelling use case to support 64-bit Tock, besides Host-based emulation. For tagged memory architectures where the only option is 64-bit. Probably ARM, maybe RISC-V. There might be some PRs pushing it upstream at some point. Might be worth putting some thought into this in advance. What would make those dealbreakers and what concerns might we have?
 * Brad: Our concern abstractly has been cannibalizing what we're good at to support an extension. So we could end up having internal APIs designed so that it's not how you'd do it with a 32-bit only OS, even though that's really our sweet spot. The tension there has been the concern.
 * Brad: It seems like there could be a way to do it. It would need to be a concious effort that it'll look strange because it's really a 32-bit API being shoved into a 64-bit system
 * Amit: So maybe the system calls are still 32-bit information, even if the registers are actually bigger. Is that what you're thinking?
 * Brad: Yes. That's a good example of tension
 * Hudson: It seems possible that there are more examples.
 * Branden: I have a hard time thinking about what the issues will be? Certainly the lower-level assembly for swapping into processes, but that would be different for any new architecture.
 * Amit: Alyssa looked into this?
 * Alyssa: Predominantly places where `usize` and `u32` are conflated. I think we resolved most of them
 * Johnathan: I don't remember the kernel side implementation. The libtock-rs side just used `u32` for a lot of things even when `u64` could have been used. Command arguments for instance, could be larger which could give more capabilities, but would be a difference. In emulation we do limit to 32-bits for emulation.
 * Alyssa: Passing pointers through command is a big issue here.
 * Johnathan: And we don't pass pointers through command in libtock-rs.
 * Alyssa: If I changed the syscall interface for 64-bit, I'd make things pointer size but not `usize`.
 * Amit: Not sure what you mean
 * Alyssa: Rust sort of supports 16-bit systems too. So `usize` can't be used directly. Some abstraction instead.
 * Branden: Presumably peripheral registers would be 64-bit too. So they would need to write to 64-bit registers in the drivers. But our register library should "just work" for that?
 * Johnathan: I strongly suspect that when these PRs come, there will be a tension. They'll have a 64-bit view of the world, compared to our 32-bit view of the world.
 * Alyssa: Really we just need to avoid pointers not being address size
 * Brad: For sure about Johnathan's point. You can always just not use upper bits. But if you expect that you can, there'll be an issue.
 * Alyssa: I think my concerns are similar to any system supporting both 32 and 64 bit, not unique to Tock
 * Brad: Well, we need to decide if we're really providing value to the 64-bit space. Do we want to support it well, or just at all? Other systems focus on 64-bit and are okay with 32-bit being clumsy.
 * Johnathan: I don't remember the resource constraints for the 64-bit systems. If they have drastically more RAM/Flash that'll also cause a difference in thinking and conflicts.
 * Amit: And again like Brad is saying, a place where Tock is maybe not bringing as much value
 * Hudson: PR #2041 had some 64-bit thoughts https://github.com/tock/tock/pull/2041
 * Hudson: This was 3 years ago. It partly waited on the author and partially a reluctance on our end. Worth looking at the discussion to see what changes were proposed
 * Branden: Going back to the start, why is tagged memory interesting to people?
 * Johnathan: CHERI is what people are very interested in
 * Amit: And broadly, tagged memory is a way of doing fine-grained memory access control
 * Alyssa: Such that exploits are much harder to gain access to memory

