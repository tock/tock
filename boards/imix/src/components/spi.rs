//! Components for SPI on the imix board.
//!
//! This provides two Components. SpiSyscallComponent provides a
//! system call interace to SPI, while SpiComponent provides a
//! virtualized client to the SPI bus. SpiSyscallComponent is used for
//! processes, while SpiComponent is used for kernel capsules that need
//! access to the SPI bus.
//!
//! Usage
//! -----
//! ```rust
//! let spi_syscalls = SpiSyscallComponent::new(mux_spi).finalize();
//! let rf233_spi = SpiComponent::new(mux_spi).finalize();
//! ```

// Author: Philip Levis <pal@cs.stanford.edu>
// Last modified: 6/20/2018

#![allow(dead_code)] // Components are intended to be conditionally included

use capsules::spi::Spi;
use capsules::virtual_spi::{MuxSpiMaster, VirtualSpiMasterDevice};
use kernel::component::Component;

pub struct SpiSyscallComponent {
    spi_mux: &'static MuxSpiMaster<'static, sam4l::spi::SpiHw>,
}

pub struct SpiComponent {
    spi_mux: &'static MuxSpiMaster<'static, sam4l::spi::SpiHw>,
}

impl SpiSyscallComponent {
    pub fn new(mux: &'static MuxSpiMaster<'static, sam4l::spi::SpiHw>) -> Self {
        SpiSyscallComponent { spi_mux: mux }
    }
}

impl Component for SpiSyscallComponent {
    type Output = &'static Spi<'static, VirtualSpiMasterDevice<'static, sam4l::spi::SpiHw>>;

    unsafe fn finalize(&mut self) -> Self::Output {
        let syscall_spi_device = static_init!(
            VirtualSpiMasterDevice<'static, sam4l::spi::SpiHw>,
            VirtualSpiMasterDevice::new(self.spi_mux, 3)
        );

        let spi_syscalls = static_init!(
            Spi<'static, VirtualSpiMasterDevice<'static, sam4l::spi::SpiHw>>,
            Spi::new(syscall_spi_device)
        );

        static mut SPI_READ_BUF: [u8; 1024] = [0; 1024];
        static mut SPI_WRITE_BUF: [u8; 1024] = [0; 1024];

        spi_syscalls.config_buffers(&mut SPI_READ_BUF, &mut SPI_WRITE_BUF);
        syscall_spi_device.set_client(spi_syscalls);

        spi_syscalls
    }
}

impl SpiComponent {
    pub fn new(mux: &'static MuxSpiMaster<'static, sam4l::spi::SpiHw>) -> Self {
        SpiComponent { spi_mux: mux }
    }
}

impl Component for SpiComponent {
    type Output = &'static VirtualSpiMasterDevice<'static, sam4l::spi::SpiHw>;

    unsafe fn finalize(&mut self) -> Self::Output {
        let spi_device = static_init!(
            VirtualSpiMasterDevice<'static, sam4l::spi::SpiHw>,
            VirtualSpiMasterDevice::new(self.spi_mux, 3)
        );

        spi_device
    }
}
