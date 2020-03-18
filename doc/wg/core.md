Tock Core Working Group (core)
==============================

- Working Group Charter
- Adopted 3/31/2020

## Goals

The core team manages and guides the development of Tock. Its responsibilities
are to:

- Design, implement, and maintain the Tock kernel and its interfaces, including
  system calls, hardware interface layers, and internal kernel APIs,
- Decide when to update the Rust compiler toolchain that Tock uses,
- Decide on the formation, scope, and lifetime of other Tock working groups,
- Decide on, articulate, and promote core principles of Tock kernel software and
  its development,
- Support any code in the Tock repository that is not supported by another
  working group.

## Members

- Niklas Adolfsson, [niklasad1](https://github.com/niklasad1), Parity Tech
- Hudson Ayers, [hudson-ayers](https://github.com/hudson-ayers), Stanford
- Brad Campbell, [bradjc](https://github.com/bradjc), UVA
- Branden Ghena, [brghena](https://github.com/brghena), UC Berkeley
- Philip Levis, [phil-levis](https://github.com/phil-levis), Stanford
- Amit Levy (chair), [alevy](https://github.com/alevy), Princeton University
- Pat Pannuto, [ppannuto](https://github.com/ppannuto), UCSD
- Johnathan Van Why, [jrvanwhy](https://github.com/jrvanwhy), Google

## Membership

The core working group membership is a subset of the people who have commit
(pull request merge) permissions on the Tock repository. It is intended to be a
smaller group that represents the major perspectives and issues, rather than a
complete group. Contributors who are actively help develop Tock will be
considered to join the core team. Generally, a core team member:

- Understands the core design principles of Tock and is capable of judging the
  effect new code contributions will have on Tock's adherence to those
  principles.
- Understands the code style and structure of Tock and can help ensure a
  reasonably consistent code base.
- Understands Tock's various stakeholders and can judge how a change to Tock
  might affect various users of Tock.

A core team member is expected to:

-Help review a percentage of new pull requests to the Tock code base. -Provide
opinions and input on substantial design decisions or major changes to Tock.
-Help test Tock prior to releases.

To join the core team, a contributor must be nominated by an existing core team
member. The nominator will open a pull request updating this document with the
new core team member in the list above. That pull request will undergo the usual
pull request review, and the member will be added to the core team if the pull
request is merged.

## Communication

The group has a weekly teleconference call. All working group members are
invited to   participate in the call. Other people may be invited to participate
to help contribute to particular topics or on-going discussions. The working
group chair decides who beyond the working group members may participate in the
call.

The working group publishes detailed notes of its calls. These will be posted
within a week of a call. This delay is to give participants an opportunity to
correct any errors or better explain points that came up. They are intended to
be a communication mechanism of the group, its discussions, the technical
issues, and decisions, not a literal transcription of what is said.

## Code Purview

The core working group is in charge of (responsible for reviewing, approving,
and merging pull requests for) all code directories that are not under the
purview of another working group. The following directories are expected to
remain under the sole purview of the core working group:

- `capsules`, although other working groups may have subdirectories
- `kernel`
- `libraries`
