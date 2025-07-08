# Tock Core Notes 2020-12-18

## Attending
 * Branden Ghena
 * Amit Levy
 * Johnathan Van Why
 * Alistair
 * Leon Schuermann
 * Brad Campbell
 * Arjun
 * Philip Levis
 * Hudson Ayers
 * Pat Pannuto
 * Vadim Sukhomlinov

## Updates
### Libtock runtime assembly
 * Johnathan: Working on libtock runtime, the new runtime for libtock-rs. Trying to get it working with the stable asm! feature. Requires standalone assembly for entry point. Might end up having to rely on another unstable nightly feature, but we'll see. You need either naked functions or global ASM, or external assembly to make this work. But toolchain is a pain for external assembly.
 * Phil: I'm a fan of external assembly
 * Amit: Does LLVM not have an assembler?
 * Johnathan: I'm not sure, more importantly I'm not sure if it comes with the Rust toolchain.
 * Johnathan: We might also lose out on inline assembly optimizations with external assembly. It seems like inline is on the track towards stabilization.
 * Amit: This is the low-level runtime, right? (yes)
 * Hudson: Rust core team has stated they hope to have inline assembly stable for 2021 rust release.
 * Brad: That's part of a larger embedded push. Does anyone know what the thinking is for resolving that last problem on the path to stabilization? If just stabilizing the new asm! macro wasn't enough, there should be some roadmap for what's missing, right?
 * Johnathan: For our use case, naked functions are the best option. 
 * Amit: It's possible that it's not an issue for other embedded projects, much like it isn't for the Tock kernel, because the entry point follows C calling conventions. Guessing here. So it could be a userspace problem.
 * Johnathan: Arguably we could make entry follow the C calling conventions, which would solve this. We are just missing a valid stack pointer, I think.
 * Amit: Doesn't changing the stack pointer in inline assembly mess with things though?
 * Johnathan: I think it would be okay. As long as the assembly block never returns.
 * Amit: So we'd have a trampoline function as the entry point. But we couldn't do what we're doing now, initializing sections of the process in Rust, it would have to be in assembly.
 * Johnathan: Yeah, it's probably undefined behavior to have it done in Rust. I think if we wanted to move to stable without external assembly, we could push on "nakedfunc", I forget the exact feature name, because there are questions about arguments, but there should be a simple to stabilize version without arguments.
 * Vadim: I did that exercise for RISC-V and moved several functions to external assembly, but it's got dependencies on toolchain. And I struggled with functions that are actually rust functions, but with some low level assembly in them. The ABI is not stabilized for Rust, which is challenging.
 * Johnathan: We could also do what the rust embedded people do, which is include pre-compiled binary in the repo for people who don't have the toolchain.
 * Amit: We could do both source and pre-compiled binary for people who don't have the toolchain.

## Tock 2.0
 * Phil: We're making progress! Thanks for everyone putting in PRs already updating syscall drivers. There are lots of examples now thanks to Leon and Hudson by working on supporting infrastructure.
 * Leon: Regarding protection against swapping appslices. I've made some progress, but it's pretty complicated. There's a PR that people should look at. I think callbacks can get away with no overhead while enforcing no swapping. I'll look at appslices over holidays.
 * Phil: One thing I'll say is that thank goodness we figured out how to trace syscalls. It's so nice to be able to look at the syscalls and arguments for debugging.
 * Vadim: I was looking at an LLVM feature for tracing in the kernel. On every edge there can be a call to a function that traces internal functions, which helps retrace on a panic.
 * Amit: Do you have an example of doing so, we'd super appreciate it. (yes)
 * Amit: I started yesterday on driver porting myself. I'd say it's been surprisingly straightforward. The complexity is just re-familiarizing with the particular driver and making sure that your changes aren't violating anything. For the most part it's a very simple interface.
 * Phil: One caveat. Except when something has very complicated usage of buffers. Handling that right, in console for example, can get pretty subtle about how to make them correct. I tried to put that in the porting document as well.

## Uncrustify in libtock-c
 * Phil: How do I uncrustify before doing a push? If I have a file with trailing whitespace, then travis fails on uncrustify.
 * Amit: You go into examples and run `format-all`. It's a shell script.

