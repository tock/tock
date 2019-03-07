//! Virtualize UDP senders and receivers
use core::cell::Cell;
use core::cmp;
use capsules::net::udp::udp_recv::UDPRecvClient;
use capsules::net::udp::udp_send::{UDPSender, UDPSendClient}; // which one?
use kernel::common::cells::{OptionalCell, TakeCell};
use kernel::common::{List, ListLink, ListNode};
use kernel::hil;
use kernel::hil::uart;
use kernel::ReturnCode;

pub struct MuxUdpSender<'a> {

}

// or IPV6Client?
pub struct MuxUdpReceiver<'a> {

}



