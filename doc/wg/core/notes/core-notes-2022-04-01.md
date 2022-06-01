# Core Working Group Meeting, April 1st 2022

## Attendees
- Hudson Ayers
- Philip Levis
- Alexandru Radovici
- Leon Schuermann
- Johnathan Van Why
- Brad Campbell
- Alyssa Haroldsen
- Branden Ghena
- Vadim Sukhomlinov
- Jett Rink


## Updates
 * Brad: No progress on stability issues by Rust community
 * Hudson: Going to EuroSec next week. Presenting a paper on Tock there.


## Ergonomic Static Apps
 * https://github.com/tock/tock/pull/3001
 * Hudson: Wondering if this is something we should be doing in the Tock kernel, or should be pushing to Makefiles for apps? Code in the kernel has a cost for everyone, even apps that don't need it. Alternatively, it seems like apps could choose addresses while building.
 * Brad: While you're developing the kernel, the address where RAM for apps starts can change. That can make it limiting about where you can put apps and how many you can put. So more flexibility in the kernel is probably convenient, because it's hard to know which addresses you might want, and it takes time to link apps for many different possible addresses.
 * Hudson: Right. So at build time you don't know where the memory region for the kernel ends.
 * Phil: I'm not sure how this would work. Doesn't the Tock kernel already handle this and it's a Tockloader issue?
 * Brad: No. Right now what the kernel does is that it looks at the loaded app, checks if it needs a fixed address, and then tries to start the apps region wherever it requested. The issue, as I understand, is that this only works if the address is well-aligned for the MPU. If it's not, creating the MPU region will fail and the process won't load.
 * Phil: I thought that when loading, it was walking forward through RAM. When loading an app, it jumps forward to a memory address if the app needs. So the issue is that if the statically assigned address doesn't align with the MPU, then allocation can fail?
 * Brad: yes
 * Leon: But won't you know the MPU alignment at app build time, for the given platform? Why would you link an application without a well-aligned address
 * Brad: Not sure
 * Hudson: I think it's a great question. I think this is coming from libtock-rs.
 * Phil: I'm trying to parse this comment in the PR. https://github.com/tock/tock/pull/3001#issuecomment-1079168845 Why doesn't 0x20004000 just work? I'll respond to the PR.
 * Hudson: Maybe when we're building apps for some fixed memory address, the fixed address we're building for we're specifying the end and the start is determined by the size? I'm pretty confused here too.
 * Brad: If the app needs more RAM, i.e., if you increase the stack. If it becomes longer than the MPU alignment, then the app might no longer be well-aligned. So if the memory requirements increase, you could go from an app alignment that works to an app alignment that doesn't work.
 * Brad: So say your app needs 2 bytes of memory. You could put it at address 2 which is aligned. But if you increase to 4 bytes of memory (for more stack size), it's still at the fixed address of 2, but that doesn't work anymore.
 * Branden: How are we picking that address anyways? How are we choosing 2?
 * Brad: It's hard-coded in the linker when the app is compiled.
 * Alexandru: Like a constant, or based on how much memory is needed?
 * Brad: I don't know libtock-rs, but it's fixed in libtock-c
 * Johnathan: It's hardcoded in libtock-rs too.
 * Leon: Okay, I think I get it. But how does this PR solve it then? Seems like a problem we can't without multiple regions.
 * Brad: Yes, so back to my original example. If we start at address 2, but need 5 bytes of memory. We start the MPU region at address 0, the app can start at 2, AND it can still have 5 bytes.
 * Leon: Doesn't this increase complexity a lot because we might have to check backwards for collisions?
 * Brad: That's correct. I think this is mitigated because the MPU interface is based on the start of memory that's available, which is already after the previous app. So I think that's going to work. Although we should double-check it.
 * Leon: We could also fix this with better checks while linking, right?
 * Hudson: We don't know where the kernel region ends though.
 * Leon: I think we might. We have to know where RAM is for our apps since they're fixed.
 * Brad: That can change, and it's theoretically fine to choose a later address and waste memory, that way if the kernel does increase you don't have to recompile your app.
 * Leon: This sounds like we would end up with a pretty suboptimal result. We could propose adding more checks in the linking process.
 * Brad: Right. The question is "why not just choose a better starting address"? That's not clear to me.
 * Branden: I'm a little worried that changing the linking process would be pretty difficult actually. Whereas this does seem to solve the problem as-is.
 * Hudson: This takes 200 bytes or so. I try to push back on these things because I don't want to waste kernel space on things that are very special-purpose and could be done somewhere else.
 * Phil: It would be nice that if you're just given binaries and can't recompile them, that you could still load them.
 * Brad: An issue is that Tockloader doesn't know where the start of application RAM would be. If you compiled an app for, say, 100 different RAM locations, you don't know how to choose. If you have one app for address 2 and one for address 16, Tockloader doesn't know which one to choose.
 * Phil: Right. In the presence of dynamic loading along with static loading, it can't know.
 * Brad: Even with only static, Tockloader would need to know MPU, and kernel memory bounds, and app memory size. Then it could pick.
 * Leon: If the kernel could tell Tockloader things, it could make more intelligent decisions.
 * Brad: Yup. But then we're adding even more code size.
 * Phil: So is this use case important? That you're given a binary, you can't recompile it, it's statically linked, and Tockloader needs to determine where to place it?
 * Alexandru: A question is how far away are we from relocatable apps? Do the GCC for Rust efforts solve this problem?
 * Brad: It seems like we need to better understand the use case. Since existing apps haven't seemed to run into this issue.
 * Johnathan: You have TI50 which is doing it's own thing and combining kernel and apps.
 * Johnathan: Upstream, libtock-rs is linked against a fixed kernel. Occasionally we have to update that and adjust the linker a little. There is some confidence that in the future we'll be able to do relocatable apps on RISC-V. But on ARM, we still won't be able to with LLVM, so we'd need GCC for Rust too.
 * Brad: Okay, so right now we have this logic that looks forward in memory and advances to meet the application requirement. If we just change that to hit the nearest MPU region, that seems fine.
 * Branden: The nearest MPU region for what size of memory?
 * Brad: The app size, plus the gap until you start memory, and the gap until the app wants its memory.
 * Branden: But you're saying that you have this info in the kernel and can make that choice.
 * Hudson: The linker script should know memory sizes, once?
 * Brad: Conceptually yes. Practically that's only known by elf2tab.
 * Jett: It's weird to me that the memory size and the alignment address are separate in the linker stuff.
 * Hudson: Okay, so we don't need a full resolution here. But I wanted to bring it up and think through what's going on with this PR. Specifically whether it should be done in the kernel at all. I'll looks some more at this on the libtock-rs side after the call and think about whether the linker could get all the right info. If that looks really hard, we could see if we can just optimize the kernel side a little.
 * Johnathan: Since the linker script is generated by libtock-rs, push comes to shove we could generate a linker script dynamically with the right size.
 * Phil: There is a future issue where, since we have signed apps, we won't be able to touch the binary but still want to load them.
 * Johnathan: But they'd be signed after the app is compiled, so if it choose the right location before then, everything's fine.
 * Phil: But I could imagine not moving it, but increasing the size it says it needs to make it work. It's not a prominent use case right now, but I just want people to keep signed apps in mind.
 * Hudson: Presumably, we're signing the TAB not the ELF. So the parameters are there and protected.

