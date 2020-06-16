use kernel::common::cells::{OptionalCell, TakeCell};
use kernel::common::registers::{register_bitfields, ReadOnly, ReadWrite};
use kernel::common::StaticRef;
use kernel::ClockInterface;

use crate::nvic;
use crate::rcc;
use crate::spi;
use crate::usart;

/// DMA controller
#[repr(C)]
struct Dma1Registers {
    /// low interrupt status register
    lisr: ReadOnly<u32, LISR::Register>,
    /// high interrupt status register
    hisr: ReadOnly<u32, HISR::Register>,
    /// low interrupt flag clear register
    lifcr: ReadWrite<u32, LIFCR::Register>,
    /// high interrupt flag clear register
    hifcr: ReadWrite<u32, HIFCR::Register>,
    /// stream x configuration register
    s0cr: ReadWrite<u32, S0CR::Register>,
    /// stream x number of data register
    s0ndtr: ReadWrite<u32>,
    /// stream x peripheral address register
    s0par: ReadWrite<u32>,
    /// stream x memory 0 address register
    s0m0ar: ReadWrite<u32>,
    /// stream x memory 1 address register
    s0m1ar: ReadWrite<u32>,
    /// stream x FIFO control register
    s0fcr: ReadWrite<u32, S0FCR::Register>,
    /// stream x configuration register
    s1cr: ReadWrite<u32, S1CR::Register>,
    /// stream x number of data register
    s1ndtr: ReadWrite<u32>,
    /// stream x peripheral address register
    s1par: ReadWrite<u32>,
    /// stream x memory 0 address register
    s1m0ar: ReadWrite<u32>,
    /// stream x memory 1 address register
    s1m1ar: ReadWrite<u32>,
    /// stream x FIFO control register
    s1fcr: ReadWrite<u32, S1FCR::Register>,
    /// stream x configuration register
    s2cr: ReadWrite<u32, S2CR::Register>,
    /// stream x number of data register
    s2ndtr: ReadWrite<u32>,
    /// stream x peripheral address register
    s2par: ReadWrite<u32>,
    /// stream x memory 0 address register
    s2m0ar: ReadWrite<u32>,
    /// stream x memory 1 address register
    s2m1ar: ReadWrite<u32>,
    /// stream x FIFO control register
    s2fcr: ReadWrite<u32, S2FCR::Register>,
    /// stream x configuration register
    s3cr: ReadWrite<u32, S3CR::Register>,
    /// stream x number of data register
    s3ndtr: ReadWrite<u32>,
    /// stream x peripheral address register
    s3par: ReadWrite<u32>,
    /// stream x memory 0 address register
    s3m0ar: ReadWrite<u32>,
    /// stream x memory 1 address register
    s3m1ar: ReadWrite<u32>,
    /// stream x FIFO control register
    s3fcr: ReadWrite<u32, S3FCR::Register>,
    /// stream x configuration register
    s4cr: ReadWrite<u32, S4CR::Register>,
    /// stream x number of data register
    s4ndtr: ReadWrite<u32>,
    /// stream x peripheral address register
    s4par: ReadWrite<u32>,
    /// stream x memory 0 address register
    s4m0ar: ReadWrite<u32>,
    /// stream x memory 1 address register
    s4m1ar: ReadWrite<u32>,
    /// stream x FIFO control register
    s4fcr: ReadWrite<u32, S4FCR::Register>,
    /// stream x configuration register
    s5cr: ReadWrite<u32, S5CR::Register>,
    /// stream x number of data register
    s5ndtr: ReadWrite<u32>,
    /// stream x peripheral address register
    s5par: ReadWrite<u32>,
    /// stream x memory 0 address register
    s5m0ar: ReadWrite<u32>,
    /// stream x memory 1 address register
    s5m1ar: ReadWrite<u32>,
    /// stream x FIFO control register
    s5fcr: ReadWrite<u32, S5FCR::Register>,
    /// stream x configuration register
    s6cr: ReadWrite<u32, S6CR::Register>,
    /// stream x number of data register
    s6ndtr: ReadWrite<u32>,
    /// stream x peripheral address register
    s6par: ReadWrite<u32>,
    /// stream x memory 0 address register
    s6m0ar: ReadWrite<u32>,
    /// stream x memory 1 address register
    s6m1ar: ReadWrite<u32>,
    /// stream x FIFO control register
    s6fcr: ReadWrite<u32, S6FCR::Register>,
    /// stream x configuration register
    s7cr: ReadWrite<u32, S7CR::Register>,
    /// stream x number of data register
    s7ndtr: ReadWrite<u32>,
    /// stream x peripheral address register
    s7par: ReadWrite<u32>,
    /// stream x memory 0 address register
    s7m0ar: ReadWrite<u32>,
    /// stream x memory 1 address register
    s7m1ar: ReadWrite<u32>,
    /// stream x FIFO control register
    s7fcr: ReadWrite<u32, S7FCR::Register>,
}

