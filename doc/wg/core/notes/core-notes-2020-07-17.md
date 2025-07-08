# Tock Core Notes 07/17/2020

Attending
 - Branden Ghena
 - Johnathan Van Why
 - Brad Campbell
 - Leon Schuermann
 - Samuel Jero
 - Hudson Ayers
 - Amit Levy
 - Phil Levis
 - Alistair
 
## Updates
 * Brad: Support for fixed-address applications keeps getting better. Code is spread across many tools. Works almost as well as the PIC apps now! Tockloader knows how to interact with them successfully. Changes aren't all released yet, but seems to be working pretty well.
 
 * Johnathan: I have a PR in libtock-rs (https://github.com/tock/libtock-rs/pull/220) that needs to be reviewed and merged. I have a bunch of follow-up PRs that need to go in.
 * Phil: Approved!

## Tock 2.0
 * Phil: Two major things we've been working on are 1) read-only allow and 2) examining system call ABI for transition between kernel and userspace. There's a first draft for read-only allow. Seems straightforward.
 * Phil: Second is what the ABI should be between the two. We pass 4 registers in and 1 back right now. Problem, we can't pass back a value and an error since there's only one. Should we do 2 or 4 is the question. If you look across architectures, Linux has a very different ABI, but it's based a lot in architecture and C calling standard. Given that the ARM ABI allows a 64-bit value in registers or in the case of containerized vectors, 128 bits (we aren't using those though).
 * Phil: The reasons to stick with 2 is that it easily transfers to C calling conventions. The reason to not is that we want 64-bits and an error code. 63-bits is possible, but *messy*. The other is that if we need more than 32-bits you could pass a buffer with allow. That's possible, but being able to directly pass in registers is more efficient and preferable. So 4 greatly increases flexibility and decreases overhead for edge cases.
 * Phil: Conclusion, if the system calls return 4 registers (r0-r3 on ARM become arguments and return values) there's no assembly overhead to this. The challenge is that to C code, this is weird. So the underlying thing would have to pass in 4 pointers to integers that would be written to. This is the lowest layer though, so developers don't actually see this. For example, 64-bit timestamp request requires passing in a pointer to a 64-bit memory location anyways. So underneath libtock-c it will take what's in, for example, r1 and r2 and coalesce them into the 64-bit value.
 * Phil: If we can pass back 4 registers, how do we use those? I put out a proposal that system calls can return an error or success. Success can have 1-3 32-bit values or a 64-bit value or a 64-bit value and a 32-bit value or none. Then any system call would always have the same success type, but each driver gets to choose.
 * Phil: Pushback originally was that Linux often does what it does for a good reason. And we should understand before we diverge. But I think we got to the bottom of why Linux does this. And because libtock-rs would really like 4 registers, it seems like the right thing to do.

 * Leon: Syscall registers in userspace. I convinced GCC to compile for ARM a wrapper that takes additional pointers to the return value. Which captures what we want to do with 4 return values. It is inlined, so we aren't constrained to C ABI. The first 2 return values would be returned as a single value, the other two would be pointers to 32-bit values. The only thing that makes this problematic is that in the case where I don't want to use all values, this still does do a memory write. So my proposed example is a wrapper for the common small case. And then some wrappers for extended bigger versions.
 * Branden: Did you write this in C instead of assembly?
 * Leon: Yes. Just the `svc` was assembly. The real key is inlining the code, which means it doesn't have to follow the C ABI and doesn't have to load or store registers based on expectations.
 * Phil: Do need to be careful that we're not relying on a specific version of GCC.
 * Leon: I think what's nice is that this is nice C code which also optimizes elegantly.
 * Phil: A little wary, but I'll have to take a look.
 
 * Brad: I'm still interested in what the libtock-c looks like if there are multiple success types. Really the question is who is responsible for handling if you get the wrong success type back? That would imply a kernel bug.
 * Phil: Or a kernel/userspace mismatch. I would say it's libtock-c's responsibility to handle this.
 * Brad: Would that be a new error type? "The kernel didn't return what I expected"
 * Phil: You could. Imagine this was a similar error, i.e. pass a pointer to a struct and there's a mismatch where the kernel thinks its a different struct format. What is to be done? A driver can always return ENOSUPPORT.
 
 * Brad: Why not get 5 registers back? RISC-V passes down 5 registers, where fifth is the system call number.
 * Amit: We can only do 4 in ARM Is one reason.
 * Brad: Not necessarily. We can pick whatever.
 * Phil: You're going to want 12 registers next!!
 * Brad: Basically, if we want to be parallel input/output, we should be parallel.
 * Amit: Right now 4 is the minimum of ARM/RISC-V. So there's no overhead for up to 4 in ARM. 5 in ARM would require overhead of an additional register that it isn't currently using. The system call number in ARM is encoded in the instruction itself, and we decode that way, not needed the extra number.
 * Brad: So it's important to say that this is driven by ARM. RISC-V is unique that it has very little support for system calls. So this is still kind of architecture specific.

## Potential Tock 1.6 Release
 * Brad: We talked about this before whether we want one. This came back up for me because there's been a lot of development lately. I think it warrants another release with testing. Notable things: "Schedulers, new systick, new alarm HIL, cdc over usb, appslice fix, better fixed address app support, etc". We definitely want the alarm updates. And the new schedulers is a really cool change too. Plus, we've had multiple broken platforms in the last couple weeks that slipped through. So that signals to me that another testing release would be useful.
 * Brad: My proposal is that it's worth it if we get the alarm and scheduler PRs merged.
 * Hudson: I agree
 * Phil: Me too. For the alarm release, it works for SAM4L and I've been running a test for 2 weeks straight now which is working still. Right now Amit and Guillaume are working on the RISC-V and nRF ports. One thing that came up was, since this changes the syscall API for alarm, should it be a new device, new calls, etc.? This will require changes to libtock-c. One option is that we have a new device and update libtock-C. Option 2 is that we have a new device with new syscalls and emulate old syscalls too. Option 3 is that we keep the old device number and add new syscalls. So we would have old set alarm and new set alarm. So old things keep working, but libtock-c would update to the new one. So 1) deprecate old and have new 2) have both 3) just add new commands to old one.
 * Brad: Which option is leading right now?
 * Phil: No significant discussion yet.
 * Brad: I think option 3 is the smoothest.
 * Hudson: Would the plan be to deprecate the old style calls?
 * Amit: Yes. We just want old apps to run still until 2.0 where we decide to break ABI anyways.
 * Hudson: Sounds good.
 * Phil: We'll likely go with 3 then. My guess is that the cost is pretty limited.
 * Amit: Other than alarm HIL, what are the other things that we want 1.6 to wait for?
 * Brad: Just scheduler and the PMP issue since the board doesn't work. But for "features", just the scheduler.
 * Hudson: Scheduler PR should be rebased and tested again before the end of next week.
 * Branden: Do you want to do an elf2tab and tockloader release too? That would give fixed-address app support.
 * Brad: I do want to do an elf2tab release once the debugging fixed apps PR is merged. I think tockloader is reasonable too.
 * Hudson: I think open-titan device parameters and API for external process types too.
 * Brad: Agreed. I'm also realising that there are a few PRs for fixed-address stuff, and that is an important update as well.
 * Brad: Sounds like there is support. So I will open an issue for this.

