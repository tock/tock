//! Tock 2.0 callback swapping prevention capsule

use core::mem;
use kernel::{
    AppId, Callback, CommandReturn, Driver, ErrorCode, Grant, GrantDefault, ProcessCallbackFactory,
};

pub const DRIVER_NUM: usize = crate::driver::NUM::CallbackSwapTest as usize;

//#[derive(GrantDefault)]
pub struct App {
    // Just a standard callback, should always work
    //#[subscribe_num = 0]
    callback_0: Callback,

    // Those two callbacks will be swapped on command 1
    //#[subscribe_num = 1]
    callback_1: Option<Callback>,
    //#[subscribe_num = 2]
    callback_2: Callback,

    // This callback has a wrong subscribe_num associated with it
    //#[subscribe_num = 4]
    callback_3: Callback,
}
impl GrantDefault for App {
    fn grant_default(_process_id: AppId, cb_factory: &mut ProcessCallbackFactory) -> Self {
	App {
	    callback_0: cb_factory.build_callback(0).unwrap(),
	    callback_1: Some(cb_factory.build_callback(1).unwrap()),
	    callback_2: cb_factory.build_callback(2).unwrap(),
	    callback_3: cb_factory.build_callback(4).unwrap(),
	}
    }
}


struct CallbackSwapTest(Grant<App>);

impl CallbackSwapTest {
    pub fn new(grant: Grant<App>) -> Self {
        CallbackSwapTest(grant)
    }
}

impl Driver for CallbackSwapTest {
    fn subscribe(
        &self,
        subscribe_num: usize,
        mut callback: Callback,
        app_id: AppId,
    ) -> Result<Callback, (Callback, ErrorCode)> {
        let res = self
            .0
            .enter(app_id, |app, _| match subscribe_num {
                0 => {
                    mem::swap(&mut app.callback_0, &mut callback);
                    Ok(())
                }
                1 => {
		    let mut callback_1 = app.callback_1.take().unwrap();
                    mem::swap(&mut callback_1, &mut callback);
		    app.callback_1.replace(callback_1);
                    Ok(())
                }
                2 => {
                    mem::swap(&mut app.callback_2, &mut callback);
                    Ok(())
                }
                3 => {
                    mem::swap(&mut app.callback_3, &mut callback);
		    Ok(())
                },
		_ => Err(ErrorCode::NOSUPPORT),
            })
            .unwrap_or_else(|err| Err(err.into()));

	match res {
	    Ok(()) => Ok(callback),
	    Err(e) => Err((callback, e)),
	}
    }

    fn command(&self, command_num: usize, _: usize, _: usize, app_id: AppId) -> CommandReturn {
        match command_num {
	    0 /* Check if exists */ => {
		CommandReturn::success()
	    },
	    1 /* Swap callbacks 1 and 2 */ => {
		self.0.enter(app_id, |app, _| {
		    let mut callback = app.callback_1.take().unwrap();
		    mem::swap(&mut callback, &mut app.callback_2);
		    app.callback_1.replace(callback);
		    CommandReturn::success()
		}).unwrap_or_else(|err| CommandReturn::failure(err.into()))
	    },
	    _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
	}
    }
}
