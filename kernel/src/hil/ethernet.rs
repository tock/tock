use crate::ErrorCode;

pub trait TransmitClient {
    fn transmitted_frame(&self, transmit_status: Result<(), ErrorCode>);
}

pub trait ReceiveClient {
    fn received_frame(&self,
        receive_status: Result<(), ErrorCode>,
        received_frame: &mut [u8],
        received_frame_length: usize
    );
}
