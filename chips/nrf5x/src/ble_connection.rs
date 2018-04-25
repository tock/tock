use core::fmt;
use ble_advertising_hil::RadioChannel;
use ble_link_layer::LLData;

// pub struct LLData {
//     pub aa: [u8; 4],
//     pub crc_init: [u8; 3],
//     win_size: u8,
//     win_offset: u16,
//     interval: u16,
//     latency: u16,
//     timeout: u16,
//     chm: [u8; 5],
//     hop_and_sca: u8 // hops 5 bits, sca 3 bits
// }

const NUMBER_CHANNELS: usize = 40;
const NUMBER_DATA_CHANNELS: usize = NUMBER_CHANNELS - 3;

type ChannelMap = [u8; NUMBER_CHANNELS];

pub struct ConnectionData {
	last_unmapped_channel: u8,
	channels: ChannelMap,
	pub conn_event_counter: u16,
	hop_increment: u8,
	number_used_channels: u8,
	pub aa: u32,
	pub crcinit: u32,
	pub transmit_seq_nbr: u8,
	pub next_seq_nbr: u8,
	pub conn_interval_start: Option<u32>,
	pub lldata: LLData

}

impl PartialEq for ConnectionData {
    fn eq(&self, other: &ConnectionData) -> bool {
        self.last_unmapped_channel == other.last_unmapped_channel
    }
}

impl Eq for ConnectionData {}

impl fmt::Debug for ConnectionData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ConnectionData {{ last_unmapped_channel: {}, conn_event_counter: {}, hop_increment: {}, number_used_channels: {}, aa: {}, crcinit {} }}",
            self.last_unmapped_channel,
            self.conn_event_counter,
            self.hop_increment,
            self.number_used_channels,
			self.aa,
			self.crcinit
        )
    }
}


impl ConnectionData {
	pub fn new(lldata: LLData) -> ConnectionData {

		let (channels, number_used_channels) = ConnectionData::expand_channel_map(lldata.chm);


	    ConnectionData {
	    	last_unmapped_channel: 0,
	    	channels,
	    	number_used_channels,
			hop_increment: lldata.hop_and_sca & 0b11111,
            conn_event_counter: 0,
			aa: (lldata.aa[0] as u32) << 24 | (lldata.aa[1] as u32) << 16 | (lldata.aa[2] as u32) << 8 | (lldata.aa[3] as u32),
			crcinit: (lldata.crc_init[0] as u32) << 16 | (lldata.crc_init[1] as u32) << 8 | (lldata.crc_init[2] as u32),
			transmit_seq_nbr: 0,
			next_seq_nbr: 0,
			conn_interval_start: None,
			lldata,
	    }
	}

	pub fn update_lldata(&mut self, lldata: LLData) {
		let (channels, number_used_channels) = ConnectionData::expand_channel_map(lldata.chm);

		self.channels = channels;
		self.number_used_channels = number_used_channels;
	}

	fn expand_channel_map(chm: [u8; 5]) -> (ChannelMap, u8) {
		let mut channels: ChannelMap = [0; NUMBER_CHANNELS];

		let mut number_used_channels = 0;

	    for i in 0..chm.len() {
	        let mut byte = chm[i];

	        for j in 0..8 {
	            let bit = (byte as u8) & 1;

	            if bit == 1 {
	                number_used_channels += 1;
	            }

	            channels[(i * 8) + j] = bit;
	            byte = byte >> 1;
	        }
	    }

        (channels, number_used_channels)
	}

	pub fn next_channel(&mut self) -> RadioChannel {
	    let unmapped_channel = (self.last_unmapped_channel + self.hop_increment) % (NUMBER_DATA_CHANNELS as u8);
	    let used = self.channels[unmapped_channel as usize] == 1;

        self.last_unmapped_channel = unmapped_channel;

	    if used {
            RadioChannel::from_channel_index(unmapped_channel).unwrap()
	    } else {

	        let mut table: ChannelMap = [0; NUMBER_CHANNELS];
	        let remapping_index = unmapped_channel % self.number_used_channels;

	        let mut idx = 0;

	        for i in 0..self.channels.len() {
	            if self.channels[i] == 1 {
	                table[idx] = i as u8;
	                idx += 1;
	            }
	        }

            RadioChannel::from_channel_index(table[remapping_index as usize]).unwrap()
	    }
	}

	pub fn next_sequence_number(&mut self, buf_head_flags: u8) -> (u8, u8, bool) {


		let (sn, nesn, _) = ConnectionData::get_data_pdu_header(buf_head_flags);

		//Does the packet carry the sequence number that I expected?
		//If true, increment next_seq_nbr
		let received_new_data_pdu: bool = (sn == self.next_seq_nbr);
		if received_new_data_pdu {
			self.next_seq_nbr = (self.next_seq_nbr + 1) % 2; //flip the bit
		} //else it is resent data an next_seq_nbr shall not be changed

		//Does my peer expect the same sequence number as I am going to send?
		//If NOT equal, my peer did receive my previous packet. I should increment tansmit_seq_nbr
		let resend_last_data_pdu: bool = (nesn == self.transmit_seq_nbr);
		if !resend_last_data_pdu {
			self.transmit_seq_nbr = (self.transmit_seq_nbr + 1) % 2; //flip the bit
		}

		(self.transmit_seq_nbr, self.next_seq_nbr, resend_last_data_pdu)

	}

	pub fn get_data_pdu_header(buf_head_flags: u8) -> (u8, u8, bool) {
		//There must at least be a 2 bytes header
		let more_data = (buf_head_flags & 0b10000) >> 4 == 1;
		let nesn = (buf_head_flags & 0b100) >> 2;
		let sn = (buf_head_flags & 0b1000) >> 3;
		(sn, nesn, more_data)
	}

	pub fn connection_interval_ended(&mut self, rx_timestamp: u32) -> (bool, Option<u32>) {

		//TODO - Perhaps add jitter in the comparison?

		let interval = (self.lldata.interval as u32) * 1000 * 5 / 4 - 1000;

		match self.conn_interval_start {
			Some(start_time) => {
				if rx_timestamp >= (interval + start_time) - 150 {
					// self.conn_interval_start = None;
					(true, Some(interval + start_time))
				} else {
					(false, Some(interval + start_time))
				}
			},
			None => {
				self.conn_interval_start = Some(rx_timestamp);
				(false, Some(interval + rx_timestamp))
			},
		}
	}
}



