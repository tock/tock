//! ARM System Control Block
//!
//! <http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.dui0553a/CIHFDJCA.html>

use kernel::common::registers::{register_bitfields, register_structs, ReadOnly, ReadWrite};
use kernel::common::StaticRef;

register_structs! {
    /// In an ARMv7-M processor, a System Control Block (SCB) in the SCS
    /// provides key status information and control features for the processor.
    ScbRegisters {
        /// CPUID Base Register
        (0x00 => cpuid: ReadOnly<u32, CpuId::Register>),

        /// Interrupt Control and State Register
        (0x04 => icsr: ReadWrite<u32, InterruptControlAndState::Register>),

        /// Vector Table Offset Register
        (0x08 => vtor: ReadWrite<u32, VectorTableOffset::Register>),

        /// Application Interrupt and Reset Control Register
        (0x0c => aircr: ReadWrite<u32, ApplicationInterruptAndReset::Register>),

        /// System Control Register
        (0x10 => scr: ReadWrite<u32, SystemControl::Register>),

        /// Configuration and Control Register
        (0x14 => ccr: ReadWrite<u32, ConfigurationAndControl::Register>),

        /// System Handler Priority Register (1-4)
        (0x18 => shp: [ReadWrite<u32, SystemHandlerPriority::Register>; 3]),

        /// System Handler Control and State Register
        (0x24 => shcsr: ReadWrite<u32, SystemHandlerControlAndState::Register>),

        /// Configurable Fault Status Register
        (0x28 => cfsr: ReadWrite<u32, ConfigurableFaultStatus::Register>),

        /// HardFault Status Register
        (0x2c => hfsr: ReadWrite<u32, HardFaultStatus::Register>),

        /// Debug Fault Status Register
        (0x30 => dfsr: ReadWrite<u32, DebugFaultStatus::Register>),

        /// MemManage Fault Address Register
        (0x34 => mmfar: ReadWrite<u32, FaultAddress::Register>),

        /// BusFault Address Register
        (0x38 => bfar: ReadWrite<u32, FaultAddress::Register>),

        /// Auxiliary Fault Status Register
        (0x3c => afsr: ReadWrite<u32, FaultAddress::Register>),

        /// 0xE000ED40-7C, Reserved for CPUID registers.
        (0x40 => _reserved0),

        /// 0xE000ED80-84, Reserved.
        (0x80 => _reserved1),

        /// Coprocessor Access Control Register
        (0x88 => cpacr: ReadWrite<u32, CoprocessorAccessControl::Register>),

        /// 0xE000ED8C, Reserved.
        (0x8c => _reserved2),

        (0x90 => @END),
    }
}

