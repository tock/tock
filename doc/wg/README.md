Tock Working Groups
===================

Working groups are focused groups to organize development around a
particular aspect of Tock.

## Motivation

Tock relies on contributions from developers with across a variety of
organizations, physical distance, timezones, and engagement. It has
also grown large in scope. This has resulted in strain on maintainers
to regularly keep up with contributions while ensuring quality and
important global policies like safety and security. It has also
resulted in contributors' work going unaddressed for long periods of
time.

Tock encompases a large and varied set of subsystems, architectures,
focus areas, libraries, ancillary tools, and documentation. Most
contributors have expertise and stake in a subset of these. Moreover,
it is impractical for any maintainer to keep up with the discussions
and direction of each part of the project. Finally, different parts of
the project should be able to move at different paces, with different
levels of scrutiny. For example, a soundness bug or performance
regression in the kernel crate can catestrophically impact all users
and should be avoided if at all possible, while a sub-optimal design
decision in an experimental user-space library is not a big deal.

To facilitate this, working groups take on responsibility for specific
sub-areas of the project. Members of a working group become experts in
that sub-area and are best able to determine appropriate scrutiny for
accepting contributions, frequency and mode of design discussions,
etc.

## Structure And Responsibilities

Tock development organizes around a core working group as well as
additional area-specific working groups. The core working group
oversees the project wholistically, defining high-level design goals
and project direction, establishing working groups, and facilitating
work that spans multiple working groups. Other working groups
facilitate contributions to specific sub-areas of the project, with
devolved decision making resposibility for accepting contributions,
design, and direction.

While working groups *oversee* development, working group members are
not expected to be the primary source of contributions. Instead,
working groups establish code review standards, define and communicate
specific design direction for their purview, and ensure relevant
contributors are both supported and effective.

### Working Group Organizational Guidelines

Each working group has a Lead who assembles the working group
membership and is responsible for its operation.

Each working group should include at least one member from the Core
Working Group to ensure that the Core Working Group is regularly
updated on the activities and motivations of the working group. The
Core Working Group member need not be the working group Lead.

Each working group, including the Core Working Group, establishes its
own rules and procedures for accepting contributions, communicating,
meeting, and making decisions. In absense of such rules, working
groups make decisions by consensus, have weekly voice calls, make
meeting notes available publicly, and communicate asynchronously via a
mailing list.

However, working groups are encouraged to avoid consensus-based
decision making as quickly as possible and establish appropriate
meeting frequency and communication mode for their needs and
membership.

## Core Working Group

The Core Working Group shepherds and oversees the Tock OS and related
tools and libraries. Importantly, it serves as a backstop for managing
contributions that fall outside the purview of existing working
groups. It also establishes new working groups to handle such
contributions, and dissolves or re-organizes existing working groups.

Formally, the Core Working Group controls who can directly commit to
all Tock project repositories and devolves the ability to commit to
specific repositories or components of repositories to other working
groups. The Core Working Group, like other working groups, establishes
its own rules for deciding how to accept contributions as well as how
to establish and disband working groups.

The Core Working Group's duties are:

- Managing and overseeing code, documentation, testing, and releases
  for the Tock project.
  - Defining and communicating overall project goals and direction.
  - Establishing and delegating responsility over components and
    sub-projects to working groups.
  - Ensuring that working groups have the people and resources needed
    to accomplish their work.
  - Ensuring working groups are accountable to their delegated
    responsibilities and the project as a whole.
  - Coordinating decisions including (but not limited to) code,
    documentation, testing, and releases that affect purviews
    delegated to more than one working group.
  - Facilitating communication channels and consensus among working
    groups.
  - Coordinating project-wide changes to teams, structures, or
    processes.

Existing Working Groups
-----------------------

- [Core](core/README.md)
- [OpenTitan](opentitan/README.md)
- [Documentation](documentation/README.md)


Retired Working Groups
----------------------

- [Legacy](legacy/README.md)
