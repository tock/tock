# Tock Meeting Notes 2025-06-18

## Attendees
 - Branden Ghena
 - Amit Levy
 - Alexandru Radovici
 - Brad Campbell
 - Leon Schuermann
 - Johnathan Van Why
 - Pat Pannuto


## Updates
### MobiSys Tutorial
 * Brad: Pushed a new release of elf2tab because we wanted to use the new signature format for signing TABs. Version 0.13 I think
 * Amit: We also merged a bunch of stuff for the MobiSys tutorial
 * Brad: Yes. Key signing and whatnot. There's still a PR for Libtock-C
 * Brad: Functionality for demo: we can inspect processes from userspace (experimental for debugging) name, ID, start/stop, them. Lets you explore Tock from an app. We can also dynamically add new apps. The new app binary is compiled inside of another app, so it's not over-the-air, but you can press a button and a new app is installed without a restart. The third piece is ECDSA signatures with multiple keys, so you can have multiple apps which are signed and make different permission policies based on which key they're signed with. In the demo we control access to the buttons based on this. Without privilege you still run, but the system call doesn't work
### Tock Office Hours
 * Amit: First one tomorrow, noon eastern. I told people not on the core working group that for now we're going to disinvite people from this meeting to streamline, but invite them to the office hours to still get to chat about their specific interests.
### Libtock-C Build System
 * https://github.com/tock/libtock-c/issues/511
 * Amit: Bobby has a partial start to a port of Libtock-C to CMake. We've had a lot of back-and-forth about using non-Make build systems to try to improve things. So this is one potential option there


