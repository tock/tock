//! Component for process printers.
//!
//! Usage
//! -----
//! ```rust
//! let process_printer = ProcessPrinterTextComponent::new().finalize(());
//! ```

use kernel::component::Component;
use kernel::static_init;

pub struct ProcessPrinterTextComponent {}

impl ProcessPrinterTextComponent {
    pub fn new() -> ProcessPrinterTextComponent {
        ProcessPrinterTextComponent {}
    }
}

impl Component for ProcessPrinterTextComponent {
    type StaticInput = ();
    type Output = &'static kernel::process::ProcessPrinterText;

    unsafe fn finalize(self, _static_buffer: Self::StaticInput) -> Self::Output {
        static_init!(
            kernel::process::ProcessPrinterText,
            kernel::process::ProcessPrinterText::new()
        )
    }
}