## Mixing Fixed Position and PIC apps
 * https://github.com/tock/tockloader/issues/82
 * Hudson: This seems like a good use case to support, and the author put a lot of work and thought into this. I wanted to raise it to get people to take a look and provide feedback.
 * Brad: I thought this would be a simple change. Just place the fixed position apps first, then the relocatable ones after. Not sure why it needs to be more complicated than that.
 * Hudson: The SAT solver is to optimize placement. It's probably okay to be suboptimal since this is an unusual case. Not sure why it needs refactoring of TBF handling and App classes though.
 * Brad: I can follow up and see if we can figure out what's really needed or not. It seems like a stretch to me that we're going to have so many apps in this case that we'll run out of flash before running out of RAM.
 * Hudson: Right. It does depend on the platform though.
 * Hudson: I guess the changes for the TBF handling code, we might assume that once we see one app that is fixed all future ones are fixed as well.
 * Brad: That was a bug and is now fixed. But TBF doesn't really have stuff for mixed, so might need changes.
 * Hudson: My thought on dependencies is that we probably don't care for Tockloader.
 * Brad: Only worried about complexity. Not actually a problem to have dependencies.
 * Hudson: Cases that would be really worried about auditing already don't really use Tockloader.
 * Brad: We just want to avoid other requirements or dependencies that make it hard to run on some platforms. Pure python dependencies are fine.
 * Brad: This SAT solver makes me nervous. I think there's a lot of native stuff here that might make Tockloader not work on some platforms.


