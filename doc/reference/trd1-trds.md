Tock Reference Document (TRD) Structure and Keywords
========================================

**TRD:** 1<br/>
**Working Group:** Kernel<br/>
**Type:** Best Common Practice<br/>
**Status:** Final<br/>
**Authors:** Philip Levis, Daniel Griffin<br/>

Abstract
-------------------------------

This document describes the structure followed by all Tock Reference
Documents (TRDs), and defines the meaning of several key words in
those documents.

1 Introduction
====================================================================

To simplify management, reading, and tracking development,
all Tock Reference Documents (TRDs) MUST have a particular
structure. Additionally, to simplify development and improve
implementation interoperability, all TRDs MUST observe the meaning of
several key words that specify levels of compliance. This document
describes and follows both.

2 Keywords
====================================================================

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT",
"SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this
document are to be interpreted as described in [TRD1].

Note that the force of these words is modified by the requirement
level of the document in which they are used. These words hold their
special meanings only when capitalized, and documents SHOULD avoid using
these words uncapitalized in order to minimize confusion.


2.1 MUST
--------------------------------------------------------------------

MUST: This word, or the terms "REQUIRED" or "SHALL", mean that the
definition is an absolute requirement of the document.

2.2 MUST NOT
--------------------------------------------------------------------

MUST NOT: This phrase, or the phrase "SHALL NOT", mean that the
definition is an absolute prohibition of the document.

2.3 SHOULD
--------------------------------------------------------------------

SHOULD: This word, or the adjective "RECOMMENDED", mean that there
may exist valid reasons in particular circumstances to ignore a
particular item, but the full implications must be understood and
carefully weighed before choosing a different course.

2.4 SHOULD NOT
--------------------------------------------------------------------

SHOULD NOT: This phrase, or the phrase "NOT RECOMMENDED" mean that
there may exist valid reasons in particular circumstances when the
particular behavior is acceptable or even useful, but the full
implications should be understood and the case carefully weighed
before implementing any behavior described with this label.

2.5 MAY
--------------------------------------------------------------------

MAY: This word, or the adjective "OPTIONAL", mean that an item is
truly optional.  One implementer may choose to include the item
because a particular application requires it or because the
implementer feels that it enhances the system while another
implementer may omit the same item.  An implementation which does not
include a particular option MUST be prepared to interoperate with
another implementation which does include the option, though perhaps
with reduced functionality. Similarly, an implementation which does
include a particular option MUST be prepared to interoperate with
another implementation which does not include the option (except, of
course, for the feature the option provides.)

2.6 Guidance in the use of these Imperatives
--------------------------------------------------------------------

Imperatives of the type defined in this memo must be used with care
and sparingly.  In particular, they MUST only be used where it is
actually required for interoperation or to limit behavior which has
potential for causing harm (e.g., limiting retransmissions)  For
example, they must not be used to try to impose a particular method
on implementors where the method is not required for
interoperability.


3 TRD Structure
====================================================================

A TRD MUST begin with a title, and then follow with a header and a
body. The header states document metadata, for management and status.
The body contains the content of the proposal.

All TRDs MUST conform to [Markdown syntax][markdown] to enable
translation to HTML and LaTeX, and for useful display in web tools.

3.1 TRD Header
--------------------------------------------------------------------

The TRD header has several fields which MUST be included, as well as
others which MAY be included. The TRD header MUST NOT include fields
which are not specified in TRD 1 or supplementary Best Common Practice
TRDs. The first five header fields MUST be included in all TRDs, in
the order stated below.  The Markdown syntax to use when composing a
header is modeled by this document's header.

The first field is "TRD," and specifies the TRD number of the
document. A TRD's number is unique. This document is TRD 1. The
TRD type (discussed below) determines TRD number assignment. Generally,
when a document is ready to be a TRD, it is assigned the smallest
available number. BCP TRDs start at 1 and all other TRDs
(Documentary, Experimental, and Informational) start at 101.

The second field, "Working Group," states the name of the working
group that produced the document. This document was produced by the
Kernel Working Group.

The third field is "Type," and specifies the type of TRD the document
is. There are four types of TRD: Best Current Practice (BCP),
Documentary, Informational, and Experimental. This document's type is Best
Current Practice.

*Best Current Practice* is the closest thing TRDs have to a standard: it
represents conclusions from significant experience and work by its
authors. Developers desiring to add code (or TRDs) to Tock SHOULD
follow all current BCPs. 