register_bitfields![u32,
    LISR [
        /// Stream x transfer complete interrupt flag (x = 3..0)
        TCIF3 OFFSET(27) NUMBITS(1) [],
        /// Stream x half transfer interrupt flag (x=3..0)
        HTIF3 OFFSET(26) NUMBITS(1) [],
        /// Stream x transfer error interrupt flag (x=3..0)
        TEIF3 OFFSET(25) NUMBITS(1) [],
        /// Stream x direct mode error interrupt flag (x=3..0)
        DMEIF3 OFFSET(24) NUMBITS(1) [],
        /// Stream x FIFO error interrupt flag (x=3..0)
        FEIF3 OFFSET(22) NUMBITS(1) [],
        /// Stream x transfer complete interrupt flag (x = 3..0)
        TCIF2 OFFSET(21) NUMBITS(1) [],
        /// Stream x half transfer interrupt flag (x=3..0)
        HTIF2 OFFSET(20) NUMBITS(1) [],
        /// Stream x transfer error interrupt flag (x=3..0)
        TEIF2 OFFSET(19) NUMBITS(1) [],
        /// Stream x direct mode error interrupt flag (x=3..0)
        DMEIF2 OFFSET(18) NUMBITS(1) [],
        /// Stream x FIFO error interrupt flag (x=3..0)
        FEIF2 OFFSET(16) NUMBITS(1) [],
        /// Stream x transfer complete interrupt flag (x = 3..0)
        TCIF1 OFFSET(11) NUMBITS(1) [],
        /// Stream x half transfer interrupt flag (x=3..0)
        HTIF1 OFFSET(10) NUMBITS(1) [],
        /// Stream x transfer error interrupt flag (x=3..0)
        TEIF1 OFFSET(9) NUMBITS(1) [],
        /// Stream x direct mode error interrupt flag (x=3..0)
        DMEIF1 OFFSET(8) NUMBITS(1) [],
        /// Stream x FIFO error interrupt flag (x=3..0)
        FEIF1 OFFSET(6) NUMBITS(1) [],
        /// Stream x transfer complete interrupt flag (x = 3..0)
        TCIF0 OFFSET(5) NUMBITS(1) [],
        /// Stream x half transfer interrupt flag (x=3..0)
        HTIF0 OFFSET(4) NUMBITS(1) [],
        /// Stream x transfer error interrupt flag (x=3..0)
        TEIF0 OFFSET(3) NUMBITS(1) [],
        /// Stream x direct mode error interrupt flag (x=3..0)
        DMEIF0 OFFSET(2) NUMBITS(1) [],
        /// Stream x FIFO error interrupt flag (x=3..0)
        FEIF0 OFFSET(0) NUMBITS(1) []
    ],
    HISR [
        /// Stream x transfer complete interrupt flag (x=7..4)
        TCIF7 OFFSET(27) NUMBITS(1) [],
        /// Stream x half transfer interrupt flag (x=7..4)
        HTIF7 OFFSET(26) NUMBITS(1) [],
        /// Stream x transfer error interrupt flag (x=7..4)
        TEIF7 OFFSET(25) NUMBITS(1) [],
        /// Stream x direct mode error interrupt flag (x=7..4)
        DMEIF7 OFFSET(24) NUMBITS(1) [],
        /// Stream x FIFO error interrupt flag (x=7..4)
        FEIF7 OFFSET(22) NUMBITS(1) [],
        /// Stream x transfer complete interrupt flag (x=7..4)
        TCIF6 OFFSET(21) NUMBITS(1) [],
        /// Stream x half transfer interrupt flag (x=7..4)
        HTIF6 OFFSET(20) NUMBITS(1) [],
        /// Stream x transfer error interrupt flag (x=7..4)
        TEIF6 OFFSET(19) NUMBITS(1) [],
        /// Stream x direct mode error interrupt flag (x=7..4)
        DMEIF6 OFFSET(18) NUMBITS(1) [],
        /// Stream x FIFO error interrupt flag (x=7..4)
        FEIF6 OFFSET(16) NUMBITS(1) [],
        /// Stream x transfer complete interrupt flag (x=7..4)
        TCIF5 OFFSET(11) NUMBITS(1) [],
        /// Stream x half transfer interrupt flag (x=7..4)
        HTIF5 OFFSET(10) NUMBITS(1) [],
        /// Stream x transfer error interrupt flag (x=7..4)
        TEIF5 OFFSET(9) NUMBITS(1) [],
        /// Stream x direct mode error interrupt flag (x=7..4)
        DMEIF5 OFFSET(8) NUMBITS(1) [],
        /// Stream x FIFO error interrupt flag (x=7..4)
        FEIF5 OFFSET(6) NUMBITS(1) [],
        /// Stream x transfer complete interrupt flag (x=7..4)
        TCIF4 OFFSET(5) NUMBITS(1) [],
        /// Stream x half transfer interrupt flag (x=7..4)
        HTIF4 OFFSET(4) NUMBITS(1) [],
        /// Stream x transfer error interrupt flag (x=7..4)
        TEIF4 OFFSET(3) NUMBITS(1) [],
        /// Stream x direct mode error interrupt flag (x=7..4)
        DMEIF4 OFFSET(2) NUMBITS(1) [],
        /// Stream x FIFO error interrupt flag (x=7..4)
        FEIF4 OFFSET(0) NUMBITS(1) []
    ],
    LIFCR [
        /// Stream x clear transfer complete interrupt flag (x = 3..0)
        CTCIF3 OFFSET(27) NUMBITS(1) [],
        /// Stream x clear half transfer interrupt flag (x = 3..0)
        CHTIF3 OFFSET(26) NUMBITS(1) [],
        /// Stream x clear transfer error interrupt flag (x = 3..0)
        CTEIF3 OFFSET(25) NUMBITS(1) [],
        /// Stream x clear direct mode error interrupt flag (x = 3..0)
        CDMEIF3 OFFSET(24) NUMBITS(1) [],
        /// Stream x clear FIFO error interrupt flag (x = 3..0)
        CFEIF3 OFFSET(22) NUMBITS(1) [],
        /// Stream x clear transfer complete interrupt flag (x = 3..0)
        CTCIF2 OFFSET(21) NUMBITS(1) [],
        /// Stream x clear half transfer interrupt flag (x = 3..0)
        CHTIF2 OFFSET(20) NUMBITS(1) [],
        /// Stream x clear transfer error interrupt flag (x = 3..0)
        CTEIF2 OFFSET(19) NUMBITS(1) [],
        /// Stream x clear direct mode error interrupt flag (x = 3..0)
        CDMEIF2 OFFSET(18) NUMBITS(1) [],
        /// Stream x clear FIFO error interrupt flag (x = 3..0)
        CFEIF2 OFFSET(16) NUMBITS(1) [],
        /// Stream x clear transfer complete interrupt flag (x = 3..0)
        CTCIF1 OFFSET(11) NUMBITS(1) [],
        /// Stream x clear half transfer interrupt flag (x = 3..0)
        CHTIF1 OFFSET(10) NUMBITS(1) [],
        /// Stream x clear transfer error interrupt flag (x = 3..0)
        CTEIF1 OFFSET(9) NUMBITS(1) [],
        /// Stream x clear direct mode error interrupt flag (x = 3..0)
        CDMEIF1 OFFSET(8) NUMBITS(1) [],
        /// Stream x clear FIFO error interrupt flag (x = 3..0)
        CFEIF1 OFFSET(6) NUMBITS(1) [],
        /// Stream x clear transfer complete interrupt flag (x = 3..0)
        CTCIF0 OFFSET(5) NUMBITS(1) [],
        /// Stream x clear half transfer interrupt flag (x = 3..0)
        CHTIF0 OFFSET(4) NUMBITS(1) [],
        /// Stream x clear transfer error interrupt flag (x = 3..0)
        CTEIF0 OFFSET(3) NUMBITS(1) [],
        /// Stream x clear direct mode error interrupt flag (x = 3..0)
        CDMEIF0 OFFSET(2) NUMBITS(1) [],
        /// Stream x clear FIFO error interrupt flag (x = 3..0)
        CFEIF0 OFFSET(0) NUMBITS(1) []
    ],
    HIFCR [
        /// Stream x clear transfer complete interrupt flag (x = 7..4)
        CTCIF7 OFFSET(27) NUMBITS(1) [],
        /// Stream x clear half transfer interrupt flag (x = 7..4)
        CHTIF7 OFFSET(26) NUMBITS(1) [],
        /// Stream x clear transfer error interrupt flag (x = 7..4)
        CTEIF7 OFFSET(25) NUMBITS(1) [],
        /// Stream x clear direct mode error interrupt flag (x = 7..4)
        CDMEIF7 OFFSET(24) NUMBITS(1) [],
        /// Stream x clear FIFO error interrupt flag (x = 7..4)
        CFEIF7 OFFSET(22) NUMBITS(1) [],
        /// Stream x clear transfer complete interrupt flag (x = 7..4)
        CTCIF6 OFFSET(21) NUMBITS(1) [],
        /// Stream x clear half transfer interrupt flag (x = 7..4)
        CHTIF6 OFFSET(20) NUMBITS(1) [],
        /// Stream x clear transfer error interrupt flag (x = 7..4)
        CTEIF6 OFFSET(19) NUMBITS(1) [],
        /// Stream x clear direct mode error interrupt flag (x = 7..4)
        CDMEIF6 OFFSET(18) NUMBITS(1) [],
        /// Stream x clear FIFO error interrupt flag (x = 7..4)
        CFEIF6 OFFSET(16) NUMBITS(1) [],
        /// Stream x clear transfer complete interrupt flag (x = 7..4)
        CTCIF5 OFFSET(11) NUMBITS(1) [],
        /// Stream x clear half transfer interrupt flag (x = 7..4)
        CHTIF5 OFFSET(10) NUMBITS(1) [],
        /// Stream x clear transfer error interrupt flag (x = 7..4)
        CTEIF5 OFFSET(9) NUMBITS(1) [],
        /// Stream x clear direct mode error interrupt flag (x = 7..4)
        CDMEIF5 OFFSET(8) NUMBITS(1) [],
        /// Stream x clear FIFO error interrupt flag (x = 7..4)
        CFEIF5 OFFSET(6) NUMBITS(1) [],
        /// Stream x clear transfer complete interrupt flag (x = 7..4)
        CTCIF4 OFFSET(5) NUMBITS(1) [],
        /// Stream x clear half transfer interrupt flag (x = 7..4)
        CHTIF4 OFFSET(4) NUMBITS(1) [],
        /// Stream x clear transfer error interrupt flag (x = 7..4)
        CTEIF4 OFFSET(3) NUMBITS(1) [],
        /// Stream x clear direct mode error interrupt flag (x = 7..4)
        CDMEIF4 OFFSET(2) NUMBITS(1) [],
        /// Stream x clear FIFO error interrupt flag (x = 7..4)
        CFEIF4 OFFSET(0) NUMBITS(1) []
    ],
    S0CR [
        /// Channel selection
        CHSEL OFFSET(25) NUMBITS(3) [],
        /// Memory burst transfer configuration
        MBURST OFFSET(23) NUMBITS(2) [],
        /// Peripheral burst transfer configuration
        PBURST OFFSET(21) NUMBITS(2) [],
        /// Current target (only in double buffer mode)
        CT OFFSET(19) NUMBITS(1) [],
        /// Double buffer mode
        DBM OFFSET(18) NUMBITS(1) [],
        /// Priority level
        PL OFFSET(16) NUMBITS(2) [],
        /// Peripheral increment offset size
        PINCOS OFFSET(15) NUMBITS(1) [],
        /// Memory data size
        MSIZE OFFSET(13) NUMBITS(2) [],
        /// Peripheral data size
        PSIZE OFFSET(11) NUMBITS(2) [],
        /// Memory increment mode
        MINC OFFSET(10) NUMBITS(1) [],
        /// Peripheral increment mode
        PINC OFFSET(9) NUMBITS(1) [],
        /// Circular mode
        CIRC OFFSET(8) NUMBITS(1) [],
        /// Data transfer direction
        DIR OFFSET(6) NUMBITS(2) [],
        /// Peripheral flow controller
        PFCTRL OFFSET(5) NUMBITS(1) [],
        /// Transfer complete interrupt enable
        TCIE OFFSET(4) NUMBITS(1) [],
        /// Half transfer interrupt enable
        HTIE OFFSET(3) NUMBITS(1) [],
        /// Transfer error interrupt enable
        TEIE OFFSET(2) NUMBITS(1) [],
        /// Direct mode error interrupt enable
        DMEIE OFFSET(1) NUMBITS(1) [],
        /// Stream enable / flag stream ready when read low
        EN OFFSET(0) NUMBITS(1) []
    ],
    S0FCR [
        /// FIFO error interrupt enable
        FEIE OFFSET(7) NUMBITS(1) [],
        /// FIFO status
        FS OFFSET(3) NUMBITS(3) [],
        /// Direct mode disable
        DMDIS OFFSET(2) NUMBITS(1) [],
        /// FIFO threshold selection
        FTH OFFSET(0) NUMBITS(2) []
    ],
    S1CR [
        /// Channel selection
        CHSEL OFFSET(25) NUMBITS(3) [],
        /// Memory burst transfer configuration
        MBURST OFFSET(23) NUMBITS(2) [],
        /// Peripheral burst transfer configuration
        PBURST OFFSET(21) NUMBITS(2) [],
        /// ACK
        ACK OFFSET(20) NUMBITS(1) [],
        /// Current target (only in double buffer mode)
        CT OFFSET(19) NUMBITS(1) [],
        /// Double buffer mode
        DBM OFFSET(18) NUMBITS(1) [],
        /// Priority level
        PL OFFSET(16) NUMBITS(2) [],
        /// Peripheral increment offset size
        PINCOS OFFSET(15) NUMBITS(1) [],
        /// Memory data size
        MSIZE OFFSET(13) NUMBITS(2) [],
        /// Peripheral data size
        PSIZE OFFSET(11) NUMBITS(2) [],
        /// Memory increment mode
        MINC OFFSET(10) NUMBITS(1) [],
        /// Peripheral increment mode
        PINC OFFSET(9) NUMBITS(1) [],
        /// Circular mode
        CIRC OFFSET(8) NUMBITS(1) [],
        /// Data transfer direction
        DIR OFFSET(6) NUMBITS(2) [],
        /// Peripheral flow controller
        PFCTRL OFFSET(5) NUMBITS(1) [],
        /// Transfer complete interrupt enable
        TCIE OFFSET(4) NUMBITS(1) [],
        /// Half transfer interrupt enable
        HTIE OFFSET(3) NUMBITS(1) [],
        /// Transfer error interrupt enable
        TEIE OFFSET(2) NUMBITS(1) [],
        /// Direct mode error interrupt enable
        DMEIE OFFSET(1) NUMBITS(1) [],
        /// Stream enable / flag stream ready when read low
        EN OFFSET(0) NUMBITS(1) []
    ],
    S1FCR [
        /// FIFO error interrupt enable
        FEIE OFFSET(7) NUMBITS(1) [],
        /// FIFO status
        FS OFFSET(3) NUMBITS(3) [],
        /// Direct mode disable
        DMDIS OFFSET(2) NUMBITS(1) [],
        /// FIFO threshold selection
        FTH OFFSET(0) NUMBITS(2) []
    ],
    S2CR [
        /// Channel selection
        CHSEL OFFSET(25) NUMBITS(3) [],
        /// Memory burst transfer configuration
        MBURST OFFSET(23) NUMBITS(2) [],
        /// Peripheral burst transfer configuration
        PBURST OFFSET(21) NUMBITS(2) [],
        /// ACK
        ACK OFFSET(20) NUMBITS(1) [],
        /// Current target (only in double buffer mode)
        CT OFFSET(19) NUMBITS(1) [],
        /// Double buffer mode
        DBM OFFSET(18) NUMBITS(1) [],
        /// Priority level
        PL OFFSET(16) NUMBITS(2) [],
        /// Peripheral increment offset size
        PINCOS OFFSET(15) NUMBITS(1) [],
        /// Memory data size
        MSIZE OFFSET(13) NUMBITS(2) [],
        /// Peripheral data size
        PSIZE OFFSET(11) NUMBITS(2) [],
        /// Memory increment mode
        MINC OFFSET(10) NUMBITS(1) [],
        /// Peripheral increment mode
        PINC OFFSET(9) NUMBITS(1) [],
        /// Circular mode
        CIRC OFFSET(8) NUMBITS(1) [],
        /// Data transfer direction
        DIR OFFSET(6) NUMBITS(2) [],
        /// Peripheral flow controller
        PFCTRL OFFSET(5) NUMBITS(1) [],
        /// Transfer complete interrupt enable
        TCIE OFFSET(4) NUMBITS(1) [],
        /// Half transfer interrupt enable
        HTIE OFFSET(3) NUMBITS(1) [],
        /// Transfer error interrupt enable
        TEIE OFFSET(2) NUMBITS(1) [],
        /// Direct mode error interrupt enable
        DMEIE OFFSET(1) NUMBITS(1) [],
        /// Stream enable / flag stream ready when read low
        EN OFFSET(0) NUMBITS(1) []
    ],
    S2FCR [
        /// FIFO error interrupt enable
        FEIE OFFSET(7) NUMBITS(1) [],
        /// FIFO status
        FS OFFSET(3) NUMBITS(3) [],
        /// Direct mode disable
        DMDIS OFFSET(2) NUMBITS(1) [],
        /// FIFO threshold selection
        FTH OFFSET(0) NUMBITS(2) []
    ],
    S3CR [
        /// Channel selection
        CHSEL OFFSET(25) NUMBITS(3) [],
        /// Memory burst transfer configuration
        MBURST OFFSET(23) NUMBITS(2) [],
        /// Peripheral burst transfer configuration
        PBURST OFFSET(21) NUMBITS(2) [],
        /// ACK
        ACK OFFSET(20) NUMBITS(1) [],
        /// Current target (only in double buffer mode)
        CT OFFSET(19) NUMBITS(1) [],
        /// Double buffer mode
        DBM OFFSET(18) NUMBITS(1) [],
        /// Priority level
        PL OFFSET(16) NUMBITS(2) [],
        /// Peripheral increment offset size
        PINCOS OFFSET(15) NUMBITS(1) [],
        /// Memory data size
        MSIZE OFFSET(13) NUMBITS(2) [],
        /// Peripheral data size
        PSIZE OFFSET(11) NUMBITS(2) [],
        /// Memory increment mode
        MINC OFFSET(10) NUMBITS(1) [],
        /// Peripheral increment mode
        PINC OFFSET(9) NUMBITS(1) [],
        /// Circular mode
        CIRC OFFSET(8) NUMBITS(1) [],
        /// Data transfer direction
        DIR OFFSET(6) NUMBITS(2) [],
        /// Peripheral flow controller
        PFCTRL OFFSET(5) NUMBITS(1) [],
        /// Transfer complete interrupt enable
        TCIE OFFSET(4) NUMBITS(1) [],
        /// Half transfer interrupt enable
        HTIE OFFSET(3) NUMBITS(1) [],
        /// Transfer error interrupt enable
        TEIE OFFSET(2) NUMBITS(1) [],
        /// Direct mode error interrupt enable
        DMEIE OFFSET(1) NUMBITS(1) [],
        /// Stream enable / flag stream ready when read low
        EN OFFSET(0) NUMBITS(1) []
    ],
    S3FCR [
        /// FIFO error interrupt enable
        FEIE OFFSET(7) NUMBITS(1) [],
        /// FIFO status
        FS OFFSET(3) NUMBITS(3) [],
        /// Direct mode disable
        DMDIS OFFSET(2) NUMBITS(1) [],
        /// FIFO threshold selection
        FTH OFFSET(0) NUMBITS(2) []
    ],
    S4CR [
        /// Channel selection
        CHSEL OFFSET(25) NUMBITS(3) [],
        /// Memory burst transfer configuration
        MBURST OFFSET(23) NUMBITS(2) [],
        /// Peripheral burst transfer configuration
        PBURST OFFSET(21) NUMBITS(2) [],
        /// ACK
        ACK OFFSET(20) NUMBITS(1) [],
        /// Current target (only in double buffer mode)
        CT OFFSET(19) NUMBITS(1) [],
        /// Double buffer mode
        DBM OFFSET(18) NUMBITS(1) [],
        /// Priority level
        PL OFFSET(16) NUMBITS(2) [],
        /// Peripheral increment offset size
        PINCOS OFFSET(15) NUMBITS(1) [],
        /// Memory data size
        MSIZE OFFSET(13) NUMBITS(2) [],
        /// Peripheral data size
        PSIZE OFFSET(11) NUMBITS(2) [],
        /// Memory increment mode
        MINC OFFSET(10) NUMBITS(1) [],
        /// Peripheral increment mode
        PINC OFFSET(9) NUMBITS(1) [],
        /// Circular mode
        CIRC OFFSET(8) NUMBITS(1) [],
        /// Data transfer direction
        DIR OFFSET(6) NUMBITS(2) [],
        /// Peripheral flow controller
        PFCTRL OFFSET(5) NUMBITS(1) [],
        /// Transfer complete interrupt enable
        TCIE OFFSET(4) NUMBITS(1) [],
        /// Half transfer interrupt enable
        HTIE OFFSET(3) NUMBITS(1) [],
        /// Transfer error interrupt enable
        TEIE OFFSET(2) NUMBITS(1) [],
        /// Direct mode error interrupt enable
        DMEIE OFFSET(1) NUMBITS(1) [],
        /// Stream enable / flag stream ready when read low
        EN OFFSET(0) NUMBITS(1) []
    ],
    S4FCR [
        /// FIFO error interrupt enable
        FEIE OFFSET(7) NUMBITS(1) [],
        /// FIFO status
        FS OFFSET(3) NUMBITS(3) [],
        /// Direct mode disable
        DMDIS OFFSET(2) NUMBITS(1) [],
        /// FIFO threshold selection
        FTH OFFSET(0) NUMBITS(2) []
    ],
    S5CR [
        /// Channel selection
        CHSEL OFFSET(25) NUMBITS(3) [],
        /// Memory burst transfer configuration
        MBURST OFFSET(23) NUMBITS(2) [],
        /// Peripheral burst transfer configuration
        PBURST OFFSET(21) NUMBITS(2) [],
        /// ACK
        ACK OFFSET(20) NUMBITS(1) [],
        /// Current target (only in double buffer mode)
        CT OFFSET(19) NUMBITS(1) [],
        /// Double buffer mode
        DBM OFFSET(18) NUMBITS(1) [],
        /// Priority level
        PL OFFSET(16) NUMBITS(2) [],
        /// Peripheral increment offset size
        PINCOS OFFSET(15) NUMBITS(1) [],
        /// Memory data size
        MSIZE OFFSET(13) NUMBITS(2) [],
        /// Peripheral data size
        PSIZE OFFSET(11) NUMBITS(2) [],
        /// Memory increment mode
        MINC OFFSET(10) NUMBITS(1) [],
        /// Peripheral increment mode
        PINC OFFSET(9) NUMBITS(1) [],
        /// Circular mode
        CIRC OFFSET(8) NUMBITS(1) [],
        /// Data transfer direction
        DIR OFFSET(6) NUMBITS(2) [],
        /// Peripheral flow controller
        PFCTRL OFFSET(5) NUMBITS(1) [],
        /// Transfer complete interrupt enable
        TCIE OFFSET(4) NUMBITS(1) [],
        /// Half transfer interrupt enable
        HTIE OFFSET(3) NUMBITS(1) [],
        /// Transfer error interrupt enable
        TEIE OFFSET(2) NUMBITS(1) [],
        /// Direct mode error interrupt enable
        DMEIE OFFSET(1) NUMBITS(1) [],
        /// Stream enable / flag stream ready when read low
        EN OFFSET(0) NUMBITS(1) []
    ],
    S5FCR [
        /// FIFO error interrupt enable
        FEIE OFFSET(7) NUMBITS(1) [],
        /// FIFO status
        FS OFFSET(3) NUMBITS(3) [],
        /// Direct mode disable
        DMDIS OFFSET(2) NUMBITS(1) [],
        /// FIFO threshold selection
        FTH OFFSET(0) NUMBITS(2) []
    ],
    S6CR [
        /// Channel selection
        CHSEL OFFSET(25) NUMBITS(3) [],
        /// Memory burst transfer configuration
        MBURST OFFSET(23) NUMBITS(2) [],
        /// Peripheral burst transfer configuration
        PBURST OFFSET(21) NUMBITS(2) [],
        /// ACK
        ACK OFFSET(20) NUMBITS(1) [],
        /// Current target (only in double buffer mode)
        CT OFFSET(19) NUMBITS(1) [],
        /// Double buffer mode
        DBM OFFSET(18) NUMBITS(1) [],
        /// Priority level
        PL OFFSET(16) NUMBITS(2) [],
        /// Peripheral increment offset size
        PINCOS OFFSET(15) NUMBITS(1) [],
        /// Memory data size
        MSIZE OFFSET(13) NUMBITS(2) [],
        /// Peripheral data size
        PSIZE OFFSET(11) NUMBITS(2) [],
        /// Memory increment mode
        MINC OFFSET(10) NUMBITS(1) [],
        /// Peripheral increment mode
        PINC OFFSET(9) NUMBITS(1) [],
        /// Circular mode
        CIRC OFFSET(8) NUMBITS(1) [],
        /// Data transfer direction
        DIR OFFSET(6) NUMBITS(2) [],
        /// Peripheral flow controller
        PFCTRL OFFSET(5) NUMBITS(1) [],
        /// Transfer complete interrupt enable
        TCIE OFFSET(4) NUMBITS(1) [],
        /// Half transfer interrupt enable
        HTIE OFFSET(3) NUMBITS(1) [],
        /// Transfer error interrupt enable
        TEIE OFFSET(2) NUMBITS(1) [],
        /// Direct mode error interrupt enable
        DMEIE OFFSET(1) NUMBITS(1) [],
        /// Stream enable / flag stream ready when read low
        EN OFFSET(0) NUMBITS(1) []
    ],
    S6FCR [
        /// FIFO error interrupt enable
        FEIE OFFSET(7) NUMBITS(1) [],
        /// FIFO status
        FS OFFSET(3) NUMBITS(3) [],
        /// Direct mode disable
        DMDIS OFFSET(2) NUMBITS(1) [],
        /// FIFO threshold selection
        FTH OFFSET(0) NUMBITS(2) []
    ],
    S7CR [
        /// Channel selection
        CHSEL OFFSET(25) NUMBITS(3) [],
        /// Memory burst transfer configuration
        MBURST OFFSET(23) NUMBITS(2) [],
        /// Peripheral burst transfer configuration
        PBURST OFFSET(21) NUMBITS(2) [],
        /// ACK
        ACK OFFSET(20) NUMBITS(1) [],
        /// Current target (only in double buffer mode)
        CT OFFSET(19) NUMBITS(1) [],
        /// Double buffer mode
        DBM OFFSET(18) NUMBITS(1) [],
        /// Priority level
        PL OFFSET(16) NUMBITS(2) [],
        /// Peripheral increment offset size
        PINCOS OFFSET(15) NUMBITS(1) [],
        /// Memory data size
        MSIZE OFFSET(13) NUMBITS(2) [],
        /// Peripheral data size
        PSIZE OFFSET(11) NUMBITS(2) [],
        /// Memory increment mode
        MINC OFFSET(10) NUMBITS(1) [],
        /// Peripheral increment mode
        PINC OFFSET(9) NUMBITS(1) [],
        /// Circular mode
        CIRC OFFSET(8) NUMBITS(1) [],
        /// Data transfer direction
        DIR OFFSET(6) NUMBITS(2) [],
        /// Peripheral flow controller
        PFCTRL OFFSET(5) NUMBITS(1) [],
        /// Transfer complete interrupt enable
        TCIE OFFSET(4) NUMBITS(1) [],
        /// Half transfer interrupt enable
        HTIE OFFSET(3) NUMBITS(1) [],
        /// Transfer error interrupt enable
        TEIE OFFSET(2) NUMBITS(1) [],
        /// Direct mode error interrupt enable
        DMEIE OFFSET(1) NUMBITS(1) [],
        /// Stream enable / flag stream ready when read low
        EN OFFSET(0) NUMBITS(1) []
    ],
    S7FCR [
        /// FIFO error interrupt enable
        FEIE OFFSET(7) NUMBITS(1) [],
        /// FIFO status
        FS OFFSET(3) NUMBITS(3) [],
        /// Direct mode disable
        DMDIS OFFSET(2) NUMBITS(1) [],
        /// FIFO threshold selection
        FTH OFFSET(0) NUMBITS(2) []
    ]
];

