# Tock Meeting Notes 2024-04-19

## Attendees
- Branden Ghena
- Brad Campbell
- Pat Pannuto
- Johnathan Van Why
- Viswajith
- Alyssa Haroldsen


## Updates
* Chatted about Makefile syntax and debugging for a bit
* Brad: Working on updating the nightly https://github.com/tock/tock/pull/3842 Still some build errors. Failing in the tests, which only run on CI. It looks like the stopgap PR didn't quite cover everything

## Libtock-C Rewrite
* Brad: I'm going to try to rebase the libtock-c rewrite PR. We should then have a better sense of how close it is to compiling
* Branden: That's multiple Makefile updates lately, right
* Brad: Yes, everything for the split of libtock and libtock-sync. The others are for openthread