*Documentary* TRDs describe a system or protocol that exists; a
documentary TRD MUST reference an implementation that a reader can
easily obtain.  Documentary TRDs simplify interoperability when 
needed, and document Tock implementations.

*Informational* TRDs provide information that is of interest to the
community. Informational TRDs include data gathered on radio behavior,
hardware characteristics, other aspects of Tock software/hardware,
organizational and logistic information,
or experiences which could help the community achieve its goals.  

*Experimental* TRDs describe a completely experimental approach to a
problem, which are outside the Tock release stream and will not
necessarily become part of it.  Unlike Documentary TRDs, Experimental
TRDs may describe systems that do not have a reference implementation.

The fourth field is "Status," which specifies the status of the TRD.
A TRD's status can be either "Draft," which means it is a work in
progress, or "Final," which means it is complete and will not change.
Once a TRD has the status "Final," the only change allowed is the
addition of an "Obsoleted By" field.

The "Obsoletes" field is a backward pointer to an earlier TRD which
the current TRD renders obsolete. An Obsoletes field MAY have multiple
TRDs listed.  For example, if TRD 121 were to replace TRDs 111 and
116, it would have the field "Obsoletes: 111, 116".

The "Obsoleted By" field is added to a Final TRD when another TRD has
rendered it obsolete. The field contains the number of the obsoleting
TRD. For example, if TRD 111 were obsoleted by TRD 121, it would have
the field "Obsoleted By: 121".

"Obsoletes" and "Obsoleted By" fields MUST agree. For a TRD to list another
TRD in its Obsoletes field, then that TRD MUST list it in the Obsoleted By
field.

The obsoletion fields are used to keep track of evolutions and modifications
of a single abstraction. They are not intended to force a single approach or
mechanism over alternative possibilities.

The final required field is "Authors," which states the names of the
authors of the document. Full contact information should not be listed
here (see Section 3.2).

There is an optional field, "Extends." The "Extends" field refers to
another TRD. The purpose of this field is to denote when a TRD represents
an addition to an existing TRD. Meeting the requirements of a TRD with an
Extends field requires also meeting the requirements of all TRDs listed 
in the Extends field.

If a TRD is a Draft, then four additional fields MUST be included:
Draft-Created, Draft-Modified, Draft-Version, and Draft-Discuss.
Draft-Created states the date the document was created, Draft-Modified
states when it was last modified. Draft-Version specifies the version
of the draft, which MUST increase every time a modification is
made. Draft-Discuss specifies the email address of a mailing list
where the draft is being discussed. Final and Obsolete TRDs MUST NOT
have these fields, which are for Drafts only.

3.2 TRD Body
--------------------------------------------------------------------

A TRD body SHOULD begin with an Abstract, which gives a brief overview
of the content of the TRD. A longer TRD MAY, after the Abstract, have
a Table of Contents. After the Abstract and Table of Contents there
SHOULD be an Introduction, stating the problem the TRD seeks to solve
and providing needed background information.

If a TRD is Documentary, it MUST have a section entitled
"Implementation," which instructs the reader how to obtain the
implementation documented.

If a TRD is Best Current Practice, it MUST have a section entitled
"Reference," which points the reader to one or more reference uses of
the practices.

The last three sections of a TRD are author information, citations,
and appendices. A TRD MUST have an author information section titled
entitled "Author's Address" or "Authors' Addresses." A TRD MAY have
a citation section entitled "Citations." A citations section MUST
immediately follow the author information section. A TRD MAY have
appendices. Appendices MUST immediately follow the citations section,
or if there is no citations section, the author information section.
Appendices are lettered.  Please refer to Appendix A for details.

4 File names
====================================================================

TRDs MUST be stored in the Tock repository with a file name of

trd[number]-[desc].md

Where number is the TRD number and desc is a short, one word description.
The name of this document is trd1-trds.md.

5 Reference
====================================================================
The reference use of this document is TRD 1 (itself).

6 Acknowledgments
====================================================================

The definitions of the compliance terms are a direct copy of
definitions taken from IETF RFC 2119. This document is heavily copied
from TinyOS Enhancement Proposal 1 (TEP 1).

7 Author's Address
====================================================================

    Philip Levis
    409 Gates Hall
    Stanford University
    Stanford, CA 94305

    phone - +1 650 725 9046

    email - pal@cs.stanford.edu

Appendix A Example Appendix
====================================================================

This is an example appendix. Appendices begin with the letter A.

[TRD1]: trd1-trds.md "Tock Reference Document (TRD) Structure and Keywords"

[markdown]: https://daringfireball.net/projects/markdown/ "Markdown"