const DMA1_BASE: StaticRef<Dma1Registers> =
    unsafe { StaticRef::new(0x40026000 as *const Dma1Registers) };

/// The DMA stream number. What other microcontrollers refer to as "channel",
/// STM32F446RE refers to as "streams". STM32F446RE has eight streams. A stream
/// transfers data between memory and peripheral.
#[repr(u8)]
#[derive(Copy, Clone)]
pub enum StreamId {
    Stream0 = 0,
    Stream1 = 1,
    Stream2 = 2,
    Stream3 = 3,
    Stream4 = 4,
    Stream5 = 5,
    Stream6 = 6,
    Stream7 = 7,
}

/// Each stream can be selected among up to eight channel requests. This is
/// basically STM32F446RE's way of selecting the peripheral for the stream.
/// Nevertheless, the use of the term channel here is confusing. Table 28
/// describes the mapping between stream, channel, and peripherals.
#[allow(dead_code)]
#[repr(u32)]
enum ChannelId {
    Channel0 = 0b000,
    Channel1 = 0b001,
    Channel2 = 0b010,
    Channel3 = 0b011,
    Channel4 = 0b100,
    Channel5 = 0b101,
    Channel6 = 0b110,
    Channel7 = 0b111,
}

/// DMA transfer direction. Section 9.5.5
#[allow(dead_code)]
#[repr(u32)]
enum Direction {
    PeripheralToMemory = 0b00,
    MemoryToPeripheral = 0b01,
    MemoryToMemory = 0b10,
}

