pub unsafe extern "C" fn safe_hapi_void(f_ptr: unsafe extern "C" fn()) {
    'loop1: loop {
        if !(*((0x400c8000i32 + 0x0i32) as (*mut usize)) == 0) {
            break;
        }
    }
    f_ptr();
    *((0x400c8000i32 + 0x0i32) as (*mut usize)) = 1usize;
}

#[allow(unused)]
pub unsafe extern "C" fn safe_hapi_aux_adi_select(
    f_ptr: unsafe extern "C" fn(u8),
    mut ut8signal: u8,
) {
    'loop1: loop {
        if !(*((0x400c8000i32 + 0x0i32) as (*mut usize)) == 0) {
            break;
        }
    }
    f_ptr(ut8signal);
    *((0x400c8000i32 + 0x0i32) as (*mut usize)) = 1usize;
}
