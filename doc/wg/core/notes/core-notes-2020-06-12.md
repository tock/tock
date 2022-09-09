# Tock Core Notes 06/12/2020

Attending
 - Brad Campbell
 - Alistair
 - Leon Schuermann
 - Pat Pannuto
 - Samuel Jero
 - Johnathan Van Why
 - Vadim Sukhomlinov
 - Branden Ghena
 - Hudson Ayers
 - Garret Kelly
 - Amit Levy

## Updates
 * Branden: Described Emilio's issue on slack with an out of tree board where
   updating the rust nightly increased the binary size of his board by 1000x.
Eventually tracked down the issue being his board having RAM before flash
instead of after, and a change in llvm-objcopy shipped with Rust caused the
issue. Ultimately using some different flags seems to have fixed it.
 * Brad: We have run into issues like this before, that fix is consistent with
    my past experiences. I think using the new flag sounds like a desirable
solution.
 * Brad: I have been working on tockloader, adding support for fixed address
   apps, by looking at a TAB file, see that there are multiple binaries for a
given architecture, and try to find a combination of multiple apps that will
work on boards where PIC is not supported. This is not pushed yet, it is in a
separate branch, but it seems to be working well. This would get us close to
having RISC-V support look a lot more like Cortex-M support.
 * Amit: And there are fixed offsets that we would compile each app to?
 * Brad: Yes: This goes with a corresponding patch to the kernel process code

## Appslice Bug
  * Leon: We do not do any checks when we have ownership of an app slice to
    check whether the memory this slice points to is still valid or belongs to
this instance of that application. This means that when you restart an app a
capsule holding an appslice gets access to process memory without the instance
of the app having shared that appslice with that particular capsule.
  * Amit: I think we can fix this without breaking any existing code that I am
    aware of, and we definitely need to fix this. Maybe this should be lumped
into the bigger discussion of how we should handle these kinds of memory
buffers.
  * Leon: I think that any of the solutions discussed in the
    unallow/unsubscribe loophole issue would fix this as well. I am not sure
how we will make sure this does not break any other code
  * Amit: The hack would be to just return an empty slice
  * Hudson: This seems like it could lead to a panic in capsules
  * Amit: Correctly written code should be written such that an app only does
    access appslices when the app is valid (such as storing them inside a
grant). Returning an empty slice is similar to returning a unit type. Similar
to how the enter function for grants just does not execute a passed function
when the app is not active
  * Leon: This could wreak havoc in capsules that do not take into account that
    the length of a slice could potentially change to 0 at any time.
  * Amit: Yes, so it is kinda gross.
  * Brad: Overall that seems reasonable, and anything that break we really
    really wanna know about. We should not have any capsules that should enter
this situation, and we have to worry about people using those as capsules when
writing a new capsule.
  * Sam: Yeah that is a bug that should be fixed ASAP if it exists.
  * Amit: I could like panic when this case hits and run all the tests we
    always run but that is probably not likely to find any existing cases
  * Brad: I think we should be able to look through all the capsules, if they
    use a grant we are good, if they are not are they doing anything else to
check that the app is still there. I should have fixed most of these when I
made changes to support restarting
  * Amit: AppSlice is basically a wrapper around AppPtr in the same module, for
    which I unsafely implemented deref in an incorrect way, but luckily nothing
uses AppSlice instead of AppPtr, so we can just change the visibility of AppPtr
to pub(crate) and get rid of deref implementation.

## Unallow/Unsibscribe discussion
  * Hudson: I think we got sidetracked last time talking about whether the
    "unsubscribe/unallow loophole" is technically "unsound" (in the Rust sense
of leading to UB) or just undesirable. My opinion is that either way these
changes are desirable, having the kernel manage subscriptions and allowed
buffers makes a lot of sense, especially from the perspective of a rust user
space that in order to be soundly implemented you need to be able to trust that
capsules are not simultanesouly modifying something that userspace has a
reference to. It seems like if we can arrive at a consensus that these changes
would be positive, we should state that on the issue, because it seems like
Guillaume would probably be interested in maybe submitting a PR along these
lines, and I think we should give him the go-ahead if we can agree that it
seems like a good idea.
  * Amit: What are folks reactions to that?
  * Johnathan: I agree
  * Leon: I think if we can technically limit the amount of damage a capsule
    can do, sound or not, we should probably do it.
  * Amit: I think that having been a voice against this last time, I am also of
    the mind that it probably makes sense, especially given that we probably
mostly agree that while we want to support apps in many languages including of
course C, ideally canonical apps would be written in Rust, and that would give
the most robust overall platform. And this makes sense even if there is some
modest overhead for this.
  * Johnathan: One thing I want to point out is that it is not unsound to have
    allows last forever, and that can be wrapped in a library in userspace. If
an unallow is guaranteed to succeed we can have a Rust userspace API that can
return a slice, if it is not guaranteed to succeed we have a really nasty error
handling case, and it is hard to handle that without panicing. That pushes
towards static buffers that last forever anyway.
  * Amit: What if the kernel upcalled into apps to pass buffers back, instead
    of a userspace initiated allow?
 * Johnathan: I think this could work, but might have a decent amount of
   overhead in the Kernel (having to track various callbacks), or would require
additional memory from the grant regions of apps amd in apps themselves.
 * (some discussion of how this could be optimized some, potentially by using
   userspace malloc when avialable) 
 * Amit: I think this is valuable if it makes the rust userland significantly
   better, otherwise its not
 * Johnathan: if unallow always succeeds, that would be ideal
 * Hudson: Just to clarify -- for the current design, can libtock-rs ever
   soundly modify a buffer after calling allow()
 * Johnathan: yes, if your reads and writes are volatile
 * Amit: I think the trick here is that a slice of bytes is kinda morally
   equivalent to a slice of cells of bytes which would in fact be safe to alias
