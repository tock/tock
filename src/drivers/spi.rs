use core::cell::RefCell;
use core::cell::Cell;
use hil::{AppId,Driver,Callback,AppSlice,Shared,NUM_PROCS};
use hil::spi_master::{SpiMaster,SpiCallback};
use core::cmp;


/* SPI operations are handled by coping into a kernel buffer for
 * writes and copying out of a kernel buffer for reads.
 *
 * If the application buffer is larger than the kernel buffer,
 * the driver issues multiple HAL operations. The len field
 * of an application keeps track of the length of the desired
 * operation, while the index variable keeps track of the 
 * index an ongoing operation is at in the buffers. */

struct App {
    callback:  Option<Callback>,
    app_read:  Option<AppSlice<Shared, u8>>,
    app_write: Option<AppSlice<Shared, u8>>,
    len:       Cell<usize>,
    index:     Cell<usize>,
}

pub struct Spi<'a, S: SpiMaster + 'a> {
    spi_master:   &'a mut S,
    busy:         Cell<bool>,
    apps:         [RefCell<Option<App>>; NUM_PROCS],
    kernel_read:  RefCell<Option<&'static mut [u8]>>,
    kernel_write: RefCell<Option<&'static mut [u8]>>,
    kernel_len:   Cell<usize>
}

impl<'a, S: SpiMaster> Spi<'a, S> {
    pub fn new(spi_master: &'a mut S) -> Spi<S> {
        Spi {
            spi_master: spi_master,
            busy: Cell::new(false),
            apps: [RefCell::new(None)],
            kernel_len: Cell::new(0),
            kernel_read : RefCell::new(None),
            kernel_write : RefCell::new(None),
        }
    }

    pub fn config_buffers(&mut self,
                          read: &'static mut [u8],
                          write: &'static mut [u8]) {
        let len = cmp::min(read.len(), write.len());
        self.kernel_len.set(len);
        self.kernel_read = RefCell::new(Some(read));
        self.kernel_write = RefCell::new(Some(write));
    }

    // Assumes checks for busy/etc. already done
    // Updates app.index to be index + length of op 
    fn do_next_read_write(&self, app: &mut App) {
        let start = app.index.get();
        let len = cmp::min(app.len.get() - start, self.kernel_len.get());
        let end = start + len;
        app.index.set(end);
        let mut kwrite = self.kernel_write.borrow_mut();
        let mut kread  = self.kernel_read.borrow_mut();
        {
            use core::slice::bytes::copy_memory;
            let src = app.app_write.as_mut().unwrap();
            let mut kwbuf = kwrite.as_mut().unwrap();
            copy_memory(&src.as_ref()[start .. end], kwbuf);
        }
        let reading = app.app_read.is_some();
        if reading {
            let mut kwrite = self.kernel_read.borrow_mut();
            self.spi_master.read_write_bytes(kwrite.take(), kread.take(), len);
        } else {
            self.spi_master.read_write_bytes(kwrite.take(), None, len);
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
                        len: Cell::new(0),
                        index: Cell::new(0),
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
                        len: Cell::new(0),
                        index: Cell::new(0),
                    })
                } else {
                    appc.as_mut().map(|app| app.app_write = Some(slice));
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
                        len: Cell::new(0),
                        index: Cell::new(0),
                    });
                } else {
                    app.as_mut().map(|a| a.callback = Some(callback));
                }
                0
            },
            _ => -1
        }
    }

    fn command(&self, cmd_num: usize, arg1: usize) -> isize {
        if self.busy.get() {
            return -1;
        }
        match cmd_num {
            0 /* read_write_byte */ => { 
                self.spi_master.read_write_byte(arg1 as u8) as isize
            },
            1 /* read_write_bytes */ => { 
                let mut app = self.apps[0].borrow_mut();
                if app.is_none() {
                    return -1;
                }
                app.as_mut().map(|mut a| {
                    // If no write buffer, return
                    if a.app_write.is_none() {
                        return -1;
                    }
                    // If write buffer too small, return
                    a.app_write.as_mut().map(|w| {
                        if w.len() < arg1 {
                            return -1;
                        }
                        0
                    });
                    a.len.set(arg1);
                    a.index.set(0);
                    self.busy.set(true);
                    self.do_next_read_write(&mut a as &mut App);
                    0
                }); 
                -1
            }
            _ => -1
        }
    }
}

#[allow(dead_code)]
fn each_some<'a, T, I, F>(lst: I, f: F)
        where T: 'a, I: Iterator<Item=&'a RefCell<Option<T>>>, F: Fn(&mut T) {
    for item in lst {
        item.borrow_mut().as_mut().map(|i| f(i));
    }
}

impl<'a, S: SpiMaster> SpiCallback for Spi<'a, S> {
    fn read_write_done(&self, 
                       writebuf: Option<&'static mut [u8]>, 
                       readbuf:  Option<&'static mut [u8]>,
                       length: usize) {
        self.apps[0].borrow_mut().as_mut().map(|app| {
            if app.app_read.is_some() {
                use core::slice::bytes::copy_memory;
                let src = readbuf.as_ref().unwrap();
                let dest = app.app_read.as_mut().unwrap(); 
                let start = app.index.get() - length;
                let end = start + length;
                copy_memory(&src[0 .. length], &mut dest.as_mut()[start .. end]);
            }

            *self.kernel_read.borrow_mut() =  readbuf;
            *self.kernel_write.borrow_mut() = writebuf;

            if app.index.get() == app.len.get() {
                app.callback.take().map(|mut cb| {
                    self.busy.set(false);
                    cb.schedule(app.len.get(), 0, 0);
                });
                app.len.set(0);
                app.index.set(0);
            } else {
                self.do_next_read_write(app);
            }
        });
    }
}

