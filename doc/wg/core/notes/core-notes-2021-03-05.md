# Tock Core Notes 03/05/2021

## Attending
 * Brad Campbell
 * Phil Levis
 * Arjun Deopujari
 * Branden Ghena
 * Gabriel Marcano
 * Vadim Sukhomlinov
 * Leon Schuermann
 * Pat Pannuto
 * Johnathan Van Why
 * Alistair
 * Hudson Ayers

## Updates

### Benchmarking Tock
- Work continues on benchmarking syscall v1
- Hacked module to get performance counters to userland as ibex core does not
  expose counters to user mode. Bug report submitted.
- Future work: more permanent userspace interfaces for counters.

### libtock-rs
- Raw syscalls PR didn't work on arm, been updated to now work on arm and rv32i
- Adding tests for Miri
- Now working on fmt::debug.

## Tock 2.0
- Almost there! Capsules and syscalls done.
- Open PR for alpha1 is not full 2.0, still need some guarantees implemented.
  - Specifically subscribe and allow.
- Working on docs, polishing comments.
- We can start testing on Monday!
- Need to chart out remainder of what needs to be done for 2.0.

### Suggested changes
- Avoid on PRs which do not merge to master.

### .map() and error handling
- 2.0 is the time to reduce .map use and force authors to think about what
  should happen if the map fails.
- How many cases exist where .map_or() has no good return value in the "or"
  case? 3 or 4? More?
  - If small number, maybe we can fix the capsules?
- If map fail not handled, capsule could hang. Not great.
- If everything is in grant, then map won't fail...grant.enter() will fail
  instead.

### GenericSyscallReturnValue
- Rename to SyscallReturn.

### Callback swapping restrictions
- New approach does not pre-allocated all grants.
- Now enforce only one callback per (Driver, Process, SubscribeId) tuple.
- Caveats:
  - Only 1 grant region per capsule.
  - Capsules must store callbacks in grants.
  - Grant creation needs the DriverNum.
- Concern about boilerplate in capsules.
- Need guide for implementing `Driver`.
- Standards for virtualized capsules?
  - Standards helpful, but there will be exceptions and customization.
- Virtualizing all capsules is hard. How to implement change without blocking
  for too long?
- Do we actually need to check for duplicate grants in code?
  - Worth it if check in userpsace is too difficult. Otherwise overhead not
    worth it.
- main.rs now needs driver nums in two places
