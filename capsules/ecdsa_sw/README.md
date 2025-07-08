ECDSA Software Implementation
=============================

This crate provides a software-based implementation of ECDSA algorithms using
the RustCrypto crates.

Supported Operations
--------------------

- Signature Verification
  - P256 (secp256r1)

Dependency Tree
---------------

```
ecdsa-sw v0.2.3-dev (/Users/bradjc/git/tock/capsules/ecdsa_sw)
├── kernel v0.2.3-dev (/Users/bradjc/git/tock/kernel)
│   ├── tock-cells v0.1.0 (/Users/bradjc/git/tock/libraries/tock-cells)
│   ├── tock-registers v0.9.0 (/Users/bradjc/git/tock/libraries/tock-register-interface)
│   └── tock-tbf v0.1.0 (/Users/bradjc/git/tock/libraries/tock-tbf)
└── p256 v0.13.2
    ├── ecdsa v0.16.9
    │   ├── der v0.7.9
    │   │   ├── const-oid v0.9.6
    │   │   └── zeroize v1.8.1
    │   ├── digest v0.10.7
    │   │   ├── block-buffer v0.10.4
    │   │   │   └── generic-array v0.14.7
    │   │   │       ├── typenum v1.17.0
    │   │   │       └── zeroize v1.8.1
    │   │   │       [build-dependencies]
    │   │   │       └── version_check v0.9.5
    │   │   ├── const-oid v0.9.6
    │   │   ├── crypto-common v0.1.6
    │   │   │   ├── generic-array v0.14.7 (*)
    │   │   │   └── typenum v1.17.0
    │   │   └── subtle v2.4.1
    │   ├── elliptic-curve v0.13.8
    │   │   ├── base16ct v0.2.0
    │   │   ├── crypto-bigint v0.5.5
    │   │   │   ├── generic-array v0.14.7 (*)
    │   │   │   ├── rand_core v0.6.4
    │   │   │   ├── subtle v2.4.1
    │   │   │   └── zeroize v1.8.1
    │   │   ├── digest v0.10.7 (*)
    │   │   ├── ff v0.13.1
    │   │   │   ├── rand_core v0.6.4
    │   │   │   └── subtle v2.4.1
    │   │   ├── generic-array v0.14.7 (*)
    │   │   ├── group v0.13.0
    │   │   │   ├── ff v0.13.1 (*)
    │   │   │   ├── rand_core v0.6.4
    │   │   │   └── subtle v2.4.1
    │   │   ├── rand_core v0.6.4
    │   │   ├── sec1 v0.7.3
    │   │   │   ├── base16ct v0.2.0
    │   │   │   ├── der v0.7.9 (*)
    │   │   │   ├── generic-array v0.14.7 (*)
    │   │   │   ├── subtle v2.4.1
    │   │   │   └── zeroize v1.8.1
    │   │   ├── subtle v2.4.1
    │   │   └── zeroize v1.8.1
    │   ├── rfc6979 v0.4.0
    │   │   ├── hmac v0.12.1
    │   │   │   └── digest v0.10.7 (*)
    │   │   └── subtle v2.4.1
    │   └── signature v2.2.0
    │       ├── digest v0.10.7 (*)
    │       └── rand_core v0.6.4
    ├── elliptic-curve v0.13.8 (*)
    ├── primeorder v0.13.6
    │   └── elliptic-curve v0.13.8 (*)
    └── sha2 v0.10.8
        ├── cfg-if v1.0.0
        ├── cpufeatures v0.2.16
        └── digest v0.10.7 (*)
```
