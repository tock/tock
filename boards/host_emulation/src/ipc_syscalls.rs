use crate::{log, log_dbg};
use std::os::unix::net::UnixDatagram;
use zerocopy::{AsBytes, FromBytes, LayoutVerified, Unaligned};

#[allow(dead_code)]
enum SyscallNum {
    YIELD,
    SUBSCIBE,
    COMMAND,
    ALLOW,
    MEMOP,
}

#[repr(C, packed)]
#[derive(Unaligned, AsBytes, FromBytes, Default, Debug, Copy, Clone)]
pub struct Syscall {
    identifier: usize,
    pub syscall_number: usize,
    pub args: [usize; 4],
}

#[repr(C, packed)]
#[derive(Unaligned, AsBytes, FromBytes, Default, Debug, Copy, Clone)]
pub struct Callback {
    pc: usize,
    args: [usize; 4],
}

#[repr(C, packed)]
#[derive(Unaligned, AsBytes, FromBytes, Default, Debug, Copy, Clone)]
pub struct KernelReturn {
    ret_val: isize,
    cb: Callback,
}

#[repr(C, packed)]
#[derive(Unaligned, AsBytes, FromBytes, Default, Debug, Copy, Clone)]
pub struct AllowsInfo {
    pub number_of_slices: usize,
}

#[repr(C, packed)]
#[derive(Unaligned, AsBytes, FromBytes, Default, Debug, Copy, Clone)]
pub struct AllowSliceInfo {
    pub address: usize,
    pub length: usize,
}

#[repr(C, packed)]
#[derive(Unaligned, AsBytes, FromBytes, Default, Debug, Copy, Clone)]
pub struct Ready {
    pub status_ok: u32,
}

impl Syscall {}

impl Callback {
    pub const fn new(pc: usize, arg0: usize, arg1: usize, arg2: usize, arg3: usize) -> Callback {
        Callback {
            pc,
            args: [arg0, arg1, arg2, arg3],
        }
    }

    pub const fn new0(pc: usize) -> Callback {
        Callback::new1(pc, 0)
    }

    pub const fn new1(pc: usize, arg0: usize) -> Callback {
        Callback::new2(pc, arg0, 0)
    }

    pub const fn new2(pc: usize, arg0: usize, arg1: usize) -> Callback {
        Callback::new3(pc, arg0, arg1, 0)
    }

    pub const fn new3(pc: usize, arg0: usize, arg1: usize, arg2: usize) -> Callback {
        Callback::new(pc, arg0, arg1, arg2, 0)
    }
}

impl KernelReturn {
    const fn new(ret_val: isize, cb: Callback) -> KernelReturn {
        KernelReturn { ret_val, cb }
    }

    pub const fn new_ret(ret_val: isize) -> KernelReturn {
        KernelReturn::new(ret_val, Callback::new0(0))
    }

    pub const fn new_cb(cb: Callback) -> KernelReturn {
        KernelReturn::new(0, cb)
    }
}

impl AllowSliceInfo {
    pub const fn new(address: usize, length: usize) -> AllowSliceInfo {
        AllowSliceInfo { address, length }
    }
}

pub const IPC_MSG_HDR_MAGIC: u16 = 0xA55A;

#[repr(C)]
pub enum IpcMsgType {
    SYSCALL,
    KERNELRETURN,
    ALLOWSINFO,
    ALLOWSLICEINFO,
    READY,
}

pub trait IntoIpcMsgType {
    fn to_ipc_msg_type() -> IpcMsgType;
}

#[repr(C, packed)]
#[derive(Unaligned, AsBytes, FromBytes, Default, Debug, Copy, Clone)]
pub struct IpcMsgHeader {
    pub magic: u16,
    pub msg_len: u16,
    pub msg_type: u16,
    pub msg_cksum: u16,
}

impl IpcMsgHeader {
    pub fn new(msg_len: u16, msg_type: u16) -> IpcMsgHeader {
        IpcMsgHeader {
            magic: IPC_MSG_HDR_MAGIC,
            msg_len,
            msg_type,
            msg_cksum: IPC_MSG_HDR_MAGIC + msg_len + msg_type,
        }
    }
}

impl IntoIpcMsgType for Syscall {
    fn to_ipc_msg_type() -> IpcMsgType {
        IpcMsgType::SYSCALL
    }
}

impl IntoIpcMsgType for KernelReturn {
    fn to_ipc_msg_type() -> IpcMsgType {
        IpcMsgType::KERNELRETURN
    }
}

impl IntoIpcMsgType for AllowsInfo {
    fn to_ipc_msg_type() -> IpcMsgType {
        IpcMsgType::ALLOWSINFO
    }
}