register_bitfields![u32,
    CpuId [
        /// Implementer code assigned by ARM. ARM implementations are 0x41.
        IMPLEMENTER     OFFSET(24)  NUMBITS(8),

        /// Implementer-defined variant number.
        VARIANT         OFFSET(20)  NUMBITS(4),

        /// Archtiecture always reads as 0xF for Cortex-M
        ARCHITECTURE    OFFSET(16)  NUMBITS(4),

        /// Implementer-defined part number.
        PARTNO          OFFSET(4)   NUMBITS(12),

        /// Implementer-defined revision number.
        REVISION        OFFSET(0)   NUMBITS(4)
    ],

    InterruptControlAndState [
        /// Non-Maskable Interrupt.
        /// Write 0 is no-op, write 1 triggers. Read returns whether NMI is active.
        /// RW.
        NMIPENDSET      OFFSET(31)  NUMBITS(1),

        /// Pendable SerVice.
        /// Write 0 is no-op, write 1 triggers. Read returns whether PendSV is active.
        /// RW.
        PENDSVSET       OFFSET(28)  NUMBITS(1),

        /// Write 1 to clear PendSV.
        /// WO.
        PENDSVCLR       OFFSET(27)  NUMBITS(1),

        /// Pendable SysTick.
        /// Write 0 is no-op, write 1 triggers. Read returns whether PendST is active.
        /// RW.
        PENDSTSET       OFFSET(26)  NUMBITS(1),

        /// Write 1 to clear PendST.
        /// WO.
        PENDSTCLR       OFFSET(25)  NUMBITS(1),

        /// Whether an excpetion will be serviced when existing debug state.
        /// RO.
        ISRPREEMPT      OFFSET(23)  NUMBITS(1),

        /// Whether an external interrupt (from NVIC) is pending.
        /// RO.
        ISRPENDING      OFFSET(22)  NUMBITS(1),

        /// Highest pending exception. Zero if none pending.
        /// RO.
        VECTACTIVE      OFFSET(0)   NUMBITS(9)
    ],

    /// Note: Software can write all 1s to `TBLOFF` and read result to learn
    /// maximum supported value.
    VectorTableOffset [
        /// Bits [31:7] of the vector table address
        /// n.b. bits [6:0] are always 0.
        TBLOFF          OFFSET(7)   NUMBITS(25)
    ],

    ApplicationInterruptAndReset [
        /// Key field. Must write 0x05FA or write is ignored. Reads as 0xFA05.
        /// RW.
        VECTKEY         OFFSET(16)  NUMBITS(16),

        /// 0=Little endian, 1=Big endian.
        /// RO.
        ENDIANNESS      OFFSET(15)  NUMBITS(1),

        /// Binary point position for priority grouping. Defaults to 0b000.
        /// RW.
        PRIGROUP        OFFSET(8)   NUMBITS(3),

        /// Writing 1 to this bit requests a Local reset. Cleared to 0b0 on reset.
        /// RW.
        SYSRESETREQ     OFFSET(2)   NUMBITS(1),

        /// Writing 1 clears all state information for exceptions.
        /// WARN: Writing this bit when not in a Debug halt is UNPREDICTABLE.
        /// WO.
        VECTCLRACTIVE   OFFSET(1)   NUMBITS(1),

        /// Writing 1 causes a local system reset.
        /// WARN: Writing this bit when not in a Debug halt is UNPREDICTABLE.
        /// WARN: Writing this and `SYSRESETREQ` is UNPREDICTABLE.
        /// WO.
        VECTRESET       OFFSET(0)   NUMBITS(1)
    ],

    SystemControl [
        SEVONPEND       OFFSET(4)   NUMBITS(1),
        SLEEPDEEP       OFFSET(2)   NUMBITS(1),
        SLEEPONEXIT     OFFSET(1)   NUMBITS(1)
    ],

    ConfigurationAndControl [
        STKALIGN        OFFSET(9)   NUMBITS(1),
        BFHFNMIGN       OFFSET(8)   NUMBITS(1),
        DIV_0_TRAP      OFFSET(4)   NUMBITS(1),
        UNALIGN_TRP     OFFSET(3)   NUMBITS(1),
        USERSETMPEND    OFFSET(1)   NUMBITS(1),
        NONBASETHRDENA  OFFSET(0)   NUMBITS(1)
    ],

    // Note: Simplified
    SystemHandlerPriority [
        PRI_N3          OFFSET(24)  NUMBITS(4),
        PRI_N2          OFFSET(16)  NUMBITS(4),
        PRI_N1          OFFSET(8)   NUMBITS(4),
        PRI_N0          OFFSET(0)   NUMBITS(4)
    ],

    SystemHandlerControlAndState [
        USGFAULTENA     OFFSET(18)  NUMBITS(1),
        BUSFAULTENA     OFFSET(17)  NUMBITS(1),
        MEMFAULTENA     OFFSET(16)  NUMBITS(1),
        SVCALLPENDED    OFFSET(15)  NUMBITS(1),
        BUSFAULTPENDED  OFFSET(14)  NUMBITS(1),
        MEMFAULTPENDED  OFFSET(14)  NUMBITS(1),
        USGFAULTPENDED  OFFSET(14)  NUMBITS(1),
        SYSTICKACT      OFFSET(11)  NUMBITS(1),
        PENDSVACT       OFFSET(10)  NUMBITS(1),
        MONITORACT      OFFSET(8)   NUMBITS(1),
        SVCALLACT       OFFSET(7)   NUMBITS(1),
        USGFAULTACT     OFFSET(3)   NUMBITS(1),
        BUSFAULTACT     OFFSET(1)   NUMBITS(1),
        MEMFAULTACT     OFFSET(0)   NUMBITS(1)
    ],

    ConfigurableFaultStatus [
        UsageFault      OFFSET(16)  NUMBITS(16),
        BusFault        OFFSET(8)   NUMBITS(8),
        MemManage       OFFSET(0)   NUMBITS(8)
    ],

    MemManageStatus [
        MMARVALID       OFFSET(7)   NUMBITS(1),
        MLSPERR         OFFSET(5)   NUMBITS(1),
        MSTKERR         OFFSET(4)   NUMBITS(1),
        MUNSTKERR       OFFSET(3)   NUMBITS(1),
        DACCVIOL        OFFSET(1)   NUMBITS(1),
        IACCVIOL        OFFSET(1)   NUMBITS(1)
    ],

    BusFaultStatus [
        BFARVALID       OFFSET(7)   NUMBITS(1),
        LSPERR          OFFSET(5)   NUMBITS(1),
        STKERR          OFFSET(4)   NUMBITS(1),
        UNSTKERR        OFFSET(3)   NUMBITS(1),
        IMPRECISERR     OFFSET(2)   NUMBITS(1),
        PRECISERR       OFFSET(1)   NUMBITS(1),
        IBUSERR         OFFSET(0)   NUMBITS(1)
    ],

    UsageFaultStatus [
        DIVBYZERO       OFFSET(9)   NUMBITS(1),
        UNALIGNED       OFFSET(8)   NUMBITS(1),
        NOCP            OFFSET(3)   NUMBITS(1),
        INVPC           OFFSET(2)   NUMBITS(1),
        INVSTATE        OFFSET(1)   NUMBITS(1),
        UNDEFINSTR      OFFSET(0)   NUMBITS(1)
    ],

    HardFaultStatus [
        DEBUGEVT        OFFSET(31)  NUMBITS(1),
        FORCED          OFFSET(30)  NUMBITS(1),
        VECTTBL         OFFSET(1)   NUMBITS(1)
    ],

    DebugFaultStatus [
        EXTERNAL        OFFSET(4)   NUMBITS(1),
        VCATCH          OFFSET(3)   NUMBITS(1),
        DWTTRAP         OFFSET(2)   NUMBITS(1),
        BKPT            OFFSET(1)   NUMBITS(1),
        HALTED          OFFSET(0)   NUMBITS(1)
    ],

    FaultAddress [
        ADDRESS         OFFSET(0)   NUMBITS(32)
    ],

    CoprocessorAccessControl [
        CP11            OFFSET(22)  NUMBITS(2),
        CP10            OFFSET(20)  NUMBITS(2),
        CP7             OFFSET(14)  NUMBITS(2),
        CP6             OFFSET(12)  NUMBITS(2),
        CP5             OFFSET(10)  NUMBITS(2),
        CP4             OFFSET(8)  NUMBITS(2),
        CP3             OFFSET(6)  NUMBITS(2),
        CP2             OFFSET(4)  NUMBITS(2),
        CP1             OFFSET(2)  NUMBITS(2),
        CP0             OFFSET(0)  NUMBITS(2)
    ]
];

