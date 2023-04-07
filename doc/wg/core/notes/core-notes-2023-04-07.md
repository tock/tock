# Tock Core Notes 2023-04-07

Attendees:
 - Hudson Ayers
 - Alyssa Haroldsen
 - Johnathan Van Why
 - Pat Pannuto
 - Alexandru 
 - Branden Ghena
 - Vlad Radulescu
 - Philip Levis
 - Vadim Sukhomlinov

## Updates

- Phil: Ordered console will be finished today, then it's ready for review.

- Hudson: we have some things from Johnathan, plus the ProcessConsole state
machine.

- Phil: I'll take a look on ProcessConsole.

- Alexandru: I will too.

## Agenda Items

- Hudson: first thing is TockWorld survey results. Amit needs to give us
access.

- Phil: I'll ping him.

- Hudson: OK, let's talk about the license issues.

- Johnathan: This is 3419 and 3417.

- Johnathan: THe only documentation I have currently is just a comment, it
shows up if you run RustDoc. This is stuff like how you configure the license
tool. I am going to improve it, and I would like to hear opinions on whether
it should be somewhere else. I suspect that people will just try to open
up src/main. (3419)

- Hudson: I think keeping documentation as is is fine, but it would be
good if --help would point them to it. So if someone types --help they
know where to look.

- Johnathan: Next thing is file format support. Some file formats do not
support line comments. XML and linker scripts. They are in the issue.
This is on me, I didn't realize this. They require
block comments. So I will fix this. THe other type is JLink, they have DOS
line endings, and it seems accidental that they have them, it's a mix.

- Branden: Seems fine to get rid of non-UNIX line endings. 

- Hudson: Presumably, adding a file with a DOS line ending will fail.

- Johnathan: Correct.

- Alyssa: Definitely add a check.

- Hudson: Anything else?

- Alyssa: There's an Eclipse coming up!

- Phil: In Indonesia, unfortunately.





