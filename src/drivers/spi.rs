use core::cell::RefCell;
use hil::{AppId,Driver,Callback,AppSlice,Shared,NUM_PROCS};
use hil::spi_master::{SpiMaster,SpiCallback};

/* SPI operations are handled by coping into a kernel buffer for
 * writes and copying out of a kernel buffer for reads.
 *
 * If the application buffer is larger than the kernel buffer,
 * the driver issues multiple HAL operations. The len field
 * of an application keeps track of the length of the desired
 * operation, while the index variable keeps track of the 
 * index an ongoing operation is at in the buffers. */

const BUF_SIZE : usize = 128;

struct App {
    callback: Option<Callback>,
    app_read: Option<AppSlice<Shared, u8>>,
    app_write: Option<AppSlice<Shared, u8>>,
    len: usize,
    index: usize,
}

pub struct Spi<'a, S: SpiMaster + 'a> {
    spi_master: &'a mut S,
    busy:  bool,
    apps: [RefCell<Option<App>>; NUM_PROCS],
    kernel_read: [u8; BUF_SIZE],
    kernel_write: [u8; BUF_SIZE],
}

impl<'a, S: SpiMaster> Spi<'a, S> {
    pub fn new(spi_master: &'a mut S) -> Spi<S> {
        Spi {
            spi_master: spi_master,
            busy: false,
            apps: [RefCell::new(None)],
            kernel_read : [0u8; BUF_SIZE],
            kernel_write : [0u8; BUF_SIZE],
        }
    }
}

impl<'a, S: SpiMaster> Driver for Spi<'a, S> {
    fn allow(&self, appid: AppId,
             allow_num: usize, slice: AppSlice<Shared, u8>) -> isize {
        let app = appid.idx();
        match allow_num {
            0 => {
                let mut appc = self.apps[app].borrow_mut();
                if appc.is_none() {
                    *appc = Some(App {
                        callback: None,
                        app_read: Some(slice),
                        app_write: None,
                        len: 0,
                        index: 0,
                    })
                } else {
                    appc.as_mut().map(|app| {
                        app.app_read = Some(slice);
                    });
                }
                0
            },
            1 => {
                let mut appc = self.apps[app].borrow_mut();
                if appc.is_none() {
                    *appc = Some(App {
                        callback: None,
                        app_read: None,
                        app_write: Some(slice),
                        len: 0,
                        index: 0,
                    })
                } else {
                    appc.as_mut().map(|app| app.app_write = Some(slice) );
                }
                0
            }
            _ => -1
        }
    }

    #[inline(never)]
    fn subscribe(&self, subscribe_num: usize, callback: Callback) -> isize {
        match subscribe_num {
            0 /* read_write */ => {
                let mut app = self.apps[0].borrow_mut();
                if app.is_none() {
                    *app = Some(App {
                        callback: Some(callback),
                        app_read: None,
                        app_write: None,
                        len: 0,
                        index: 0,
                    });
                } else {
                    app.as_mut().map(|a| a.callback = Some(callback) );
                }
                0
            },
            _ => -1
        }
    }

    fn command(&self, cmd_num: usize, arg1: usize) -> isize {
        match cmd_num {
            0 /* read_write */ => { 
                self.spi_master.read_write_byte(arg1 as u8); 
                1
            },
            _ => -1
        }
    }
}

fn each_some<'a, T, I, F>(lst: I, f: F)
        where T: 'a, I: Iterator<Item=&'a RefCell<Option<T>>>, F: Fn(&mut T) {
    for item in lst {
        item.borrow_mut().as_mut().map(|i| f(i));
    }
}

impl<'a, S: SpiMaster> SpiCallback for Spi<'a, S> {
    fn read_write_done(&self) {
        self.apps[0].borrow_mut().as_mut().map(|app| {
            app.callback.take().map(|mut cb| {
                cb.schedule(app.len, 0, 0);
            });
            app.len = 0;
        });
    }
}

