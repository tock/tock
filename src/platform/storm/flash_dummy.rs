/// A dummy flash client to test flashcalw functionality at the platofrm level (so board/chip specific)
use sam4l::flashcalw;
use hil::flash::{FlashController, Client};
use core::mem;


// tests the flash driver for the flashcalw...
pub unsafe fn flash_dummy_test() {
    
    println!("Flashcalw Sam4L testing beginning...");
    println!("Configuring...");
    flashcalw::flash_controller.configure();
    
    println!("Testing Meta Info...");
    assert_eq!(flashcalw::flash_controller.get_page_size(), 512);
    assert_eq!(flashcalw::flash_controller.get_number_pages(), 1024);

    println!("Testing Read, Write and Erase...");
    println!("\tTest One:");
    let mut buff : [usize;128] = [0; 128];
    let mut buffy : [u8;512] = [0; 512];
    flashcalw::flash_controller.write_page(42, &buffy);
    println!("\t\tWritten page");
    flashcalw::flash_controller.read_page(42, &mut buff);
    println!("\t\tRead page");
    for i in 0..128 {
        println!("\t\t\t{}", buff[i]);
        //assert_eq!(buff[i], 0);   
    }
    assert!(false);
    println!("\tTest Two:");
    flashcalw::flash_controller.erase_page(42);
    flashcalw::flash_controller.read_page(42, &mut buff);
    for i in 0..128 {
        assert_eq!(buff[i], 1);
    }
    flashcalw::flash_controller.read_page(42, &mut buff);
    
    println!("\tTest Three:");
    //generate a random page and see that it's also written correctly
    for i in 0..512 {
        buffy[i] = 4;
        //buffy[i] = rand::random::<u8>();
    }
    flashcalw::flash_controller.write_page(42, &buffy);
    flashcalw::flash_controller.read_page(42, &mut buff);

    let mut checker : [u8;512] = unsafe { mem::transmute(buff) };

    for i in 0..512{
        assert_eq!(buffy[i], checker[i]);
    }
    
    println!("Done testing Sam4L Flashcalw.");

}
