/// A dummy flash client to test flashcalw functionality at the platofrm level (so board/chip specific)
use sam4l::flashcalw;
use hil::flash::{FlashController, Client};
use core::mem;



// tests the flash driver for the flashcalw...
pub unsafe fn flash_dummy_test() {
    
    println!("Flashcalw Sam4L testing beginning...");
    
    println!("Configuring...");
    flashcalw::flash_controller.configure();
    //unlock any locks on the flash to all writing...
    flashcalw::flash_controller.lock_all_regions(false);
    println!("Configured!");

    println!("Testing Meta Info...");
    test_meta_info();
    println!("Passed Meta Info...");

    println!("Testing Read, Write and Erase");
    
    println!("\tTesting basic r/w to a page ONCE!");
    test_read_write(40, 4);
    test_read_write(41, 5);
    test_read_write(42, 6);
    println!("\tPassed basic r/w ONCE!");

    
    //testing with QPR
    assert_eq!(flashcalw::flash_controller.quick_page_read(40), false);
    assert_eq!(flashcalw::flash_controller.quick_page_read(41), false);
    assert_eq!(flashcalw::flash_controller.quick_page_read(42), false);

    println!("\tTesting erase, quick read and read");
    test_erase(40);
    test_erase(41);

    println!("\tTesting erase & read");
    test_erase_read(43);
    test_erase_read(42);
    test_erase_read(41);
    test_erase_read(40);
    println!("\tPassed erase & read!");
    
    println!("All literate! Passed Read, Write and Erase!");

    println!("Done testing Sam4L Flashcalw.");

}


/// This function primarily tests meta information for the chip on the 
/// the FireStorm - ATSAM4LC8C. For other ATSAM4L chips, calculations using the 
/// flash size asserts might fail (as they might not have the same flash size).
pub unsafe fn test_meta_info() {
    assert_eq!(flashcalw::flash_controller.get_page_size(), 512);
    assert_eq!(flashcalw::flash_controller.get_flash_size(), 512 << 10);
    assert_eq!(flashcalw::flash_controller.get_number_pages(), 1024);
    assert_eq!(flashcalw::flash_controller.get_page_count_per_region(), 64);
    let mut pg_num = 0u32;
    for i in 0..16 {
        assert_eq!(flashcalw::flash_controller.get_page_region(pg_num as i32), i);
        assert_eq!(flashcalw::flash_controller.get_region_first_page_number(i),
            pg_num);
        pg_num += 64;
    }
}

/// Erases page page_num and test whether both QPR and Read confirm it's been erased...
pub unsafe fn test_erase(page_num : i32) {
    flashcalw::flash_controller.enable_ahb();
    let qpr_read = flashcalw::flash_controller.flashcalw_erase_page(page_num, true);
    if !qpr_read { assert!(false); }
    
    println!("Erased page {} with error code:{}.", page_num, 
        flashcalw::flash_controller.debug_error_status());

    let mut read_buffer = [0usize; 128];
    let expected_bit_pattern = [255u8; 512];
    flashcalw::flash_controller.read_page(page_num as usize, &mut read_buffer);
    
    let output_bits : [u8;512]  = mem::transmute(read_buffer);

    //compare raw bits
    for i in 0..512 {
        assert_eq!(expected_bit_pattern[i], output_bits[i]);
    }
}

/// Erases a page and tests whether the page is now 'erased'.
pub unsafe fn test_erase_read(page_num : i32) {
    flashcalw::flash_controller.erase_page(page_num);
    
    let mut read_buffer = [0usize; 128];
    let expected_bit_pattern = [255u8; 512];
    flashcalw::flash_controller.read_page(page_num as usize, &mut read_buffer);
    
    let output_bits : [u8;512]  = mem::transmute(read_buffer);

    //compare raw bits
    for i in 0..512 {
        assert_eq!(expected_bit_pattern[i], output_bits[i]);
    }
}

/// Writes some u8 value repeatedly to a page, and see tests whether
/// the same pattern is read.
pub unsafe fn test_read_write(page_num : i32, value : u8) {
    let expected_bit_pattern = [value; 512];
    let read_val = write_and_read(page_num, value);
    let output_bits : [u8;512]  = mem::transmute(read_val);

    //compare raw bits
    for i in 0..512 {
        assert_eq!(expected_bit_pattern[i], output_bits[i]);
    }
}

/// Writes a particular value to some page and returns what is read from that
/// value.
pub unsafe fn write_and_read(page_num : i32, value : u8) -> [usize;128] {
    let buff = [value; 512];
    let mut read_buffer = [0usize; 128];
    //write to the page...
    flashcalw::flash_controller.write_page(page_num as usize, &buff);
    flashcalw::flash_controller.read_page(page_num as usize, &mut read_buffer);
    read_buffer
}
