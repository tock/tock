# Tock Meeting Notes 11/17/23 ===================================

## Attendees
- Hudson Ayers
- Amit Levy
- Leon Schuermann
- Branden Ghena
- Alyssa Haroldsen
- Jonathan Van Why
- Andrew Imwalle
- Tyler Potyondy
- Alexandru


## Updates
- Leon: PMP implementation is still waiting on upstream fixes in the upstream
  RISCV implementation to trickle down, then they can update an external
  dependency and get that merged. The dependency is a softcore CPU
  implementation
- Tyler: Submitted CPS-IoT tutorial proposal. Minimal 1-pager that Pat helped
  with. Hopefully happening in the Spring. CPS-IoT is IPSN + RTAS + IoTDI + 1
  other. Mix of real time and sensor net conferences.
- Tyler: Goal is to advertise Tock as a sensor platform you can easily get up
  and running for IoT / networking research.
- Amit: Probably will be a bit different than other tutorials as a result

## Networking Group Update
- Leon has been making some buffer management updates. Effectively, trying to
  find a way around the Rust compiler to have a set of abstractions for
  representing buffers that allow us to verify at compile time that a set of
  network layers conform to hardware’s expectation about reserved headroom and
  space toward the end of a buffer

## TockWorld Planning Update
- Survey coming out soon with times / dates surrounding June
- If there is anyone not on this call or that Brad/Pat might not obviously
  think of, send those folks to Amit

## Yield Wait-For System Call discussion
- https://github.com/tock/tock/pull/3577 
- Amit: I added a summary at the bottom of the PR
- Amit: Goal is to find a path forward
- **Reading Break**
- Amit: My suggestion is that we accept some unanswered questions and focus on
  just finding a path that seems viable and then we can learn from how it goes
- Amit: Summary of Yield-WaitFor-NoCallback (YWFNC): There are two problems
  with current Yield system call: it is too easy to write app code that will
  blow up your stack, in the case that there are calls to yield() within a
  callback. Additionally, there are requirements for application code that
  operates synchronously (efficiency or safety reasons). Currently those are
  implemented by faking blocking calls but that is often insufficient.
- Amit: Competing goal is to support better patterns without having two
  parallel worlds of Yield and “blocking yield” that capsules and apps have to
  implement in parallel. The goal is for modifications in capsules to support
  both paths to be minimal
- Johnathan: That characterization seems reasonable
- Amit: Johnathan can you elaborate on this from the libtock-rs perspective?
- Johnathan: My syscall tunnel app work has given me some perspective on this
  problem. It has to run normal yields until [garbled]. So that is an
  interesting thing I have not seen represented in the TRD.
- Amit: What I was getting at is that there are optimizations you can do in C
  like only ever registering one callback once, but doing the same thing in
  Rust does not work safely
- Johnathan: That is true with the current libtock-rs APIs, but it should be
  possible with the pin-based APIs using statics and a sync-wrapper
- Amit: But it still does not solve the re-entrancy issue more generally
- Johnathan: Yeah re-entrancy is a pain
- Alyssa: There could be an up call registered for an entirely different
  subscription which could contain a print, so does YWF, other unrelated
  upscales can be invoked, right?
- Amit: The semantics of YWFNC, no callbacks period run on that process
- Alyssa: I see, no callbacks on the process, not the subscription
- Amit: A significant difference from what I had proposed originally, but that
  is a good point that that is necessary to actually address the reentrancy
  issue
- Alyssa: I want to clarify that if I have code with a subscribe which has an
  up call waiting for a stack variable. Would it be correct to replace that
  with no subscription at all, just a command then a YWFNC. 
- Amit: That is correct, I believe. The data that would have been passed in a
  callback will be returned in YWFNC. And the subscribe number is passed to the
  kernel in the YWFNC.
- Alyssa: Can you add in your summary at the bottom a description of
  yield-result? 
- Alyssa: What is “Yield-Result”? Whether an upcall occurred or not?
- Johnathan: Yes
- Amit: Where are we seeing this?
- Alyssa: I ctrl-f’ed `yield-result`
- Amit: I think that is a mistake actually. YWFNC should not have a
  `yield-result`.
- Amit: You found an inconsistency. This is here because the TRD includes
  multiple versions of yield-waitfor and some of them do this. We can take away
  one of the arguments if we stick with YWFNC. There is no way for an up call
  to be called!
- Alyssa: I propose that we use the same registers as other yield calls to
  reduce kernel overhead
- Johnathan: They overlap with the yield number if you try to put them in the
  same spot as subscribe
- Leon: I think we currently always extract the same registers anyway and then
  route them to the right calls
- Amit: Alyssa is probably right that we end up with more register shuffling
  this way
- Alyssa: Yeah, but sounds like it is a moot point.
- Amit: Question for Alyssa: it was your suggestion that we have an optional
  subscribe, the idea was that YWF has similar semantics to YWNFC except that
  you optionally pass in a callback, and if you do so that is called before
  returning. Is that accurate that you suggested that?
- Alyssa: I think so — looking for my comment
- Amit: There seems like a lot of buy-in for no callback right now
- Alyssa: I do like my option for a bit more control and it seems it would work
  well for Rust as well
- Amit: My suggestion is that we just go for it, and once we have accumulated
  some experience and evidence with this design we can evaluate it better. Even
  with that suggestion there are two things I think might be worth hashing out
  ahead of time. There are other potential costs in terms of reentrancy /
  correctness issues that a design might incur. And it would be good to have
  some way of evaluating whether on net we have improved in those dimensions. 
- Amit: We also want some gauge for what would count as a code size or
  performance benefit. It seems likely there is overhead to doing any of this
  in the kernel. We want to see some benefit on the application side and that
  would be nice to quantify.
- Amit: In Pat’s prototype C application there was some benefit and we want to
  see if we can do better with a better implementation. It would be nice to
  have some idea of at what point that tradeoff is positive: 1 application? 10?
  What complexity application?
- Hudson: Are you advocating an order here?
- Amit: Not necessarily since it seems we are gonna go with this anyway. Just
  want a sense for what are good benchmarks.
- Alyssa: I think the kernel implementation having less than 500 bytes of
  overhead would be good
- Alyssa: The benefits from blocking command were hard to quantify because of
  other simultaneous changes but we would like to see similar performance. We
  can ask Jett and see if he remembers
- Amit: If there are two scenarios, one is we get a benefit on the application
  side on each driver that uses this pattern, and the other is we get it for
  each callsite of that driver, it seems like maybe the latter is better? Do we
  have a feeling on this?
- Hudson: A great test would be, once we implement this, if Ti50 uses it, does
  their size improve or worsen?
- Amit: Say we don’t merge this or update libtock-c / libtock-rs in the main
  branches, because that would be sort of a commitment. Instead we go off into
  a corner and try it out, I think we want a metric for if it worked.
- Amit: My concern is that our design might be bad in a way that is really
  close to being the right thing but far enough off that we don’t see any
  benefits, and I don’t want to end up there. 
- Hudson: I propose savings of ~2.5kB for 5 applications versus the 500 byte
  estimate of overhead in the kernel.
- Alyssa: Jett thinks he has the numbers for savings from blocking syscall and
  will look for them.
