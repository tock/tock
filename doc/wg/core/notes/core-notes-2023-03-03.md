# Tock Core Notes 2022-03-03

Attendees:
- Hudson
- Alexandru
- Branden
- Amit
- Johnathan
- Alyssa
- Leon

## Updates
  
Leon: PR to split capsules is in. 

Leon: I'm continuing to work on optimizing RISC-V context switches.

Amit: Could you explain them optimizations?

Leon: We used to stack all kernel registers, so that the assembly block could do anything.
The current change marks all registers as clobbered, rather than stacking/unstacking. So
now LLVM is automatically stack everything we need, and nothing more, rather than manually
stacking everything.

Alyssa: Is there an artifact?

Leon: #3407

Leon: I'm also wondering about this for userspace. But one thing is we don't want to leak
anything to userspace. 

Branden: Which direction is this saving or not saving, right now?

Leon: Only relating to kernel registers from the kernel context. 

Branden: Doesn't this mean we could leak information?

Leon: No, we restore the entire application register file. We might potentially have
some application contents until they are overwritten by the kernel.

Branden: And we trust the kernel.

Hudson: I have rebased deferred call PR and addressed all of the comments. Thank
you for the soundness review, Alyssa. There's still the TRD 1 issue, needing to update
TRDs that have deferred calls in them, but I don't want to block on that. #3382

Hudson: Let's get to the agenda. Tock registers. Johnathan?

Johnathan: I'd like to get eyes on this. I have some ideas to change Tock registers
to fix the unsoundness issues and support testing. It's a complex, interconnected design,
which I had trouble describing. I really want help writing the design document, I need
help explaining the design. 

Phil: I said I would help.

Branden: I had a tough time looking at all of the traits.

Hudson: I've started to look through it some, but I don't quite understand it all yet.

Alyssa: Please send me the PR?

Johnathan: Sounds like lots of people will take a look, so we can table this to
next week.

Hudson: What's your porting plan?

Johnathan: I was assuming we could keep the existing macro in place, and start
changing things over. It's a huge change, will require a lot of testing.

Phil: Can we do this for the next release? Make it a milestone?

Branden: Not everything is switched over from even the last one.

Phil: Yeah, let's clean this up and try to pay off this technical debt.

Hudson: Alex would like to talk about Rust applications.

Alex: We have a CAN driver, which receives synchronous from the bus.
We need to signal this to userspace. In C, you get one notification,
it starts filling in data, you get another notification, and can start
processing the messages. You can do this like ADC, but I don't see how
to do this in Rust. Maybe I have a buffer in the capsule, I copy it 
into a buffer swapped with the application?

Johnathan: I believe you can modify allow read-write to modify the reference
and it would be sound. When you have nested scopes, it'll be unallowed when 
the inner scope begins. You can't move the reference into the outer scope,
this is the concern, that this is the dangling reference issue.

Alex: So I could have a shorter lifetime than the original.

Johnathan: If outer scope allows the buffer, and the inner scope swaps,
this should be safe.

Alex: But when inner scope finishes, it unallows my buffer.

Alyssa: Wouldn't this shrink the lifetime?

Alex: If I exit the scope, it will be unallowed. I was wondering if someone
who is using libtock-rs has bumped into this?

Johnathan: Nope.

Alyssa: I don't fully understand the problem, can you message me on Slack?

Johnathan: I'll try to open a group chat between the three of us.

Alex: That would be great, I would appreciate it a lot.

Hudson: That was everything on the agenda for today. Anything else?

Alyssa: We want into some issues with the UART trait and wanting to
do some unsafe magic. I can gather the details, but to get around it,
we would have to change the interface to the trait. Essentially replacing
the mut ref with some wrapper around it. It's specifically because of
protection guarantees on references. If you pass a mutable reference, it
has to be valid for the function, even if you never use it after you 
invalidate it?

Leon: Don't we use static mutable?

Alyssa: Makes no difference. It has nothing to do with lifetimes. I want to try to 
identify the larger problem. It's one of those things, where if we were using
async this wouldn't be a problem. But we needed to create an unsafe interface
and that unsafe interface is unsound. I'll talk to Jett and see if I can get
more info.

Hudson: An issue would be great.





