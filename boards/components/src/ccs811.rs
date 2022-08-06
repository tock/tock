//! Components for the BME280 Humidity, Pressure and Temperature Sensor.
//!
//! Usage
//! -----
//! ```rust
//!     let ccs811 =
//!         Ccs811Component::new(mux_i2c, 0x77).finalize(components::ccs811_component_helper!());
//!     let temperature = components::temperature::TemperatureComponent::new(
//!         board_kernel,
//!         capsules::temperature::DRIVER_NUM,
//!         ccs811,
//!     )
//!     .finalize(());
//!     let humidity = components::humidity::HumidityComponent::new(
//!         board_kernel,
//!         capsules::humidity::DRIVER_NUM,
//!         ccs811,
//!     )
//!     .finalize(());
//! ```

use capsules::ccs811::Ccs811;
use capsules::virtual_i2c::{I2CDevice, MuxI2C};
use core::mem::MaybeUninit;
use kernel::component::Component;
use kernel::dynamic_deferred_call::DynamicDeferredCall;
use kernel::{static_init, static_init_half};

// Setup static space for the objects.
#[macro_export]
macro_rules! ccs811_component_helper {
    () => {{
        use capsules::ccs811::Ccs811;
        use core::mem::MaybeUninit;
        static mut BUF1: MaybeUninit<Ccs811<'static>> = MaybeUninit::uninit();
        &mut BUF1
    };};
}

pub struct Ccs811Component {
    i2c_mux: &'static MuxI2C<'static>,
    i2c_address: u8,
    deferred_caller: &'static DynamicDeferredCall,
}

impl Ccs811Component {
    pub fn new(
        i2c: &'static MuxI2C<'static>,
        i2c_address: u8,
        deferred_caller: &'static DynamicDeferredCall,
    ) -> Self {
        Ccs811Component {
            i2c_mux: i2c,
            i2c_address,
            deferred_caller,
        }
    }
}

static mut I2C_BUF: [u8; 6] = [0; 6];

impl Component for Ccs811Component {
    type StaticInput = &'static mut MaybeUninit<Ccs811<'static>>;
    type Output = &'static Ccs811<'static>;

    unsafe fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let ccs811_i2c = static_init!(I2CDevice, I2CDevice::new(self.i2c_mux, self.i2c_address));
        let ccs811 = static_init_half!(
            static_buffer,
            Ccs811<'static>,
            Ccs811::new(ccs811_i2c, &mut I2C_BUF, self.deferred_caller)
        );

        ccs811_i2c.set_client(ccs811);
        ccs811.initialize_callback_handle(
            self.deferred_caller.register(ccs811).unwrap(), // Unwrap fail = no deferred call slot available for CCS811
        );
        ccs811.startup();
        ccs811
    }
}
