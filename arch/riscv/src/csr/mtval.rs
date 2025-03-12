// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use kernel::utilities::registers::register_bitfields;

// mtval contains the address of an exception
// On CHERI, in the event of CHERI exceptions, it has a different format
register_bitfields![usize,
    pub mtval [
        exception_addr OFFSET(0) NUMBITS(crate::XLEN) [],
        cause OFFSET(0) NUMBITS(5) [
            NONE                    = 0x00,
            LENGTH                  = 0x01,
            TAG                     = 0x02,
            SEAL                    = 0x03,
            TYPE                    = 0x04,
            PERM_SOFT               = 0x08,
            REPRESENT               = 0x0a,
            UNALIGNED               = 0x0b,
            GLOBAL                  = 0x10,
            PERM_EXECUTE            = 0x11,
            PERM_LOAD               = 0x12,
            PERM_STORE              = 0x13,
            PERM_LOAD_CAP           = 0x14,
            PERM_STORE_CAP          = 0x15,
            PERM_STORE_LOCAL_CAP    = 0x16,
            PERM_SEAL               = 0x17,
            PERM_ASR                = 0x18,
            PERM_CINVOKE            = 0x19,
            PERM_CINVOKE_IDC        = 0x1a,
            PERM_UNSEAL             = 0x1b,
            PERM_SET_CID            = 0x1c,
        ],
        cap_idx OFFSET(5) NUMBITS(6) [
            // All the bit patterns from 0-31 are the GPRs as you would expect.
            // The CSRs are as follows:
            PCC       = 0b100000,
            DDC       = 0b100001,

            UTCC      = 0b100100,
            UTDC      = 0b100101,
            UScratchC = 0b100110,
            UEPCC     = 0b100111,

            STCC      = 0b101100,
            STDC      = 0b101101,
            SScratchC = 0b101110,
            SEPCC     = 0b101111,

            MTCC      = 0b111100,
            MTDC      = 0b111101,
            MScratchC = 0b111110,
            MEPCC     = 0b111111,
        ],
        // Top bit indicates CSR vs GPR
        cap_idx_type OFFSET(10) NUMBITS(1) [
            GPR = 0,
            CSR = 1,
        ],
    ]
];
