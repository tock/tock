# Tock Core Notes 2020-10-02

## Attending
 * Johnathan Van Why
 * Philip Levis
 * Amit Levy
 * Leon Schuermann
 * Alistair
 * Brad Campbell
 * Hudson Ayers
 * Vadim Sukhomlinov

# Updates
-None

# 1.6
- Amit: only release blocker is e310 interrupts PR
- Decided last time that interrupt changes shouldn't block that PR.
- Phil/Alistair any concerns with that PR?
- Phil: I will take a look.

- Amit: Assuming we merge that, ready to test 1.6.
- Timeline? Typically 1 or 2 weeks?
- Hudson: I'm ready to help test.
- Amit: I can test, but have few boards.
- JVW: I can test H1.
- Amit: We can test after we merge the e310 pr!
- Phil: I agree that we can merge the pr, change interrupts later.

# Soundness Bug in Grants
- Hudson: Bug comes from entering grants twice, and getting two mutable
  references the same memory.
- A couple capsules do this.
- Probably not an issue, now, but would be if Rust changed its mutable aliasing
  optimizations.
- Leading solution is having some sort of lock when grant is entered.
- Amit: Like TakeCell.

- JVW: Issue with allow() and an app sharing the same app slices multiple times.
- Leon: Known issue, but different issue.

- Hudson: Proposed fix should break fairly little.
- Amit: Might want to get this in for 1.6.

- Hudson: Want a low-overhead fix.
- Brad: Could put the flag in the grant region itself (only used when grant is
  actually created).

- Amit: Using 0 for the unused pointer is a nice hack since it works as null.

- Leon: If we use all ones, we should check.
- Brad: However, it is impossible for a grant region to ever be all ones based
  on how the layout of apps works.
- Leon: It would be good to document in code what these values mean.

- Brad: Why are we not considering static approaches?
- Various reasons the note taker didn't quite capture.
- Lot to do with the dynamic nature of grants.

# Tock 2.0
- Phil: What to do with unused registers in syscall?
- Consensus: no issues, but still want to specify.

- Phil: API for return values in kernel. Specific capsule syscalls can only
  return one success type and one failure type. Static checking versus dynamic
  dispatch. Can we get checking without overhead?
- Leon: My approach avoids dynamic dispatch, but might have some overhead.
- Leon: Might return types to be more than u32.
- Phil: Yes, and this would allow for more static checking.

# Tock 1.6
- Amit: Hudson and I will look at grant soundness.
