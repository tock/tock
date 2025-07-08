Tock Documentation
==================

General kernel documentation is in the [Tock Book](https://book.tockos.org/doc).
Information about Tock policies and development practices is here. This folder
also contains documentation on [syscall interfaces](reference).

For short tutorials and longer courses on how to use Tock, see the [Tock OS
Book](https://book.tockos.org).

Tock Policies
-------------

### Interface Details
- **[Syscall Interfaces](syscalls)** - API between userland and the kernel.
- **[Internal Kernel Interfaces](reference)** - Hardware Interface Layers (HILs) for kernel components.

### Tock Setup and Development
- **[Getting Started](Getting_Started.md)** - Installing the Tock toolchain and programming hardware.
- **[Repository Structure](Repository.md)** - How the tock/ repo is organized.
- **[Nested Boards](NestedBoards.md)** - How Tock supports nesting board platforms.
- **[Out of Tree Boards](OutOfTree.md)** - Best practices for maintaining boards not in Tock master.
- **[Style](Style.md)** - Stylistic aspects of Tock code.
- **[External Dependencies](ExternalDependencies.md)** - Policy for including external dependencies.

### Management of Tock
- **[Working Groups](wg)** - Development groups for specific aspects of Tock.
- **[Code Review Process](CodeReview.md)** - Process for pull request reviews.
- **[Tock Management](Maintenance.md)** - Management processes for Tock, including releases.
- **[Security Protocol](SecurityProtocol.md)** - Procedures for security vulnerability reporting, response, and disclosure.