/// DMA data size. Section 9.5.5
#[allow(dead_code)]
#[repr(u32)]
enum Size {
    Byte = 0b00,
    HalfWord = 0b01,
    Word = 0b10,
}

struct Msize(Size);
struct Psize(Size);

/// DMA transfer mode. Section 9.5.10
#[allow(dead_code)]
#[repr(u32)]
enum FifoSize {
    Quarter = 0b00,
    Half = 0b01,
    ThreeFourths = 0b10,
    Full = 0b11,
}

#[allow(dead_code)]
enum TransferMode {
    Direct,
    Fifo(FifoSize),
}

/// List of peripherals managed by DMA1
#[allow(non_camel_case_types, non_snake_case)]
#[derive(Copy, Clone, PartialEq)]
pub enum Dma1Peripheral {
    USART2_TX,
    USART2_RX,
    USART3_TX,
    USART3_RX,
    SPI3_TX,
    SPI3_RX,
}

impl Dma1Peripheral {
    // Returns the IRQ number of the stream associated with the peripheral. Used
    // to enable interrupt on the NVIC.
    pub fn get_stream_irqn(&self) -> u32 {
        match self {
            Dma1Peripheral::SPI3_TX => nvic::DMA1_Stream7,
            Dma1Peripheral::USART2_TX => nvic::DMA1_Stream6,
            Dma1Peripheral::USART2_RX => nvic::DMA1_Stream5,
            Dma1Peripheral::USART3_TX => nvic::DMA1_Stream3,
            Dma1Peripheral::SPI3_RX => nvic::DMA1_Stream2,
            Dma1Peripheral::USART3_RX => nvic::DMA1_Stream1,
        }
    }

