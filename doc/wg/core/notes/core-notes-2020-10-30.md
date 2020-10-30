# Tock Core Notes 2020-10-30

## Attending
 * Branden Ghena
 * Amit Levy
 * Leon Schuermann
 * Alistair
 * Hudson Ayers
 * Johnathan Van Why
 * Pat Pannuto
 * Brad Campbell
 * Phil Levis
 * Vadim Sukhomlinov


## Updates
 * Amit: Tock 1.6 is released! Thanks to Brad for managing and others for testing.
 * Hudson: There is now synchronous panic output for the Nano33. So it's getting closer to normal board support. Still needs tockloader support.
 * Phil: Students at Stanford have Tock booting on nRF53, which has multiple (separate) cores and trustzone M. And the students are working on distributed system calls. There is separate memories for the two, but there is a shared memory region and some signalling peripheral.


## Tock 2.0 Progress
 * https://github.com/tock/tock/blob/tock-2.0-dev/kernel/src/syscall.rs
 * Leon: been working on syscall implementation. Design is almost settled. We implemented syscall.rs and are happy that the implementation matches the design. We're looking at the generated assembly to see if everything is optimized well enough. Cortex-M assembly Leon is working on looks pretty good. Phil is looking at RISC-V assembly.
 * Phil: One of the concerns I had was the identifier space. That we have return values with certain identifiers, which won't match the corresponding enum. This turns out to not be a problem because a move is necessary anyways. Not a big deal.
 * Phil: On the RISC-V side, the generated assembly which matches the enum doesn't have a switch table but is rather a series of if-else's. This is bad because it's a bunch of branches rather than a single jump, especially if it turns out to be the common case.
 * Amit: Do we know why?
 * Phil: Not yet.
 * Leon: As far as I can tell, I'm not aware if there's an instruction that you need.
 * Branden: Don't you just need an indirect jump? That's definitely in RISC-V
 * Vadim: Depends on a bunch of things in the compiler. Compiler might think that branches are faster.
 * Phil: Work in progress.
 * Phil: So after we feel comfortable with this, the next step will be changing over the traits. There are some things like read-only allow we need to add there. At some point, we'll get to the stage of changing system call capsules, at which point we'll need assistance.
 * Leon: One question to bring up on call is the potential unsubscribe semantics and how it would work. We've talked a lot about unallow, which probably has a lot of commonality to unsubscribe. Guarantee that function will no longer be called in the future is important, especially for a heap-allocated function.
 * Phil: My guess is like unallow, the kernel will have to be able to say "no". But if it says yes, it works for sure. Options include a new call for unallow. Or you could subscribe with a new NULL value and always get the old value back (where value is function pointer here). Would love to hear people's thoughts on this.
 * Vadim: What about automatic unsubscribe after a call?
 * Phil: What if the callback wants to invoke driver again? It would need to re-subscribe which could lead you to miss a "signal" from the driver. Race condition where events can get lost.
 * Amit: Plus the performance overhead for the common case where you register a callback once and use it forever is worse.
 * Vadim: I'm having a complex case now where I have to allow a bunch of things because they don't fit in commands. I was thinking if allow command could have multiple slices that would help to minimize number of syscalls.
 * Phil: You've brought this up before. Part of the challenge is that cases where syscall overhead is significant compared to what you're doing is narrow. But they do occur. One case could be AES because it's pretty fast. In those cases maybe we need a different syscall or maybe some form of allow that has a buffer which is a structure with multiple things in it. We do want to make sure all edge cases are possible, but adding lots of system complexity for it is tricky. It will be good to have concrete cases for short operations, but many syscalls. My guess is crypto is where this will occur.
 * Phil: With allow, the semantics are clearer, especially if we follow the rust world when thinking. For subscribe and code, it's harder. We usually think about it as read-only, but for code chunks allocated on the heap, it's more complicated.
 * Amit: There's also userdata associated with callbacks. And that userdata has ownership properties. My intuition is that it seems clean to have similar semantics for allow/unallow and subscribe/unsubscribe.
 * Phil: Challenge is that a slice of zero length has clear meaning in allow. There isn't a clear analogy for code. So we have to pick something that is a "null function".
 * Vadim: Is there a check that allow memory is in the right process?
 * Phil: Oh yes! There must be. That's why we need read-only allow, because right now it only allows memory from RAM, not from flash addresses.
 * Leon: My proposal would be to discuss implementation details on mailing list from here for unsubscribe.
 * Leon: Also, by migrating to 2.0 syscall interface, we change where errorcode and success registers are. Moving away from this, we could just have an errorcode enum for error cases.
 * Phil: So right now there's success and successwithvalue, we should be able to get rid of these now.
 * Leon: So in the kernel it's used everywhere. In the long term we can deprecate returncode and change to errorcode. I implemented try-from-returncode which converts to errorcode.
 * Amit: I'm a bit lost. Is there a question?
 * Leon: I think it's an invasive change if we deprecate the returncode and wanted to check people's thoughts.
 * Phil: I think it'll be clearer when we get there. I think there is a lot of interest in moving to rust results. We'll have to figure out the path when we get there.
 * Amit: I think 2.0 is the opportunity to fix things in invasive ways that lead to categorically better.
 * Hudson: Do we have to replace everywhere? Or could we just replace in some places and translate before calling to lower drivers?
 * Leon: Yes we can go with that and I think it's a fine idea in the short run. I think in the long run we really do want to deprecate returncode. Having errors and successes combined together is complicated.
 * Phil: Yes. I think the key cases there are the HILs, which should be changed. Start with UART, Timer, GPIO, and see how it goes.


