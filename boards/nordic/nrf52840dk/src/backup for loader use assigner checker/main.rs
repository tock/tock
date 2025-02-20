// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Tock kernel for the Nordic Semiconductor nRF52840 development kit (DK).

#![no_std]
// Disable this attribute when documenting, as a workaround for
// https://github.com/rust-lang/rust/issues/62184.
#![cfg_attr(not(doc), no_main)]
#![deny(missing_docs)]

use core::ptr::addr_of_mut;

use kernel::debug;
use kernel::platform::{KernelResources, SyscallDriverLookup};
use kernel::{capabilities, create_capability};
use nrf52840dk_lib::{self, PROCESSES};

use capsules_extra::screen::Screen;
use core::cell::Cell;
use kernel::component::Component;
// use kernel::hil::screen::Screen;
use kernel::static_init;
use kernel::syscall::SyscallDriver;
use nrf52840::gpio::Pin;
use nrf52840::interrupt_service::Nrf52840DefaultPeripherals;

// /// I2C pins for all of the sensors.
// const I2C_SDA_PIN: Pin = Pin::P1_10;
// const I2C_SCL_PIN: Pin = Pin::P1_11;

// State for loading and holding applications.
// How should the kernel respond when a process faults.
const FAULT_RESPONSE: capsules_system::process_policies::PanicFaultPolicy =
    capsules_system::process_policies::PanicFaultPolicy {};

// Number of concurrent processes this platform supports.
const NUM_PROCS: usize = 8;

// type Screen = components::ssd1306::Ssd1306ComponentType<nrf52840::i2c::TWI<'static>>;
type ScreenDriver = components::screen::ScreenComponentType;

struct Platform {
    base: nrf52840dk_lib::Platform,
    eui64_driver: &'static nrf52840dk_lib::Eui64Driver,
    ieee802154_driver: &'static nrf52840dk_lib::Ieee802154Driver,
    udp_driver: &'static capsules_extra::net::udp::UDPDriver<'static>,
    screen: &'static ScreenDriver, // add screen driver
    soil_value: core::cell::Cell<u32>, //add parameter to store 'soil' from app 'soil-moisture-sensor'
                                       // ipc: kernel::ipc::IPC<{ NUM_PROCS as u8 }>,
}

impl SyscallDriverLookup for Platform {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn kernel::syscall::SyscallDriver>) -> R,
    {
        match driver_num {
            capsules_extra::eui64::DRIVER_NUM => f(Some(self.eui64_driver)),
            capsules_extra::net::udp::DRIVER_NUM => f(Some(self.udp_driver)),
            capsules_extra::ieee802154::DRIVER_NUM => f(Some(self.ieee802154_driver)),
            capsules_extra::screen::DRIVER_NUM => f(Some(self.screen)),
            _ => self.base.with_driver(driver_num, f),
        }
    }
}

type Chip = nrf52840dk_lib::Chip;

impl KernelResources<Chip> for Platform {
    type SyscallDriverLookup = Self;
    type SyscallFilter = <nrf52840dk_lib::Platform as KernelResources<Chip>>::SyscallFilter;
    type ProcessFault = <nrf52840dk_lib::Platform as KernelResources<Chip>>::ProcessFault;
    type Scheduler = <nrf52840dk_lib::Platform as KernelResources<Chip>>::Scheduler;
    type SchedulerTimer = <nrf52840dk_lib::Platform as KernelResources<Chip>>::SchedulerTimer;
    type WatchDog = <nrf52840dk_lib::Platform as KernelResources<Chip>>::WatchDog;
    type ContextSwitchCallback =
        <nrf52840dk_lib::Platform as KernelResources<Chip>>::ContextSwitchCallback;

    fn syscall_driver_lookup(&self) -> &Self::SyscallDriverLookup {
        self
    }
    fn syscall_filter(&self) -> &Self::SyscallFilter {
        self.base.syscall_filter()
    }
    fn process_fault(&self) -> &Self::ProcessFault {
        self.base.process_fault()
    }
    fn scheduler(&self) -> &Self::Scheduler {
        self.base.scheduler()
    }
    fn scheduler_timer(&self) -> &Self::SchedulerTimer {
        self.base.scheduler_timer()
    }
    fn watchdog(&self) -> &Self::WatchDog {
        self.base.watchdog()
    }
    fn context_switch_callback(&self) -> &Self::ContextSwitchCallback {
        self.base.context_switch_callback()
    }
}

// impl Platform {
//     fn u32_to_str(mut value: u32, buffer: &mut [u8]) -> &str {
//         let mut i = buffer.len();
//         while value != 0 {
//             i -= 1;
//             buffer[i] = b'0' + (value % 10) as u8;
//             value /= 10;
//         }
//         core::str::from_utf8(&buffer[i..]).unwrap_or("")
//     }

