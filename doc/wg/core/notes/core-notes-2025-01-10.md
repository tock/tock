# Tock Meeting Notes 2025-01-10

## Attendees

- Johnathan Van Why
- Amit Levy
- Leon Schuermann
- Tyler Potyondy
- Branden Ghena
- Alexandru Radovici
- Benjamin Prevor
- Brad Campbell
- Hudson Ayers

## Updates

### Tock Release 2.2:
- Leon: Tock 2.2 is released. 2 years between Tock 2.1 and Tock 2.2. Please let me know if there are any issues people find with this release.

### Treadmill:
- Leon: Treadmill is now a required check for merging PRs. We now have a 3rd github action that reports success and does nothing else for pushes to a PR, and performs actual tests when merging. This is a bit hacky, but is mostly working.
- Branden: Now Treadmill doesn't run until someone clicks merge?
- Leon: It never ran before merge, we cannot have it run automatically. We have a planned change to test PRs when they are opened.
- Branden: Is there a way for developers to manually run tests on branches?
- Leon: This is only possible for branches that originate from the Tock repository.
- Leon: Failures on PRs when merging are now reported and placed within the PR.
- Leon: Please let Ben or myself know if anyone experiences issues or failures with Treadmill.

### PSOC Board Support (https://github.com/tock/tock/pull/4300):
- Alex: We added an infineon board. This is the PSOC. 
- Leon: I saw this has a cortex-m0+ and a cortex-m4. Is there any reason you targeted the M0+?
- Alex: I will get back to you on this. I think it has something to do with how the code is uploaded.
- Branden: I remember PSCOC having some hardware state machine similar to the RP2040. Are you going to try to support this?
- Alex: No, not at this time. We selected this because we wanted to support an infineon chip and this was the chip infineon recommended us to work with. 

### Meeting Logistics:
- Amit: Moving forward we will switch these meetings to be Wednesdays 9:15-10am PST.
- Amit: This change will start next week Wednesday. I will send an announcement.

## Treadmill Summary
- Ben: Treadmill is a system Leon and I have been developing to automate testing and allow remote users to obtain a development environment that has a direct USB connection to a Tock supported board. 
- Ben: All tests currently written are integrated in the CI (https://github.com/tock/tock-hardware-ci/tree/main/hwci/tests).
- Ben: One of the primary challenges was designing the framework for the board tests.
- Ben: As we expand the suite of boards, the framework we have designed should be extensible to new and other board architectures.
- Ben: These tests are now automated for all PRs that are merged. 
- Ben: Tyler has written a suite of tests for networking that we will be integrating into treadmill.
- Ben: My current focus has been to improve the treadmill cli tool. I am trying to simplify the process of enqueuing jobs and selecting the hardware a given job should run on.
- Ben: Other pain points are the login process and obtaining ssh keys etc.
- Ben: In summary, our two aims are to allow CI testing and also provide an environment for people to remotely develop on hardware.
- Leon: For the release tests, I added some board definitions. I copied Ben's work to also include imix and an FPGA board. This mostly worked out of the box which was great.
- Ben: Here are some pointers to the board class implementation:
  - https://github.com/tock/tock-hardware-ci/blob/main/hwci/boards/nrf52dk.py
  - https://github.com/tock/tock-hardware-ci/blob/main/hwci/boards/tockloader_board.py
- Amit: One question, say we wanted to do a release tomorrow, what would that look like using Treadmill?
- Ben: Because Treadmill is running as PRs enter the merge queue, this will be more incremental. For a release, we would want to have a "run all tests" option that would exercise all subsystems, boards, and tests.
- Amit: Treadmill is going to do a bunch of testing. What is missing between that and what we would want to do manually for a release?
- Leon: Currently, we do not have all tests and boards supported, so this would require some manual testing. As we develop this further, hopefully the number of boards requiring manual testing would decrease.
- Leon: As we add more boards and tests, I don't think we can or should run the full suite of all tests for every PR as this would take too long.
- Leon: We need to develop an intelligent way for determining how to select which tests need to be executed for a given PR.
- Amit: In terms of coverage, how many boards is treadmill testing?
- Leon: We have a wip imix and litex boards. NRF is the only automated board.
- Leon: Many of these tests are UART and GPIO. 
- Leon: Other tests require uncommenting lines in the kernel. We haven't tackled this problem.
- Branden: One of the original goals was parallelism. If we had multiple NRF boards or boards at different universities, what is our status with this?
- Leon: We can add this using github actions to have a python script that uses some plan / config to match tests to available board resources.
- Branden: Having this would help with the testing bottle neck and also improve the stability of the system.
- Tyler: Would Treadmill's design cause headaches when deploying across universities (such as with campus IT security concerns).
- Leon: All outgoing traffic goes through a VPN. One way we can mitigate concerns from campus IT security is by only allowing Tock CI testing jobs to run. This would mean we are not giving users direct access to execute arbitrary code 
- Ben: Here is the link for the cli tool: https://github.com/treadmill-tb/treadmill/tree/main/cli.
- Brad: For the tests requiring kernel changes, this should use the configuration boards options. 
- Amit: This seems to be like the correct approach.

## TockWorld Planning 
- Alex: We have discussed having TockWorld scheduled around RustConf.
- Amit: We are still waiting for Microsoft to confirm time/dates.
- Alex: The RustConf website says the dates will be announced in January. 
- Amit: It would be nice to have the RustConf schedule so we can be careful not to overlap with the main conference.
- Amit: We should get ready to announce, blocking on confirmation on RustConf and also venue availability.

## Tock Summer School Offering
- Alex: We have a summer school that we host: https://www.ipworkshop.ro/.
- Alex: If anyone is interested to join for this, please let me know.
