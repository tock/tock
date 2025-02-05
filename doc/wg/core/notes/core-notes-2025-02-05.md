# Tock Meeting Notes 2025-02-05

## Attendees
 - Kat Fox
 - Pat Pannuto
 - Ben Prevor
 - Hudson Ayers
 - Leon Schuermann
 - Brad Campbell
 - Alexandru Radovici
 - Viswajith Rajan Govinda
 - Johnathan Van Why
 - Amit Levy


## Updates

- Vish: Much progress on https://github.com/tock/tock/pull/3941
  - Implemented for Sequential 
  - Is this only for PIC?
  - Sequential: intended to mean layout of process binaries in flash is back-to-back with TBF headers.
    - Q: does RAM have to be sequential as well? Is that assumed?


- Pat: MobiSys'25 is in Anaheim in June
  - Tock tutorial there?
  - Conference tutorials helpful for deadlines for development and trying out new tutorials.
  - Could do a tutorial centered around root-of-trust with Tock.
  - Leon: it would be helpful to have insight in how people are using Tock for RoT.
    - How would this be different from our existing HOTP key tutorial?
    - More lecture-based. 1) What is RoT?, 2) What does Tock have to do for RoT?, 3) Try out some aspects with Tock.
  - Brad: would MobiSys attendees want this?
  - Hudson: my concern is we would need a lot of code development to support a proper RoT.
    - JVW: my recollection is there are many components in TI50 that are not upstream.
  - Leon: it would probably be hard to get a lot of info in exactly what would be required for Tock to be truly viable as a RoT.
  - Pat: do we need full functionality, or would one RoT task be sufficient to demonstrate the idea?
    - JVW: probably trusted firmware updates is the most "RoT-y"
  - TODO: need to decide whether to do a MobiSys tutorial, and if so, what topic.


## PR Review

- https://github.com/tock/tock/pull/4330
  - merge
- https://github.com/tock/tock/pull/4324
  - staging queue created
- https://github.com/tock/tock/pull/4300
  - supports a custom version of openocd
  - merge
- https://github.com/tock/tock/pull/4228
  - mark as last call
- https://github.com/tock/tock/pull/4250
  - rebase
  - need final reviews
- https://github.com/tock/tock/pull/4255
  - still differing viewpoints
