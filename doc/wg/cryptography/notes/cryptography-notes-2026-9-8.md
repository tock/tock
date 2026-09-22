# Tock Cryptography WG Meeting Notes

**Date:** 9-8-26

**Participants:**
  - Amit Levy
  - Hans Martin
  - Roy Bachynskyi
  - Alex
  - Tyler Potyondy
  - Alex

## Modular Arithmetic - https://github.com/theonlytruealex/Elliptic-Curves-Modular-Arithmetic
- Amit: As a reminder, the arithmetic HIL is what is changed from last week.
- Alex: For context, the old HIL from last week chained together operations where you first setup all the operations then passed into this the numbers. The issue was this is you then had a state machine in the client and in the driver. This is what I removed from last week.
- Alex: The new version now is operation then number without chaining.
- Alex: The reason for all these traits was to avoid runtime checks to ensure the operation is supported. Previously was just an enum.
- Amit: High level, this seems pretty good.
- Amit: Is it correct that the interface is read_number() provides next operation and operand? 
- Alex: Yes. If you want to do the next operation you give operation and a number. Otherwise if finished just return none.
- Amit: Does the hardware store the running sum?
- Alex: Yes. If it didn't you'd need to move the result every time. It is much more efficient to keep the running result inside the driver.
- Amit: Sorry, I meant does the hardware not the driver keep the intermediate result?
- Alex: From what I've seen, no.
- Bobby: One piece of feedback I'd give is to give the op value as an input to start. Right now there is two different data paths to get data into the driver.
- Amit: The idea is that these are chained computations so the formula the client wants to do is perhaps addition => multiplication => division. In each case, the client only needs to provide the right side of the operand. Maybe it's best to think of read_number() as read next operation and what you're returning is subtract two or multiple 17 etc.
- Alex: Yes exactly.
- Hans: How does this model work in an async context? 
- Amit: What do you mean by sync here?
- Hans: Right now every result is returning the result and operation. What happens if you have multiple operations where you are maybe sharing the hardware?
- Amit: This is a good point. One issue with this design and sharing the hardware is that your only option is to interleave full chains of computations. If your client has a long chain, you need to complete the full chain before another client can do anything. 
- Amit: This is opposed to an interface that is simpler and if it was modeling the hardware directly (one operation async then give result back) then each operation would be stateless and it is much clearer how you would virtualize this. 
- Amit: If you don't know the next operation, you aren't required to return the next operation in the callback.
- Alex: This could theoretically be abused if you had an infinite stream of calculations. This is true for aes or digest.
- Tyler: I agree with this and that this isn't an issue. The client is a capsule and a capsule is also permitted in Tock to do a while(1) loop.
- Bobby: One larger point I think all this gets at is this design has some inherent statefullness.
- Bobby: Statefullness requires the implementer of the HIL to absorb some complexity and business logic and by extension do some buffering and consume some memory. All of this in a vacuum I would argue should be in a capsule. In this example, the HIL implementation has to maintain an accumulator, buffer result of previous operation internally, and then present this chainable interface for clients. 
- Amit: Alex you've mentioned some efficiencies you were targeting in this design. Can you elaborate some more on this.
- Alex: In this design, if you're going to be using a result anyways for the new operation, we avoid needing to copy/pass the result across from the driver to the capsule multiple times.
- Bobby: Pluton has a very similar facility with arbitrary precision arithmetic operations. If accumulation is needed, we store this in the client and not the driver. For us this was the most efficient data path where we treat the client data buffer as the source of truth. Hardware is only responsible for copying from this client buffer into hardware.
- Bobby: Maybe there is other hardware where this wouldn't be the case. It seems that maybe what is optimal for one isn't for the other.
- Alex: Just to round up what we've said, everyone here would be open to making this simpler with reading the number back and forth?
- Amit: Yes, certainly if this works as efficiently. This seems like the right starting point. One thing I'd like to mention to highlight/ask. Thinking about this in crypto interfaces more generally, the HIL you have now is more in line with the data movement patterns look like in other crypto HILs we are imagining. It very well may be that this is an exception and it would be interesting to understand maybe why the patterns/rules do not apply.
- Bobby: To add to Amit's answer, I think that would be the less "controversial" and more "naive" approach, while admittedly not as optimized for long-running chained equations
- Bobby: Would others be amenable to me opening a driver mutex PR since some of this work relies on it?
- Group: Yes.
- Amit: One note, we may want to include a usecase with this to showcase how the driver mutex would be used.