    pub fn get_stream<'a>(&self) -> &'a Stream<'static> {
        unsafe { &DMA1_STREAM[usize::from(StreamId::from(*self) as u8)] }
    }
}

impl From<Dma1Peripheral> for StreamId {
    fn from(pid: Dma1Peripheral) -> StreamId {
        match pid {
            Dma1Peripheral::SPI3_TX => StreamId::Stream7,
            Dma1Peripheral::USART2_TX => StreamId::Stream6,
            Dma1Peripheral::USART2_RX => StreamId::Stream5,
            Dma1Peripheral::USART3_TX => StreamId::Stream3,
            Dma1Peripheral::SPI3_RX => StreamId::Stream2,
            Dma1Peripheral::USART3_RX => StreamId::Stream1,
        }
    }
}

pub struct Stream<'a> {
    streamid: StreamId,
    client: OptionalCell<&'a dyn StreamClient>,
    buffer: TakeCell<'static, [u8]>,
    peripheral: OptionalCell<Dma1Peripheral>,
}

pub static mut DMA1_STREAM: [Stream<'static>; 8] = [
    Stream::new(StreamId::Stream0),
    Stream::new(StreamId::Stream1),
    Stream::new(StreamId::Stream2),
    Stream::new(StreamId::Stream3),
    Stream::new(StreamId::Stream4),
    Stream::new(StreamId::Stream5),
    Stream::new(StreamId::Stream6),
    Stream::new(StreamId::Stream7),
];

pub trait StreamClient {
    fn transfer_done(&self, pid: Dma1Peripheral);
}

