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
 Is it on Signpost? How do we test it?
 - amit: It is on Signpost.  I think its safe to treat it the same way as the other Signpost capsules and defer testing until some 
 Signpost-specific testing.  Its useful for boards with a flash chip but we dont have any boards like that.
 - Leon: I could try to test it with a qemu emulator I have but I cant promise that will work.
 - Pat: I wont be able to do Signpost testing for a couple of weeks as I am to leave San Diego.
 - Brad: For Tockloader, I just pushed a commit to automatically reconnect Tockloader to a board if the board goes away for a small amount 
 of time (ex: reset).
  
## https://github.com/tock/libtock-rs/pull/269 (implications of 64-bit platforms for syscalls)
 - Johnathan: PR references the implementation of a trait I already merged into libtock-rs which represents system calls.  Trait designed for
 current 2.0 ABI with no support for 64-bit platforms.  Can we resolve comments conerning this in PR (related to design issues of 32-bit vs 64-bit)?
 - Amit: What is the delta between supporting 64-bit and 32-bit? Is it just a matter of "usize" vs "u32"?
 - Leon: I think we (Leon and Phil) concluded that we dont have a 64-bit ABI and shouldnt care about it at the moment.  Later on, we might want to
 think about the implications of supporting 64-bit platforms.  Passing buffer refs/lengths/app data, these should be platform-widthed fields.  Should capsules have the same integer-width interface with userspace on every platform?  I think its fine to define the ABI for only 32 bits for now.  We might want to think more about implications of 64-bit later on.  
 - Amit: Is supporting 64-bit untenable/messy or is it a matter of losing 32 bits of possible data if we use "usize" instead of "u32"?
 - alistair: 64-bit ABI should return a 64-bit value so we should return "usize" instead of u32.  We are returning "u32" now.
 - phil: What happens if process compiled for 32-bits calling into 64 bit kernel?
 - amit: It makes sense that 32-bit code shouldnt work on 64-bit hardware.  This is similar to running an ARM process on RISCV hardware.
 - phil: Are there 64-bit platforms that do not have virtual memory?
 - alistair:  There are 64-bit platforms without MMUs.  There could possibly be in the future.  Im not sure.
 - phil: It seemed unlikely there would be 64-bit platforms without virtual memory.
 - alistair: There was a PR on running Tock on a 64-bit RISCV board.
 - amit:  Some fields are fundamentally bit-width-specific like allow which passes a pointer which is "usize" on both ends.  Some cases where passing "usize" makes sense and sometimes it doesnt (receiving data from a 32-bit temperature sensor where there is a max value).  How much does it hurt to kick can down road on a 64-bit ABI?
 - Johnathan: Two spots where Alistair called a change to "usize": 1) return value from raw yield (this can be u8 to make structs pack tighter in memory) and 2) is commannd number, subscribe number, allow number.
 - Leon: Weve only cared about 32-bit and kernel code uses "usize" and u32 as interchangeable.  These changes should be discussed when a 64-bit 
 ABI is being designed and not now.
 - Johnathan: There is one 64-bit platform which emulated a 32-bit platform  that this struct supports (unit test environment for libtock-rs supports 64-bit platforms) but its still a 64-bit platform so maybe it is a real ABI?
 - Leon: I think the subdriver number should be fixed-width but I would be careful about changing values from "usize" to "u32" or vice versa
 so this should be dealt with at a later release.
 - Johnathan: I need to get much of libtock-rs ported to and working on 2.0 in 2 weeks so I cant wait on a 64-bit ABI design so I dont want to block this PR on that.
 - Amit: We would need discussion and a working model to validate what needs to be fixed-width or variable-width.  How much does kicking the can 
 down the road hurt us?  After merging this PR, does integrating 64-bit platforms become much harder? 
 - phil: We decided to use "u32" throughout the kernel to be explicit about size but started incorporating "usize" to accomodate with Rust preferences.  Supporting 64-bit platforms will be a pretty significant effort.  Transitioning only this part of things to 64 bits should be only a small fraction of that.
 - Johnathan:  This particular trait is "RawSysCalls".  This should be internal between libtock platform, libtock runtime, and libtock unit tests.
 Only these 3 crates would need to change to match a change in the ABI.  Higher level calls like "subscribe" shouldnt need to change much.
 - Leon: Rust integer-safety would help us in refactoring and whether we want to use "u32" or "usize".
 - Alistair: Im not opposed to kicking the can down the road on this but it might bite us in the end.
 - Amit: Will it hurt us more in the future than if we made the change now?
 - Alistair: It might bother people using these syscalls (since they are public) if we change the calls at a later date.
 - Johnathan: I dont anticipate a lot of users of "RawSysCalls".
 - Amit: Can we mark these features as deprecated or private?  
 - Johnathan: We could.  I probably should have had a comment on "RawSysCalls" on to only use them to implement SysCalls. I could make it impossible to call these function from a crate other than libtock platform.
 - Amit: An unstable warning is better I think.  We should make clear that this is a volatile interface.
 - Alistair: I agree. That is the best option to me.
 - Johnathan:  I can mark these as deprecated and give an error message upon use.
 - Amit: I think there is a way to mark an interface as unstable.  The "unstable attribute".
 - Johnathan: I think that is internal to the toolchain.
 - Amit: We should try to communicate as loudly as we can that this is an unstable interface.
 - Alistair: I think we throw a comment in.  Making it uncallable is a bit too extreme.
 - Johnathan: Okay. Ill add those comments and make them more clear about the volatility of this interface.
 - Amit: I think we are all satisfied with this.

 ## Usage of Generics in System Calls
 - Phil: references https://github.com/tock/tock/pull/2387
 - Phil: When I was porting I2C master, it was taking a generic type of the underlying I2C but I pulled it out because we generally transitioned to using `dyn`.  Hudson pointed out this is an extra vtable lookup (more cycles) and suggested keep using generics for syscall driver types. So when should we use each? Is there a consensus?  One might want to use generics for alarm where timing is of great importance.
 - Amit: Early on, I was in favor of generics because `dyn` types add a layer of indirection but dont result in different compilation artifacts for different instantiations of a type.  Generics can also get unwieldy if several layers of generics depend on each others.  This is an aesthetic question.  How much generic nesting should we tolerate?
 - Leon: For drivers, typically there is only one instance of each of them per board.  I think we should not rely on vtables.  I dont think multiple layers of indirection from using generics is an excuse to not use this optimization of rust?
 - Brad: I agree that our code should not necessarily be in easy to read.  I dont want to force PR authors to rewrite code if they use `dyn`.
 - Leon: My previous comment applies only to drivers.  I was not making a blanket list.  Generic structs around the kernel could use `dyn`.
 - Amit: One must use dyn for some features such as polymorphic data structures.
 - Hudson: Every virtualization capsule also requires this.
 - Amit:  How about for syscall capsules (driver trait implementors), we prefer generics?  If the types get too unwieldy, we can change it. Is that reasonable?
 - Phil: Is this an aesthetic guideline?
 - Amit: If you end up with a complex stack of things, it can be hard to read and write and instantiate so it is aesthetic preference in a sense.
 - Phil: What if prefer generics unless the stack or number of generics is greater than "N", then use `dyn`.
 - Amit: That is reasonable.
 - Leon: We could have common generic subexpressions in type aliases which can also be generics and which can reduce the visual clutter.
 - Amit: That's true.  The only problem with Phil's solution is that it would vary from board to board.  Levels of virtualization and nesting can vary across boards.  It's hard to have a general rule which works across boards.  Then again, using generics motivates one to not create a lot of nesting. 
 - Leon: Generics dont only get rid of vtables but also are a gateway optimization to inlining.  This helps in trimming down the code.
 - Phil: Syscall drivers are at the top of the stack but what of virtualizers?  Its hard to make these tradeoff judgements without hard numbers on memory usage and cycles.
 - Amit: That is true.
 - Phil: I am more concerned with code size than CPU cycles.  If there is 5% saving on code size by using generics I think is worth it.  I'll look at the i2c and see if I should put generic back in.  Over the next month or so, we should get numbers to generate more specific guidelines on this.
 - Amit: An undergraduate e-mailed me concerning the completion of an undergraduate thesis on Tock.
 - Phil: Let us coordinate with him. 

 ## Exit System Call
 - Phil: references https://github.com/tock/tock/pull/2378 
 - Phil: There is now exit-terminate (app terminates) and exit-restart (app restarts).  The kernel can do whatever it wants, however.  The call is a preference.
 - Brad: PR looks good but it conflicts with stack pointer changes.  It has a large diff with master branch.  Did you change the restart function?
 - Phil: I pulled the restart function into a trait.
 - Hudson: First, we can merge master into 2.0.
 - Leon: I can do that soon.
 - Pat: Can the exit syscall fail?
 - Phil: No (references documentation).  However, What happens if you invoke exit with a bad exit identifier?  That can fail.
 - Amit:  Exit with a 0 or 1 must always succeed as outlined in the documentation.  An identifier other than these two variants would lead to undefined behavior.
 - Brad: `try_restart` function or restart should be exposed.  When do you call each?
 - Phil: `try_restart` has a policy in it and it invokes the policy before calling restart which is the mechanism.
 - Brad: Do we need restart exposed?
 - Phil: If some part of the kernel wants an app restarted, it must go through the restart policy.
 - Brad: It depends on who gets to determine the policy.  The policy has been configured by the board.  Should it be a global kernel policy or a policy from userspace?  I think this should be a separate PR if we are changing how restart is designed.
 - Phil: I think it is the latter.
 - Brad: I created it as the former.
 - Phil: If it is a global kernel policy, a process can try to find out what the policy is and then circumvent the policy.
 - Phil: If we pull restart out of the trait so the mechanism isnt exposed the kernel then the `ProcessType` decides the policy.
 - Brad: The board decides the policy.  The `ProcessType` implements the policy.  The code is specified by the board.
 - Phil: By stating the global kernel policy is in the process, we should expect those policies are constrained to the state the process has access to?
 - Brad: I dont think thats necessarily true.  The restart function can have other state and can access the policy's state.
 - Phil: Do you suggest we move restart mechanism out of `ProcessType` trait?
 - Brad: Yes, for this exit syscall PR.  We can revisit the restart policy later.
 - Phil: Okay. Ill do that.
Fin.
