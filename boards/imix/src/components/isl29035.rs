//! Components for the ISL29035 on the imix board.
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
//! let isl = Isl29035Component::new(mux_i2c, mux_alarm).finalize();
//! let ambient_light = AmbientLightComponent::new(mux_i2c, mux_alarm).finalize();
//! ```

// Author: Philip Levis <pal@cs.stanford.edu>
// Last modified: 6/20/2018

#![allow(dead_code)] // Components are intended to be conditionally included

use capsules::ambient_light::AmbientLight;
use capsules::isl29035::Isl29035;
use capsules::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use capsules::virtual_i2c::{I2CDevice, MuxI2C};
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::hil;
use kernel::hil::time::Alarm;
use kernel::static_init;

pub struct Isl29035Component {
    i2c_mux: &'static MuxI2C<'static>,
    alarm_mux: &'static MuxAlarm<'static, sam4l::ast::Ast<'static>>,
}

impl Isl29035Component {
    pub fn new(
        i2c: &'static MuxI2C<'static>,
        alarm: &'static MuxAlarm<'static, sam4l::ast::Ast<'static>>,
    ) -> Self {
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

impl Component for Isl29035Component {
    type Output = &'static Isl29035<'static, VirtualMuxAlarm<'static, sam4l::ast::Ast<'static>>>;

    unsafe fn finalize(&mut self) -> Self::Output {
        let isl29035_i2c = static_init!(I2CDevice, I2CDevice::new(self.i2c_mux, 0x44));
        let isl29035_virtual_alarm = static_init!(
            VirtualMuxAlarm<'static, sam4l::ast::Ast>,
            VirtualMuxAlarm::new(self.alarm_mux)
        );
        let isl29035 = static_init!(
            Isl29035<'static, VirtualMuxAlarm<'static, sam4l::ast::Ast>>,
            Isl29035::new(isl29035_i2c, isl29035_virtual_alarm, &mut I2C_BUF)
        );
        isl29035_i2c.set_client(isl29035);
        isl29035_virtual_alarm.set_client(isl29035);
        isl29035
    }
}

pub struct AmbientLightComponent {
    board_kernel: &'static kernel::Kernel,
    i2c_mux: &'static MuxI2C<'static>,
    alarm_mux: &'static MuxAlarm<'static, sam4l::ast::Ast<'static>>,
}

impl AmbientLightComponent {
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        i2c: &'static MuxI2C<'static>,
        alarm: &'static MuxAlarm<'static, sam4l::ast::Ast<'static>>,
    ) -> Self {
        AmbientLightComponent {
            board_kernel: board_kernel,
            i2c_mux: i2c,
            alarm_mux: alarm,
        }
    }
}

impl Component for AmbientLightComponent {
    type Output = &'static AmbientLight<'static>;

    unsafe fn finalize(&mut self) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        let isl29035_i2c = static_init!(I2CDevice, I2CDevice::new(self.i2c_mux, 0x44));
        let isl29035_virtual_alarm = static_init!(
            VirtualMuxAlarm<'static, sam4l::ast::Ast>,
            VirtualMuxAlarm::new(self.alarm_mux)
        );
        let isl29035 = static_init!(
            Isl29035<'static, VirtualMuxAlarm<'static, sam4l::ast::Ast>>,
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
