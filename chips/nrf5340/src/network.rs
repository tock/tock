// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Starting the network core from the application core.

use kernel::utilities::StaticRef;
use kernel::utilities::registers::interfaces::{ReadWriteable, Writeable};
use kernel::utilities::registers::{ReadWrite, register_bitfields, register_structs};

use crate::gpio::Pin;

register_structs! {
    pub ResetRegisters {
        (0x000 => _reserved0),
        (0x614 => network_forceoff: ReadWrite<u32, ForceOff::Register>),
        (0x618 => @END),
    }
}

register_structs! {
    pub SpuRegisters {
        (0x000 => _reserved0),
        (0x700 => ramregion_perm: [ReadWrite<u32, Perm::Register>; 64]),
        (0x800 => @END),
    }
}

register_structs! {
    /// The PIN_CNF registers of one GPIO port, at the same base address as
    /// [`GpioRegisters`](crate::gpio::GpioRegisters).
    pub PinCnfRegisters {
        (0x000 => _reserved0),
        (0x700 => pin_cnf: [ReadWrite<u32, PinCnf::Register>; 32]),
        (0x780 => @END),
    }
}

register_bitfields! [u32,
    ForceOff [
        FORCEOFF OFFSET(0) NUMBITS(1) [
            Release = 0,
            Hold = 1
        ]
    ],
    PinCnf [
        MCUSEL OFFSET(28) NUMBITS(3) [
            AppMcu = 0,
            NetworkMcu = 1
        ]
    ],
    Perm [
        SECATTR OFFSET(4) NUMBITS(1) [
            NonSecure = 0,
            Secure = 1
        ]
    ]
];

pub struct NetworkCore {
    reset: StaticRef<ResetRegisters>,
    spu: StaticRef<SpuRegisters>,
    ports: [StaticRef<PinCnfRegisters>; 2],
}

impl NetworkCore {
    pub const fn new(
        reset: StaticRef<ResetRegisters>,
        spu: StaticRef<SpuRegisters>,
        ports: [StaticRef<PinCnfRegisters>; 2],
    ) -> Self {
        Self { reset, spu, ports }
    }

    /// Let the network core access the 8 KB region of application RAM that
    /// holds `address`, by making it non-secure. The network core is a
    /// non-secure bus master. Keep kernel and process memory out of that
    /// region.
    pub fn share_ram(&self, address: usize) {
        let region = (address - 0x2000_0000) / 0x2000;
        self.spu.ramregion_perm[region].modify(Perm::SECATTR::NonSecure);
    }

    /// Hand `pins` to the network core, then release it from reset.
    ///
    /// The network core runs whatever is in its flash. If that is empty, it
    /// faults, which does not affect the application core.
    pub fn start(&self, pins: &[Pin]) {
        for &pin in pins {
            self.ports[pin as usize / 32].pin_cnf[pin as usize % 32]
                .modify(PinCnf::MCUSEL::NetworkMcu);
        }
        self.reset
            .network_forceoff
            .write(ForceOff::FORCEOFF::Release);
    }
}
