// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Low level interface for Key-Value (KV) Stores
//!
//! The KV store implementation in Tock has three levels, described below.
//!
//! 1. **Hardware Level**: This level is the interface that writes a buffer to the
//!    hardware. This will generally be writing to flash, although in theory it
//!    would be possible to write to other mediums.
//!
//!    An example of the HIL used here is the Tock Flash HIL.
//!
//! 2. **KV System Level**: This level can be thought of like a file system. It
//!    is responsible for taking save/load operations and generating a buffer to
//!    pass to level 1. This level is also in charge of generating hashes and
//!    checksums.
//!
//!    This level allows generating a key hash but otherwise operates on hashed
//!    keys. This level is not responsible for permission checks.
//!
//!    This file describes the HIL for this level.
//!
//! 3. **KV Store**: This is a user friendly high level API. This API is used
//!    inside the kernel and exposed to applications to allow KV operations. The
//!    API from this level should be high level, for example set/get/delete on
//!    unhashed keys. This level is in charge of enforcing permissions.
//!
//!    This level is also in charge of generating the key hash by calling into
//!    level 2.
//!
//! The expected setup inside Tock will look like this:
//!
//! ```text
//! +-----------------------+
//! |                       |
//! |  Capsule using K-V    |
//! |                       |
//! +-----------------------+
//!
//!    capsules::kv_store
//!
//! +-----------------------+
//! |                       |
//! |  K-V in Tock          |
//! |                       |
//! +-----------------------+
//!
//!    hil::kv_system (this file)
//!
//! +-----------------------+
//! |                       |
//! |  K-V library          |
//! |                       |
//! +-----------------------+
//!
//!    hil::flash
//! ```
