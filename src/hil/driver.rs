use AppId;
use AppSlice;
use Callback;
use Shared;

pub trait Driver {
    fn subscribe(&mut self, subscribe_type: usize, callback: Callback) -> isize;
    fn command(&mut self, cmd_type: usize, r2: usize) -> isize;

    #[allow(unused)]
    fn allow(&mut self, app: AppId, allow_type: usize, slice: AppSlice<Shared, u8>) -> isize {
        -1
    }
}

