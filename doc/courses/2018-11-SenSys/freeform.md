# Free-form Experimentation
---

## Course Agenda

- [Introduction](README.md)
- Part 1: [Getting started with Tock](environment.md)
- Part 2: [Application Basics](application.md)
- Part 3: [Client Delivery](client.md)
- **Part 4: [Free-form Play](freeform.md)**

---

These are seedling ideas, feel free to try one of these or anything else that
strikes your fancy!

## Kernel Hacking

 - Understanding board configuration:
    - Look into the four-process limitation, where does it come from, can we add more processes, how?
    - How are pins mapped and buses configured? Could we do something like tie the LED to be a SPI status indicator?

 - Add a new capsule to the kernel (write it, include it in the crate,
   initialize it in a boot sequence).
    - Can you send 802.15.4 packets directly from the within the kernel?

## Userland Hacking

 - What other sensors and interfaces are available?
    - Write a thermal alarm app?
    - Write a light-change triggering motion detector app?
    - Understand how the timer interface works and virtual timers?

 - Execution model
    - What happens when there are multiple concurrent interrupts?
    - How do blocking calls and callbacks interplay?

## End-to-End Comprehension

 - What happens when your app returns from main? When will it run again (if at all)?

 - Can you diagram what happens when a process makes a blocking system call
   (context switch, interrupt setup, wait_for, interrupt handling, kernel
    thread, resuming process)?

