# Tock Core Notes 2021-10-15

## Attendees

 - Amit Levy
 - Brad Campbell
 - Gabe Marcano
 - Hudson Ayers
 - Jett Rink
 - Johnathan Van Why
 - Leon Schuermann
 - Pat Pannuto
 - Philip Levis
 - Vadim Sukhomlinov
 - Alexandru Radovici

## Updates
  N/A

## Litex CI
### Overview
 - Pat: As I understand basically what's happening here is we have this Litex platform 
   which is running in Verilator which basically is giving you a *real piece of hardware that 
   happens to be running in software*. So it's simulating actual hardware as if effectively 
   it was a hardware platform and it's kind of interesting middle ground that exists. 
 - Pat: What my undergrads have been very slowly working on is getting a system set up where you can 
   do sort of federated hardware testing. It will run NRF52840 devices attached to Raspberry Pis 
   and the Raspberry Pis will be acting as endpoints that Github could talk to and run 
   all the hardware tests. You can run it on your own hardware and it will respond to the Github CI saying
   if it did work and if it integrates pretty cleanly.
 - Pat: That got hung up on things that we can talk about in that time, but the general 
   idea is that it allows for hardware platform owners to keep their hardware platform test 
   infrastructure in-house while allowing our cloud CI access it. 
 - Pat: The Litex is kind of an interesting situation because it is a new hardware platform 
   but in principle it could run anywhere because it's just software, so you could run 
   it in the cloud, you could run it in individual development machines, etc. 
 - Pat: The question is should we be treating it like a hardware platform that is something that 
   is run in a cloud instance owned by Leon, because he owns the Litex platform, or is it something 
   that we should be treating as just another piece of generic software CI that runs in an array of
   Github runners. 
 - Pat: I think that's sort of the high level that is currently being talked about here.