//     /// 更新屏幕以显示 `soil` 数据
//     fn update_screen(&self) {
//         let soil_value = self.soil_value.get(); // 获取当前的 soil 数据

//         // 转换 soil 数据为字符串
//         let mut buffer = [0u8; 16];
//         let soil_str = Self::u32_to_str(soil_value, &mut buffer); // 将 soil 数据转为字符串

//         // 调用 screen 驱动更新屏幕
//         self.screen.command(200, soil_str, soil_str.len(), 0); // 将数据显示在屏幕左上角
//     }
// }

/// Main function called after RAM initialized.
#[no_mangle]
pub unsafe fn main() {
    let ieee802154_ack_buf = static_init!(
        [u8; nrf52840::ieee802154_radio::ACK_BUF_SIZE],
        [0; nrf52840::ieee802154_radio::ACK_BUF_SIZE]
    );

    // Initialize chip peripheral drivers
    let nrf52840_peripherals = static_init!(
        Nrf52840DefaultPeripherals,
        Nrf52840DefaultPeripherals::new(ieee802154_ack_buf)
    );

    // set up circular peripheral dependencies
    nrf52840_peripherals.init();
    let base_peripherals = &nrf52840_peripherals.nrf52;

    let (board_kernel, base_platform, chip, default_peripherals, mux_alarm) =
        nrf52840dk_lib::start();

    //--------------------------------------------------------------------------
    // IEEE 802.15.4 and UDP
    //--------------------------------------------------------------------------

    let (eui64_driver, ieee802154_driver, udp_driver) =
        nrf52840dk_lib::ieee802154_udp(board_kernel, default_peripherals, mux_alarm);

    //--------------------------------------------------------------------------
    // SCREEN INITIALIZATION
    //--------------------------------------------------------------------------

    const SCREEN_I2C_SDA_PIN: Pin = Pin::P1_10;
    const SCREEN_I2C_SCL_PIN: Pin = Pin::P1_11;

    let i2c_bus = components::i2c::I2CMuxComponent::new(&nrf52840_peripherals.nrf52.twi1, None)
        .finalize(components::i2c_mux_component_static!(nrf52840::i2c::TWI));
    nrf52840_peripherals.nrf52.twi1.configure(
        nrf52840::pinmux::Pinmux::new(SCREEN_I2C_SCL_PIN as u32),
        nrf52840::pinmux::Pinmux::new(SCREEN_I2C_SDA_PIN as u32),
    );
    nrf52840_peripherals
        .nrf52
        .twi1
        .set_speed(nrf52840::i2c::Speed::K400);

    // I2C address is b011110X, and on this board D/C̅ is GND.
    let ssd1306_sh1106_i2c = components::i2c::I2CComponent::new(i2c_bus, 0x3c)
        .finalize(components::i2c_component_static!(nrf52840::i2c::TWI));

    // Create the ssd1306 object for the actual screen driver.
    // #[cfg(feature = "screen_ssd1306")]
    // let ssd1306_sh1106 = components::ssd1306::Ssd1306Component::new(ssd1306_sh1106_i2c, true)
    //     .finalize(components::ssd1306_component_static!(nrf52840::i2c::TWI));

    // #[cfg(feature = "screen_sh1106")]
    let ssd1306_sh1106 = components::sh1106::Sh1106Component::new(ssd1306_sh1106_i2c, true)
        .finalize(components::sh1106_component_static!(nrf52840::i2c::TWI));

    let screen = components::screen::ScreenComponent::new(
        board_kernel,
        capsules_extra::screen::DRIVER_NUM,
        ssd1306_sh1106,
        None,
    )
    .finalize(components::screen_component_static!(1032));

    ssd1306_sh1106.init_screen();

    // // 配置 I2C 引脚
    // default_peripherals.twi0.configure(
    //     nrf52840::pinmux::Pinmux::new(nrf52840dk_lib::I2C_SCL_PIN as u32),
    //     nrf52840::pinmux::Pinmux::new(nrf52840dk_lib::I2C_SDA_PIN as u32),
    // );

    // // 创建 I2C 总线
    // let i2c_bus = components::i2c::I2CMuxComponent::new(&default_peripherals.twi1, None)
    //     .finalize(components::i2c_mux_component_static!(nrf52840::i2c::TWI));

    // let i2c_bus = components::i2c::I2CMuxComponent::new(&base_peripherals.twi1, None)
    //     .finalize(components::i2c_mux_component_static!(nrf52840::i2c::TWI));
    // base_peripherals.twi1.configure(
    //     nrf52840::pinmux::Pinmux::new(I2C_SCL_PIN as u32),
    //     nrf52840::pinmux::Pinmux::new(I2C_SDA_PIN as u32),
    // );

    // // 创建 SSD1306 屏幕组件
    // let ssd1306_i2c = components::i2c::I2CComponent::new(i2c_bus, 0x3C) // 0x3C 是屏幕的 I2C 地址
    //     .finalize(components::i2c_component_static!(nrf52840::i2c::TWI));

    // let ssd1306 = components::ssd1306::Ssd1306Component::new(ssd1306_i2c, true)
    //     .finalize(components::ssd1306_component_static!(nrf52840::i2c::TWI));

    // // 初始化 Screen 组件
    // let screen = components::screen::ScreenComponent::new(
    //     board_kernel,
    //     capsules_extra::screen::DRIVER_NUM,
    //     ssd1306,
    //     Some(ssd1306),
    // )
    // .finalize(components::screen_component_static!(1032));

    let platform = Platform {
        base: base_platform,
        eui64_driver,
        ieee802154_driver,
        udp_driver,
        screen,
        // ipc: kernel::ipc::IPC::new(
        //     board_kernel,
        //     kernel::ipc::DRIVER_NUM,
        //     &memory_allocation_capability,
        // ),
        soil_value: core::cell::Cell::new(0),
    };

    // //--------------------------------------------------------------------------
    // // SCREEN TESTING
    // //--------------------------------------------------------------------------

    // debug!("Testing screen communication...");
    // // let process_id = kernel::process::ProcessId::new_external(
    // //     board_kernel,
    // //     1,
    // //     0,
    // //     &process_management_capability,
    // // ); // 创建虚拟进程 ID，用于测试

    // // // 手动实现 ExternalProcessCapability
    // // unsafe impl kernel::capabilities::ExternalProcessCapability
    // //     for kernel::capabilities::ProcessManagementCapability
    // // {
    // // }
    // // let process_management_capability: &dyn kernel::capabilities::ProcessManagementCapability =
    // //     create_capability!(kernel::capabilities::ProcessManagementCapability);

    // // 创建 ProcessId（使用 new_external）
    // let process_management_capability: &dyn kernel::capabilities::ProcessManagementCapability =
    //     &create_capability!(kernel::capabilities::ProcessManagementCapability);

    // // // 创建 ProcessId（使用 new_external）
    // // let process_management_capability =
    // //     create_capability!(kernel::capabilities::ProcessManagementCapability);
    // let process_id = kernel::process::ProcessId::new_external(
    //     board_kernel,
    //     1,                              // 唯一标识符
    //     0,                              // 索引
    //     &process_management_capability, // 能力
    // );
    // // 初始化屏幕（清屏并准备写入）
    // ssd1306.init_screen();

    // // // 初始化屏幕：清屏
    // // screen.command(0, 0, 0, process_id); // `0` 通常用于初始化屏幕

    // // 设置显示区域 (x=10, y=10, width=8, height=16)
    // let x_y = (10 << 16) | 10; // 起始位置
    // let width_height = (8 << 16) | 16; // 区域大小
    // screen.command(100, x_y, width_height, process_id);

    // // 写入显示数据：数字 "1"
    // let digit_one: [u8; 16] = [
    //     0b00011000, 0b00111000, 0b00011000, 0b00011000, 0b00011000, 0b00011000, 0b00011000,
    //     0b00011000, 0b00011000, 0b00011000, 0b00011000, 0b00011000, 0b00011000, 0b00011000,
    //     0b01111110, 0b01111110,
    // ];
    // screen.command(
    //     200,
    //     digit_one.as_ptr() as usize,
    //     digit_one.len(),
    //     process_id,
    // );

    // debug!("Screen test complete. Check display for '1'.");

    // These symbols are defined in the linker script.
    extern "C" {
        /// Beginning of the ROM region containing app images.
        static _sapps: u8;
        /// End of the ROM region containing app images.
        static _eapps: u8;
        /// Beginning of the RAM region for app memory.
        static mut _sappmem: u8;
        /// End of the RAM region for app memory.
        static _eappmem: u8;
    }

    let process_management_capability =
        create_capability!(capabilities::ProcessManagementCapability);
    kernel::process::load_processes(
        board_kernel,
        chip,
        core::slice::from_raw_parts(
            core::ptr::addr_of!(_sapps),
            core::ptr::addr_of!(_eapps) as usize - core::ptr::addr_of!(_sapps) as usize,
        ),
        core::slice::from_raw_parts_mut(
            core::ptr::addr_of_mut!(_sappmem),
            core::ptr::addr_of!(_eappmem) as usize - core::ptr::addr_of!(_sappmem) as usize,
        ),
        &mut *addr_of_mut!(PROCESSES),
        &FAULT_RESPONSE,
        &process_management_capability,
    )
    .unwrap_or_else(|err| {
        debug!("Error loading processes!");
        debug!("{:?}", err);
    });

    // 更新屏幕显示 soil 值
    // platform.update_screen();

    let main_loop_capability = create_capability!(capabilities::MainLoopCapability);
    board_kernel.kernel_loop(
        &platform,
        chip,
        Some(&platform.base.ipc),
        &main_loop_capability,
    );
}
