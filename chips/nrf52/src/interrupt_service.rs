/// Interface for handling interrupts on a hardware chip.
///
/// Each chip (or chip version) must implement this trait to handle specific
/// interrupts. When an interrupt (identified by number) has triggered and
/// should be handled, the implementation of this trait will be called with the
/// interrupt number. The implementation can then handle the interrupt, or
/// return `false` to signify that it does not know how to handle the interrupt.
///
/// This functionality is given this `InterruptService` interface so that
/// multiple objects can be chained together to handle interrupts for a chip.
/// This is useful for code organization and removing the need for duplication
/// when multiple variations of a specific microcontroller exist. Then a shared,
/// base object can handle most interrupts, and variation-specific objects can
/// handle the variation-specific interrupts.
///
/// To simplify structuring the Rust code when using `InterruptService`, the
/// interrupt number should be passed "top-down". That is, an interrupt to be
/// handled will first be passed to the `InterruptService` object that is most
/// specific. If that object cannot handle the interrupt, then it should
/// maintain a reference to the second most specific object, and return by
/// calling to that object to handle the interrupt. This continues until the
/// base object handles the interrupt or decides that the chip does not know how
/// to handle the interrupt. For example, consider a `nRF52840` chip that
/// depends on the `nRF52` crate which in turn depends on the `nRF5` crate. If
/// all three have specific interrupts they know how to handle, the flow would
/// look like:
///
/// ```ignore
///           +---->nrf52840
///           |        |
///           |        |
///           |        v
///           |      nrf52
///           |        |
///           |        |
///           |        v
/// kernel-->nrf5     nrf5
/// ```
/// where the kernel instructs the `nrf5` crate to handle interrupts, and if
/// there is an interrupt ready then that interrupt is passed through the crates
/// until something can service it.
pub trait InterruptService {
    /// Service an interrupt, if supported by this chip. If this interrupt number is not supported,
    /// return false.
    unsafe fn service_interrupt(&self, interrupt: u32) -> bool;
}
