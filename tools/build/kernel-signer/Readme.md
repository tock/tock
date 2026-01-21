Signer for Kernel Images
========================

Tool for signing a kernel image using ECDSA P256 key and SHA 256 hash.

The user does not have to do anything. Generally hooked to the Makefile
of the board, this script should sign the kernel when it is compiled.