## Command Result Type for Pointers
 * https://github.com/tock/tock/pull/3003
 * Jett: I made a proposal for a new success variant for Command, "success with pointer". Different semantics from u32 or u64. There are no guarantees about validity or accessibility of the pointer from the kernel.
 * Phil: I thought we were going to return an offset into an existing pointer instead?
 * Jett: Two use cases: returning a pointer into a kernel-owned buffer where the app doesn't know the address. Another hypothetical use case is some kind of system manager info where an app might want to know something about its own address space, which would involve giving it a pointer.
 * Phil: What's an example of wanting to receiving a pointer into the kernel?
 * Jett: We have a kernel-owned buffer that's leased to applications through some driver, with contents. The application can access it, and the kernel has to MPU share it first making it exclusive to that app. The application reads, modifies, and lets the kernel know when it's done.
 * Phil: Right, this is the case where we have one kernel buffer rather than a buffer in each app. So we move access to it around rather than have each app take up space making their own.
 * Jett: Yes.
 * Jett: So, this is a first-class way to return a pointer. Drivers still need an API where it makes sense. But this is just the method. Gives a name for the thing that we need. Assume we have a pointer, this return is for it, and will make it work in a platform-independent way.
 * Phil: Question, for memop, it returns a bunch of pointers already?
 * Jett: Probably. Allow takes a pointer already. There are other system calls that receive or return pointers. This allows capsules to have a return-variant for command that they can return, that's well-defined and platform independent for when returning a pointer makes sense.
 * Leon: One concern, in Tock 2.0 we have a set of system call classes which have pointers because they must interact with pointers, like allow or memop. Then commands which, conceptually, should work exactly the same on whatever platform you're running on. So, I'm fine with a new return variant that would be able to return a pointer for these cases. I wouldn't want to increase u32 to usize, because something developed on a 64-bit platform could break on a 32-bit platform. So the driver trait should be limited in the arguments to accept 32-bit values rather than staying the same across platforms. So how are we going to reflect this change in the arguments of commands?
 * Jett: I did the least-amount-of-change approach. I stayed with the restriction. We didn't need to pass in pointers as inputs, so command only takes the arguments as it has been. The change is that pointers can be returned from the kernel with command, but apps passing pointers down should go through allow or memop. And then command could return offsets like Phil said earlier. I think command inputs don't ever need to be usize, I think we can stay with u32.
 * Leon: Okay, I think that makes sense to me.
 * Phil: The concern with 64-bit is that you're in a 64-bit platform for testing, right?
 * Jett: Yes. That was the impetus of this discussion. But I think it's an important design point even apart from host-emulation.
 * Phil: So how does memop work?
 * Jett: Allow and Memop use `usize`. That's part of their definition. It's just that there's not command variant for capsule code that works with pointers. So this introduces that new variant.
 * Phil: But with memop, there's a series of calls you make to get your start addresses, flash addresses, etc. That's all success-with-u32. So it's fundamental that the whole ABI works for u32 architectures.
 * Jett: But that memop stuff happens in the board/arch sections of the kernel. So that is all architecture-dependent and handles it all appropriately. It's not defined for 64-bit systems how memop should work. I think it's clear what you ought to do, even if it's not really defined for 64-bit. I think it's okay to have `usize` for architecture-dependent code, like memop. But capsule is very much not platform-dependent. So that's where this is sticking.
 * Phil: I'm still trying to understand. It can be that the structure definitions versus actual bits that are passed. Technically, the return type for memop is success-with-u32 (https://github.com/tock/tock/blob/master/doc/reference/trd104-syscalls.md#46-memop-class-id-5). So the fact that the Rust structure holds it as a usize is okay. But ultimately userspace is supposed to assume that it's a 32-bit value. Is the host emulation actually passing a 64-bit value and getting away with it? How does that work?
 * Jett: Good question. I'll have to look at that.
 * Phil: So I think the question I asked last time, is there a reason you don't just run the host emulation in 32-bit mode?
 * Jett: We run it natively on our machines.
 * Alyssa: I don't want to revamp our toolchain just for that.
 * Phil: Well, you are asking to revamp the kernel for this.
 * Alyssa: I think it's a bug to not have a usize return type. In my opinion, you should allow some variable-width data in a return type like this.
 * Leon: There are two issues here, actually. The memop stuff could be addressed with a 64-bit ABI, but we would still be having this discussion about drivers and command.
 * Jett: I agree. Related, but two separate issues.
 * Jett: So, here we're focusing on the issue of adding the new success variant with pointer. That will help when we think about a 64-bit ABI. It's still well-defined for 32-bit ABIs too.
 * Phil: I think this is a thorny issue. There's also that there's a command returning a pointer into kernel memory. I don't see any significant issue with that, but it's different from Tock's traditional memory model.
 * Jett: Giving capsules the ability to return a pointer seems useful. Maybe it's a kernel pointer. Maybe it's an application pointer. But it's a valid return type that I think upstream should allow. Again, no guarantees on accessibility. This is just a value, but it's a usize-width value.
 * Hudson: I think it would be good to look over the prior call notes too. We talked about this two weeks ago on a call that Phil missed, so I want to make sure we're not re-hashing issues too much.
 * Phil: Oh! Definitely. Jett and I could talk about this offline too, grabbing other interested parties. I want to put some thought into this issue, for sure.
 * Jett: Sounds good

