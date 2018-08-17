//! Interface for sending and receiving packets.
//!
//! Hardware independent interface for an arbitrary radio. Note that
//! configuration commands are asynchronous and must be committed with a call to
//! config_commit. For example, calling set_address will change the source
//! address of packets but does not change the address stored in hardware used
//! for address recognition. This must be committed to hardware with a call to
//! config_commit. Please see the relevant TRD for more details.

use returncode::ReturnCode;

#[derive(Debug, Clone, Copy)]
pub enum RfcOperationStatus {
    Idle,
    Pending,
    Active,
    Skipped,
    SendDone,
    CommandDone,
    LastCommandDone,
    RxOk,
    TxDone,
    Setup,
    Invalid,
}

pub enum State {
    Start,
    Pending,
    CommandStatus(RfcOperationStatus),
    Done,
    Invalid,
}

pub trait RadioConfig {
    fn set_tx_client(&self, &'static TxClient);
    fn set_rx_client(&self, &'static RxClient, receive_buffer: &'static mut [u8]);
    //fn power_up(&self);
    //fn power_down(&self);
    //fn push_state(&self);
    //fn pop_state(&self) -> State;
    fn set_receive_buffer(&self, receive_buffer: &'static mut [u8]);
}

pub trait TxClient {
    fn send_done(&self, buf: &'static mut [u8], result: ReturnCode);
}

pub trait RxClient {
    fn receive(&self, buf: &'static mut [u8], frame_len: usize, crc_valid: bool, result: ReturnCode);
}

pub trait Radio: RadioConfig + RadioAttrs {}

pub trait RadioAttrs {
    fn transmit(&self, tx_buf: &'static mut [u8], frame_len: usize) -> (ReturnCode, Option<&'static mut [u8]>);
    fn push_state(&self);
    fn pop_state(&self) -> State;

}
