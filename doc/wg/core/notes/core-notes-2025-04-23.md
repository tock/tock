# Tock Meeting Notes 2025-04-09

## Attendees
 - Branden Ghena
 - Amit Levy
 - Brad Campbell
 - Leon Schuermann
 - Lawrence Esswood
 - Hudson Ayers
 - Benjamin Prevor
 - Johnathan Van Why
 - Alexandru Radovici
 - Pat Pannuto


## Updates
 * Leon: On the edge of merging tock Ethernet support into master. Then userspace will be added quickly. I'll plan to write up a blog post about it. https://github.com/tock/tock/pull/4409
 * Amit: Maybe with availability of Thread in userspace and LWIP in userspace, that might be a good opportunity to think about how to do an IPC layer for using those. Good test case for them
 * Leon: Agreed


## CHERI Support
 * https://github.com/tock/tock/pull/4365
 * Amit: Specifically, how to support RISC-V assembly in a target-specific way. I created this PR, but it's Lawrence's work
 * Amit: In this, there are changes to the RISC-V crate which are at the heart of the question
 * Lawrence: One of the things is the EASM macro. It's probably unnecessary and can be removed
 * Lawrence: More interesting to discuss is the attempt to change which instructions are being used in different architectures. There are different sized loads and stores for different platforms 32, 64 or 128 bits. Otherwise the assembly is the same, just loads and stores changing. Example: https://github.com/tock/tock/pull/4365/files#diff-4b96340b56046ae24d8ac90c1e8a7cb5789285e71bfb4ca0e3fd0dfc8f233bb4 Where there are 4s or Ws, there's a call to a macro to store X where X is the size on this platform. I think this is good actually (compared to EASM which could be removed)
 * Amit: An example line of assembly would be `ldptr!() ptrreg!("s1") ", 1*{CLEN_BYTES}(s0)`. This is a load into a register of some stuff, but instead of having a load instruction directly, these are macros that turn into actual instructions based on the concrete architecture of the platform.
 * Lawrence: `ldptr` is the load instruction. `ptrreg` is a register large enough to hold the data, similar to x86 ax eax rax.
 * Pat: In other contexts, assemblers have pseudo-instructions which have the same semantics as this. Maybe these don't exist yet for RISC-V
 * Lawrence: I don't know if there is a loadX for RISC-V yet for 32/64 bit. I'm not aware of it. Certainly not once CHERI is involved
 * Leon: So we have an assembly block like switch-to-process which is very similar between architectures and just needs to change the size of registers. I sympathize with the desire to de-duplicate, but it's very helpful when debugging to read the fully existing assembly code. I think these macros lose this immediate visibility. Maybe there's a middle ground, like having the generated assembly stored somewhere. But when I view this file as a standalone file, I'm worried that it's hard to debug
 * Lawrence: I normally look at an objdump when debugging, rather than the source.
 * Leon: You lose all the context and comments with the tools I've used
 * Amit: What about `cargo expand` or something like that.
 * Lawrence: I have tooling that can expand those like that
 * Leon: That works for the EASM which is macros, but not for the conditionals in the assembly, as those are evaluated by the assembler
 * Lawrence: They are just macros that expand in Rust based on the architecture. So Rust expansion not assembler
 * Lawrence: There are also for ranges, but we could remove those. It's the loads and stores I want to keep. Otherwise we'd have four different versions of the code
 * Pat: You end up having to read straight-line assembly where there are four possible options for each line though. I would say, this is currently generic across all RISC-V compared to ARM where we have a cortex-m folder with generic stuff and have subfolders with v6m v7m v7me etc for things that are architecture-specific. How much would be changing here?
 * Lawrence: It would be an exact copy-paste just changing one letter of the load/store
 * Leon: Okay, idea, maybe we could have a common template that architectures use but we actually include the fully expanded assembly in the files and use CI to make sure that the files match the template
 * Amit: My intuition is that feels unnecessarily brittle
 * Pat: It's the case that the ldptr macro just turns into a single assembly instruction, none of the expand into multiple lines or semantics. So in that context, we're just filling a gap in the tooling where there should be a pseudo-instruction doing what these macros do. And so we're essentially creating them.
 * Branden: That seems more acceptable to me. Just one macro for a single assembly instruction
 * Pat: And I'd go so far as to document these as pseudo-instructions with good doc comments about exactly what it could expand into
 * Lawrence: So ditch the easm and the for-range, but keep the ldptr macro
 * Brad: Do we know why RISC-V isn't doing this yet? A pseudo-instruction?
 * Lawrence: I don't know. They should have one
 * Amit: What's the equivalent in ARM? Is there one?
 * Pat: A load machine word thing. I don't remember
 * Lawrence: I think it's `ldr` in ARM maybe?
 * Pat: It's a little different. It's not exactly a pseudo-instruction, it just has different operation on 32-bit and 64-bit
 * Amit: Actually the guide says it's a pseudo-instruction.
 * Lawrence: The addition here is that CHERI needs to handle it to
 * Amit: So when we hit 32-bit and 64-bit ARM, we might be able to avoid the issue there if there's a pseudo-instruction
 * Leon: Okay, I think this alleviates most of my concerns too
 * Leon: There are other ways that this file is dynamic and concerns me, but we can discuss those on the PR itself
 * Lawrence: One more thing from the PR. How much stuff should go in the global config object?
 * Brad: We have this capability pointer type, and we know it's different on cortex-m and CHERI. We need different implementations. We have relied on Rust to have something that does this for us. But we can't do that here. I figured that when we wanted to support CHERI we'd have a big if-def on two different implementations CHERI or non-CHERI. It's fixed based on the hardware you're using, so that makes it more acceptable to me
 * Lawrence: I had the framework in mind, that as a developer works on Tock, they may be touching things that affect other platforms. So it would start to get out-of-hand if CI doesn't test all the configuration options and I want to keep if-defs out of kernel code altogether. So the compiler can use if statements to choose. But if the layout of some struct changes, that gets harder. So I added some new type that can wrap other types so it looks like one thing, but if you ever touch the code you have to accept that it's a union of multiple things and they must handle both cases. So in the long term, I'm hoping this could remove more nasty problems with weird architectures that only happen at compile time.
 * Pat: One place we have run into this elsewhere is the syscall interface and MPU traits. The kernel has an abstract notion of hanging on to a reference for something, but the specific implementation can change out and is in the arch-specific trait.
 * Lawrence: We could inject types via associated types in some zero-sized type. But we don't have somewhere good to put these as it would hit everywhere in Tock today. I tried that refactor and it touches basically everywhere and is too much. So making it generic is hard because it's in too many places
 * Leon: A question about layout. What I like about these macros is that they force you to acknowledge that there are two types. But what's confusing me is that if for example "TIfCfg"
 * Lawrence: It's not a union type underneath the hood. It's actually one of two options at compile-time. So you're restricted to the interfaces which are available.
 * Leon: So in syscall.rs line `ddc: TIfCfg!(is_cheri, CapabilityPtr)`. You have to handle both cases, either unit type or capability pointer type. But the size could be accessed from assembly
 * Lawrence: Yes, but never guess the size and always use sizeof and alignof and never hardcode things. Size of structs could always change
 * Leon: So in this particular case, we don't actually get a benefit. We have to reason about both cases
 * Lawrence: Users can just treat capabilityptr as an opaque type that we can just move around
 * Leon: But internally it uses TIfCfg to choose, so internally it has to handle both cases
 * Leon: Okay, I maybe understand this now. I'm questioning the utility still, as I think it's very similar to an if-def in practice. We combine a compile-time parameter with a choice between two options
 * Lawrence: Yes, but this won't give compile-errors. You can't forget to initialize some field
 * Lawrence: In the capabilityptr file, line 116, we have an implementation of `add_assign` which forces you to implement both cases
 * Leon: Okay, so this doesn't affect anyone external. This infrastructure is for ergonomics for internal implementation of correct code
 * Lawrence: I also used this for reference counts for buffers based on whether DMA is used or not. Compilers are good at eliminating functions, but not fields in structures. So that's what this is for. Allows adding fields conditionally but safely
 * Leon: Okay that seems useful. But it should really be its own PR. It's a whole thing to reason about on its own.
 * Lawrence: I would want to land the config first though so I wouldn't have to update capabilityptr
 * Brad: I'd warn against that.
 * Pat: There are a bunch of ways this TIfConfig is used. map and TIfConfg and if statements, a bunch of mechanisms. I worried that this is pretty hard to explain and understand by new people
 * Alex: My personal opinion is that this goes the direction of the linux kernel as impossible to read
 * Amit: If the alternative is #defines, then this is still harder than we'd hope, but at least it doesn't have the downside of invisible compile-time bugs unless you have the right combination of configurations
 * Lawrence: Right. The code looks similar in both cases, but it would be compile-time choices rather than matches
 * Pat: I do strongly agree with hating #defines. But this looks like very non-standard Rust. The interface that we're adding here is challenging. Maybe there's a way to rename or encapsulate this so you don't have to learn a ton to understand what's going on
 * Leon: I think why this looks weird to me is that it's using these at the wrong layer. We have two capability pointers which can switch based on platform. Right now, this code makes choices everywhere for which thing is actually used. Maybe we could have the two implementations have the same trait, and switch between them with a single config rather than having this everywhere. We'd generally prefer to have two implementations and switch between the two once, rather than having fine-grained choices deep in the implementation
 * Lawrence: You could do both. You could wrap capability pointer around two and call the right functions. If you have two implementations and modify the trait you sometimes notice if there's a change, but not necessarily if there's a new associated type. I think there's a way to wrap them so you'd never get a miscompile
 * Lawrence: But what about adding the `ddc` field. There's really just one extra field that only needs to exist sometimes.
 * Leon: On the `ddc` thing in particular, I think your advantage doesn't shine, because we rely on the final layout of this type in the assembly. So we wouldn't lose much in that use case with regular #defines.
 * Lawrence: There is one piece of code in Rust that accesses it too, which was dangerous
 * Amit: I suspect what's going on is the following. This changeset is extracted from more proposed changes to come which have more uses of the TIfConfig. This went through a round here where using not this and using #defines was seen to be terrible. And maybe an issue is that could be incorrect, but also that we here don't get to see that whole process or other places where its useful. So we have to be careful not to remove this, then want it back later. This could be the least-evil thing. Maybe that process is worth it so we can see
 * Lawrence: For the `ddc` field, there was a version with the if-config. We could compare those two implementations for `ddc`. Compared to capabilityptr where there maybe is a good alternative like Leon said. This will be more useful when subbing out fields here and there
 * Amit: Okay so what we should do for now is look at this PR without the ifconfigs, and then wait for them to feel necessary and propose them then.
 * Leon: The two uses are the `ddc` and the `capabilityptr` right now?
 * Lawrence: I think so? Oh, and also the return type from the MMU configuration which is a whole crazy thing. Chooses which variants of an enum is possible in different builds. Lets the compiler remove some match arms in some platforms. The programmer has to write all of them. Uses the `never` type for that
 * Leon: Okay, link to the `ddc` with regular configs if you can. Then we can consider and I agree with what Amit proposed
 * Brad: There is more to address in this PR but this is a start. There are MPU changes which are significant on their own

