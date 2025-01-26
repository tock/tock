# Tock Core WG meeting notes 2024-11-01

## Attendees

- Amit Levy
- Leon Schuermann
- Benjamin Prevor
- Brad Campbell
- Hudson Ayers
- Johnathan Van Why
- Pat Pannuto
- Alexandru Radovici

## Updates

### Treadmill-CI:

- GPIO tets are working, mpu_walk_region and button_test are working
- RPI5 libaries for gpio created small issues button_test
- Within one week nrf tests in tock repo

### Add StreamingProcessSlice helper, based on ProcessSliceBuffer design

- [https://github.com/tock/tock/pull/4208](https://github.com/tock/tock/pull/4208)
- Adds StreamingProcessSlice wrapper for streaming data from kernel to userspace
- Based on ProcessSliceBuffer design with modified semantics
- Improves documentation and buffer handling code
- One can use StreamingProcessSlice as a receive buffer for IPC
- It's hard to test because there is no upstream ADC driver
- Amit merged the PR

### PR Reviews

- Discussing a PR about capability pointers (formerly meta pointers)
- Concerns about naming conventions and potential confusion with hardware capabilities
- Discussion about whether to merge current PR and handle naming changes later
- Need for clear documentation of design and abstraction layers

### EWSN Tutorial Planning

- Tutorial happening in ~5 weeks
- Expected attendees: Brad, Tyler, Anthony, and Pat
- Hardware currently in San Diego
- Total conference attendance ~130, tutorial attendance TBD
- Action items:
  - Schedule planning call for next week
  - Verify nothing is broken
  - Polish existing content
  - Brad working on nonvolatile storage driver upgrade
  - Switch to new streaming slices discussed earlier
  - Resolve infrastructure build questions
  - Consider sustainability of tutorial materials

### GCC/Build System Discussion

- https://github.com/tock/libtock-c/pull/470
- Debate about printing commands during build process
- Discussion about whether to suppress command output with @ symbol
- Concerns about consistency with rest of build system

### CHERI Implementation Progress

- Meta pointer PR renamed to capability pointer
### x86 port progress

- Two parallel efforts for x86 external clemency:
  1. Rewriting (Bobby)
  2. Rendering necessary parts including big flags (Zane)
- Ongoing discussion about naming conventions and type definitions
- Need for TRD (technical requirements document) updates or extension

## Action Items

1. Schedule EWSN tutorial planning call for next week
2. Review capability pointer PR in more depth
3. Synthesize current PR changes and identify controversial/separable parts
4. Further discussion needed on x86 kernel PR comment
5. Review TRD 104 section 3.2 regarding return values

## Open Questions

- Relationship between capability pointers and IPC
- How to handle tutorial environment setup for attendees
- Definition and scope of capability pointers in non-CHERI architectures
- Whether to merge current PR and handle naming changes later
