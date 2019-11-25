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
//! let isl = Isl29035Component::new(mux_i2c, mux_alarm)
//!     .finalize(isl29035_component_helper!(sam4l::ast::Ast));
//! let ambient_light = AmbientLightComponent::new(board_kernel, sensors_i2c, mux_alarm)
//!     .finalize(isl29035_component_helper!(sam4l::ast::Ast));
//! ```

// Author: Philip Levis <pal@cs.stanford.edu>
// Last modified: 6/20/2018

#![allow(dead_code)] // Components are intended to be conditionally included

use core::mem::MaybeUninit;

use capsules::ambient_light::AmbientLight;
use capsules::isl29035::Isl29035;
use capsules::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use capsules::virtual_i2c::{I2CDevice, MuxI2C};
use kernel::capabilities;
use kernel::component::Component;
use kernel::hil;
use kernel::hil::time;
use kernel::hil::time::Alarm;
use kernel::{static_init, static_init_half};

// Setup static space for the objects.
#[macro_export]
macro_rules! isl29035_component_helper {
    ($A:ty) => {{
        use capsules::isl29035::Isl29035;
        use core::mem::MaybeUninit;
        static mut BUF1: MaybeUninit<VirtualMuxAlarm<'static, $A>> = MaybeUninit::uninit();
        static mut BUF2: MaybeUninit<Isl29035<'static, VirtualMuxAlarm<'static, $A>>> =
            MaybeUninit::uninit();
        (&mut BUF1, &mut BUF2)
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

// This should really be an option, such that you can create either
// an Isl29035 component or an AmbientLight component, but not both,
// such that trying to take the buffer out of an empty option leads to
// a panic explaining why. Right now it's possible for a board to make
// both components, which will conflict on the buffer. -pal

static mut I2C_BUF: [u8; 3] = [0; 3];

impl<A: 'static + time::Alarm<'static>> Component for Isl29035Component<A> {
    type StaticInput = (
        &'static mut MaybeUninit<VirtualMuxAlarm<'static, A>>,
        &'static mut MaybeUninit<Isl29035<'static, VirtualMuxAlarm<'static, A>>>,
    );

    type Output = &'static Isl29035<'static, VirtualMuxAlarm<'static, A>>;

    unsafe fn finalize(&mut self, static_buffer: Self::StaticInput) -> Self::Output {
        let isl29035_i2c = static_init!(I2CDevice, I2CDevice::new(self.i2c_mux, 0x44));
        let isl29035_virtual_alarm = static_init_half!(
            static_buffer.0,
            VirtualMuxAlarm<'static, A>,
            VirtualMuxAlarm::new(self.alarm_mux)
        );
        let isl29035 = static_init_half!(
            static_buffer.1,
            Isl29035<'static, VirtualMuxAlarm<'static, A>>,
            Isl29035::new(isl29035_i2c, isl29035_virtual_alarm, &mut I2C_BUF)
        );
        isl29035_i2c.set_client(isl29035);
        isl29035_virtual_alarm.set_client(isl29035);
        isl29035
    }
}

pub struct AmbientLightComponent<A: 'static + time::Alarm<'static>> {
    board_kernel: &'static kernel::Kernel,
    i2c_mux: &'static MuxI2C<'static>,
    alarm_mux: &'static MuxAlarm<'static, A>,
}

impl<A: 'static + time::Alarm<'static>> AmbientLightComponent<A> {
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        i2c: &'static MuxI2C<'static>,
        alarm: &'static MuxAlarm<'static, A>,
    ) -> Self {
        AmbientLightComponent {
            board_kernel: board_kernel,
            i2c_mux: i2c,
            alarm_mux: alarm,
        }
    }
}

impl<A: 'static + time::Alarm<'static>> Component for AmbientLightComponent<A> {
    type StaticInput = (
        &'static mut MaybeUninit<VirtualMuxAlarm<'static, A>>,
        &'static mut MaybeUninit<Isl29035<'static, VirtualMuxAlarm<'static, A>>>,
    );
    type Output = &'static AmbientLight<'static>;

    unsafe fn finalize(&mut self, static_buffer: Self::StaticInput) -> Self::Output {
        let grant_cap = capabilities::MemoryAllocationCapability::new();

        let isl29035_i2c = static_init!(I2CDevice, I2CDevice::new(self.i2c_mux, 0x44));
        let isl29035_virtual_alarm = static_init_half!(
            static_buffer.0,
            VirtualMuxAlarm<'static, A>,
            VirtualMuxAlarm::new(self.alarm_mux)
        );
        let isl29035 = static_init_half!(
            static_buffer.1,
            Isl29035<'static, VirtualMuxAlarm<'static, A>>,
            Isl29035::new(isl29035_i2c, isl29035_virtual_alarm, &mut I2C_BUF)
        );
        isl29035_i2c.set_client(isl29035);
        isl29035_virtual_alarm.set_client(isl29035);
        let ambient_light = static_init!(
            AmbientLight<'static>,
            AmbientLight::new(isl29035, self.board_kernel.create_grant(&grant_cap))
        );
        hil::sensors::AmbientLight::set_client(isl29035, ambient_light);
        ambient_light
    }
}