## Callback arguments and return codes
* https://github.com/tock/tock/issues/2235#issuecomment-745789840
 * Hudson: Since we're moving away from returncode at the system call boundary, with the plan to remove it for result and errorcode, that means at this point we're no longer passing returncode across system calls except for callbacks. I think it's still the most straightforward way to send a result in a single usize across the syscall boundary that can be either success or failure. Even revised 2.0 capsules still use returncode for callbacks. For libtock-c this makes it challenging to return from a library function presenting a synchronous interface. Typically those call several system calls, returning the error if there was one, or if not block on callback and return result from callback. Which is great when all of those are returncodes, but now systemcalls use returncodes, and even though there is a wrapper, it's less idiomatic. Maybe we do still want to use returncode everywhere in libtock-c, but I wanted to see people's thoughts. It's strange to have returncode only used in one place.
 * Leon: I agree it's weird and am for removing them. Definitely don't want two duplicate enums that _mostly_ overlap. We could pass a command result in there. I think that's a good option if we want to distinguish between different callback types which we can register, which have different numbers and types of arguments. Like command does currently. If we do this, we shouldn't just return commandresult, but should have another wrapper that is specific for callback to make this more expressive.
 * Phil: I think we're bumping up against C and Rust having different expressiveness. We should still use returncode in libtock-c since it's more idiomatic C, I think. Lots of callbacks don't necessarily return failure or success, which makes this tricky. We also have userdata for callbacks which might be complicated.
 * Leon: This could be one of the reasons we resort to using a specific wrapper-type around the generic syscall return value for the callbacks. In which we, for instance, define the ABI differently and in these values we return app data.
 * Branden: I didn't follow.
 * Leon: We currently return the command result value and it wraps the common value in the kernel, which is describing different collections of usize values. What Hudson suggests in the PR is that we could just pass the commandresult type in rust into the schedule method on callback. This won't work because we also have appdata that we need to pass into userspace, which would make this five values to pass to the function requiring more overhead, like using the stack. So instead we could have a different wrapper around this value in the kernel and have an ABI that would make a callback only scheduled on three user defined values plus the appdata value.
 * Hudson: The problem is that there isn't a standard way to represent a usize success/failure in the kernel without returncode. The thing that makes the C implementation easiest probably makes the kernel and libtock-rs implementations less clean.
 * Leon: And the reason we want to remove returncode is that it's not idiomatic in Rust. Which will only really effect the kernel, because the ABI won't be rust idiomatic. Plus it's 31 bits. There's no native type for 31-bits in rust (or c) so we implicitly cut off a bit to store if it's a success or failure.
 * Amit: I think it's not necessary or useful to have the ABI not match rust. In the case of callbacks, it's fine for the ABI to be agnostic of the languages in userspace. Maybe an answer is that returncode doesn't make sense in the kernel but is a fine utility in the kernel for system call drivers to use as a convention for signalling success.
 * Phil: Imagine I write a syscall driver and the first thing it returns in the callback is a resultcode that is specific to that driver. We wouldn't object to that. It's okay to have complex return codes for a weird interface. The idea that you might be passing values that represent errors or results seems normal for callbacks and is up to the syscall driver and libtock-c. Touching what Amit said, we should provide a standard one and let people alternatively create their own. ENOACK doesn't make sense for everything, which might want something like EPOWERLOST instead.
 * Leon: One potential solution is a supertype on the errorcode enum in the kernel, which could also indicate success, but not success with value. Which would avoid the 31/32 bit problem. So a superset would also include the success variant.
 * Hudson: The stated goal is to remove successWithValue before 2.0 anyways.
 * Leon: But we should make sure that the discriminators between errorcode and returncode are the same. And this would be a good chance to move returncode away from negative signed integer return codes to just usize and positive numbers, which is easier across syscall boundaries.
 * Hudson: One problem is that returncode and errorcode use different underlying integer values.
 * Leon: This was introduced exactly for that reason. It's much easier to think about. And the syscall trace with raw values is much easier to think about as unsigned values rather than two's complement. And since we didn't need to differentiate between success and error cases anymore, unsigned values seemed better.
 * Hudson: It just adds this necessity for a conversion between errorcode and returncode where it could otherwise be free.
 * Leon: So we could change returncode once successwithvalue is gone.
 * Hudson: Yes. That could be nice. Because if I'm going to schedule a callback and I want the first value to be an error, it shouldn't matter whether I'm doing returncode.into() or errorcode.into(). If we don't have those be the same, you'll have cases where you have sometimes one and sometimes the other. And the userspace won't notice and it'll be a pretty big pain point for the users.
 * Leon: We can enforce that every value is a returncode in the callback.
 * Hudson: We just suggested we don't want to confine them to a single type.
 * Amit: In the idea world, would we be happy with the idea of not having returncode at all and there being a standard converstion of errorcode to a usize and everyone just used that?
 * Phil: No. Because then we need an extra field for denoting success or failure. This was okay in the syscall ABI since we're giving more data, but callbacks are a little weirder. One solution is that successwithvalue goes away, success is zero. Then errorcode is a subset of returncode values, and you should be able to handle them okay. So callbacks send returncodes, but if someone hands an errorcode it will still work transparently.
 * Leon: Just for communicating the value for userspace, we could reserve a special value never used in the kernel otherwise that represents success. 
 * Amit: So we only have errorcode in the kernel. As a convention, userspace drivers and kernel drivers have success or failure, and kind of failure, that is encoded in one of the arguments of a callback has the space of values that errorcode has, plus an additional value that denotes success, probably zero. So it could be relatively convenient to write that in the kernel in a driver capsules. And in the userspace, zero is just success and 1-13 are types of errors.
 * Hudson: I think I prefer that to what we have now where the errorcode mapping is different from what it looks like in the kernel.
 * Leon: And there's an advantage because if returncode exists, people will use it by accident.
 * Phil: It does still need to exist in the C userpace. (agreement)
 * Hudson: Only for C though. For rust userspace, we'd represent it just like the kernel does again.
 * Amit: The ABI is just some numbers, and we'd re-encode however necessary. So it's just a little bit of backbreaking work to do the transition.
 * Hudson: I think it's actually easy as long as you use the same numbers currently used by returncode.
 * Leon: We're going to use the same numbers as currently used by errorcode, and can use the regular into function to translate.
 * Amit: That would require changing userspace. Which isn't not an option, but is more work. Probably for the payoff of debugging and tracing be more friendly.

