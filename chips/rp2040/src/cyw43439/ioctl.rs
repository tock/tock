// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2025.

pub(super) const CUR_ETHERADDR_IOCTL: &core::ffi::CStr = c"cur_etheraddr";

#[derive(Clone, Copy, Debug)]
pub(super) enum Ioctl {
    Get = 0,
    Set = 2,
}

#[derive(Clone, Copy, Debug)]
pub(super) enum IoctlCommand {
    Up = 2,
    Down = 3,
    SetInfra = 20,
    SetAuth = 22,
    SetSsid = 26,
    SetChannel = 30,
    Disassoc = 52,
    SetAntdiv = 64,
    SetGmode = 110,
    SetAp = 118,
    SetWsec = 134,
    SetBand = 142,
    SetWpaAuth = 165,
    GetVar = 262,
    SetVar = 263,
    SetWsecPmk = 268,
}
