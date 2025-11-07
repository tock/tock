// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2025.

//! Definition of common operations that are independent of
//! the bus implementation.

use super::{RegAddr, RegLen};

#[derive(Clone, Copy)]
pub(crate) enum BackplaneTask {
    // TODO(?): Maybe move the function pointer to an enum
    Read(RegLen, RegAddr, Option<fn(u32, &mut u8) -> ()>),
    Write(RegLen, RegAddr, u32),
    WaitMs(u32),
}

impl BackplaneTask {
    pub(crate) const fn write(addr: RegAddr, val: u32, len: RegLen) -> Self {
        BackplaneTask::Write(len, addr, val)
    }

    // `jmp` is a function that should return the value of the next index from the
    // operations list based on the current operation's result.
    pub(crate) const fn read(addr: RegAddr, len: RegLen, jmp: Option<fn(u32, &mut u8)>) -> Self {
        BackplaneTask::Read(len, addr, jmp)
    }
}

/// Common `eq` function that can be used as `jmp` function for the `ReadFailable` register
/// operation.
pub(crate) fn eq<const VALUE: u32, const IDX_OK: u8, const IDX_FAIL: u8>(val: u32, idx: &mut u8) {
    if val == VALUE {
        *idx += IDX_OK
    } else {
        *idx += IDX_FAIL
    }
}

/// Common `mask` function that can be used as `jmp` function for the `ReadFailable` register
/// operation.
pub(crate) fn mask<const MASK: u32, const IDX_EQ_0: u8, const IDX_NEQ_0: u8>(
    val: u32,
    idx: &mut u8,
) {
    if (val & MASK) == 0 {
        *idx += IDX_EQ_0
    } else {
        *idx += IDX_NEQ_0
    }
}

/// Disable a core
pub(crate) mod core_disable {
    use super::{mask, BackplaneTask as Task};
    use crate::{bus, utils};
    use bus::RegLen::Byte;

    #[inline]
    pub(crate) const fn ops<const BASE_ADDR: u32>() -> [Task; 7] {
        [
            Task::read(BASE_ADDR + utils::AI_RESETCTRL_OFFSET, Byte, None),
            Task::read(
                BASE_ADDR + utils::AI_RESETCTRL_OFFSET,
                Byte,
                Some(mask::<{ utils::AI_RESETCTRL_BIT_RESET }, 1, 6>),
            ),
            Task::write(BASE_ADDR + utils::AI_IOCTRL_OFFSET, 0, Byte),
            Task::read(BASE_ADDR + utils::AI_IOCTRL_OFFSET, Byte, None),
            Task::WaitMs(1),
            Task::write(
                BASE_ADDR + utils::AI_RESETCTRL_OFFSET,
                utils::AI_RESETCTRL_BIT_RESET,
                Byte,
            ),
            Task::read(BASE_ADDR + utils::AI_RESETCTRL_OFFSET, Byte, None),
        ]
    }
}

/// Reset a core
pub(crate) mod core_reset {
    use super::BackplaneTask as Task;
    use crate::{bus, utils};
    use bus::RegLen::Byte;

    #[inline]
    pub(crate) const fn ops<const BASE_ADDR: u32>() -> [Task; 7] {
        [
            Task::write(
                BASE_ADDR + utils::AI_IOCTRL_OFFSET,
                utils::AI_IOCTRL_BIT_FGC | utils::AI_IOCTRL_BIT_CLOCK_EN,
                Byte,
            ),
            Task::read(BASE_ADDR + utils::AI_IOCTRL_OFFSET, Byte, None),
            Task::write(BASE_ADDR + utils::AI_RESETCTRL_OFFSET, 0, Byte),
            Task::WaitMs(1),
            Task::write(
                BASE_ADDR + utils::AI_IOCTRL_OFFSET,
                utils::AI_IOCTRL_BIT_CLOCK_EN,
                Byte,
            ),
            Task::read(BASE_ADDR + utils::AI_IOCTRL_OFFSET, Byte, None),
            Task::WaitMs(1),
        ]
    }
}