impl<'a> Stream<'a> {
    const fn new(streamid: StreamId) -> Stream<'a> {
        Stream {
            streamid: streamid,
            buffer: TakeCell::empty(),
            client: OptionalCell::empty(),
            peripheral: OptionalCell::empty(),
        }
    }

    pub fn set_client(&self, client: &'a dyn StreamClient) {
        self.client.set(client);
    }

    pub fn handle_interrupt(&self) {
        self.clear_transfer_complete_flag();

        self.client.map(|client| {
            self.peripheral.map(|pid| {
                client.transfer_done(*pid);
            });
        });
    }

    pub fn setup(&self, pid: Dma1Peripheral) {
        self.peripheral.set(pid);

        // Setup is called before interrupts are enabled on the NVIC
        self.disable_interrupt();
        self.disable();

        // The numbers below are from Section 1.2 of AN4031. It looks like these
        // settings can be set only once. Trying to set them again, seems to
        // generate a hard-fault even when the stream is disabled.
        //
        // 8
        self.set_transfer_mode_for_peripheral();
        // 9
        self.set_data_width_for_peripheral();
    }

    pub fn do_transfer(&self, buf: &'static mut [u8], len: usize) {
        self.disable_interrupt();

        // The numbers below are from Section 1.2 of AN4031
        //
        // NOTE: We only clear TC flag here. Trying to clear any other flag,
        //       generates a hard-fault
        // 1
        self.disable();
        self.clear_transfer_complete_flag();
        // 2
        self.set_peripheral_address();
        // 3
        self.set_memory_address(&buf[0] as *const u8 as u32);
        // 4
        self.set_data_items(len as u32);
        // 5
        self.set_channel();
        // 9
        self.set_direction();
        self.set_peripheral_address_increment();
        self.set_memory_address_increment();
        self.interrupt_enable();
        // 10
        self.enable();

        // NOTE: We still have to enable DMA on the peripheral side
        self.buffer.replace(buf);
    }

    pub fn abort_transfer(&self) -> (Option<&'static mut [u8]>, u32) {
        self.disable_interrupt();

        self.disable();

        (self.buffer.take(), self.get_data_items())
    }

    pub fn return_buffer(&self) -> Option<&'static mut [u8]> {
        self.buffer.take()
    }

    fn set_channel(&self) {
        self.peripheral.map(|pid| {
            match pid {
                Dma1Peripheral::SPI3_TX => unsafe {
                    // SPI3_RX Stream 7, Channel 0
                    DMA1.registers
                        .s7cr
                        .modify(S7CR::CHSEL.val(ChannelId::Channel0 as u32));
                },
                Dma1Peripheral::USART2_TX => unsafe {
                    // USART2_TX Stream 6, Channel 4
                    DMA1.registers
                        .s6cr
                        .modify(S6CR::CHSEL.val(ChannelId::Channel4 as u32));
                },
                Dma1Peripheral::USART2_RX => unsafe {
                    // USART2_RX Stream 5, Channel 4
                    DMA1.registers
                        .s5cr
                        .modify(S5CR::CHSEL.val(ChannelId::Channel4 as u32));
                },
                Dma1Peripheral::USART3_TX => unsafe {
                    // USART3_TX Stream 3, Channel 4
                    DMA1.registers
                        .s3cr
                        .modify(S3CR::CHSEL.val(ChannelId::Channel4 as u32));
                },
                Dma1Peripheral::SPI3_RX => unsafe {
                    // SPI3_RX Stream 2, Channel 0
                    DMA1.registers
                        .s2cr
                        .modify(S2CR::CHSEL.val(ChannelId::Channel0 as u32));
                },
                Dma1Peripheral::USART3_RX => unsafe {
                    // USART3_RX Stream 1, Channel 4
                    DMA1.registers
                        .s1cr
                        .modify(S1CR::CHSEL.val(ChannelId::Channel4 as u32));
                },
            }
        });
    }

    fn set_direction(&self) {
        self.peripheral.map(|pid| {
            match pid {
                Dma1Peripheral::SPI3_TX => unsafe {
                    // SPI3_TX Stream 7
                    DMA1.registers
                        .s7cr
                        .modify(S7CR::DIR.val(Direction::MemoryToPeripheral as u32));
                },
                Dma1Peripheral::USART2_TX => unsafe {
                    // USART2_TX Stream 6
                    DMA1.registers
                        .s6cr
                        .modify(S6CR::DIR.val(Direction::MemoryToPeripheral as u32));
                },
                Dma1Peripheral::USART2_RX => unsafe {
                    // USART2_RX Stream 5
                    DMA1.registers
                        .s5cr
                        .modify(S5CR::DIR.val(Direction::PeripheralToMemory as u32));
                },
                Dma1Peripheral::USART3_TX => unsafe {
                    // USART3_TX Stream 3
                    DMA1.registers
                        .s3cr
                        .modify(S3CR::DIR.val(Direction::MemoryToPeripheral as u32));
                },
                Dma1Peripheral::SPI3_RX => unsafe {
                    // SPI3_RX Stream 2
                    DMA1.registers
                        .s2cr
                        .modify(S2CR::DIR.val(Direction::PeripheralToMemory as u32));
                },
                Dma1Peripheral::USART3_RX => unsafe {
                    // USART3_RX Stream 1
                    DMA1.registers
                        .s1cr
                        .modify(S1CR::DIR.val(Direction::PeripheralToMemory as u32));
                },
            }
        });
    }

    fn set_peripheral_address(&self) {
        self.peripheral.map(|pid| {
            match pid {
                Dma1Peripheral::SPI3_TX => unsafe {
                    // SPI3_TX Stream 7
                    DMA1.registers.s7par.set(spi::SPI3.get_address_dr());
                },
                Dma1Peripheral::USART2_TX => unsafe {
                    // USART2_TX Stream 6
                    DMA1.registers.s6par.set(usart::USART2.get_address_dr());
                },
                Dma1Peripheral::USART2_RX => unsafe {
                    // USART2_RX Stream 5
                    DMA1.registers.s5par.set(usart::USART2.get_address_dr());
                },
                Dma1Peripheral::USART3_TX => unsafe {
                    // USART3_TX Stream 3
                    DMA1.registers.s3par.set(usart::USART3.get_address_dr());
                },
                Dma1Peripheral::SPI3_RX => unsafe {
                    // SPI3_RX Stream 2
                    DMA1.registers.s2par.set(spi::SPI3.get_address_dr());
                },
                Dma1Peripheral::USART3_RX => unsafe {
                    // USART3_RX Stream 1
                    DMA1.registers.s1par.set(usart::USART3.get_address_dr());
                },
            }
        });
    }

    fn set_peripheral_address_increment(&self) {
        self.peripheral.map(|pid| {
            match pid {
                Dma1Peripheral::SPI3_TX => unsafe {
                    // SPI3_TX Stream 7
                    DMA1.registers.s7cr.modify(S7CR::PINC::CLEAR);
                },
                Dma1Peripheral::USART2_TX => unsafe {
                    // USART2_TX Stream 6
                    DMA1.registers.s6cr.modify(S6CR::PINC::CLEAR);
                },
                Dma1Peripheral::USART2_RX => unsafe {
                    // USART2_RX Stream 5
                    DMA1.registers.s5cr.modify(S5CR::PINC::CLEAR);
                },
                Dma1Peripheral::USART3_TX => unsafe {
                    // USART3_TX Stream 3
                    DMA1.registers.s3cr.modify(S3CR::PINC::CLEAR);
                },
                Dma1Peripheral::SPI3_RX => unsafe {
                    // SPI3_RX Stream 2
                    DMA1.registers.s2cr.modify(S2CR::PINC::CLEAR);
                },
                Dma1Peripheral::USART3_RX => unsafe {
                    // USART3_RX Stream 1
                    DMA1.registers.s1cr.modify(S1CR::PINC::CLEAR);
                },
            }
        });
    }

    fn set_memory_address(&self, buf_addr: u32) {
        self.peripheral.map(|pid| {
            match pid {
                Dma1Peripheral::SPI3_TX => unsafe {
                    // SPI3_TX Stream 7
                    DMA1.registers.s7m0ar.set(buf_addr);
                },
                Dma1Peripheral::USART2_TX => unsafe {
                    // USART2_TX Stream 6
                    DMA1.registers.s6m0ar.set(buf_addr);
                },
                Dma1Peripheral::USART2_RX => unsafe {
                    // USART2_RX Stream 5
                    DMA1.registers.s5m0ar.set(buf_addr);
                },
                Dma1Peripheral::USART3_TX => unsafe {
                    // USART3_TX Stream 3
                    DMA1.registers.s3m0ar.set(buf_addr);
                },
                Dma1Peripheral::SPI3_RX => unsafe {
                    // SPI3_RX Stream 2
                    DMA1.registers.s2m0ar.set(buf_addr);
                },
                Dma1Peripheral::USART3_RX => unsafe {
                    // USART3_RX Stream 1
                    DMA1.registers.s1m0ar.set(buf_addr);
                },
            }
        });
    }

    fn set_memory_address_increment(&self) {
        self.peripheral.map(|pid| {
            match pid {
                Dma1Peripheral::SPI3_TX => unsafe {
                    // SPI3_TX Stream 7
                    DMA1.registers.s7cr.modify(S7CR::MINC::SET);
                },
                Dma1Peripheral::USART2_TX => unsafe {
                    // USART2_TX Stream 6
                    DMA1.registers.s6cr.modify(S6CR::MINC::SET);
                },
                Dma1Peripheral::USART2_RX => unsafe {
                    // USART2_RX Stream 5
                    DMA1.registers.s5cr.modify(S5CR::MINC::SET);
                },
                Dma1Peripheral::USART3_TX => unsafe {
                    // USART3_TX Stream 3
                    DMA1.registers.s3cr.modify(S3CR::MINC::SET);
                },
                Dma1Peripheral::SPI3_RX => unsafe {
                    // SPI3_RX Stream 2
                    DMA1.registers.s2cr.modify(S2CR::MINC::SET);
                },
                Dma1Peripheral::USART3_RX => unsafe {
                    // USART3_RX Stream 1
                    DMA1.registers.s1cr.modify(S1CR::MINC::SET);
                },
            }
        });
    }

    fn get_data_items(&self) -> u32 {
        match self.streamid {
            StreamId::Stream0 => unsafe { DMA1.registers.s0ndtr.get() },
            StreamId::Stream1 => unsafe { DMA1.registers.s1ndtr.get() },
            StreamId::Stream2 => unsafe { DMA1.registers.s2ndtr.get() },
            StreamId::Stream3 => unsafe { DMA1.registers.s3ndtr.get() },
            StreamId::Stream4 => unsafe { DMA1.registers.s4ndtr.get() },
            StreamId::Stream5 => unsafe { DMA1.registers.s5ndtr.get() },
            StreamId::Stream6 => unsafe { DMA1.registers.s6ndtr.get() },
            StreamId::Stream7 => unsafe { DMA1.registers.s7ndtr.get() },
        }
    }

    fn set_data_items(&self, data_items: u32) {
        match self.streamid {
            StreamId::Stream0 => unsafe {
                DMA1.registers.s0ndtr.set(data_items);
            },
            StreamId::Stream1 => unsafe {
                DMA1.registers.s1ndtr.set(data_items);
            },
            StreamId::Stream2 => unsafe {
                DMA1.registers.s2ndtr.set(data_items);
            },
            StreamId::Stream3 => unsafe {
                DMA1.registers.s3ndtr.set(data_items);
            },
            StreamId::Stream4 => unsafe {
                DMA1.registers.s4ndtr.set(data_items);
            },
            StreamId::Stream5 => unsafe {
                DMA1.registers.s5ndtr.set(data_items);
            },
            StreamId::Stream6 => unsafe {
                DMA1.registers.s6ndtr.set(data_items);
            },
            StreamId::Stream7 => unsafe {
                DMA1.registers.s7ndtr.set(data_items);
            },
        }
    }

    fn set_data_width_for_peripheral(&self) {
        self.peripheral.map(|pid| match pid {
            Dma1Peripheral::SPI3_TX => {
                self.stream_set_data_width(Msize(Size::Byte), Psize(Size::Byte))
            }
            Dma1Peripheral::USART2_TX => {
                self.stream_set_data_width(Msize(Size::Byte), Psize(Size::Byte))
            }
            Dma1Peripheral::USART2_RX => {
                self.stream_set_data_width(Msize(Size::Byte), Psize(Size::Byte))
            }
            Dma1Peripheral::USART3_TX => {
                self.stream_set_data_width(Msize(Size::Byte), Psize(Size::Byte))
            }
            Dma1Peripheral::SPI3_RX => {
                self.stream_set_data_width(Msize(Size::Byte), Psize(Size::Byte))
            }
            Dma1Peripheral::USART3_RX => {
                self.stream_set_data_width(Msize(Size::Byte), Psize(Size::Byte))
            }
        });
    }

    fn stream_set_data_width(&self, msize: Msize, psize: Psize) {
        match self.streamid {
            StreamId::Stream0 => unsafe {
                DMA1.registers.s0cr.modify(S0CR::PSIZE.val(psize.0 as u32));
                DMA1.registers.s0cr.modify(S0CR::MSIZE.val(msize.0 as u32));
            },
            StreamId::Stream1 => unsafe {
                DMA1.registers.s1cr.modify(S1CR::PSIZE.val(psize.0 as u32));
                DMA1.registers.s1cr.modify(S1CR::MSIZE.val(msize.0 as u32));
            },
            StreamId::Stream2 => unsafe {
                DMA1.registers.s2cr.modify(S2CR::PSIZE.val(psize.0 as u32));
                DMA1.registers.s2cr.modify(S2CR::MSIZE.val(msize.0 as u32));
            },
            StreamId::Stream3 => unsafe {
                DMA1.registers.s3cr.modify(S3CR::PSIZE.val(psize.0 as u32));
                DMA1.registers.s3cr.modify(S3CR::MSIZE.val(msize.0 as u32));
            },
            StreamId::Stream4 => unsafe {
                DMA1.registers.s4cr.modify(S4CR::PSIZE.val(psize.0 as u32));
                DMA1.registers.s4cr.modify(S4CR::MSIZE.val(msize.0 as u32));
            },
            StreamId::Stream5 => unsafe {
                DMA1.registers.s5cr.modify(S5CR::PSIZE.val(psize.0 as u32));
                DMA1.registers.s5cr.modify(S5CR::MSIZE.val(msize.0 as u32));
            },
            StreamId::Stream6 => unsafe {
                DMA1.registers.s6cr.modify(S6CR::PSIZE.val(psize.0 as u32));
                DMA1.registers.s6cr.modify(S6CR::MSIZE.val(msize.0 as u32));
            },
            StreamId::Stream7 => unsafe {
                DMA1.registers.s7cr.modify(S7CR::PSIZE.val(psize.0 as u32));
                DMA1.registers.s7cr.modify(S7CR::MSIZE.val(msize.0 as u32));
            },
        }
    }

    fn set_transfer_mode_for_peripheral(&self) {
        self.peripheral.map(|pid| match pid {
            Dma1Peripheral::SPI3_TX => {
                self.stream_set_transfer_mode(TransferMode::Fifo(FifoSize::Full));
            }
            Dma1Peripheral::USART2_TX => {
                self.stream_set_transfer_mode(TransferMode::Fifo(FifoSize::Full));
            }
            Dma1Peripheral::USART2_RX => {
                self.stream_set_transfer_mode(TransferMode::Fifo(FifoSize::Full));
            }
            Dma1Peripheral::USART3_TX => {
                self.stream_set_transfer_mode(TransferMode::Fifo(FifoSize::Full));
            }
            Dma1Peripheral::SPI3_RX => {
                self.stream_set_transfer_mode(TransferMode::Fifo(FifoSize::Full));
            }
            Dma1Peripheral::USART3_RX => {
                self.stream_set_transfer_mode(TransferMode::Fifo(FifoSize::Full));
            }
        });
    }

    fn stream_set_transfer_mode(&self, transfer_mode: TransferMode) {
        match self.streamid {
            StreamId::Stream0 => unsafe {
                match transfer_mode {
                    TransferMode::Direct => {
                        DMA1.registers.s0fcr.modify(S0FCR::DMDIS::CLEAR);
                    }
                    TransferMode::Fifo(s) => {
                        DMA1.registers.s0fcr.modify(S0FCR::DMDIS::SET);
                        DMA1.registers.s0fcr.modify(S0FCR::FTH.val(s as u32));
                    }
                }
            },
            StreamId::Stream1 => unsafe {
                match transfer_mode {
                    TransferMode::Direct => {
                        DMA1.registers.s1fcr.modify(S1FCR::DMDIS::CLEAR);
                    }
                    TransferMode::Fifo(s) => {
                        DMA1.registers.s1fcr.modify(S1FCR::DMDIS::SET);
                        DMA1.registers.s1fcr.modify(S1FCR::FTH.val(s as u32));
                    }
                }
            },
            StreamId::Stream2 => unsafe {
                match transfer_mode {
                    TransferMode::Direct => {
                        DMA1.registers.s2fcr.modify(S2FCR::DMDIS::CLEAR);
                    }
                    TransferMode::Fifo(s) => {
                        DMA1.registers.s2fcr.modify(S2FCR::DMDIS::SET);
                        DMA1.registers.s2fcr.modify(S2FCR::FTH.val(s as u32));
                    }
                }
            },
            StreamId::Stream3 => unsafe {
                match transfer_mode {
                    TransferMode::Direct => {
                        DMA1.registers.s3fcr.modify(S3FCR::DMDIS::CLEAR);
                    }
                    TransferMode::Fifo(s) => {
                        DMA1.registers.s3fcr.modify(S3FCR::DMDIS::SET);
                        DMA1.registers.s3fcr.modify(S3FCR::FTH.val(s as u32));
                    }
                }
            },
            StreamId::Stream4 => unsafe {
                match transfer_mode {
                    TransferMode::Direct => {
                        DMA1.registers.s4fcr.modify(S4FCR::DMDIS::CLEAR);
                    }
                    TransferMode::Fifo(s) => {
                        DMA1.registers.s4fcr.modify(S4FCR::DMDIS::SET);
                        DMA1.registers.s4fcr.modify(S4FCR::FTH.val(s as u32));
                    }
                }
            },
            StreamId::Stream5 => unsafe {
                match transfer_mode {
                    TransferMode::Direct => {
                        DMA1.registers.s5fcr.modify(S5FCR::DMDIS::CLEAR);
                    }
                    TransferMode::Fifo(s) => {
                        DMA1.registers.s5fcr.modify(S5FCR::DMDIS::SET);
                        DMA1.registers.s5fcr.modify(S5FCR::FTH.val(s as u32));
                    }
                }
            },
            StreamId::Stream6 => unsafe {
                match transfer_mode {
                    TransferMode::Direct => {
                        DMA1.registers.s6fcr.modify(S6FCR::DMDIS::CLEAR);
                    }
                    TransferMode::Fifo(s) => {
                        DMA1.registers.s6fcr.modify(S6FCR::DMDIS::SET);
                        DMA1.registers.s6fcr.modify(S6FCR::FTH.val(s as u32));
                    }
                }
            },
            StreamId::Stream7 => unsafe {
                match transfer_mode {
                    TransferMode::Direct => {
                        DMA1.registers.s7fcr.modify(S7FCR::DMDIS::CLEAR);
                    }
                    TransferMode::Fifo(s) => {
                        DMA1.registers.s7fcr.modify(S7FCR::DMDIS::SET);
                        DMA1.registers.s7fcr.modify(S7FCR::FTH.val(s as u32));
                    }
                }
            },
        }
    }

    fn enable(&self) {
        match self.streamid {
            StreamId::Stream0 => unsafe { DMA1.registers.s0cr.modify(S0CR::EN::SET) },
            StreamId::Stream1 => unsafe { DMA1.registers.s1cr.modify(S1CR::EN::SET) },
            StreamId::Stream2 => unsafe { DMA1.registers.s2cr.modify(S2CR::EN::SET) },
            StreamId::Stream3 => unsafe { DMA1.registers.s3cr.modify(S3CR::EN::SET) },
            StreamId::Stream4 => unsafe { DMA1.registers.s4cr.modify(S4CR::EN::SET) },
            StreamId::Stream5 => unsafe { DMA1.registers.s5cr.modify(S5CR::EN::SET) },
            StreamId::Stream6 => unsafe { DMA1.registers.s6cr.modify(S6CR::EN::SET) },
            StreamId::Stream7 => unsafe { DMA1.registers.s7cr.modify(S7CR::EN::SET) },
        }
    }

    fn disable(&self) {
        match self.streamid {
            StreamId::Stream0 => unsafe { DMA1.registers.s0cr.modify(S0CR::EN::CLEAR) },
            StreamId::Stream1 => unsafe { DMA1.registers.s1cr.modify(S1CR::EN::CLEAR) },
            StreamId::Stream2 => unsafe { DMA1.registers.s2cr.modify(S2CR::EN::CLEAR) },
            StreamId::Stream3 => unsafe { DMA1.registers.s3cr.modify(S3CR::EN::CLEAR) },
            StreamId::Stream4 => unsafe { DMA1.registers.s4cr.modify(S4CR::EN::CLEAR) },
            StreamId::Stream5 => unsafe { DMA1.registers.s5cr.modify(S5CR::EN::CLEAR) },
            StreamId::Stream6 => unsafe { DMA1.registers.s6cr.modify(S6CR::EN::CLEAR) },
            StreamId::Stream7 => unsafe { DMA1.registers.s7cr.modify(S7CR::EN::CLEAR) },
        }
    }

    fn clear_transfer_complete_flag(&self) {
        match self.streamid {
            StreamId::Stream0 => unsafe {
                DMA1.registers.lifcr.write(LIFCR::CTCIF0::SET);
            },
            StreamId::Stream1 => unsafe {
                DMA1.registers.lifcr.write(LIFCR::CTCIF1::SET);
            },
            StreamId::Stream2 => unsafe {
                DMA1.registers.lifcr.write(LIFCR::CTCIF2::SET);
            },
            StreamId::Stream3 => unsafe {
                DMA1.registers.lifcr.write(LIFCR::CTCIF3::SET);
            },
            StreamId::Stream4 => unsafe {
                DMA1.registers.hifcr.write(HIFCR::CTCIF4::SET);
            },
            StreamId::Stream5 => unsafe {
                DMA1.registers.hifcr.write(HIFCR::CTCIF5::SET);
            },
            StreamId::Stream6 => unsafe {
                DMA1.registers.hifcr.write(HIFCR::CTCIF6::SET);
            },
            StreamId::Stream7 => unsafe {
                DMA1.registers.hifcr.write(HIFCR::CTCIF7::SET);
            },
        }
    }

    // We only interrupt on TC (Transfer Complete)
    fn interrupt_enable(&self) {
        match self.streamid {
            StreamId::Stream0 => unsafe { DMA1.registers.s0cr.modify(S0CR::TCIE::SET) },
            StreamId::Stream1 => unsafe { DMA1.registers.s1cr.modify(S1CR::TCIE::SET) },
            StreamId::Stream2 => unsafe { DMA1.registers.s2cr.modify(S2CR::TCIE::SET) },
            StreamId::Stream3 => unsafe { DMA1.registers.s3cr.modify(S3CR::TCIE::SET) },
            StreamId::Stream4 => unsafe { DMA1.registers.s4cr.modify(S4CR::TCIE::SET) },
            StreamId::Stream5 => unsafe { DMA1.registers.s5cr.modify(S5CR::TCIE::SET) },
            StreamId::Stream6 => unsafe { DMA1.registers.s6cr.modify(S6CR::TCIE::SET) },
            StreamId::Stream7 => unsafe { DMA1.registers.s7cr.modify(S7CR::TCIE::SET) },
        }
    }

    // We only interrupt on TC (Transfer Complete)
    fn disable_interrupt(&self) {
        match self.streamid {
            StreamId::Stream0 => unsafe { DMA1.registers.s0cr.modify(S0CR::TCIE::CLEAR) },
            StreamId::Stream1 => unsafe { DMA1.registers.s1cr.modify(S1CR::TCIE::CLEAR) },
            StreamId::Stream2 => unsafe { DMA1.registers.s2cr.modify(S2CR::TCIE::CLEAR) },
            StreamId::Stream3 => unsafe { DMA1.registers.s3cr.modify(S3CR::TCIE::CLEAR) },
            StreamId::Stream4 => unsafe { DMA1.registers.s4cr.modify(S4CR::TCIE::CLEAR) },
            StreamId::Stream5 => unsafe { DMA1.registers.s5cr.modify(S5CR::TCIE::CLEAR) },
            StreamId::Stream6 => unsafe { DMA1.registers.s6cr.modify(S6CR::TCIE::CLEAR) },
            StreamId::Stream7 => unsafe { DMA1.registers.s7cr.modify(S7CR::TCIE::CLEAR) },
        }
    }
}

pub struct Dma1 {
    registers: StaticRef<Dma1Registers>,
    clock: Dma1Clock,
}

pub static mut DMA1: Dma1 = Dma1::new();

impl Dma1 {
    const fn new() -> Dma1 {
        Dma1 {
            registers: DMA1_BASE,
            clock: Dma1Clock(rcc::PeripheralClock::AHB1(rcc::HCLK1::DMA1)),
        }
    }

    pub fn is_enabled_clock(&self) -> bool {
        self.clock.is_enabled()
    }

    pub fn enable_clock(&self) {
        self.clock.enable();
    }

    pub fn disable_clock(&self) {
        self.clock.disable();
    }
}

struct Dma1Clock(rcc::PeripheralClock);

impl ClockInterface for Dma1Clock {
    fn is_enabled(&self) -> bool {
        self.0.is_enabled()
    }

    fn enable(&self) {
        self.0.enable();
    }

    fn disable(&self) {
        self.0.disable();
    }
}
