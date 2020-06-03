
// pub struct NineDofComponent {
//     board_kernel: &'static kernel::Kernel,
//     i2c_mux: &'static MuxI2C<'static>,
//     gpio: &'static dyn gpio::InterruptPin,
// }

// impl NineDofComponent {
//     pub fn new(
//         board_kernel: &'static kernel::Kernel,
//         i2c: &'static MuxI2C<'static>,
//         gpio: &'static dyn gpio::InterruptPin,
//     ) -> NineDofComponent {
//         NineDofComponent {
//             board_kernel: board_kernel,
//             i2c_mux: i2c,
//             gpio: gpio,
//         }
//     }
// }

// impl Component for NineDofComponent {
//     type StaticInput = ();
//     type Output = &'static NineDof<'static>;

//     unsafe fn finalize(self, _s: Self::StaticInput) -> Self::Output {
//         let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

//         let fxos8700_i2c = static_init!(I2CDevice, I2CDevice::new(self.i2c_mux, 0x1f));
//         let fxos8700 = static_init!(
//             fxos8700cq::Fxos8700cq<'static>,
//             fxos8700cq::Fxos8700cq::new(fxos8700_i2c, self.gpio, &mut fxos8700cq::BUF)
//         );
//         fxos8700_i2c.set_client(fxos8700);
//         self.gpio.set_client(fxos8700);

//         let ninedof = static_init!(
//             NineDof<'static>,
//             NineDof::new(fxos8700, self.board_kernel.create_grant(&grant_cap))
//         );
//         hil::sensors::NineDof::set_client(fxos8700, ninedof);

//         ninedof
//     }
// }
