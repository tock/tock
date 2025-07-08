# Tock Core Notes 05/15/2020

Attending
 - Brad Campbell
 - Alistair
 - Johnathan Van Why
 - Amit Levy
 - Leon Schurmann
 - Hudson Ayers
 - Branden Ghena
 - Philip Levis
 - Vadim  Sukhomlinov
 - Samuel Jero
 - Pat Pannuto
 - Andrey Pronin

## Updates

- Hudson: Working on getting Github Actions CI working.

## Testing

- Phil: Goal is to do testing with board-independent names. See email on
  tock-dev for my proposed design.
- Phil: First, do we agree on the three design goals?
- Amit: Strongly agree.
- Leon: How do tests signal success or failure? Is there a bool?
- Phil: Agree we need some mechanism, but that should live outside of Tock code.
- Leon: We want the check to stay in sync with the test.
- Amit: I think Go handles this well.
- Phil: Yes, this has been done, I'm sure we can copy something.
- Phil: Namespacing objects means capsule setup will change.
- Amit: So there will be global statics?
- Phil: Yes.
- Andrey: Want to be able to run a full test suite, and support boards that
  don't have loadable apps.
- Phil: This only covers kernel tests.
- Andrey: What series of tests would be run?
- Amit: Would there be test conflicts? Would they be compiled together?
- Phil: Could re-run or re-compile on test failure.
- Phil: My intent was not to prescribe the exact test format.
- Branden: Could read/write flash between tests for any state.
- Leon: I would be concerned about flash write cycles.
- Andrey: And the performance implications of that.
- Phil: This issue has to do with cleanup after tests, we don't want it to be
  too hard to write tests.
- Phil: I will look into test cleanup and running a series of tests.
- Andrey: There could be a test runner which tells the board which test to run.
- Samuel: If state is shared, really need to build separately. If there are
  failures, want to know what the cause is.
- Phil: Multiple tests can also lead to timing issues (example: iterating many
  virtual uart devices).
- Phil: I will start on this after teensy port to 1.5.
- Amit: Getting some testing going is great, no need to stall on perfection.
- Phil: We do want an encompassing design.
- Hudson: Is it going to be ok to have global statics? We have been moving away
  from them in other contexts.
- Leon: They could be encompassed in a trait.
- Amit: static_init! solves a different problem.
- Hudson: But it also restricts multiple references.
- Leon: I think we can work around this issue.
- Phil: Can still do static_init!.
- Hudson: Why do we need namespace then, if the traits match I can just pass in
  an object.
- Phil: Hmm, something to think about.


## License

- Pat: Need to add headers. PR adds draft header text. Year needed for
  copyright. Need to agree on that.
- Phil: Useful for last updates and authors when thinking about code files as
  separate documents.
- Johnathan: Most projects do not do per-file authors.
- Pat: We can disentangle license issues from authors/maintainers issue.
- Phil: I agree, but these are documents.
- Johnathan: There are questions about how to implement the automation.
- Pat: Want CI to enforce.
- Leon: There could be a wrapper text, and CI does byte-by-byte comparison.
- Johnathan: I want to avoid having a wrapper.
- Amit: Our case should be easy to actually do this. Should CI check for year?
- Pat: Yes, check for text and year.
- Amit: Can there be multiple copyright blocks?
- Johnathan: That is a question for legal.
- Johnathan: But yes, should be able to build CI tool.
- Brad: Should not be able to make comments about this in a PR; if CI is happy then the PR is good to go w.r.t. licensing.
- Amit: Byte-by-byte check should work.
- Samuel: Makefiles includes?
- Pat: Yes, I think so.
