// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

//! Component for constructing the `x86_q35::Pc` chip instance on x86 PC boards.

use core::mem::MaybeUninit;

use kernel::component::Component;
use x86::mpu::PagingMPU;
use x86::registers::bits32::paging::{PD, PT};
use x86::Boundary;

use x86_q35::pit::Pit;
use x86_q35::serial::{SerialPortComponent, COM1_BASE, COM2_BASE, COM3_BASE, COM4_BASE};
use x86_q35::vga_uart_driver::VgaText;
use x86_q35::Pc;

pub struct X86PcComponent<'a> {
    pd: &'a mut PD,
    pt: &'a mut PT,
}

impl<'a> X86PcComponent<'a> {
    /// ## Safety
    /// Only one instance should ever be created for the lifetime of the kernel.
    pub unsafe fn new(pd: &'a mut PD, pt: &'a mut PT) -> Self {
        Self { pd, pt }
    }
}

impl Component for X86PcComponent<'static> {
    type StaticInput = (
        <SerialPortComponent as Component>::StaticInput,
        <SerialPortComponent as Component>::StaticInput,
        <SerialPortComponent as Component>::StaticInput,
        <SerialPortComponent as Component>::StaticInput,
        &'static mut MaybeUninit<VgaText<'static>>,
        &'static mut MaybeUninit<Pc<'static>>,
    );
    type Output = &'static Pc<'static>;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        // ### Safety
        // - Running in ring zero.
        // - Single instantiation across kernel lifetime.
        unsafe {
            x86::init();
            x86_q35::pic::init();

            let pd: &mut PD = &mut *core::ptr::from_mut(self.pd);
            x86_q35::vga::new_text_console(pd);
        }

        let com1 = unsafe { SerialPortComponent::new(COM1_BASE).finalize(s.0) };
        let com2 = unsafe { SerialPortComponent::new(COM2_BASE).finalize(s.1) };
        let com3 = unsafe { SerialPortComponent::new(COM3_BASE).finalize(s.2) };
        let com4 = unsafe { SerialPortComponent::new(COM4_BASE).finalize(s.3) };

        let pit = unsafe { Pit::new() };

        let vga = s.4.write(VgaText::new());
        kernel::deferred_call::DeferredCallClient::register(vga);

        let paging = unsafe {
            let pd_addr = core::ptr::from_ref(self.pd) as usize;
            let pt_addr = core::ptr::from_ref(self.pt) as usize;
            PagingMPU::new(self.pd, pd_addr, self.pt, pt_addr)
        };
        paging.init();

        let syscall = Boundary::new();

        s.5.write(Pc::new(com1, com2, com3, com4, pit, vga, syscall, paging))
    }
}

#[macro_export]
macro_rules! x86_pc_component_static {
    () => {{
        (
            (kernel::static_buf!(x86_q35::serial::SerialPort<'static>),),
            (kernel::static_buf!(x86_q35::serial::SerialPort<'static>),),
            (kernel::static_buf!(x86_q35::serial::SerialPort<'static>),),
            (kernel::static_buf!(x86_q35::serial::SerialPort<'static>),),
            kernel::static_buf!(x86_q35::vga_uart_driver::VgaText<'static>),
            kernel::static_buf!(x86_q35::Pc<'static>),
        )
    }};
}
