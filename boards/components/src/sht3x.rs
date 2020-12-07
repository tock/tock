use capsules::sht3x::SHT3x;
use capsules::virtual_i2c::I2CDevice;
use core::mem::MaybeUninit;
use kernel::component::Component;
use kernel::static_init_half;

// Setup static space for the objects.
#[macro_export]
macro_rules! sht3x_i2c_component_helper {
    ($i2c_mux: expr) => {{
        use capsules::sht3x::SHT3x;
        use capsules::virtual_i2c::I2CDevice;
        use core::mem::MaybeUninit;
        let temperature_humidity_i2c =
            components::i2c::I2CComponent::new($i2c_mux, SHT3x::BASE_ADDR)
                .finalize(components::i2c_component_helper!());
        static mut sht3x: MaybeUninit<SHT3x<'static>> = MaybeUninit::uninit();
        (&temperature_humidity_i2c, &mut sht3x)
    };};
}

pub struct SHT3xI2CComponent {}

impl SHT3xI2CComponent {
    pub fn new() -> SHT3xI2CComponent {
        SHT3xI2CComponent {}
    }
}

impl Component for SHT3xI2CComponent {
    type StaticInput = (
        &'static I2CDevice<'static>,
        &'static mut MaybeUninit<SHT3X<'static>>,
    );
    type Output = &'static SHT3X<'static>;

    unsafe fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let sht3x = static_init_half!(
            static_buffer.1,
            SHT3x<'static>,
            SHT3x::new(static_buffer.0, &mut capsules::sht3x::BUFFER)
        );
        static_buffer.0.set_client(sht3x);

        sht3x
    }
}
