//! Component for Real Time Clock initialisation.
//!
//! Usage
//! -----
//! ```rust
//! use components::date_time::DateTimeComponent;
//! let date_time = components::date_time::DateTimeComponent::new(
//!     board_kernel,
//!     &peripherals.rtc,
//!  ).finalize(());
//! ```
//!
//! Author Irina Bradu <irinabradu.a@gmail.com>

use capsules::date_time::DateTime;
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::hil::date_time;
use kernel::static_init;

pub struct DateTimeComponent {
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
    rtc: &'static dyn date_time::DateTime<'static>,
}

impl DateTimeComponent {
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        driver_num: usize,
        rtc: &'static dyn date_time::DateTime<'static>,
    ) -> DateTimeComponent {
        DateTimeComponent {
            board_kernel,
            driver_num,
            rtc,
        }
    }
}

impl Component for DateTimeComponent {
    type StaticInput = ();
    type Output = &'static DateTime<'static>;

    unsafe fn finalize(self, _s: Self::StaticInput) -> Self::Output {
        let grant_dt = create_capability!(capabilities::MemoryAllocationCapability);
        let grant_date_time = self.board_kernel.create_grant(self.driver_num, &grant_dt);

        let date_time = static_init!(
            capsules::date_time::DateTime<'static>,
            capsules::date_time::DateTime::new(self.rtc, grant_date_time)
        );
        date_time::DateTime::set_client(self.rtc, date_time);
        date_time
    }
}
