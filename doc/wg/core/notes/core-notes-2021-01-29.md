# Tock Core Notes 2021-01-29

## Attending
 * Pat Pannuto
 * Amit Levy
 * Alistair
 * Leon Schuermann
 * Arjun Deopujari
 * Philip Levis
 * Johnathan Van Why
 * Hudson Ayers
 * Brad Campbell
 * Vadim Sukhomlinov

## Updates 
 - phil: For Tock 2.0, almost all system call capsules are done as we have a handful left.  One that I have a question on is "SD card".
 How do we test it?
 - amit: It is on Signpost.  I think its better to treat it the same way as the other Signpost capsules and defer testing until some 
 Signpost-specific testing.  We don't have boards which utilize it.
 - Brad: For Tockloader, I pushed a commit to reconnect Tockloader to a board if the board goes away for a small amount of time (ex: reset).
  
## https://github.com/tock/libtock-rs/pull/269
 - Johnathan: PR references the implementation of a trait I already merged into libtock-rs which represents system calls.  Trait designed for
 current 2.0 ABI with no support for 64-bit platforms.  Can we resolve comments conerning this in PR?
 - Amit: What is the delta between supporting 64-bit and 32-bit?
 - Leon: I think we (Leon and Phil) concluded that we dont have a 64-bit ABI and shouldnt care about it at the moment.  Later on, we might want to
 think about the implications of supporting 64-bit platforms (capsule code-userspace interfaces, buffer references).
 - Amit: Is supporting 64-bit untenable/messy or inefficient (in terms of memory)?
 - alistair: 64-bit ABI should return a 64-bit value so we should return "usize" instead of u32.
 - phil: What happens if process compiled for 32-bits calling into 64 bit kernel?
 - amit: Makes sense that 32-bit code shouldnt work on 64-bit hardware
 - phil: Are there 64-bit platforms that do not have virtual memory?
 - alistair:  There could possibly be in the future.
 - amit:  Some things are bit-width specific like allow which passes a pointer which is "usize" on both ends.  Some cases where passing "usize" makes sense and sometimes it doesnt (receiving data from a 32-bit temperature sensor where there is a max value).  How much does it hurt to kick
 can down road?
 - Johnathan: Two spots where Alistair called a change to "usize": 1) return from raw yield (this can be u8 to make structs pack tighter in memory) 
 and 2) is commannd number/subscribe number/allow number.
 - Leon: Weve only cared about 32-bit and kernel code usese "usize" and u32 as interchangeable.  These changes should be discussed when a 64-bit 
 ABI is being designed.
 - Johnathan: There is one 64-bit platform which emulated a 32-bit platform but its still a 64-bit platform so maybe it is a real ABI?
 - Leon: I think the subdriver number should be fixed-width but I would be careful about changing values from "usize" to "u32" or vice versa
 and this should be dealt with at a later release.
 - Johnathan: I need to get much of libtock-rs ported to 2.0 in 2 weeks so I cant wait on a 64-bit ABI design so I dont want to block this PR on that.
 - Amit: We would need discussion and a working model to validate what needs to be fixed-width or variable-width.  How much does kicking the can 
 down the road hurt us? 
 - phil: We decided to use "usize" throughout the kernel.  Supporting 64-bit platforms will be a pretty significant effort.  Transitioning only
 this part of things to 64 bits should be only a small fraction of that.
 - Johnathan:  This particular trait is "RawSysCalls".  This should be internal between libtock platform, libtock runtime, and libtock unit tests.
 Only these 3 crates would need to change to match a change in the ABI.  Higher level calls like "subscribe" shouldnt need to change much.
 - Leon: Rust integer-safety would help us in refactoring and whether we want to use "u32" or "usize".
 - Alistair: Im not opposed to kicking the can down the road on this but it might bite us in the end.
 - Amit: Will it hurt us more in the future than if we made the change now?
 - Alistair: It might bother people using these syscalls (since they are public).
 - Johnathan: I dont anticipate a lot of users of "RawSysCalls".
 - Amit: Can we mark these features as deprecated or private?
 - Johnathan: We could.  I probably should have had a comment on "RawSysCalls" on to only use them to implement SysCalls. I could make it impossible to call these function from a crate other than libtock platform.
 - Amit: An unstable warning is better I think.  We should make clear that this is a volatile interface.
 - Alistair: I agree.
 - Johnathan:  I can mark these as deprecated and give an error message upon use.
 - Amit: I think there is a way to mark an interface as unstable.
 - Johnathan: I think that is internal to the toolchain.
 - Alistair: I think we throw a comment in.  Making it uncallable is a bit too extreme.
 - Johnathan: Okay. Ill add those comments and make them more clear about the volatility of this interface.

 ## Usage of Generics in System Calls
 - Phil: references https://github.com/tock/tock/pull/2387
 - Phil: When I was porting I2C master, it was taking a generic type of the underlying I2C but I pulled it out because we generally transitioned to using dyn.  Hudson pointed out this is an extra vtable lookup (more cycles) and we suggested keep using generics for syscall driver types. So when should we use each? Is there a consensus?
 - Amit: Early on, I was in favor of generics because dyn types add a layer of indirection.  This is an aesthetic question.  How much generic nesting should we tolerate in favor of the "right" thing.
 - Leon: For drivers, typically there is only one instance of each of them per board.  I think we should not rely on vtables.
 - Brad: I agree that our code should not necessarily be in easy to read.  I dont want to force PR authors to rewrite code if they use dyn.
 - Leon: My previous comment applies only to drivers.  I was not making a blanket list.
 - Amit: One must use dyn for some features such as polymorphic data structures.
 - Hudson: Every virtualization capsule also requires this.
 - Amit:  How about for syscall capsules (driver trait implementors), we prefer generics.  If the types get too unwieldy we can change it. Is that reasonable?
 - Phil: Is this an aesthetic thing?
 - Amit: If you end up with a complex stack of things, it can be hard to read and write and instantiate so it is aesthetic preference in a sense.
 - Phil: What if prefer generics unless the stack or number of generics is greater than "N", then use dyn.
 - Amit: That is reasonable.
 - Leon: We could have common generic subexpressions in type aliases which can also be generics and which can reduce the visual clutter.
 - Amit: That's true.  The only problem with Phil's solution is that it would vary from board to board.  It's hard to have a general rule which works across boards.
 - Leon: Generics dont only get rid of vtables but also are a gateway to inlining.  This helps in trimming down the code.
 - Phil: Syscall drivers are at the top of the stack but what of virtualizers?  Its hard to make these tradeoff judgements without hard numbers on memory usage and cycles.
 - Amit: That is true.
 - Phil: I am more concerned with code size than cycles.  If there is 5% saving on code size by using generics I think is worth it.  Ill look at i2c and see if I should put generic back in.  Over the next month or so, we should get numbers to generate more specific guidelines on this.

 ## Exit System Call
 - Phil: references https://github.com/tock/tock/pull/2378 
 - Phil: There is now exit-terminate (app terminates) and exit-restart (app restarts).  The kernel can do whatever it wants, however.
 - Brad: PR looks good but it conflicts with stack pointer changes.  It has a large diff with master branch.
 - Phil: I pulled the restart function into a trait.
 - Hudson: First, we can merge master into 2.0.
 - Leon: I can do that soon.
 - Pat: Can exit fail?
 - Phil: What happens if you invoke exit with a bad exit identifier?  That can fail.
 - Amit:  Exit with a 0 or 1 must always succeed as outlined in the documentation.
 - Brad: try_restart or restart should be exposed.  When do you call each?
 - Phil: try_restart has a policy in it and it invokes the policy before calling restart.
 - Brad: Do we need restart exposed?
 - Phil: If some part of the kernel wants an app restarted it must go through the restart policy.
 - Brad: It depends on who gets to determine the policy.  Should it be a global kernel policy or userspace?  I think this should be
 a separate PR if we are changing how restart is designed.
 - Brad: The board decides the policy.
 - Phil: I thought the process type implements the policy?
 - Brad: It doesnt have to.
 - Phil: We expect the policy are constrained to the state the process has.
 - Brad: Restart policy can have different states coupled to the process
 - Phil: Do you suggest we move restart function out of processtype trait?
 - Brad: Yes, for this exit syscall PR.
 - Phil: Okay. Ill do that.
Fin.