mutably (if you are using volatile)
 * Johnathan: Not quite that: you cant have a reference to shared memory, but
   you can have a pointer, and use volatile when reading from / writing to that
pointer.
 * Johnathan: Userspace just cannot have a rust reference type -- there can't
   be any rust references pointing into that memory
 * Amit: that is what cell would enforce?
 * Johnathan: No thats not right cell doesnt do that
 * Amit: Refcell?
 * Johnathan: reference to cell is a reference to memory as far as rust is
   concerned
 * Johnathan: Cell cannot be safely mutated by a thread that rust is unaware of
   (such as the kernel)
 * Amit: The arguments that you are making seem to imply that what we are
   currently doing is not unsafe
 * Johnathan: What we are currently doing is unsafe, this is also true for the
   tock-registers crate
 * Amit: Can you explain this a bit more
 * Johnathan: in tock registers you have volatile cell. In rust anything that
   invokes UB in LLVM is UB in Rust. LLVM is allowed to insert arbitrary reads
and writes under aliasing assumptions that memory is not going to change unless
the compiler changes it. THis is true even for unsafe cell. As such having a
reference to an MMIO register still invokes UB the minute that register
changes. This means that the compiler can insert arbitrary references to MMIO
registers.
 * Amit: We are writing to a specific substantiation of LLVM bc we have to, I
   guess my question is are the consequences of this "potentially UB" limited
for the types that we are using? This should not break type encapsulation
otherwise
 * Johnathan: It is UB, so technically that is not true, in practice I have not
   observed those issues
 * Amit: popping back up..
 * Johnathan: when userspace allows memory to the kernel, it can hold pointers
   to that memory, but not references, because if it had references, it would
allow userspace to have UB in the userspace when the kernel mutates the memory.
 * Amit: Aha..so is this specific to Rust in that volatile in C does not result
   in the same behavior.
 * Johnathan: Unclear to me, that is a hard question. Arguably you could end up
   with a data race.
 * Amit: I am claiming that a data race is not a problem 
 * Johnathan: data race is UB in C/C++/Rust
 * Amit: Is it not a problem for the kernel bc the kernel is able to lock the
   appslice while it is operating on it
 * Johnathan: Yes
 * Amit: So if we want to handle these things safely from userspace, and the
   point of sharing a slice is to allow two-way communication, we need to
either trust the capsule to give us the buffer back or we need to have a
mechanism in the kernel to ensure that?
 * Johnathan: If you want userspace to be able to reuse the memory arbitrarily,
   yes. But the kernel model is workable, userspace can work with it
 * Amit: Right, so if you had to send multiple packets, you would need multiple
   userspace buffers for it?
 * Johnathan: Yeah, it causes lots of buffer allocation issues
 * Amit: I think what you are pointing out which I had not considered before.
   This is not an issue only from the perspective of breaking functionality,
but even if you only ever reuse that buffer with the same capsule, userspace
has to trust the capsule in order to trust *its own* type safety!
 * Johnathan: My current plan is to change the libtock-rs API to make that not
   a problem
 * Sam: ...
 * Amit: Basically right now you have to trust a capsule with more than just
   the memory you have allowed to them, or there can be arbitrary UB in
userspace
 * Amit: This discussion was convincing to me that we should have a way to
   enforce this in the kernel
 * Amit: If we could enforce this in the kernel, and unallow was guaranteed to
   succeed, would that fix these issues?
 * Johnathan: Yes, and we would get a beautiful API
 * Amit: What if it could fail, but was enforced if it did succeed? There might
   be lots of userspace overhead?
 * Johnathan: Right, then we would probably need two APIs
 * Amit: Thinking through this, my sense is that if we do not reach all the way
   down into the hardware (pass process buffers down to hardware) I think we
should be able to write this such that unallow always succeeds.
 * Sam: Does that cause bigger problems?
 * Leon: I think that if we wanted to give hardware rights to the userspace
   buffers with for instance DMA I think that it is still possible to make the
call always succeed and if it doesn't succeed then that is the fault of the
capsule and the capsule should block up until the DMA succeeds or is aborted.
 * Amit: Oh that is a good point, there is no way for the app to fail if it
   cant run
 * Amit: I am more convinced after this discussion that this makes sense to
   pursue. Any dissenting voices?
 * Brad: As someone who cannot even compile libtock-rs, I am more interested in
   the fact that this would make it harder to write poor capsules, because all
of these things would be enforced behind a grant so you can't store ad-hoc
process state in a capsule
 * Amit: Sorry, what can you not do then?
 * Brad: (explains again)
 * Hudson: Brad is not dissenting, just saying he likes it for a different
   reason
 * Amit: Oh okay good, so we have consensus, so we should politely and
   thankfully ask Guillaume to go ahead with a PR
 * Leon: I have another approach, Can I open a PR with this other approach so
   we can compare them both
 * Amit: Yeah that makes sense, leave a comment on the issue so everyone is
   aware there will be competing things to considere

## Tock 2.0 Approach
* Amit: Do people have thoughts on how we should go about this? Phil and I
  should be ramping back up soon
* Brad: I am in favor of the seperate branch, and just want us to not keep
  adding features and making this take even longer
* Amit: I think we should focus some time in each meeting on discussing the
  progress we are making toward 2.0
* Amit: How do we manage forward-porting changes and fixes and features to the
  master branch
* Brad: One other approach is put all syscall changes as new syscalls, then we
  can issue all PRs to master
* Brad: Then atomically flip at release time
* Hudson: I love it
* Amit: It could be hard if these changes require changes to capsules dependent
  on the new syscalls
* Brad: I am happy with this compromise