### Thoughts
  - Pat: My thoughts are that conceptually there's something elegant about treating a hardware platform 
    as a hardware platform whether it's a Verilated one or physical piece of hardware. 
  - Pat: I think that would be nice to validate that this federated hardware infrastructure works nicely. 
  - Pat: On the flip side of it, doing that actually ends up being a bit more work than it would otherwise, 
    because now you need a *physical instance* of the Litex hardware that in practice would be in a AWS endpoint. 
    Whereas if you didn't do that you could just take advantage of the existing Github runners mechanism that 
    spins up the physical instances of hardware to run on them.
  - Pat: Which side that line you fall on?
  - Leon: I think that is a very good summary. 
  - leon: It explains why even I'm technically in favor of doing it in software. But I mean, 
    I'm pretty biased as, it just would mean that I would have to maintain less code and instances 
    and infrastructure in general. 
  - Leon: One interesting detail maybe about the Litex platform that makes it very special is that it's 
    a hardware platform technically which evolves. So there's regular PRs which I do which update 
    the bitstream or pinned revisions of the hardware description language and it's not like, 
    as I understand, OpenTitan where they work towards sort of a defined revision of hardware which is 
    then set in stone. This I think it's just like a rolling release fashion always evolving. 
  - Leon: So we then have this interesting disparity of it being technically a hardware platform but we 
    would also have to make sure that the hardware we're providing for running these tests on, which would 
    be a VM, is always using the exact same revision as the one we're targeting in Tock. 
  - Leon: Whereas running it in this Github actions mechanism, as an actual test infrastructure attached to the code,
    we have to make sure that we always use the revisions which we are targeting. That's just a detail.
  - Pat: Well, this is kind of an important detail cause I think that's what it is differenting it from being an 
    actual hardware platform. The significance of the hardware CI is to say, *look the NRF52840 is like 
    silicon, this sits and is never going to change, whenever you need to work on this silicon* no matter what. 
  - Pat: Whereas Litex keeps changing, it's a moving target. It's not this hardware platform that exist 
    that everybody has copies of and is immutable. It's highly mutable and so for that reason leaving 
    it in software makes a bit more sense. 
  - Pat: I think I didn't totally follow that, like what is Litex trying to achieve like, 
    taking from a 10,000 foot view? Who wants a hardware platform that is always changing underneeth them? 
    I was wondering what the goal is just because it is so actively under development or do they have 
    plans on stabilizing the *hardware* eventually?
  - Leon: I think what you would do is when you run it in production on FPGA, you would just kinda revision and 
    work on that. Because it's only targeting FPGA, it's never meant to be custom silicon, 
    except that people are doing that. The original idea was that you're just having an SoC generated 
    for an FPGA, so you could potentially update the hardware of your product every half a year or so.
  - Pat: I think my instinct is if we ever added one of these silicon revisions, hardware revisions of Litex, 
    because somebody built a board around that we wanted to use, we want to use the durational hardware CI mechanism 
    because that is a fixed point. But as long as we're sitting on top of this weird and mutable hardware platform, we 
    should leave it all software to move with the target as it moves.
  - Leon: There's also a target for an actual FPGA and I would love to have that included in the hardware CI platform. It 
    would still put us in the same weird position where we would need to match the bitstream program onto the FPGA to 
    have the same sort of layout as the Tock code expects, but I suppose that could be figured out later. 
  - Leon: I didn't really want to make this discussion only about Litex, but also because you brought up that 
    this might be potentially interesting for QEMU. My question was whether that sort of this reasoning
    follows for QEMU as well.
  - Pat: I think does that, but what I would like to do is I'd like to get some of the actual hardware platforms up running 
    and integrated and, once that's really working well, then it would make sense to move the QEMU and stuff into 
    the hardware CI. 
  - Pat: Right now we have this obnoxious phonomenon where every month or two we have a new commit to move the QEMU Hash and 
    there's a lot of maintenance and setup of the QEMU environment for relatively little benefit to the 
    average Tock developer. 
  - Pat: I think it would make sense to have QEMU as hardware platform in the cloud that exist as a physical piece 
    of hardware *that just happens to be emulated* that's available to people.
  - Pat: I think that once the hardware CI becomes a little bit more mature, it's worth moving over but right 
    now QEMU works and so why mess with something that is working in order to move it into a new thing.
  - Leon: That makes sense.
  - Leon: If we ever start moving QEMU or Litex emulator in the future into the hardware CI mechanism, we 
    would also need to think about how people can get funding for when they want to provide an infrastructure. This 
    is Verilator, it simulates actual hardware, *it eats CPUs for breakfast*. That is also one concern.
  - Leon: Okay, I think that sounds like sort of a consensus of just leaving it as it is, as a software based platform at 
    first.
  - Pat: I think so. I think we're still firmly in the try a bunch of different approaches with this platform CI 
    and see what works best and if the hardware CI is coming up slower than I would like it to, let's not block 
    this on that.
  - Leon: Yes, makes sense. Thanks so much for providing these insights.
  - Pat: Cool
  - Hudson: Yeah, I weighting on not blocking this on that, because I do think this is neat and tests stuff 
    that currently none of the other QEMU tests cover so I think it would be nice to get it in.
  - Pat: Just as a micro update on the other side of the interest, the physical hardware CI ... . Each of 
    the individual tests works great, but when you try to wire up all of the wired connections permanently, 
    the Raspberry Pi gets unhappy because some pins are multiplexed as I2C and SPI, and when you're doing 
    an SPI test the I2C then gets in a bad state and it's surprisingly hard to reset the Pi's I/O controller. 
    He's fighting like driver issues on the Raspberry Pi, for one there's unexpected junk on the bus 
    and really just needs to force every GPIO to tri-state which should be easier than it is, but 
    that's what's going on there. So they all work in isolation, they just don't work in the untouched unattended 
    situation.
  - Phil: Which Pi?
  - Pat: 4B I think.
  - Phil: A lot of it depends on the exact processor, right?
  - Pat: Yes. They will dig a bit and they will get there.
  - Hudson: Cool, well that's exciting that is still going on.

