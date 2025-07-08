AES GCM Capsule
===============

This crate contains software support for
[AES-GCM](https://en.wikipedia.org/wiki/AES-GCM-SIV).
This capsule doesn't perform any AES operations, instead it relies on
existing implmenetations to perform the AES operations and instead manages
the operations and hashing to support GCM.

This capsule uses the extenal
[ghash crate](https://github.com/RustCrypto/universal-hashes/tree/master/ghash)
as part of Rust-crypto to implement AES GCM on top of existing AES
implementions.

## Cargo tree

```
capsules-aes-gcm v0.1.0 (/var/mnt/scratch/alistair/software/tock/tock/capsules/aes_gcm)
├── enum_primitive v0.1.0 (/var/mnt/scratch/alistair/software/tock/tock/libraries/enum_primitive)
├── ghash v0.4.4
│   ├── opaque-debug v0.3.0
│   └── polyval v0.5.3
│       ├── cfg-if v1.0.0
│       ├── cpufeatures v0.2.7
│       ├── opaque-debug v0.3.0
│       └── universal-hash v0.4.1
│           ├── generic-array v0.14.7
│           │   └── typenum v1.16.0
│           │   [build-dependencies]
│           │   └── version_check v0.9.4
│           └── subtle v2.4.1
├── kernel v0.1.0 (/var/mnt/scratch/alistair/software/tock/tock/kernel)
│   ├── tock-cells v0.1.0 (/var/mnt/scratch/alistair/software/tock/tock/libraries/tock-cells)
│   ├── tock-registers v0.8.1 (/var/mnt/scratch/alistair/software/tock/tock/libraries/tock-register-interface)
│   └── tock-tbf v0.1.0 (/var/mnt/scratch/alistair/software/tock/tock/libraries/tock-tbf)
└── tickv v1.0.0 (/var/mnt/scratch/alistair/software/tock/tock/libraries/tickv)
```