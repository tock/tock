# Tock Network WG Meeting Notes

- **Date:** November 30, 2023
- **Participants:**
    - Alex Radovici
    - Tyler Potyondy
    - Branden Ghena
- **Agenda**
    1. Updates
    2. OpenThread in Tock Design
- **References:**
    - [Tock OpenThread Designs](./2023-11-30/tyler_openthread_designs.pdf)
    - [Leon's Encapulation Paper](https://dl.acm.org/doi/10.1145/3625275.3625397)


## Updates
- None


## OpenThread in Tock Design
* [OpenThread Design Diagrams](./2023-11-30/tyler_openthread_designs.pdf)
* Tyler: Overview want full thread networking, the parsing and stuff is tricky. OpenThread C library does this well. So we want to re-use their implementation. Other OSes do this: Zephyr, FreeRTOS, etc. Bespoke implementation was good to start, but for most of the requirements they don't seem to timing sensitive and Tock can handle them from Userspace (we hope).
* Tyler: So I have some design ideas for today. Three, but I think the third is most useful.
* Tyler: Design 1 - We have a Thread capsule controller which can talk to Apps and to an OpenThread app-service. Apps would have to make syscalls. Thread would have to make syscalls. It's the obvious design, but a lot of syscalls and latency. Possibly some timing issues with Thread. Maybe would work?
* Tyler: Designb 2 - We have a the same apps, but no Thread capsule. IPC from apps to OpenThread for Thread requests. Only OpenThread would do syscalls to 15.4, crypto, rng. In practice, our IPC implementation might require syscalls anyways, so this isn't really an improvement.
* Tyler: Design 3 - Use Leon's "encapsulated library" option. So OpenThread would be an encapsulated crate, like Leon presented at Tockworld this summer. So apps would do syscalls to a Controller capsule that talks to it, but no full context switches or syscalls are needed between the Controller capsule and OpenThread library. At least, I believe they would be lower cost calls.
* Tyler: My first implementation is going to go with Design 3, but just have a C dependency to start for testing purposes. We'd put OpenThread in a crate with the Foregin Function Interface, then pull in that crate into Tock and link against it / compile it. I'm not 100% familiar with linking or compiling, so not 100% sure how this will work. Open to thoughts.
* Alex: I did this with a C++ binary. It did seem to work. The smaller the interface the better. Exceptions can propogate too, which is bad.
* Tyler: How did you create the interface?
* Alex: For the ESP, we couldn't use bindgen, I guessed the interface signatures. Bindgen did work in a different project, but didn't do Enums well. C will keep an Enum in the smallest size possible, but Bindgen will always keep Enums as 4 bytes. So you need a flag when you compile the C code to keep Enums at the maximum size
* Tyler: So Alex, for the ESP32 you needed to reverse engineer the library. But if you have the source did you use Bindgen?
* Alex: Yes. Bindgen worked, but we only did C interfaces not C++.
* Tyler: Did you use the CC crate to compile?
* Alex: I'm not sure. But I can connect you with the person who did this, Dan.
* Branden: So the ESP32 had a binary blob.
* Alex: What would happen with the ESP32 is that the function call would go through, but the kernel will still crash. Not sure why
* Alex: Bindgen has two ways of dealing with Enums. One way is defined constants, the other is Rust enums. But the Rust enums are different from the C enums from the compiler. Different representations led to crashes.
* Tyler: Did you do static linking of the library? Or compile it as part of the Rust project?
* Alex: Not sure
* Tyler: One tricky part is that OpenThread does have a big build system. Which we don't want.
* Alex: How does Zephyr do this? They have to include it in their build somehow.
* Tyler: I did that approach. But a lot of what happens with building OpenThread for Zephyr is through West and I was having a hard time understanding it. A lot of unrelated stuff.
* Branden: So you want to compile it as part of Tock, not as a seperate, already-linked binary blob?
* Tyler: I think. There are build scripts for OpenThread, which has a lot of nested build scripts. The first specifies the platform which invokes CMake and Ninja and eventually it links the OpenThread repo into the platform-specific stuff. That's just one big script and then you flash it.
* Branden: If you were just going to compile OpenThread as a binary blob, adding it to the kernel should be easy from a memory standpoint: just reserve a chunk of memory in the LD file. I'm honestly not sure how to connect up functions to that blob and calls from that blob though.
* Alex: So, Tock needs to be asynchronous in the kernel. Is OpenThread async, or does it wait for a while?
* Tyler: That's a good question. I don't think it's a huge concern for just playing around with it. The way I thought about this: if OpenThread hangs, is there a way to reclaim control from it? I don't have an obvious answer here. I do doubt OpenThread is async, but I'm not sure if we could have an interrupt that takes control back.
* Alex: You'd need a different stack. I don't remember, but don't think, that Leon's stuff supports that. You'd never be able to continue running OpenThread again, if you mess up its stack.
* Tyler: Looking at Leon's paper (https://dl.acm.org/doi/10.1145/3625275.3625397), I think there is a separate stack?
* Branden: Basically, we're thinking about treating it like a process or like a capsule. If a capsule hangs, the kernel hangs. If a process hangs, we can timeslice it and come back later, or even restart it.
* Alex: But can you continue it afterwards if you time slice it? Would the network still work?
* Tyler: Not sure. Thread does save stuff to nonvolatile memory, so just restarting should maybe be okay and not even time things out in the best case. Maybe there are ways to work around this.
* Tyler: So the question is whether OpenThread is asynchronous enough to keep the kernel running.
* Alex: Yes, if incoming interrupts stop it. And we can wrestle control back. We'd HAVE to treat it as a process but avoid a context switch? But that code would need to be trusted, as it could theoretically stop its sandbox if it's in supervisor mode?
* Tyler: I do think Leon set up the encapsulated function to be untrusted
* Alex: Too much speculation here. Need to read paper and/or talk to Leon and get back to you
* Tyler: Yeah. Great points though. Definitely concerns I shared
* Branden: One extra concern for you. Be aware that Leon's encapsulation stuff is a research project, not a fully functional library of its own. So you're going to have to think of yourself as a developer for that project if you want to get it working, not just a user.
* Tyler: Agreed. That was my expectation and I've talked with Leon about that.
* Branden: Going way back, I also think that Design 1 could be a good place to start. It certainly seems easier than getting Design 3 working in any form.
* Tyler: Yeah, and it would be good to have for comparison for a potential paper. So I'd probably do both anyways. Pat was pushing for Design 3 to start.
* Branden: Finally, another repo for you that uses OpenThread from my previous research lab. I can put you in contact with the person who worked on it (Neal Jackson). https://github.com/lab11/nrf52x-base
* Alex: Generally, I'm excited about the encapsulated C code idea. There are lots of cases where there are existing C libraries that we want to be able to reuse rather than rewrite.
* Branden: Goal was always to do those in userland, but that doesn't seem to have panned out.
* Alex: MPU regions has always hurt that. Alignment requirements for MPU in ARMv7 makes apps use way more memory than you need. In automotive they just use the MPU for separating 1 or 2 domains. But tasks or processes are never in separate memory regions. It wasn't designed for apps. Not sure how this works on RISC-V and ARMv8 is much better, I believe.
* Tyler: What is ARMv7 versus ARMv8?
* Alex: Newer ARM cores. Some Cortex-R
* Branden: Or Cortex-M33 or things like that. nRF53 series, I think. But nRF52 is ARMv7
* Tyler: Can't we load lots of processes now on nRF52s?
* Branden: So, the problem is memory. Because of the alignment issues on the MPU, you need to waste a ton of space for each application, unless they're toy applications.
* Alex: We had a big application to controlled a screen. It MUST be the first application due to alignment issues, for example.
* Alex: Looking at the Cortex-M33 documentation for MPU alignment. For ARMv8, all regions must be 32-byte aligned. Region size must also be a multiple of 32 bytes. So that's WAY better.