## x86 ABI
 * https://github.com/tock/tock/pull/4452
 * Amit: This PR is at a stalemate where it's unclear how to proceed. To summarize: in an attempt to support Virtual Memory, this PR first changes the ABI for x86 to make it work with both a flat, shared memory address space but also with virtual memory where the virtual address of the process stack is not the same as the kernel's view of physical memory. So putting arguments on the stack is tricky, but this new PR doesn't conform with the C ABI, so there are possible implications for userspace and performance costs.
 * Amit: So, high-level we want to figure out whether to accept these changes to the ABI or not. I'm not sure how we go about deciding on that
 * Alex: Background, in a non-flat memory space, putting arguments on the stack becomes tricky because the stack could be at a page boundary. We could map that page, read args, unmap page, which costs a lot of cycles. So we tried to use registers instead.
 * Alex: The problem is when yield returns. All system calls work and return arguments, but yield makes a function call happen instead. Bobby made this look like a C return function for yield. I think there will be other architectures where this will be tricky. This is about the upcall. When you yield, the actual yield doesn't return. The kernel simulates you calling another function, then after the upcall returns, that returns to the yield location. So the kernel continues execution from a different function
 * Pat: The point here, which is a good one possibly for Tock 3.0, right now the yield syscall imposes a specific userspace function call ABI, which is not quite the same thing as a syscall ABI. You could not use the non-C ABI for linking, for example. That's something that works cleanly in ARM and RISC-V, but it's less clean in x86.
 * Amit: Why is it less clean in x86?
 * Pat: There are more function calling conventions
 * Alex: 64-bit uses functions for calling. But 32-bit uses a stack. So we enforce that the args are on the stack
 * Amit: Why is that an issue?
 * Alex: So on ARM those args are in the registers, but they get pushed into the stack automatically. In x86 if you want to push things to the stack you have to do it manually
 * Amit: In ARM we push them to stack and the return pops them
 * Pat: There is a difference. In ARM, caller-saved registers are pushed by hardware. In RISC-V and x86 I'm not sure that happens.
 * Amit: It does not
 * Pat: So x86 and RISC-V look similar here
 * Alex: You still need to write the upcall with the C calling convention
 * Amit: I'm still not getting the problem. Just do that? What's the issue?
 * Alex: It's tricky. If this is a synchronous system call, that's okay. We could map the pages in memory to write to them. But if we're running another process that process's memory could be mapped at the moment.
 * Pat: The user-kernelspace boundary setprocessfunction is tricky. You suspended the running process and the function gets called at some point later to write into the process. Later on we actually enqueue that process. So at the window of time when you call the set process fucntion, the MPU isn't configured for that process, which makes it awkward
 * Alex: You could keep it in state and just write it just before switching back
 * Alex: So, there are multiple solutions. One was to use registers, which is just less code
 * Pat: Could you have a generic trampoline which is the function that's always the upcall handler? I think that won't work cleanly, nevermind
 * Amit: The only place setprocessfunction is actually called is in the scheduler when we're about to schedule a process. It's separated, but we do push arguments onto the stack at the moment when we plan to switch to the process
 * Leon: I think this mimics the MPU and PMP implementation. If you configure it for an application, it does a simple check to see if it's already good and returns. So we could add that check to setprocessfunction, which could check as a no-op, or configure if necessary
 * Leon: On a higher-level, we're missing the point as to why this PR stalled. We're making changes to an ABI which isn't in a stable release. So we have conflicting requirements of not breaking things for downstream users but also haven't promised any stability guarantees. I actually think this new register approach is better
 * Amit: Despite not being stable upstream, it is widely deployed and used. It's unclear to me because I don't have a sense that this is for-sure going to work out for MMU configuration, so there's risk of churn here. That doesn't mean we don't merge this, but maybe it should be merged and we should look at the whole thing before assessing that it's actually good.
 * Leon: I do fully agree with that. I think from the kernel side, we could maintain both of the variants for a brief time. We could envisions implementations of the boundary struct for a virtual memory system using registers and a flat address space implementation using the stack. I'm worried that we can't transparently switch things out in userspace without having to change code quite a bit
 * Pat: The long term here was why I opened the tracking issue about having Tock as a proper triple in toolchains. (https://github.com/tock/tock/issues/4464) The long-term vision is that you could have a toolchain that's i386-tock-mem and i386-tock-reg and everything would just work. There's a path that's not painful, it's just very long term
 * Leon: So is it true that there's no way to have both of the user interfaces compatible from the perspective of someone using libtock-c?
 * Pat: If you want to pass an arbitrary function pointer to be called, that has to follow cdecl, which means it must use the stack.
 * Leon: What if we stored a function pointer in a global static and had a trampoline function that called it?
 * Pat: I think there's probably something viable there with careful coordination
 * Amit: Okay, so is it a fair assessment that we don't know for sure that this is the right way forward? And if that's true, does it make sense to have something like an x86-next arch crate which moves rapidly, can break things, and more clearly doesn't interfere with current downstream usage. What I want to avoid is people having to continuously change their downstream to track upstream Tock
 * Alex: I will say that we don't know if this current change is good or bad. It seems good for codesize and speed, but we'll have to see.
 * Amit: Yeah, really it depends more on virtual memory issues that aren't implemented yet that we'll have to try out
 * Alex: So if we wanted an x86-next crate, how would we ever get to a point of merging the two together? I'm worried we'll diverge and not be able to combine them
 * Branden: Would it be so bad to have an x86-flat and an x86-virtual architecture? Certainly, we have to make the userspace implementations different
 * Leon: The other option is that we could just have virtual use this same existing interface, even if it's awkward, then make a bigger switch later if necessary
 * Brad: I want to second Amit in that ABIs were the one thing we stabilized. It's been a while since we had a release, but we're going to release sooner rather than later. So I think we should be careful about making changes. Similarly, we don't have a TRD for x86 at all, but we really should, so that should be on the roadmap too. We should have a way to be able to check whether what we're changing is compatible or not.
 * Brad: Or we could say it's fully not stable without a TRD, but I'm worried we're in a tricky intermediate ground
 * Amit: I agree. Having an x86 TRD should be a pre-req for stabilizing and an action item
 * Amit: Summary, two actions. First, the PR should be modified to move into a new, separate crate. I'll put a comment on the PR about that. Second, the x86 working group (which is almost established) will start drafting an x86 TRD.


## Userspace Readable Allow
 * Leon: Someone reached out on Matrix channel why userspace readable allows don't have any users. I mistakenly said that it's just aliased to userspace read/write allow system calls. But they mentioned that we have a separate entry for it from Alistair's work (https://github.com/tock/tock/blob/fda330e991a2f63254c76ffe63d7069e3d860507/kernel/src/syscall_driver.rs#L272-L289) but it's not implemented the same way and has a separate syscall method. So this was either an oversight in porting back in the day, or an omission that was fine then but maybe isn't now.
 * Leon: So, I wanted to know if anyone knew about this and had thoughts
 * Brad: My understanding was that there was no concious decision to not have it in the same way. But this is just sort-of what happens in an open-source project. I meant to get to this and update it, but never got around to it. So I think we should do the same porting we did with others, make them all consistent, and that's that.
 * Leon: Who's responsible for that?
 * Brad: Whoever feels motivated I guess
 * Amit: This could be an opportunity to recruit a new contributor. One of us could suggest shepherding the person who brought it up to deal with it. I think this isn't difficult with the right pointers.
 * Leon: It's not difficult, but I'm not sure they had a use case and wanted to do anything with it.
 * Brad: While I like the idea Amit, this requires changing some of the most unsafe code in the kernel. On the surface it's not that hard, but once you look at the diff you'll be worried about it. It touches some gnarly parts of grants.
 * Amit: In that case, I can take it on my plate. Can I ask that we maybe bring it up to remind me?
 * Brad: This is low on the list of things I'd like you to work on though... It's one of those things where you can just use it today like allow used to work. We only have like two uses so porting isn't too bad. But I think it's going to require going through the entire grant file checking for usage of data structures.
 * Leon: Part two is whether anyone is using this and why not. Presumably it works?
 * Amit: It has one use upstream, but I haven't heard of anyone asking questions about it
 * Leon: So maybe we should just leave it be, or we could take actions on it. It's got a special place in the set of infrastructure that's upstream and touches really core places in the kernel, but no one is using it. So it could be possible to use it to break semantics and it's uncomfortable to have a not-well-tested/maintained thing that's so critical.
 * Brad: We should test it better. I'd be surprised if it didn't work since it's just an allow. My impression is that we're still in the period where maybe there is a use. It mirrors things in other systems, but requires the right user to come along to want it. The semantics question is important, but hopefully we thought about that when merging it.
 * Leon: The key difference is that this is the only allow that's refusable or where a capsule could mess up buffers. That could break promises we've made in documentation
 * Brad: But the updated implementation would fix that?
 * Leon: Yes. That plus some tests would be sufficient.
 * Leon: Outcome is that someone should fix this. I added it to a task list somewhere pretty far down.

