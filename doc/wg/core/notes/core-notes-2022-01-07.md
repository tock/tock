# Tock Core Notes 2022-01-07

Attendees:
- Hudson Ayers
- Branden Ghena
- Philip Levis
- Amit Levy
- Alexandru Radovici
- Jett Rink
- Leon Schuermann
- Vadim Sukhomlinov
- Johnathan Van Why

## Updates

### Asynchronous Process Loading

- Phil: continuing to work on asynchronous process loading for
  signature verification. Made good progress. Starting the
  implementation with credentials which are just SHA hashes, given the
  AES interface is not finalized yet.

  Will result in 3 coupled PRs to elf2tab, Tockloader, Tock.

- Leon: Do you need software implementations of SHA or AES? Have them
  lying around, would need to update them to the new interfaces.

- Phil: Don't think so.

  Hope to have it done by end of the month.

### Book "Getting Started with Secure Embedded Systems"

- Alexandru: Our Tock book titled "Getting Started with Secure
  Embedded Systems" is released. Made a PR to add it to the main
  README. Not sure where it should go, but people should be able to
  find it.

  https://link.springer.com/book/10.1007/978-1-4842-7789-8#toc

- Hudson: This is great. We want others to be able to find it.

### Code Size Analysis

- Hudson: Working on a paper about some of the code size issues when
  working with Tock on Ti50. Planning to submit it to LCTES. It has
  some interesting insights, will share when it is ready.

## libtock-c PR #274

- Hudson: it is possible that one could call `delay_ms` and the kernel
  could fail to set the alarm. This would cause the app to yield
  indefinitely. Terrible to debug.

  Proposed solution is to check the places where it could fail and
  print errors.

  Don't like the solution: some people would prefer to rather not
  print things. Also, it makes small apps significantly larger (blink
  app goes from ~1400 bytes to ~2400 bytes), as for sleeping the
  entire console machinery would need to be included.

  Fixed I could imagine:
  1. some mechanism for making the printing configurable by apps to be
     able to opt-out.
  2. errors are returned when possible, when not possible the app
     would simply not yield.
  3. user-provided function pointer for user-defined error handling
     when setting an alarm fails.
  4. have `delay_ms` exit the app when failing to set an alarm.

  Complicating this, the libtock-c alarm driver has a virtualization
  layer such that a single app can set multiple alarms
  simultaneously. When an alarm fires, in the callback for that alarm,
  it will check if there are future alarms and then set those. When an
  alarm is requested it is enqueued in this structure. If an alarm
  fails later in the queue, it's not trivial to propagate this error
  to the original caller.

- Branden: what is causing the kernel to return an error when setting
  the alarm?

- Hudson: don't actually know. Apparently this has been happening on
  OpenTitan. However, given that it is technically possible for alarm
  to return an error, we should probably handle those.

- Phil: second Branden's question. Are we paying a cost for an
  exceedingly rare edge case?

- Branden: it appears that only the first time a timer is set it could
  fail? Because if a second timer will be set, this is in response to
  the first timer, which then must've already worked.

- Hudson: not impossible, but it would be pretty unusual and involve
  a bug in the kernel alarm driver.

- Phil: another approach could be that, if this is of concern, a
  developer should roll their own solution.

- Alexandru: looking at the kernel, it does not seem like `set_alarm`
  returns anything which could indicate an error.

- Hudson: essentially it could fail for the same reason any call can
  fail, which is not being able to enter the Grant.

- Branden: presumably this is the first time a timer is set?

- Hudson: yes.

- Alexandru: `delay_ms` does not guarantee that it sleeps the exact
  amount of time, but it might sleep for longer. We could define it to
  just not sleep at all if it errors.

- Phil: the alarm interface is designed to never fail, as it has
  wrapping semantics. We need to find out what is generating this
  failure.

- Hudson: it seems the only reasons for failure is if deprecated
  command 4 is called, or the Grant region cannot be allocated.

- Branden: I don't hate that they are paying attention to this issue,
  so returning an error seems like a good option.

  It would not hurt backwards compatibility, and it would only every
  occur on the first call (if this is related to Grant allocation
  failure).

- Phil: the hypothesis is that this is a Grant problem, we should
  verify that.
