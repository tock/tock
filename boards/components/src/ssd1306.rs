use core::mem::MaybeUninit;
use capsules_extra::bus;
use capsules_extra::ssd1306;
use kernel::component::Component;
use kernel::dynamic_deferred_call::DynamicDeferredCall;

#[macro_export]
macro_rules! ssd1306_component_static {
    ($B: ty, $(,)?) => {{
        let buffer = kernel::static_buf!([u8; drivers::ssd1306::BUFFER_SIZE]);
        let app_write_buffer = kernel::static_buf!(
            [u8; drivers::ssd1306::WIDTH * drivers::ssd1306::HEIGHT / 8
                + drivers::ssd1306::BUFFER_PADDING]
        );
        let bus_write_buffer = kernel::static_buf!(
            [u8; drivers::ssd1306::WIDTH * drivers::ssd1306::HEIGHT / 8
                + drivers::ssd1306::BUFFER_PADDING]
        );
        let aux_write_buffer = kernel::static_buf!(
            [u8; drivers::ssd1306::WIDTH * drivers::ssd1306::HEIGHT / 8
                + drivers::ssd1306::BUFFER_PADDING]
        );
        let command_sequence = kernel::static_buf!(
            [drivers::ssd1306::ScreenCommand; drivers::ssd1306::SEQUENCE_BUFFER_SIZE]
        );
        let ssd1306 = kernel::static_buf!(drivers::ssd1306::SSD1306<'static, $B>);
        (
            ssd1306,
            command_sequence,
            buffer,
            app_write_buffer,
            bus_write_buffer,
            aux_write_buffer,
        )
    };};
}

pub struct SSD1306Component<B: 'static + bus::Bus<'static>> {
    bus: &'static B,
    deferred_caller: &'static DynamicDeferredCall,
}

impl<B: 'static + bus::Bus<'static>> SSD1306Component<B> {
    pub fn new(
        bus: &'static B,
        deferred_caller: &'static DynamicDeferredCall,
    ) -> SSD1306Component<B> {
        SSD1306Component {
            bus,
            deferred_caller,
        }
    }
}

impl<B: 'static + bus::Bus<'static>> Component for SSD1306Component<B> {
    type StaticInput = (
        &'static mut MaybeUninit<ssd1306::SSD1306<'static, B>>,
        &'static mut MaybeUninit<[ssd1306::ScreenCommand; ssd1306::SEQUENCE_BUFFER_SIZE]>,
        &'static mut MaybeUninit<[u8; ssd1306::BUFFER_SIZE]>,
        &'static mut MaybeUninit<
            [u8; ssd1306::HEIGHT * ssd1306::WIDTH / 8 + ssd1306::BUFFER_PADDING],
        >,
        &'static mut MaybeUninit<
            [u8; ssd1306::HEIGHT * ssd1306::WIDTH / 8 + ssd1306::BUFFER_PADDING],
        >,
        &'static mut MaybeUninit<
            [u8; ssd1306::HEIGHT * ssd1306::WIDTH / 8 + ssd1306::BUFFER_PADDING],
        >,
    );

    type Output = &'static ssd1306::SSD1306<'static, B>;

    fn finalize(self, static_memory: Self::StaticInput) -> Self::Output {
        let command_sequence = static_memory.1.write(
            [ssd1306::ScreenCommand {
                id: ssd1306::CommandId::Nop,
                parameters: None,
            }; ssd1306::SEQUENCE_BUFFER_SIZE],
        );
        let command_arguments = static_memory.2.write([0; ssd1306::BUFFER_SIZE]);
        let app_write_buffer = static_memory
            .3
            .write([0; ssd1306::HEIGHT * ssd1306::WIDTH / 8 + ssd1306::BUFFER_PADDING]);
        let bus_write_buffer = static_memory
            .4
            .write([0; ssd1306::HEIGHT * ssd1306::WIDTH / 8 + ssd1306::BUFFER_PADDING]);
        let aux_write_buffer = static_memory
            .5
            .write([0; ssd1306::HEIGHT * ssd1306::WIDTH / 8 + ssd1306::BUFFER_PADDING]);

        let ssd1306 = static_memory.0.write(ssd1306::SSD1306::new(
            self.bus,
            command_sequence,
            command_arguments,
            app_write_buffer,
            bus_write_buffer,
            aux_write_buffer,
            self.deferred_caller,
        ));
        self.bus.set_client(ssd1306);

        ssd1306.initialize_callback_handle(self.deferred_caller.register(ssd1306).unwrap());
        ssd1306
    }
}
