Tock Tools
==========

Various scripts, testing infrastructure, and helpers related to Tock.

 - **`build/`**
   Support infrastructure for building Tock. These are things that are used
   universally, i.e., both during local development work and invoked during CI
   or other builds.
 - **`ci/`**
   Support infrastructure for Tock CI. These are things that are generally
   only invoked as part of fully automated build or testing procedures.
 - **`debugging-and-development/`**
   These are tools primarily designed for interactive work sessions. These
   include general tools such as memory-use analyzers as well as tools for
   specific use cases, such as mock'ing portions of a USB interface.
 - **`repo-maintenance/`**
   One-off tools that are CI-like in their nature, but are not (yet) automated.
   These are periodically run by project maintainers.
