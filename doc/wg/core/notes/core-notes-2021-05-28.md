# Tock Core Notes 2021-05-28

Attending:
- Hudson Ayers
- Jefferson Chien
- Arjun Deopujari
- Branden Ghena
- Philip Levis
- Amit Levy 
- Arvin Lin
- Andrew Malty
- Gabriel Marcano
- Pat Pannuto
- Alexandru Radovici
- Leon Schuermann
- Jack Sheridan
- Vadim Sukhomlinov
- Anthony Quiroga
- Johnathan Van Why

## Updates?

### Libtock-c fuzzing

Alexandru: a student is working on fuzzing libtock-c, seems to crash. Would love help figuring out what is actually happenning

Phil: experience is dumps for panic has all the information necessary but hard to find

## Hardware-in-the-loop CI

Arvin, Jack & Jefferson demo hardware-in-the-loop CI

- Have been working on this project for past quarter at UCSD
- Workflow:
     1. Developer pushes code to repo
     2. GitHub action triggers an action on a raspberry pi
     3. Downloads version of code
     4. Test harness builds Tock, install on target device, build tests from libtock-c
     5. Run some tests in Python
     6. Test results show up in log

- Arvin demonstrates a live demo!
  - Demo is using the NRF52840DK
  - We see the raspberry PI observing outputs from the NRF52DK pins
  - Add tests is as simple as adding another test name to the config. Let's see it what happens if we add another test? Expect it to run both tests.

Branden: How does the python script interpret the tests from libtock-c?
Arvin: Comeback to this

Phil: are name formats defined by test framework? Or just arbitrary
Arvin: Just arbitrary
Phil: suggest more descriptive test names

Johnathan: How many devices involved?
Arvin: RasPI is running the tests and the GitHub actions, so only one machine in addition to the target.

Leon: Is it possible for others to have others providing different boards?
Jack: Idea is anyone from anywhere could spin up a PI with their target board, will listen for a _particular_ board in the Tock repo.

Branden: Please share slide deck? Video? [Slides are located here](https://docs.google.com/presentation/d/1rWZU8UPYhbEYJ44ri1KKlzCbSIzo_eYPDP9AnRPs0DY/edit?usp=sharing)

You can specify different Raspberry PIs for the same board by giving each board an ID

  - Amit: why would you want this?
  - Arvin: so you can parallelize long running tests
  - Branden: What links `tests.all` or `tests.[some_id]` to a specific Raspbery PI IP address?
  - Arvin: when setting up a test runner, you specify a label for each Raspberry PI (e.g. nrf52dk). The raspberry pi specifies its own specific ID---it will get all test, but filter based on the id in the config.
  - Jack: Note that the raspberry pi performs the outgoing connection persistently
  - Hudson: If you specify multiple RasPIs to run, does it load balance or run on all?
  - Arvin: we believe it runs on all

  - Alexandru: Whoever wants to use their own runner needs to add a file as a workflow. When they send a PR, they would need to block that file from being added to the main Tock repo?
  - Arvin: If we have standard labels, you would do it on the same label with your own repository
  - Alexandru: If I clone the repo, and my repo on GitHub I don't have that runner, is this a runner?
  - Arvin: Not, it's simply ignored

## ReadOnly Allows

Leon: Two issues:

     1. Some discussion regarding guaranteeing atomicity
     2. How do we want to deal with AppSlices when allowed to kernel? Can userspace read them?

### Leon + Alistair discussion: General semantics
Currently, single implementation of vDSO system call with well-defined limited
functionality. Realized through a driver implementation that resides in the
kernel crate that could not be changed by different boards.

Conversely, could make this a trait based system that is passed in by board

Phil: Agree in general with Leon's approach, but... in practice vDSO in Linux
has been around for a while and only used for a few things. Are there really
many use cases? If not, better to just optimize for the one use case. Want use cases that need updating at very low cost ...

Leon: Maybe at least move it to Alarm capsule, so the that the kernel is not
calling to _specific_ capsule, which is strange.

Hudson: This is also sort of blocked on updates to TRD 104

## TRD 104

### Hudson's summary

Current wording: in order for userspace to access buffer, it must unallow the
buffer, even though not enforceable by kernel.

Unfortunately this adds overhead. If you want to read from buffer, you must
first unallow the buffer. In many cases, it's possible to write application
that it's not necessary. As long as capsule doesn't write to buffer...

Similar case with ReadOnlyAllow

Question: should we relax this constraint?

Problem: would constrains us from future optimizations should MPUs become more
granular.

Benefit: could be more performant in particular in high throughput cases

## Process console PR:

Brad commented on mailing list: printing out capsules is too much, so let's
just cut out printing out the capsules.

Issues: process console replicates a bunch of stuff that' in panic.
Symbol table question: whether the kernel should export is memory symbols

Process console wants to print out the kernel memory map, so kernel in current PR exports symbols

Brad suggests passing symbols into process console rather than relying on exported symbols
