//! Components for the ISL29035 sensor.
//!
//! This provides two Components, Isl29035Component, which provides
//! direct access to the ISL29035 within the kernel, and
//! AmbientLightComponent, which provides the ambient light system
//! call interface to the ISL29035. Note that only one of these
//! Components should be instantiated, as AmbientLightComponent itself
//! creates an Isl29035Component, which depends on a static buffer: if you
//! allocate both, then the two instances of Isl29035Component will conflict
//! on the buffer.
//!
//! Usage
//! -----
//! ```rust
//! let isl29035 = Isl29035Component::new(mux_i2c, mux_alarm)
//!     .finalize(components::isl29035_component_static!(sam4l::ast::Ast));
//! let ambient_light =
//!     AmbientLightComponent::new(board_kernel, extra_capsules::ambient_light::DRIVER_NUM, isl29035)
//!         .finalize(components::ambient_light_component_static!());
//! ```

// Author: Philip Levis <pal@cs.stanford.edu>
// Last modified: 6/20/2018

use core::mem::MaybeUninit;
use core_capsules::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use core_capsules::virtual_i2c::{I2CDevice, MuxI2C};
use extra_capsules::ambient_light::AmbientLight;
use extra_capsules::isl29035::Isl29035;
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::hil;
use kernel::hil::time::{self, Alarm};

// Setup static space for the objects.
#[macro_export]
macro_rules! isl29035_component_static {
    ($A:ty $(,)?) => {{
        let alarm = kernel::static_buf!(core_capsules::virtual_alarm::VirtualMuxAlarm<'static, $A>);
        let i2c_device = kernel::static_buf!(core_capsules::virtual_i2c::I2CDevice<'static>);
        let i2c_buffer = kernel::static_buf!([u8; extra_capsules::isl29035::BUF_LEN]);
        let isl29035 = kernel::static_buf!(
            extra_capsules::isl29035::Isl29035<
                'static,
                core_capsules::virtual_alarm::VirtualMuxAlarm<'static, $A>,
            >
        );

        (alarm, i2c_device, i2c_buffer, isl29035)
    };};
}

#[macro_export]
macro_rules! ambient_light_component_static {
    () => {{
        kernel::static_buf!(extra_capsules::ambient_light::AmbientLight<'static>)
    };};
}

pub struct Isl29035Component<A: 'static + time::Alarm<'static>> {
    i2c_mux: &'static MuxI2C<'static>,
    alarm_mux: &'static MuxAlarm<'static, A>,
}

impl<A: 'static + time::Alarm<'static>> Isl29035Component<A> {
    pub fn new(i2c: &'static MuxI2C<'static>, alarm: &'static MuxAlarm<'static, A>) -> Self {
        Isl29035Component {
            i2c_mux: i2c,
            alarm_mux: alarm,
        }
    }
}

impl<A: 'static + time::Alarm<'static>> Component for Isl29035Component<A> {
    type StaticInput = (
        &'static mut MaybeUninit<VirtualMuxAlarm<'static, A>>,
        &'static mut MaybeUninit<I2CDevice<'static>>,
        &'static mut MaybeUninit<[u8; extra_capsules::isl29035::BUF_LEN]>,
        &'static mut MaybeUninit<Isl29035<'static, VirtualMuxAlarm<'static, A>>>,
    );
    type Output = &'static Isl29035<'static, VirtualMuxAlarm<'static, A>>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let isl29035_i2c = static_buffer.1.write(I2CDevice::new(self.i2c_mux, 0x44));
        let isl29035_i2c_buffer = static_buffer
            .2
            .write([0; extra_capsules::isl29035::BUF_LEN]);
        let isl29035_virtual_alarm = static_buffer.0.write(VirtualMuxAlarm::new(self.alarm_mux));
        isl29035_virtual_alarm.setup();

        let isl29035 = static_buffer.3.write(Isl29035::new(
            isl29035_i2c,
            isl29035_virtual_alarm,
            isl29035_i2c_buffer,
        ));
        isl29035_i2c.set_client(isl29035);
        isl29035_virtual_alarm.set_alarm_client(isl29035);
        isl29035
    }
}

pub struct AmbientLightComponent<L: 'static + hil::sensors::AmbientLight<'static>> {
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
    light_sensor: &'static L,
}

impl<L: 'static + hil::sensors::AmbientLight<'static>> AmbientLightComponent<L> {
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        driver_num: usize,
        light_sensor: &'static L,
    ) -> Self {
        AmbientLightComponent {
            board_kernel: board_kernel,
            driver_num: driver_num,
            light_sensor,
        }
    }
}

impl<L: 'static + hil::sensors::AmbientLight<'static>> Component for AmbientLightComponent<L> {
    type StaticInput = &'static mut MaybeUninit<AmbientLight<'static>>;
    type Output = &'static AmbientLight<'static>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        let ambient_light = static_buffer.write(AmbientLight::new(
            self.light_sensor,
            self.board_kernel.create_grant(self.driver_num, &grant_cap),
        ));
        hil::sensors::AmbientLight::set_client(self.light_sensor, ambient_light);
        ambient_light
    }
}
