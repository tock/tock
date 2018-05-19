use sam4l;
use capsules::isl29035::Isl29035;
use capsules::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use capsules::virtual_i2c::{I2CDevice, MuxI2C};
use kernel::component::Component;

pub struct Isl29035Component {
    i2c_mux: &'static MuxI2C<'static>,
    alarm_mux: &'static MuxAlarm<'static, sam4l::ast::Ast<'static>>,
}

impl Isl29035Component {
    pub fn new(i2c: &'static MuxI2C<'static>, alarm: &'static MuxAlarm<'static, sam4l::ast::Ast<'static>>) -> Self {
        Isl29035Component {
            i2c_mux: i2c,
            alarm_mux: alarm,
        }
    }
}

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
            Isl29035::new(
                isl29035_i2c,
                isl29035_virtual_alarm,
                &mut I2C_BUF
            )
        );
        isl29035_i2c.set_client(isl29035);
        isl29035_virtual_alarm.set_client(isl29035);
        isl29035
    }
}

