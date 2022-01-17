# Tock Core Notes 01/14/2022

## Attendees
 * Branden Ghena
 * Hudson Ayers
 * Phil Levis
 * Jett Rink
 * Pat Pannuto
 * Johnathan Van Why
 * Vadim Sukhomlinov
 * Leon Schuermann
 * Alexandru Radovici


## Updates
 * Phil: From Alistair, he's been working for a while on getting cryptographic traits into kernel HILs. One that's really important is #2839 for RSA. https://github.com/tock/tock/pull/2839 Sometimes RSA keys are big and in Flash, but sometimes they're in RAM. Because passing buffers in asynchronous buffers need to be mut so you don't cast away mutability, this means RSA has two pairs of traits, one for mutable and one for immutable. We tried this a few ways and decided that mut/immut buffers shouldn't be in HILs. Passing down a mutable buffer and getting immutable back is a bad failure case.
 * Phil: So, we should have people take a look at https://github.com/tock/tock/pull/2839 as it's important.
 * Hudson: Small update, working with Alyssa on runtime/linker scripts for libtock-rs. Internally we're trying to port over to libtock-rs 2.0 APIs. Stuff is mostly working with help from Johnathan. Hopefully we'll be able to start porting drivers over soon.


## Crypto and Mutability
 * Phil: Same challenge from RSA keys. The idea of some crypto data coming from Flash is going to be recurring in crypto traits.
 * Phil: Issue I'm running into is verified process loading. Checking the integrity of an application image. The challenge is that there are lots of ways to do this, but a cryptographic hash over the text and metadata of binary is often the case. But the API for digest, the HIL for this, requires a mutable buffer. But code in Flash is immutable. https://github.com/tock/tock/blob/master/kernel/src/hil/digest.rs So this means we need parallel APIs, one for mutable and one for immutable.
 * Phil: Things that use lots of data particularly have this problem. AES is only small blocks and you can copy into RAM. But for things like hashes and public keys, it matters.
 * Phil: Right now, the best idea we could come up with is having these parallel traits. Other options seem to require using unsafe and panic if things go wrong.
 * Jett: Potentially ignorant question. Why can't we just expose the immutable interface? Is there a time we need the references to be mutable?
 * Phil: So if I pass a mutable buffer into this interface. If I want to pass it in immutably, I can't have a mutable reference too.
 * Jett: So this is within the kernel. Nothing to do with apps.
 * Phil: Right. So eventually I get the buffer back and it's immutable, but I can't get the mutable version back.
 * Jett: Yeah, it seems like an abstraction for what type it _used_ to be seems useful.
 * Phil: This is exactly what the mut_immut_buffer was for. So I can act on something like an immutable buffer, but I can pull the mutable out at the end if it really is that. So this could be the solution. OpenTitan group we talked about this and tried using this. The challenge is that moment when you've gotten the mut_immut_buffer back as the client, and want it to be the mutable buffer again, if there was a bug in the lower level implementation where it gave you the wrong thing back, what do you do?
 * Jett: We've had issues where you pass in a buffer, some slicing happens, and you only get part of the buffer back. It's a similar issue if there are bugs.
 * Phil: Yes. There's a bug in some low-level implementation that your client is being forced to handle. Slicing is a good analogy. You can split slices but you can't put them back together. So instead we said we'd have parallel traits. And the implementation actually uses mut_immut_buffer, but it can detect the error right where it happens, rather than kicking the can to the client.
 * Jett: I would think that the capsule that gave the buffer that gave the buffer, it tries to cast right when it gets it and notices. Whereas if the underlying thing handles it, it might just never give a callback if it's the wrong type. Potentially.
 * Phil: So the question is whether the capsule can respond to the error. Our model is that this is a serious memory leak problem.
 * Jett: Right, like slicing.
 * Hudson: I remember reading a blog post about a network stack handling this. I think this is it: https://lab.whitequark.org/notes/2016-12-13/abstracting-over-mutability-in-rust/ The technique they use is that the trait takes a generic type, either mutable or immutable buffers. So if we're willing to assume that for a given implementation we'll either pass mutable or immutable, but never both, this is an approach we could take maybe.
 * Phil: For clients, yes. But the underlying implementation needs to do both.
 * Hudson: Could the lower-level implementation be generic too, so it doesn't care and always returns what you gave it?
 * Phil: You'll have code replication for the two types though, right?
 * Hudson: If you use both at compile time. But if an implementation only hashes either RAM or Flash, that should be fine. And you don't have to duplicate source code.
 * Jett: If you need one instantiation that needs to manage hardware, there's a problem there.
 * Hudson: Oh, I see what you mean. You can't monomorphize the lowest level object.
 * Branden: Maybe I didn't understand, but why can't we use mut/immut? Just because there could be a bug?
 * Phil: If you pass mut/immuts around, it's pretty easy for the underlying implementation to mix up the buffers because they all look the same. In contrast, with the parallel tracks, you quickly realize when there's an issue. So it's easier for implementers to write bugs, which is why we avoided it. And we found the parallel tracks didn't greatly increase code size, so they were a better solution.
 * Branden: That makes sense.
 * Jett: We could treat slicing and mutability as one problem that we have one solution for. We could have a "passable" buffer we send in, and make it so that it can't be changed and guarantees to be the same thing you gave.
 * Phil: But couldn't it send you a different "passable buffer" back? Leasable buffer gets at this.
 * Jett: It could mix up its clients, yeah.
 * Phil: With process loading, it's synchronous right now and it has slices and splits them, but when you pop back on the stack you get the original thing back. You slice RAM and Flash for processes. When this becomes asynchronous, this gets trickier. Right now you back out after allocation, but for asynch you can't split the slices until you're SURE you want them, because you can't undo it.
 * Phil: Let me explain again. With process loading, you have a big chunk of RAM. Loading processes carves pieces off of that slice. So it might give 8 KB to a process and hang on to 32 kB for the rest, if it starts with 40. If it fails, you can just pop out of the function, and you're back to code that still holds the 40 KB. If it's asynchronous and split into 8 and 32, there's no way to put them back together, because there's no way to return to the point where we had 40 kB.
 * Branden: And there's no way in Rust to recombine?
 * Phil: There are crates, but with lots of unsafe. So we wanted to avoid that.
 * Phil: And for processes, we tear off chunks of _mutable_ RAM. So we can't undo the slice.
 * Phil: So, I wanted to share to get people thinking about the idea in case anyone has an interesting solution.
 * Phil: For now, I'm going to split Digest into parallel tracks for immutable and mutable.
 * Jett: And you said there's not that much code size overhead?
 * Phil: No, it's a pretty thin wrapper. And the code to decide is something the client would need anyways.
 * Jett: That's a good trade then to make the check happen at compile time.


## Kernel Releases
 * Jett: What's the release schedule? When would we release 2.1?
 * Hudson: In the pre-2.0 days, it was whenever someone got motivated for a new release. We didn't have a particularly regular schedule.
 * Pat: We tried time-based releases, but it didn't work so well given that Tock moves in bursts. So instead whenever, we decide to do a release we do. So you could push for one, if you want one.
 * Jett: We're not in a rush. But also the longer we wait the more validation work it is.
 * Pat: Here's the official release policy: https://github.com/tock/tock/blob/master/doc/Maintenance.md#preparing-a-release


## Libtock-RS Releases

 * Alexandru: For libtock-rs, that will never be compatible with 2.0, right? It'll need 2.1 because allows don't work with 2.0.
 * Johnathan: It should work fine for 2.0
 * Alexandru: It shouldn't work because we moved allows from capsules into the kernel.
 * Johnathan: Oh, I see what you mean. Yeah, it won't be compatible. So we could probably consider it released as of 2.1
 * Hudson: That's as good a motivation as any.
 * Johnathan: I do want to remove the 1.0 stuff at some point. Although I'm not sure when that will happen. Need agreement with others, including Alistair.
 * Hudson: Right, at this point there's no development that's relevant to the 1.0 crates.


