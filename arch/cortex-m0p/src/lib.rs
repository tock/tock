//! Shared implementations for ARM Cortex-M0+ MCUs.

#![crate_name = "cortexm0p"]
#![crate_type = "rlib"]
#![no_std]

pub mod mpu {
    pub type MPU = cortexm::mpu::MPU<8>;
}

// Re-export the base generic cortex-m functions here as they are
// valid on cortex-m0.
pub use cortexm::support;

pub use cortexm::initialize_ram_jump_to_main;
pub use cortexm::nvic;
pub use cortexm::print_cortexm_state as print_cortexm0_state;
pub use cortexm::syscall;
pub use cortexm::systick;
pub use cortexm::unhandled_interrupt;
pub use cortexm0::generic_isr;
pub use cortexm0::hard_fault_handler;
pub use cortexm0::svc_handler;
pub use cortexm0::systick_handler;

// Mock implementation for tests on Travis-CI.
#[cfg(not(any(target_arch = "arm", target_os = "none")))]
pub unsafe extern "C" fn switch_to_user(
    _user_stack: *const u8,
    _process_regs: &mut [usize; 8],
) -> *mut u8 {
    unimplemented!()
}
