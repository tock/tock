RSA Software Implementation
===========================

This crate provides a software-based implementation of RSA algorithms using
the RustCrypto RSA crate.

Dependency Tree
---------------

```
rsa-sw v0.1.0 (/Users/bradjc/git/tock/capsules/rsa_sw)
├── kernel v0.1.0 (/Users/bradjc/git/tock/kernel)
│   ├── tock-cells v0.1.0 (/Users/bradjc/git/tock/libraries/tock-cells)
│   ├── tock-registers v0.8.1 (/Users/bradjc/git/tock/libraries/tock-register-interface)
│   └── tock-tbf v0.1.0 (/Users/bradjc/git/tock/libraries/tock-tbf)
└── rsa v0.9.2
    ├── byteorder v1.4.3
    ├── const-oid v0.9.2
    ├── digest v0.10.7
    │   ├── block-buffer v0.10.4
    │   │   └── generic-array v0.14.7
    │   │       └── typenum v1.16.0
    │   │       [build-dependencies]
    │   │       └── version_check v0.9.4
    │   ├── const-oid v0.9.2
    │   └── crypto-common v0.1.6
    │       ├── generic-array v0.14.7 (*)
    │       └── typenum v1.16.0
    ├── num-bigint-dig v0.8.2
    │   ├── byteorder v1.4.3
    │   ├── lazy_static v1.4.0
    │   │   └── spin v0.5.2
    │   ├── libm v0.2.7
    │   ├── num-integer v0.1.45
    │   │   └── num-traits v0.2.15
    │   │       └── libm v0.2.7
    │   │       [build-dependencies]
    │   │       └── autocfg v1.1.0
    │   │   [build-dependencies]
    │   │   └── autocfg v1.1.0
    │   ├── num-iter v0.1.43
    │   │   ├── num-integer v0.1.45 (*)
    │   │   └── num-traits v0.2.15 (*)
    │   │   [build-dependencies]
    │   │   └── autocfg v1.1.0
    │   ├── num-traits v0.2.15 (*)
    │   ├── rand v0.8.5
    │   │   ├── rand_chacha v0.3.1
    │   │   │   ├── ppv-lite86 v0.2.17
    │   │   │   └── rand_core v0.6.4
    │   │   └── rand_core v0.6.4
    │   ├── smallvec v1.10.0
    │   └── zeroize v1.6.0
    ├── num-integer v0.1.45 (*)
    ├── num-iter v0.1.43 (*)
    ├── num-traits v0.2.15 (*)
    ├── pkcs1 v0.7.5
    │   ├── der v0.7.6
    │   │   ├── const-oid v0.9.2
    │   │   └── zeroize v1.6.0
    │   ├── pkcs8 v0.10.2
    │   │   ├── der v0.7.6 (*)
    │   │   └── spki v0.7.2
    │   │       └── der v0.7.6 (*)
    │   └── spki v0.7.2 (*)
    ├── pkcs8 v0.10.2 (*)
    ├── rand_core v0.6.4
    ├── sha2 v0.10.6
    │   ├── cfg-if v1.0.0
    │   ├── cpufeatures v0.2.7
    │   └── digest v0.10.7 (*)
    ├── signature v2.1.0
    │   ├── digest v0.10.7 (*)
    │   └── rand_core v0.6.4
    ├── spki v0.7.2 (*)
    ├── subtle v2.5.0
    └── zeroize v1.6.0
```
