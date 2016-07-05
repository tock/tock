/// A dummy flash client to test flashcalw functionality at the platofrm level (so board/chip specific)
use sam4l::flashcalw;
use hil::flash::{FlashController, Client};

// tests the flash driver for the flashcalw...
pub unsafe fn flash_dummy_test() {
    flashcalw::flash_controller.configure();
    assert_eq!(flashcalw::flash_controller.get_page_size(), 512);
    assert_eq!(flashcalw::flash_controller.get_flash_size(), 1024);

    let mut buff : [usize;129] = [0; 129];
    let mut buffy : [u8;129] = [0; 129];
    flashcalw::flash_controller.write_page(42, &buffy);
    flashcalw::flash_controller.read_page(42, &mut buff);
    for i in 0..129 {
        assert_eq!(buff[i], 0);   
    }
    
    flashcalw::flash_controller.erase_page(42);
    flashcalw::flash_controller.read_page(42, &mut buff);
    for i in 0..129 {
        assert_eq!(buff[i], 1);
    }
    flashcalw::flash_controller.read_page(42, &mut buff);

}
