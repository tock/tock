//! Accessing information about main flash layout, including location
//! of kernel and applications.

pub fn kernel_start_address() -> u32 {
    unsafe {
        extern "C" {
            static _stext: *const u32;
        }
        (&_stext as *const *const u32) as u32
    }
}

pub fn kernel_end_address() -> u32 {
    unsafe {
        extern "C" {
            static _etext: *const u32;
        }
        (&_etext as *const *const u32) as u32
    }
}

pub fn apps_start_address() -> u32 {
    unsafe {
        extern "C" {
            static _sapps: *const u32;
        }
        (&_sapps as *const *const u32) as u32
    }
}

pub fn apps_end_address() -> u32 {
    unsafe {
        extern "C" {
            static _eapps: *const u32;
        }
        (&_eapps as *const *const u32) as u32
    }
}
