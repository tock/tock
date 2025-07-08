# Tock Network WG Meeting Notes

- **Date:** February 05, 2024
- **Participants:**
    - Branden Ghena
    - Leon Schuermann
- **Agenda**
    1. Quick check-in on status
- **References:**
    - [3833](https://github.com/tock/tock/issues/3833)


## OpenThread
- Branden: Tyler posted https://github.com/tock/tock/issues/3833 about OpenThread progress
- Leon: Yes. I think the libtock-c route is the major thrust at this point. Likely to work faster. Still working on encapsulated functions from a research prospective, but it's going to take a while
- Leon: As a backup, we could just link the C code directly into the kernel if timing stuff really didn't work.

## Buffer Management
- Leon: I pushed buffer management stuff and need to get in contact with Alex's student. Next steps really need a prototype example.