const SCB: StaticRef<ScbRegisters> = unsafe { StaticRef::new(0xE000ED00 as *const ScbRegisters) };

/// Allow the core to go into deep sleep on WFI.
///
/// The specific definition of "deep sleep" is chip specific.
pub unsafe fn set_sleepdeep() {
    SCB.scr.modify(SystemControl::SLEEPDEEP::SET);
}
/// Do not allow the core to go into deep sleep on WFI.
///
/// The specific definition of "deep sleep" is chip specific.
pub unsafe fn unset_sleepdeep() {
    SCB.scr.modify(SystemControl::SLEEPDEEP::CLEAR);
}

/// Software reset using the ARM System Control Block
pub unsafe fn reset() {
    SCB.aircr.modify(
        ApplicationInterruptAndReset::VECTKEY.val(0x05FA)
            + ApplicationInterruptAndReset::PRIGROUP.val(0b111)
            + ApplicationInterruptAndReset::SYSRESETREQ::SET,
    );
}

/// relocate interrupt vector table
pub unsafe fn set_vector_table_offset(offset: *const ()) {
    SCB.vtor.set(offset as u32);
}

/// Disable the FPU
#[cfg(all(target_arch = "arm", target_os = "none"))]
pub unsafe fn disable_fpca() {
    SCB.cpacr
        .modify(CoprocessorAccessControl::CP10::CLEAR + CoprocessorAccessControl::CP11::CLEAR);

    llvm_asm!("dsb");
    llvm_asm!("isb");

    if SCB.cpacr.read(CoprocessorAccessControl::CP10) != 0
        || SCB.cpacr.read(CoprocessorAccessControl::CP11) != 0
    {
        panic!("Unable to disable FPU");
    }
}

// Mock implementation for tests on Travis-CI.
#[cfg(not(any(target_arch = "arm", target_os = "none")))]
pub unsafe fn disable_fpca() {
    unimplemented!()
}
