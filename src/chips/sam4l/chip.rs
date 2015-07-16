use core::prelude::*;
use queue;
use ast;
use gpio;
use adc;
use usart;
use hil;
use nvic;

pub static mut CHIP : Option<Sam4l> = None;

#[allow(dead_code)]
pub struct Sam4l {
    pub queue: queue::InterruptQueue,
    pub ast: ast::Ast,
    pub usarts: [usart::USART; 4],
    pub adc: adc::Adc,
    pub pa00: gpio::GPIOPin, pub pa01: gpio::GPIOPin, pub pa02: gpio::GPIOPin,
    pub pa03: gpio::GPIOPin, pub pa04: gpio::GPIOPin, pub pa05: gpio::GPIOPin,
    pub pa06: gpio::GPIOPin, pub pa07: gpio::GPIOPin, pub pa08: gpio::GPIOPin,
    pub pa09: gpio::GPIOPin, pub pa10: gpio::GPIOPin, pub pa11: gpio::GPIOPin,
    pub pa12: gpio::GPIOPin, pub pa13: gpio::GPIOPin, pub pa14: gpio::GPIOPin,
    pub pa15: gpio::GPIOPin, pub pa16: gpio::GPIOPin, pub pa17: gpio::GPIOPin,
    pub pa18: gpio::GPIOPin, pub pa19: gpio::GPIOPin, pub pa20: gpio::GPIOPin,
    pub pa21: gpio::GPIOPin, pub pa22: gpio::GPIOPin, pub pa23: gpio::GPIOPin,
    pub pa24: gpio::GPIOPin, pub pa25: gpio::GPIOPin, pub pa26: gpio::GPIOPin,
    pub pa27: gpio::GPIOPin, pub pa28: gpio::GPIOPin, pub pa29: gpio::GPIOPin,
    pub pa30: gpio::GPIOPin, pub pa31: gpio::GPIOPin,

    pub pb00: gpio::GPIOPin, pub pb01: gpio::GPIOPin, pub pb02: gpio::GPIOPin,
    pub pb03: gpio::GPIOPin, pub pb04: gpio::GPIOPin, pub pb05: gpio::GPIOPin,
    pub pb06: gpio::GPIOPin, pub pb07: gpio::GPIOPin, pub pb08: gpio::GPIOPin,
    pub pb09: gpio::GPIOPin, pub pb10: gpio::GPIOPin, pub pb11: gpio::GPIOPin,
    pub pb12: gpio::GPIOPin, pub pb13: gpio::GPIOPin, pub pb14: gpio::GPIOPin,
    pub pb15: gpio::GPIOPin, pub pb16: gpio::GPIOPin, pub pb17: gpio::GPIOPin,
    pub pb18: gpio::GPIOPin, pub pb19: gpio::GPIOPin, pub pb20: gpio::GPIOPin,
    pub pb21: gpio::GPIOPin, pub pb22: gpio::GPIOPin, pub pb23: gpio::GPIOPin,
    pub pb24: gpio::GPIOPin, pub pb25: gpio::GPIOPin, pub pb26: gpio::GPIOPin,
    pub pb27: gpio::GPIOPin, pub pb28: gpio::GPIOPin, pub pb29: gpio::GPIOPin,
    pub pb30: gpio::GPIOPin, pub pb31: gpio::GPIOPin,

    pub pc00: gpio::GPIOPin, pub pc01: gpio::GPIOPin, pub pc02: gpio::GPIOPin,
    pub pc03: gpio::GPIOPin, pub pc04: gpio::GPIOPin, pub pc05: gpio::GPIOPin,
    pub pc06: gpio::GPIOPin, pub pc07: gpio::GPIOPin, pub pc08: gpio::GPIOPin,
    pub pc09: gpio::GPIOPin, pub pc10: gpio::GPIOPin, pub pc11: gpio::GPIOPin,
    pub pc12: gpio::GPIOPin, pub pc13: gpio::GPIOPin, pub pc14: gpio::GPIOPin,
    pub pc15: gpio::GPIOPin, pub pc16: gpio::GPIOPin, pub pc17: gpio::GPIOPin,
    pub pc18: gpio::GPIOPin, pub pc19: gpio::GPIOPin, pub pc20: gpio::GPIOPin,
    pub pc21: gpio::GPIOPin, pub pc22: gpio::GPIOPin, pub pc23: gpio::GPIOPin,
    pub pc24: gpio::GPIOPin, pub pc25: gpio::GPIOPin, pub pc26: gpio::GPIOPin,
    pub pc27: gpio::GPIOPin, pub pc28: gpio::GPIOPin, pub pc29: gpio::GPIOPin,
    pub pc30: gpio::GPIOPin, pub pc31: gpio::GPIOPin
}