## Grant soundness issues
 * https://github.com/tock/tock/pull/2137
 * https://github.com/tock/tock/issues/2135
 * Amit: We went back and forth about whether this change was good, and it's been lingering for a while now. My changes to proximity exhibit exactly the kind of behavior Hudson was worried about.
 * Hudson: Summary: today it's possible to enter a grant and inside the closure enter that grant again, which is unsound. Unfortunately, there's a decent amount of code in the tock kernel that relies on this. Helper functions, like in the proximity driver, search for things among the grants. If called within a grant, today they work as expected. With Amit's fix, they would NOT work as expected, because they would silent skip the already entered grant. This is a problem because it's really easy to make the mistake of writing functions that work in some places but not in others. I had proposed that we should panic. The problem is that sometimes you _do_ want to iterate all the other grants you didn't enter, which we could make a iter_others function for. Basically, today this "works" unsoundly but fixes introduce panics. So this has sort of lingered here, but it would be nice to reach a conclusion.
 * Leon: From the old discussion, we essentially decided that any changes to the type system, for instance returning an iterator or options, would be way too complicated for various reasons, and a change that would break too much in the kernel.
 * Amit: It would be a bummer to write code for that.
 * Hudson: And they way everyone would write code for that would just panic if it wasn't what they expected anyways.
 * Leon: So essentially any changes to the type system are essentially too complex to do and wouldn't led to developer-expected behavior.
 * Amit: Having now touched and thought through one of these drivers, I think that there is probably a resolution here. There are two solutions proposed.
 * Amit: One is what I did which was simple to implement and made sure that we never re-entered a grant. So all the places, including iter, lazily don't enter a grant if it has already been entered. That has problems like Hudson mentioned that you test in a certain way and it seems to work, but sometimes in production there's an edge case based on the user apps that fails.
 * Amit: Two is what Hudson proposed to effectively panic instead of silently doing nothing. The benefit is that would hopefully be caught earlier in testing. The downside is that there is a hidden panic that's abstracted away from the developers and they won't expect it or be on the lookout for it.
 * Amit: So we first have to avoid soundness issues. The second most important thing is to avoid a buggy capsule breaking the whole system. Avoiding bugs in a particular capsule is third.
 * Amit: So a thing we could still do dynamically, is instead of panic-ing, instead of iterating at all, the iterator is empty. So the bug would presumably be demonstrated pretty easily, but capsules wouldn't panic during production.
 * Hudson: So you could still have iter_others for cases when you do want to iterate grants we aren't in.
 * Leon: I think returning an empty iter should take this subtle bug and make it a bit more visible. But it still isn't satisfying to me. It would be nice to solve this at the type level.
 * Amit: Yeah. We're just hoping it surfaces the bug earlier.
 * Leon: If a capsule wanted to handle this gracefully, how could it actually check whether the grant enter worked?
 * Hudson: This isn't something capsules can handle gracefully. When this is possible, the capsule is actually wrong. So there shouldn't be error handling code to catch it.
 * Amit: There should be a way to structure capsule code so it never does this.
 * Hudson: Yes. Unfortunately, the easiest way often does this.
 * Amit: Alternatively, we could _only_ have iter_other. So it's clear from the name that it will _never_ iterate the grant that's already entered.
 * Brad: But isn't the call going to be like self.apps.iter_unentered? It's not clear what that would mean.
 * Amit: Suppose we came up with a better name.
 * Brad: You fixed the proximity code. Was it hard?
 * Amit: I haven't fixed it yet. That's a thing I can check.
 * Brad: So then the question is how hard is it to special case the grant you're currently in and look at the others.
 * Amit: With the fix that I proposed, skipping the entered grants, the proximity driver would be easy to fix, but in a brittle way.
 * Phil: The one that I think will be tough is the alarms. You're often scanning and looking for the minimum one.
 * Amit: With hudson's proposed change, it would be tricker to fix proximity, but the resulting code would probably be clearer. (No offense, meant, the code is good.) I don't know how complex alarm would be though. Basically, you would end up keeping track of more things outside of the grant in global state. Which is now handled by lazily iterating through the grants. That state could be kept globally instead. This is a reasonable pattern, but it's not clear that it's easy to do correctly.
 * Brad: That scares me. It's always hard to keep duplicate state right.
 * Hudson: what if instead of capsules holding a grant, they held a takecell over a grant? You'd then have static checks, requiring a mutable reference to the grant region to iterate through. The flip side is that you have to put grants back whenever you take them out. Which isn't great, but the upside of the static checks.
 * Leon: There should be ways to use refcell, with map.
 * Amit: Takecell also has map. They're the same these days, except takecell doesn't panic. That's how I'd use it. I'd map over the takecell and in the closure apps.enter().
 * Leon: How does this look from a memory perspective? Would this be an extra word?
 * Amit: I believe it would be an extra word. It's possible that would go away because technically the grant is a nonzero type. But let's just say it would be an extra word in the worst case.
 * Hudson: For the proximity example, let's say you're calling map in both places. Would this be caught at compile time?
 * Amit: I think it wouldn't be caught at compile time. This is the right idea, it's just not completely there.
 * Hudson: So just renaming iter would be better than now. All these options are suboptimal but the status quo is most suboptimal.
 * Amit: So lets see how hard it is to write currently offending capsules in way that doesn't incur this problem.
 * Hudson: I think it's not that hard.
 * Amit: If it's not hard then maybe panic plus an additional iter_other function is good enough.
 * Phil: That's what alarm would do (iter_other). I'm porting alarm to 2.0, so I'll reread all of this. You have a caller and are resetting the caller's alarm and also need to iterate over everyone else's alarm. Or maybe you could set it and then iterate over all of them later. My guess is that code may have to be heavily rewritten.
 * Amit: Because alarm is time sensitive, it may matter more. I think it's representative of the hardest case.
 * Phil: We might have to totally rewrite the code again. Which isn't _that_ bad. And the API is better now.

## Next meeting
 * Amit: Next two Fridays are Christmas and New Year's day, so let's reconvene on Friday January 8th. (agreement)