impl IntoIpcMsgType for AllowSliceInfo {
    fn to_ipc_msg_type() -> IpcMsgType {
        IpcMsgType::ALLOWSLICEINFO
    }
}

impl IntoIpcMsgType for Ready {
    fn to_ipc_msg_type() -> IpcMsgType {
        IpcMsgType::READY
    }
}

pub fn send_raw(socket: &UnixDatagram, bytes: &[u8]) {
    let sent = match socket.send(bytes) {
        Ok(len) => len,
        Err(e) => {
            panic!("socket send err {}", e);
        }
    };
    if sent != bytes.len() {
        panic!(
            "EmulationError send partialMessage {} expected {} ",
            sent,
            bytes.len()
        );
    }
}

pub fn send_bytes(socket: &UnixDatagram, bytes: &[u8]) {
    log_dbg!("SEND: bytes {:X?}", bytes);
    send_raw(socket, bytes);
}

pub fn send_msg<T>(socket: &UnixDatagram, _id: usize, msg: &T)
where
    T: AsBytes + Sized + IntoIpcMsgType + std::fmt::Debug,
{
    let ipc_len = std::mem::size_of::<T>() as u16;
    let ipc_type = T::to_ipc_msg_type() as u16;
    let ipc_hdr = IpcMsgHeader::new(ipc_len, ipc_type);

    log_dbg!("HDR: SEND: msg {:x?}", ipc_hdr);
    send_raw(socket, ipc_hdr.as_bytes());
    log_dbg!("SEND: msg {:x?}", msg);
    send_raw(socket, msg.as_bytes());
}

pub fn recv_bytes(sock: &UnixDatagram, buf: &mut [u8]) -> usize {
    let rx_len = match sock.recv(buf) {
        Ok(len) => len,
        Err(e) => {
            panic!("APP  : recv_bytes: error {}", e);
        }
    };
    if rx_len != buf.len() {
        panic!(
            "APP  : recv_bytes got {} bytes expected {}",
            rx_len,
            buf.len()
        );
    }
    return rx_len;
}

pub fn recv_raw(sock: &UnixDatagram, buf: &mut std::vec::Vec<u8>) {
    let len = sock.recv(buf.as_mut_slice()).unwrap();
    if len != buf.len() {
        panic!("Received bytes {} expected {}", len, buf.len());
    }
}

pub fn recv_msg<T>(sock: &UnixDatagram) -> T
where
    T: Sized + Clone + FromBytes + Unaligned + IntoIpcMsgType + std::fmt::Debug,
{
    let mut buf: Vec<u8> = Vec::new();
    let msg_len = std::mem::size_of::<IpcMsgHeader>();
    buf.resize_with(msg_len, Default::default);

    recv_raw(sock, &mut buf);

    let ipc_hdr: &IpcMsgHeader =
        match LayoutVerified::<_, IpcMsgHeader>::new_unaligned(buf.as_mut_slice()) {
            Some(hdr) => hdr.into_ref(),
            None => {
                panic!("Wrong bytes {:x?}", buf.as_slice());
            }
        };

    if ipc_hdr.magic != IPC_MSG_HDR_MAGIC {
        panic!("APP  : Wrong hdr magic {:x?}", ipc_hdr);
    }
    if ipc_hdr.msg_len != std::mem::size_of::<T>() as u16 {
        panic!(
            "APP  : Wrong hdr {:x?} len expected {} ",
            ipc_hdr,
            std::mem::size_of::<T>() as u16
        );
    }
    if ipc_hdr.msg_type != T::to_ipc_msg_type() as u16 {
        panic!(
            "APP  :Wrong hdr {:x?} type expected {} ",
            ipc_hdr,
            T::to_ipc_msg_type() as u16
        );
    }
    if ipc_hdr.msg_cksum != ipc_hdr.magic + ipc_hdr.msg_len + ipc_hdr.msg_type {
        panic!(
            "APP  : Wrong hdr {:x?} chksum expected {} ",
            ipc_hdr,
            ipc_hdr.magic + ipc_hdr.msg_len + ipc_hdr.msg_type
        );
    }

    let msg_len = std::mem::size_of::<T>();
    buf.resize_with(msg_len, Default::default);
    recv_raw(sock, &mut buf);

    let msg: &T = match LayoutVerified::<_, T>::new_unaligned(buf.as_mut_slice()) {
        Some(msg) => msg.into_ref(),
        None => {
            panic!("Wrong bytes {:x?}", buf.as_slice());
        }
    };
    log_dbg!("RECV: {:x?}", msg);
    return msg.clone();
}
