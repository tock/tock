# Tock Network WG Meeting Notes

- **Date:** May 05, 2024
- **Participants:**
    - Alex Radovici
    - Branden Ghena
    - Leon Schuermann
    - Tyler Potyondy
- **Agenda:**
    1. Updates
    2. LwIP in Libtock-C
    3. Encapsulated Functions
- **References:**
    - [BLE Support Planning](https://github.com/tock/tock/issues/4400)
    - [LwIP in Libtock-C](https://github.com/tock/libtock-c/pull/494)


## Updates
### RPi Pico 2 WiFi
- Alex: Darius my student is working on the RPi Pico 2. I've got him working on the PIO and the WiFi
- Branden: Awesome. My students pushed an SPI implementation to Tock for the RPi Pico 1, but it seems to not actually be what you need for the WiFi chip, as it uses a somewhat-strange single-wire SPI where you switch from TX to RX on the same wire. That needs a different PIO program. My students ran out of time and are no longer working on it
- Alex: The RPi Pico 2 and 1 have seemingly identical WiFi setups since Embassy shares a driver for them. We'll be working on the PIO first, then getting WiFi to respond. The goal is to just scan nearby WiFi networks.
- Branden: Definitely hard, but doable. The two big parts seemed to be 1) a different PIO SPI program and using it and 2) the packet structure the WiFi chip expects which has multiple layers. I did a search and have a few resources that could be useful if your student wants them
- Alex: Yes, I'll send an email connecting everyone
### BLE Work
- Tyler: I think people saw this, but a student at UCSD is interested in BLE support. https://github.com/tock/tock/issues/4400 I'll get more information soon but they might be starting on something.
### Async Driver Support
- Alex: There are a lot of drivers using async Rust written for embassy, which even includes a Bluetooth stack. I've been lately considering how to add an async engine to the Tock kernel. It would be optional, and doesn't work yet, but this could be helpful if people want to add written drivers to the kernel
- Leon: What would that look like?
- Alex: I'm trying with the timer first. You would implement the embedded HAL on top of the Tock capsules, then have an execution engine which could do one future at a time. It might be useful for simpler drivers, and if it works we could try it with more complicated network drivers. People are writing a lot of drivers out there, but they're async because it's easier. This could bloat code size and might need to be nightly rust with an allocator, but it would fail fast.
- Leon: You only need dynamic allocation if you have recursion
- Alex: Not quite. You need to box the future. Call a function, unwind the whole stack, then when the client comes back call the future again. Who owns the future is the problem
- Leon: But the key here is if you know how many futures you possibly have, you're static
- Alex: The problem is the size of them. It's an opaque type, but you can name it, then you can compute the allocation statically at compile-time. But I don't know if that will work yet
- Leon: At a high level, this would be a super interesting exploration. I don't know if upstream will want it, but it sounds very intriguing, especially if it doesn't require modifying the kernel
- Alex: It shouldn't require changing the kernel. It could just be a capsule which would be optional
- Alex: I feel that I'm close, but it's not compiling yet. But I do plan for it to fail at board initialization, not at run time. It does have external dependencies though
- Branden: Sounds awesome
- Alex: I want the drivers as drop-ins. Lets someone use them downstream. Upstream would just have the executor.
- Branden: Would those drivers have unsafe stuff?
- Alex: Most of the drivers don't need unsafe. Just the embedded HAL


## LwIP Libtock-C PR
* https://github.com/tock/libtock-c/pull/494
* Branden: Is there anything you need for this PR?
* Leon: No. It still seems to work. There are small conflicts on a rebase I need to fix
* Branden: I have a few small comments that are open too. But easy to fix
* Tyler: Is any of this going into Treadmill yet?
* Leon: Not yet, but it should. Haven't gotten to it.
* Tyler: What does this run on?
* Leon: It runs on multiple targets. But I've been debugging on the QEMU board.
* Branden: Let us know when you need things here, like approvals


## Encapsulated Functions
* Leon: I'm back working on encapsulated functions, but sort-of foundational work making this code more usable and less of just a research hack. Documentation and a better workflow to use it. That'll be my focus for the next few weeks.
* Leon: We do currently have a publish version from the paper. It's pretty rough though so I don't know if I recommend it
* Branden: Is the goal a PR to Tock?
* Leon: That would be great. I don't know how likely that is. You do need some external crates that need a bunch of unsafe. But I think this would be similar in scope to our current policies on external libraries, especially if this is just used for optional libraries. Board developers could just to incorporate this or avoid it
* Leon: To say differently, this doesn't need to be part of Tock upstream
* Branden: How would someone use it then?
* Leon: There's a base crate with common infrastructure, you could pull in. There's a runtime that depends on the Tock kernel, for example to modify the MPU. Then there's a component that modifies Rust bindgen. So the way you use those three is to combine the bindings for the library with this runtime component and your crate depends on both of these.
* Branden: So instead of being in Tock, this would be a separate repo which would depend on Tock
* Leon: That's one way. Or you could have another crate in capsules that pulls this in, defines stuff, and gives an example of using it. The only thing that would ever need to include this is if you use that capsule though.
* Leon: Quite similar to the crypto capsules now which has an external dependency
* Branden: Okay, so there would be a PR to Tock, but it would be making a new crate that uses this as  an example. And that would tell us if we broke things in future Tock updates?
* Leon: One part would need to get added. The runtime could be included in Tock. For example the Tock-PMP runtime would go into Tock and depend on the external "omniglot" infrastructure which is independent of Tock
* Branden: That makes sense to me. What example would you include if you did?
* Leon: We have cryptolib, littlefs, and lwip
* Branden: Maybe something from Alex's side of the world?
* Alex: We'd be interested in a certified C driver. But I'm not sure if the encapsulated function stuff would require changes.
* Leon: For hardware access specifically, we need to not expose them if they can do DMA accesses. If the peripheral can do DMA and you can pass them a pointer and length, that could break everything. There are also other implications about shuffling data to-and-from the foreign library. It can't access the kernel's memory, so you have to allocate memory in the encapsulation. So it's hard to say if we would be able to run certified code or not. Depends on the exact details
* Leon: We might be considering BLE inside encapsulated functions as a future effort
* Tyler: Could be neat. Has timing constraints which makes userland implementations infeasible


