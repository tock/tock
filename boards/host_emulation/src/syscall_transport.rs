use crate::ipc_syscalls::{self as ipc, IntoIpcMsgType};
use zerocopy::{AsBytes, FromBytes, Unaligned};

use std::os::unix::net::UnixDatagram;
use std::path::{Path, PathBuf};

use crate::Result;
use std::thread::sleep;
use std::time::Duration;

#[derive(Default)]
pub struct SyscallTransport {
    rx_path: PathBuf,
    tx_path: PathBuf,
    rx: Option<UnixDatagram>,
    tx: Option<UnixDatagram>,
}

impl SyscallTransport {
    pub fn open(rx_path: PathBuf, tx_path: PathBuf) -> Result<SyscallTransport> {
        let tx = UnixDatagram::unbound()?;
        let rx = UnixDatagram::bind(&rx_path)?;

        Ok(SyscallTransport {
            rx_path,
            tx_path,
            rx: Some(rx),
            tx: Some(tx),
        })
    }

    pub fn tx_path(&self) -> &Path {
        self.tx_path.as_path()
    }

    pub fn rx_path(&self) -> &Path {
        self.rx_path.as_path()
    }

    pub fn send_bytes(&self, _id: usize, bytes: &[u8]) {
        ipc::send_bytes(&self.tx.as_ref().unwrap(), bytes);
    }

    pub fn send_msg<T>(&self, id: usize, msg: &T)
    where
        T: AsBytes + Sized + IntoIpcMsgType + std::fmt::Debug,
    {
        ipc::send_msg::<T>(&self.tx.as_ref().unwrap(), id, msg);
    }

    pub fn recv_msg<T>(&self) -> T
    where
        T: Sized + Clone + FromBytes + Unaligned + IntoIpcMsgType + std::fmt::Debug,
    {
        ipc::recv_msg::<T>(&self.rx.as_ref().unwrap())
    }

    pub fn recv_bytes(&self, buf: &mut [u8]) -> usize {
        ipc::recv_bytes(&self.rx.as_ref().unwrap(), buf)
    }

    pub fn wait_for_connection(&self) {
        let mut cnt = 0;
        while cnt < 100 {
            if self.tx.as_ref().unwrap().connect(&self.tx_path).is_ok() {
                return;
            }
            sleep(Duration::from_millis(100));
            cnt = cnt + 1;
        }
        panic!("Connection failed");
    }
}
