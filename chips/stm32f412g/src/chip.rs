use crate::interrupt::Stm32f412gInterruptService;
use stm32f4xx::chip::Stm32f4xx;

pub type Chip = Stm32f4xx<Stm32f412gInterruptService>;

pub unsafe fn new() -> Chip {
    Stm32f4xx::new(Stm32f412gInterruptService::new())
}