#[allow(dead_code)]
impl Sam4l {
    pub fn new() -> Sam4l {

        Sam4l {
            queue: queue::InterruptQueue::new(),
            ast: ast::Ast::new(),
            usarts: [
                usart::USART::new(usart::Location::USART0),
                usart::USART::new(usart::Location::USART1),
                usart::USART::new(usart::Location::USART2),
                usart::USART::new(usart::Location::USART3),
            ],
            adc: adc::Adc::new(),
            pa00: gpio::GPIOPin::new(gpio::Pin::PA00),
            pa01: gpio::GPIOPin::new(gpio::Pin::PA01),
            pa02: gpio::GPIOPin::new(gpio::Pin::PA02),
            pa03: gpio::GPIOPin::new(gpio::Pin::PA03),
            pa04: gpio::GPIOPin::new(gpio::Pin::PA04),
            pa05: gpio::GPIOPin::new(gpio::Pin::PA05),
            pa06: gpio::GPIOPin::new(gpio::Pin::PA06),
            pa07: gpio::GPIOPin::new(gpio::Pin::PA07),
            pa08: gpio::GPIOPin::new(gpio::Pin::PA08),
            pa09: gpio::GPIOPin::new(gpio::Pin::PA09),
            pa10: gpio::GPIOPin::new(gpio::Pin::PA10),
            pa11: gpio::GPIOPin::new(gpio::Pin::PA11),
            pa12: gpio::GPIOPin::new(gpio::Pin::PA12),
            pa13: gpio::GPIOPin::new(gpio::Pin::PA13),
            pa14: gpio::GPIOPin::new(gpio::Pin::PA14),
            pa15: gpio::GPIOPin::new(gpio::Pin::PA15),
            pa16: gpio::GPIOPin::new(gpio::Pin::PA16),
            pa17: gpio::GPIOPin::new(gpio::Pin::PA17),
            pa18: gpio::GPIOPin::new(gpio::Pin::PA18),
            pa19: gpio::GPIOPin::new(gpio::Pin::PA19),
            pa20: gpio::GPIOPin::new(gpio::Pin::PA20),
            pa21: gpio::GPIOPin::new(gpio::Pin::PA21),
            pa22: gpio::GPIOPin::new(gpio::Pin::PA22),
            pa23: gpio::GPIOPin::new(gpio::Pin::PA23),
            pa24: gpio::GPIOPin::new(gpio::Pin::PA24),
            pa25: gpio::GPIOPin::new(gpio::Pin::PA25),
            pa26: gpio::GPIOPin::new(gpio::Pin::PA26),
            pa27: gpio::GPIOPin::new(gpio::Pin::PA27),
            pa28: gpio::GPIOPin::new(gpio::Pin::PA28),
            pa29: gpio::GPIOPin::new(gpio::Pin::PA29),
            pa30: gpio::GPIOPin::new(gpio::Pin::PA30),
            pa31: gpio::GPIOPin::new(gpio::Pin::PA31),

            pb00: gpio::GPIOPin::new(gpio::Pin::PB00),
            pb01: gpio::GPIOPin::new(gpio::Pin::PB01),
            pb02: gpio::GPIOPin::new(gpio::Pin::PB02),
            pb03: gpio::GPIOPin::new(gpio::Pin::PB03),
            pb04: gpio::GPIOPin::new(gpio::Pin::PB04),
            pb05: gpio::GPIOPin::new(gpio::Pin::PB05),
            pb06: gpio::GPIOPin::new(gpio::Pin::PB06),
            pb07: gpio::GPIOPin::new(gpio::Pin::PB07),
            pb08: gpio::GPIOPin::new(gpio::Pin::PB08),
            pb09: gpio::GPIOPin::new(gpio::Pin::PB09),
            pb10: gpio::GPIOPin::new(gpio::Pin::PB10),
            pb11: gpio::GPIOPin::new(gpio::Pin::PB11),
            pb12: gpio::GPIOPin::new(gpio::Pin::PB12),
            pb13: gpio::GPIOPin::new(gpio::Pin::PB13),
            pb14: gpio::GPIOPin::new(gpio::Pin::PB14),
            pb15: gpio::GPIOPin::new(gpio::Pin::PB15),
            pb16: gpio::GPIOPin::new(gpio::Pin::PB16),
            pb17: gpio::GPIOPin::new(gpio::Pin::PB17),
            pb18: gpio::GPIOPin::new(gpio::Pin::PB18),
            pb19: gpio::GPIOPin::new(gpio::Pin::PB19),
            pb20: gpio::GPIOPin::new(gpio::Pin::PB20),
            pb21: gpio::GPIOPin::new(gpio::Pin::PB21),
            pb22: gpio::GPIOPin::new(gpio::Pin::PB22),
            pb23: gpio::GPIOPin::new(gpio::Pin::PB23),
            pb24: gpio::GPIOPin::new(gpio::Pin::PB24),
            pb25: gpio::GPIOPin::new(gpio::Pin::PB25),
            pb26: gpio::GPIOPin::new(gpio::Pin::PB26),
            pb27: gpio::GPIOPin::new(gpio::Pin::PB27),
            pb28: gpio::GPIOPin::new(gpio::Pin::PB28),
            pb29: gpio::GPIOPin::new(gpio::Pin::PB29),
            pb30: gpio::GPIOPin::new(gpio::Pin::PB30),
            pb31: gpio::GPIOPin::new(gpio::Pin::PB31),

            pc00: gpio::GPIOPin::new(gpio::Pin::PC00),
            pc01: gpio::GPIOPin::new(gpio::Pin::PC01),
            pc02: gpio::GPIOPin::new(gpio::Pin::PC02),
            pc03: gpio::GPIOPin::new(gpio::Pin::PC03),
            pc04: gpio::GPIOPin::new(gpio::Pin::PC04),
            pc05: gpio::GPIOPin::new(gpio::Pin::PC05),
            pc06: gpio::GPIOPin::new(gpio::Pin::PC06),
            pc07: gpio::GPIOPin::new(gpio::Pin::PC07),
            pc08: gpio::GPIOPin::new(gpio::Pin::PC08),
            pc09: gpio::GPIOPin::new(gpio::Pin::PC09),
            pc10: gpio::GPIOPin::new(gpio::Pin::PC10),
            pc11: gpio::GPIOPin::new(gpio::Pin::PC11),
            pc12: gpio::GPIOPin::new(gpio::Pin::PC12),
            pc13: gpio::GPIOPin::new(gpio::Pin::PC13),
            pc14: gpio::GPIOPin::new(gpio::Pin::PC14),
            pc15: gpio::GPIOPin::new(gpio::Pin::PC15),
            pc16: gpio::GPIOPin::new(gpio::Pin::PC16),
            pc17: gpio::GPIOPin::new(gpio::Pin::PC17),
            pc18: gpio::GPIOPin::new(gpio::Pin::PC18),
            pc19: gpio::GPIOPin::new(gpio::Pin::PC19),
            pc20: gpio::GPIOPin::new(gpio::Pin::PC20),
            pc21: gpio::GPIOPin::new(gpio::Pin::PC21),
            pc22: gpio::GPIOPin::new(gpio::Pin::PC22),
            pc23: gpio::GPIOPin::new(gpio::Pin::PC23),
            pc24: gpio::GPIOPin::new(gpio::Pin::PC24),
            pc25: gpio::GPIOPin::new(gpio::Pin::PC25),
            pc26: gpio::GPIOPin::new(gpio::Pin::PC26),
            pc27: gpio::GPIOPin::new(gpio::Pin::PC27),
            pc28: gpio::GPIOPin::new(gpio::Pin::PC28),
            pc29: gpio::GPIOPin::new(gpio::Pin::PC29),
            pc30: gpio::GPIOPin::new(gpio::Pin::PC30),
            pc31: gpio::GPIOPin::new(gpio::Pin::PC31),
        }
    }

    pub unsafe fn service_pending_interrupts(&mut self) {
        use nvic::NvicIdx;
        let q = &mut self.queue as &mut hil::queue::Queue<nvic::NvicIdx>;
        while q.has_elements() {
           let interrupt = q.dequeue();
           match interrupt {
             NvicIdx::ASTALARM => self.ast.handle_interrupt(),
             NvicIdx::USART3   => self.usarts[3].handle_interrupt(),
             NvicIdx::ADCIFE   => self.adc.handle_interrupt(),
             _ => {}
           }
           nvic::enable(interrupt);
        }
    }

    pub fn has_pending_interrupts(&mut self) -> bool {
        let q = &mut self.queue as &mut hil::queue::Queue<nvic::NvicIdx>;
        q.has_elements()
    }
}