## Application ID proposal
 * https://groups.google.com/g/tock-dev/c/aduN7fHWXdI/m/bLjo0_TpAQAJ
 * Johnathan: I expected this to be more controversial. Reading the background will give some insight into why it's that way, but under heading "Proposal v2" is the real important bits.
 * Branden: New TLV elements go in headers, right?
 * Johnathan: Yes
 * Phil: TLV elements are board-specific? Can you unpack that?
 * Johnathan: That's one of the more awkward parts of allowing boards to determine things. There will be one new type for TBF Headers, but the contents are board-specific. There could be multiple values for different board-specific headers under a single T.
 * Phil: I think instead, each Type should specific a single Value type. So there can be multiple Types and the board will ask for a specific one and only a specific one.
 * Johnathan: But boards are going to determine encoding, which means the allocating of Ts gets weird.
 * Phil: Can we just come up with a couple Ts that cover 99% of cases? Just application ID, verified ID, etc.
 * Johnathan: But you need to specify algorithm for verified ID, which could be something rather novel for OpenTitan case.
 * Amit: Phil's proposal leaves it up to the board to decide what appID to give to a particular process. By convention that will be taken from TLV header in process if found. Then maybe there are 2-3 types that most boards will use.
 * Johnathan: Then there's a TLV for each type of appID supported. So OpenTitan would need their own TLV element for their ID signature scheme.
 * Amit: So there would be different types for signed-by-google or signed-by-someone else? Wouldn't you want signatures from different sources of authorities?
 * Leon: So that could be part of the type? Some portion of bits go in type?
 * Johnathan: Maybe
 * Amit: I think this is great. Because decision is left to the board and TLVs are extensible, everything is an option. Even much more complex setups.
 * Phil: I agree. Everything else in proposal looks great to me.
 * Vadim: One question about alignment. Might need to be u32s.
 * Leon: AppIds are a large size here which is unfortunate. Dynamic is bad for several reasons. But maybe compile-time choosable would be good. Then we could parameterize kernel based on type.
 * Johnathan: So the thing about switching to u32 from u8 is that you get into endianness issues. We could make it a struct and set alignment.
 * Vadim: We could make some const generic size, which could change it to fit your needs.
 * Johnathan: So make size generic, fix alignment concern, and then deal with multiple types of TLV entries. I think I'll make a proposal v3 with those changes. And we can discuss again.
 * Amit: Nitpicky, but why is the in-ram data type in the process struct an Option? Process should always have appID?
 * Johnathan: That's a mistake. Will fix.
 * Phil: So back to multiple TLVs. It is a namespace management question. So you can have hierarchical or flat. The cost of a flat space where each scheme has a different T is that all high-level code has to understand each new T. But hierarchical you can just ignore. Which means you don't have to go back and modify when adding something new. There are middle grounds. One that says this is just an ID, one that says this is a signed ID and there will be a type inside for scheme, one more for this is a new signature different from the first signed ID type.
 * Branden: Why 48-bytes?
 * Johnathan: Storage and Syscalls only need 32-bits. For secureboot, crypto keys, and IPC, we really want long IDs. I think they kind of have to for security.
 * Amit: Plus 48-bytes is long enough that generating app ID with a hash of a human readable key will be "universally unique".
 * Leon: Plus with diversity of a generic type, boards that don't care and are constrained can just go straight for 32-bit types.
 * Johnathan: I had kind of assumed originally that it would be super hard to do generically. But I think the language has improved which will make it better.
 * Leon: I have a mostly-worked-through example that will improve this.
 * Amit: Plus in the common case, the 48-byte IDs would live mostly in flash. Plus right now we use arbitrary strings, which are reasonably long anyways.
 * Brad: Why is 48-bytes better for unique IDs? Won't many people just call themselves "test" or something that all hash the same?
 * Amit: We can add more path java-style to make them more unique. Basically I was saying that more than 32-bits is really helpful to having UUIDs.
* Brad: I'm a little worried about "testapp" because those are exactly the users that we want this to "just work" for. People assigning keys are already gonna figure this out.
* Amit: So those cases won't set anything in the TLV and the board will just automatically create one.
* Johnathan: Could have elf2tab default to generating an app ID by hashing the package name. This would have the effect of making binaries with the same package name map to the same app ID. Our documentation could then describe that different binaries should have different names. Would be complex internally, but with a simple rule for users.
* Phil: I've been reviving TRDs as I'm doing Tock 2.0. Johnathan would you be interested in writing something like that up for this?
* Johnathan: I'm already writing something along those lines up for libtock-rs.
* Phil: Important for people to understand why it is this way and also understand important points if they are making changes.