## Libtock-rs
### Overview
  - Johnathan: For the past couple of years I've been assuming that libtock-rs apps would look a lot like
  and would have a similar code layout to the Tock kernel where you have a bunch of 
  independent components that all stick around statically and have the same two phase code 
  patterns for running asynchronous operations where they call into the other's component and they get a callback 
  when it's done. 
  - Johnathan: One drawback of that design is when you put `allow` into that framework, because everything's static, the 
  buffers you end up sharing with `allow` need to be static.
  - Johnathan: Now that I'm looking at the code bases that make use of libtock-rs, like OpenSK, Ti50 which is not publicly 
  available, but which I've seen and also Manticore which is intended to be ported on top of it, all of 
  them want to keep buffers on the stack. Being able to share things that have a non-static lifetime is pretty important 
  to them. I realized about a month ago that the API design I was using doesn't really work for them. So I've been looking for an alternative way to implement the `allow` and also the `subscribe` system calls so that the data that is 
  shared with `allow` and the drivers can be kept on the local stack frame rather than having to be static objects.
  - Johnathan: I'm having a hard time coming up with an interface that works that isn't ridiculously complex or just painful 
  to to use and also maintain. If anyone has an idea for what those APIs should look like that might be 
  sound please send them my way.
  - Phil: Can you show us some examples of what you said as ridiculous complex API? My guess is that this 
  is something you're discovering is really *devil's in the details*. It's gonna be hard at least for me to give 
  a lot of help or insight without seeing the details. I'd hate to spend four hours or eight hours and 
  just rediscover all the things you knew, and in fact, you already have gotten past that point easilly 
  and can tell me the sixteen things that we'll discover next.
  - Johnathan: One prototype I havet is what 
  I just [linked](https://github.com/jrvanwhy/libtock-rs/blob/wip/console/src/lib.rs) there, that's 
  kind of a prototype for how `allow` would soundly work on the stack.  It uses `Pin`. The unfortunate thing 
  about it is it means the drivers cannot call `allow` itself, instead the code using the drivers, 
  application level code, would have to make  the `allow` call and that's kind of shifting the responsibility 
  for `allow` to code that it doesn't belong in, but that's in order to make sure that the `unallow` 
  happens in time. If we attempt to do this with `subscribe` then you have the additional challenge of dealing 
  with callbacks and injecting a function as a generic, which is really difficult.
  - Johnathan: I don't have a lot of examples. This is the only real code that I've really fully prototyped out. Which is 
  why I was not planning to pursue this in this week's meeting.
  - Johnathan: This design gets really messy when you try to extend it to `subscribe`. I am considering making all drivers
  that requires `subscribe` static objects, as `subscribe` works with static references, but have the `allow` system calls
  work with stack local types. That combination might be doable and that is the avenue that I am currently exploring.
  - Johnathan: The big challanges are:
    - Allow: If you share something with the kernel with `allow` that has a non-static lifetime, you have to make sure that
    the memory is `unallowed` before it gets used for something else. For `allow_readonly` it's a threat model consideration,
    while for `allow_readwrite` it's a soundness consideration.
    - Subscribe: If the user data passed into `subscribe` is a reference with a non-static lifetime, you have to make sure that
    the `unsubscribe` happens before the lifetime ends. 

### Thoughts
  - Leon: I am not enteirly sure that I got everything right there. I remeber you had an issue with knowing what type
  of buffer was shared with the kernel in an asynchronous design, as the kernel only stores a ponter and a length which you
  get back. Is that still an issue?
  - Johnathan: Yes, that is still an issue. The bigger issue with that design is that it is all static references.
  - Loen: That doesn't work for the kernel, right? My questions would have been, if the problem of static buffers 
  did not exist, whether it could have been an option to store the annotation if whether it is a mutable or immutable
  buffer somewhere in the user space application's memory.
  - Johnathan: The challenge there is where? The allocation is tricky.
  - Phil: I have a question. I understand why in the general case this is going to be really hard. Is there some way 
  to constrain how this is done, to then have a simple idom, something along the lines: *here is a programming 
  construction you use where it automatically `allows` at the start of a block and then revokes at the end of the block*, 
  and inside you have a blocking call?
  - Johnathan: I think if I was better at meta programming I would do that.
  - Phil: Well then you're forcing someone, you're basically saying, *look here's a block, where I am going to `allow` at 
  the beginning of the block and I'm gonna revoke the end of the block* and then *as long as I can show that the thing's I'm 
  `allowing` lifetime is longer than lifetime of that block, which isn't a problem on the stack, then you're okay*. It's kind 
  of like the C++ construction for mutexes where a block can run for a mutex, so you hold the mutex for that block.
  - Jett: I've actually dealt with this recently in code and the one thing with the difference between mutexes and this, is
  that there's a safe function called `mem::forget`. You can call that anytime. When you call `mem::forget` in safe code it can't cause undefined behavior. It can cause bad stuff, like you poison your mutexs, but it's still not undefined 
  behaviour. The tricky part here is with sharing memory to the kernel. If you call `mem::forget` on the shared object 
  you won't `unsubscribe` or `unallow`, and so now you're breaking the aliasing rules, as the kernel does have access to 
  the same memory that the rust application does. This is undefined behaviour.
  - Jett: It gets really tricky really fast with an object that relies on `drop`, because you can call safe 
  function that prevent `drop` from happening.
  - Jett: That's one of the things that you have to be really careful about `mem::forget`, and that's one of the gotchas with that the `Drop`. You can cause incorrect behavior with `mem::forget`, stuff you would not want, but it has to still be defined and not break aliasing rules.
  - Phil: `mem::forget` does not run the destructor.
  - Jett: Exactly, so if you're depending on the `Drop` `impl` to get called to `unsubscribe` or to `unallow` then all of 
  a sudden, you're in undefined behavior teritory. I recently did this last week or two weeks ago when we did a 
  stack based thing. I can send examples out. We didn't rely on `drop`.
  - Phil: If I `mem::forget`, I have to have a reference to `mem::forget`?
  - Jett: The object that is storing your context. You make a wrapping API and you get and object that is your context,
  and the object goes of of scope, it would `unallow` or `unsubscribe`. This is how I understood your example of using a 
  mutex.
  - Phil: No, it's just a block or a meta programming structure which says *hey, I `allowed` at the beginning of this block
  and I rovoke at the end of the block*.
  - Jett: As long as we are not relying on the `Drop` trait it is overcomeable.
  - Phil: I don't understand the connection with `mem::forget`. If I `allow`, I should not have a reference anymore. 
  I would get the reference back by doing another `allow` and revoking the previous one.
  - Leon: The issue is that you need to store a context for the `allowed` buffer. For instance, its lifetime or 
  whether is an immutable or mutable buffer.
  - Phil: I see, ok.
  - Jett: That's one of the tricky constraints with all of this for sure.
  - Hudson: Jonathan, when you say you would do it that way if you were better at meta programming, do you think that 
  maybe the best option is to have a few people who like to write macros look at this and try to attack it that way?
  - Johnathan: Maybe that might be the case and I've been wanting to bring in a Miguel Young, who's really, really good 
  at that programming stuff. 
  - Jonathan: The two issues that I have with that approach are:
    1. it's gonna take me a really long time to figure out how to do it and 
    2. it potentially leads to a codebase that you need really good expertise to maintain.
  - Jonathan: Yes, that would be really helpful, and maybe once it's built it would be maintainable.
  - Hudson: Would that mean that any app calling `allow` would be calling a macro?
  - Jonathan: I'm thinking it would be calling a function with generics and a whole lot of type inference.
  - Jonathan: The other thing that I forgot to mention is that we do need to support concurrent operations.
  Every single user, except Ti50, wants to be able to do a console read with a timeout. That involves two
  different syscall drivers with two different `subscribes` that run in parallel.
  - Jonathan: I think the meta programming approach would be that you would have operations. Each opration 
  would have a type level list of which `allow`s and `subscribe`s it needs, and then you combine those operations 
  via another type level list into an *awful* generic argument into a function. The function materializes all of 
  the on-stack structures that you need to do those `allow` and `subscribe` calls and guarantees that `unallow`
  and `unsubsribe` are called correctly and invokes the callback.
  - Hudson: The issue with the `static_init!` as a macro is that the macro only gets expended once at compile time,
  so if you put the macro into a function and than call the function multiple times, the body is the same, regardless
  of the arguments of the function as the arguments are runtime options. The idea here is that because these are
  going to be generic functions, you will end up with a lot of monomorphized functions and, as a result, the expanded
  macro will be different in each of these functions. That is how we are hiding the macro from the user, 
  despite the fact that we need different expanded code for each call.
  - Johnathan: I wasn't even thinking that there would be a macro involved.
  - Johnathan: This would be like C style meta programming. There is probably a macro based solution too, but I didn't 
  think it was.
  - Leon: With procedural macros it is pretty easy to keep state while compiling, so that might actually be a solution.
  - Phil: For system calls where there is a standard model of `allow`, `subscribe`, `command`, upcall, 
  handlaing this should be much easier than when you have a series of upcalls, like an alarm. If you introduce 
  some blocking operation, than you can tie the lifetime of what's going on to that operation. 
  - Phil: Then you got things like received a packet. That seems much tougher.
  - Johnathan: OpenSK also does a thing where it is waiting for button press and blinks lights. That is a little more 
  complex than just a single main blocking call with the cancellation operation. I was hoping to simplify it down 
  to *you do one main thing and then you have some cancellation conditions that might involve a `subscribe`*, 
  but then looking through the OpenSK code base killed that idea.
  - Leon: We can't use something like closures for that, right? Because the order of things is not always fixed.
  - Johnathan: I'm not really sure what you mean by *use closure for that*, because there's a lot of spots 
  for callbacks to happen.
  - Loen: Never mind.
  - Hudson: It seems unlikely that we're gonna propose any concrete API right now given that Jonathan has 
  spend a month on this, but I think this is good that we were able introduce this to people. And then Jonathan 
  you can share stuff as you come along and then people will have some idea of the trade-offs here.
  - Johnathan: Yes.

### Way of working
  - Hudson: We could meet again in a smaller group as well to try to flush out a more concrete API or decide 
  between a couple of options.
  - Phil: This is a discussion that works better in writing. Could we discuss this on an issue or a PR or over a
  mailing list or over email?
  - Johnathan: I am leaning over issues or PRs, as markdown highlighting is very nice compared to email. I would
  like if people would open an issue for each idea. I don't want to have one single thread where different 
  ideas are going to be mixed up with each other. PRs are also fine for that.
  - Phil: Why would you have one issue per idea rather than a PR conversation?
  - Johnathan: I think that if we have multiple ideas that we are discussing back and forth they are going to get mixed
  up, especially when we are trying to come up with a working option. If we have multiple working options, one discussion
  to choose between them is reasonable.
  - Phil: That was my concern, that if everything is in a separate issue you can't comapre. But if we have an idea, 
  let's flush it out in an issue and that let's have another one where we discuss tradeoffs sounds a great idea.
  - Johnathan: Another way that I did this was to sumbit a document to the `docs` folder.
  - Phil: Have you written down what are the requirements? Something like * if the solution is clean good code for these 6
  use cases, than we're in good shape*.
  - Johnathan: That is something that I need to do.
  - Phil: That would be really helpful. Then we could say *this thing can do A, B and C, but D is terible*. It can range from
  simple things like *sleep* to *regular callbacks*. It would be useful to show how the high level application could 
  look like using libtock-rs. Then we've got to make sure you can map it down to low level instructions.
  - Johnathan: I will try to do that. That's something I have definitely attempted.
  - Phil: It can be a living document. We could realize we need to separate these into two cases which have slightly 
  different challenges, but that's fine.
  