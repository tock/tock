impl PacketConfig for Radio<'_> {
    pub fn packet_rssi(&self) -> u8 {
        // let freq = self.frequency.get() as f64;
        let rssi_base;
        if freq < 868E6 {
            rssi_base = 164;
        } else {
            rssi_base = 157;
        }
        self.register_return(RegMap::RegPktRssiValue) - rssi_base
    }

    pub fn packet_snr(&self) -> f32 {
        self.register_return(RegMap::RegPktSnrValue) as f32 * 0.25
    }

    #[allow(arithmetic_overflow)]
    pub fn packet_frequency_error(&self) -> i64 {
        let mut freq_error;
        freq_error = self.register_return(RegMap::RegFreqErrorMsb) & 0b111;
        freq_error <<= 8;
        freq_error += self.register_return(RegMap::RegFreqErrorMid);
        freq_error <<= 8;
        freq_error += self.register_return(RegMap::RegFreqErrorLsb);

        if self.register_return(RegMap::RegFreqErrorMsb) & 0b1000 != 0 { // Sign bit is on
             //freq_error 24288; // B1000'0000'0000'0000'0000
        }

        let f_xtal = 32E6 as f64; // Fxosc: crystal oscillator (Xtal) frequency (2.5. Chip Specification, p. 14)
        let f_error =
            ((freq_error << 24) as f64 / f_xtal) * (self.get_signal_bandwidth() as f64 / 500000.0); // p. 37

        return f_error as i64;
    }

    pub fn set_power(&self, mut level: i8, output_pin: u8) {
        if output_pin == 0 {
            // Rfo
            if level < 0 {
                level = 0;
            } else if level > 14 {
                level = 14;
            }

            self.register_write(RegMap::RegPaConfig, 0x70 | level as u8);
        } else {
            // Pa Boost
            if level > 17 {
                if level > 20 {
                    level = 20;
                }

                // subtract 3 from level, so 18 - 20 maps to 15 - 17
                level -= 3;

                // High Power +20 dBm Operation (Semtech Sx1276/77/78/79 5.4.3.)
                self.register_write(RegMap::RegPaDac, 0x87);
                self.set_ocp(140);
            } else {
                if level < 2 {
                    level = 2;
                }
                //Default value PaHf/Lf or +17dBm
                self.register_write(RegMap::RegPaDac, 0x84);
                self.set_ocp(100);
            }

            self.register_write(RegMap::RegPaConfig, PA_BOOST | (level as u8 - 2));
        }
    }

    pub fn set_frequency(&self, frequency: u64) {
        self.frequency.set(frequency);

        let frf = (frequency << 19) / 32000000;

        self.register_write(RegMap::RegFrfMsb, (frf >> 16) as u8);
        self.register_write(RegMap::RegFrfMid, (frf >> 8) as u8);
        self.register_write(RegMap::RegFrfLsb, (frf >> 0) as u8);
    }

    pub fn get_spreading_factor(&self) -> u8 {
        self.register_return(RegMap::RegModemConfig2) >> 4
    }

    pub fn set_spreading_factor(&self, mut sf: u8) {
        if sf < 6 {
            sf = 6;
        } else if sf > 12 {
            sf = 12;
        }

        if sf == 6 {
            self.register_write(RegMap::RegDetectionOptimize, 0xc5);
            self.register_write(RegMap::RegDetectionThreshold, 0x0c);
        } else {
            self.register_write(RegMap::RegDetectionOptimize, 0xc3);
            self.register_write(RegMap::RegDetectionThreshold, 0x0a);
        }

        self.register_write(
            RegMap::RegModemConfig2,
            (self.register_return(RegMap::RegModemConfig2) as u8 & 0x0f) | ((sf << 4) & 0xf0),
        );
        self.set_ldo_flag();
    }

    pub fn get_signal_bandwidth(&self) -> f64 {
        let bw = (self.register_return(RegMap::RegModemConfig1) >> 4) as u8;

        match bw {
            0 => return 7.8E3,
            1 => return 10.4E3,
            2 => return 15.6E3,
            3 => return 20.8E3,
            4 => return 31.25E3,
            5 => return 41.7E3,
            6 => return 62.5E3,
            7 => return 125E3,
            8 => return 250E3,
            _ => return 500E3,
        }
    }

    pub fn set_signal_bandwidth(&self, sbw: f64) {
        let bw: u8;

        if sbw <= 7.8E3 {
            bw = 0;
        } else if sbw <= 10.4E3 {
            bw = 1;
        } else if sbw <= 15.6E3 {
            bw = 2;
        } else if sbw <= 20.8E3 {
            bw = 3;
        } else if sbw <= 31.25E3 {
            bw = 4;
        } else if sbw <= 41.7E3 {
            bw = 5;
        } else if sbw <= 62.5E3 {
            bw = 6;
        } else if sbw <= 125E3 {
            bw = 7;
        } else if sbw <= 250E3 {
            bw = 8;
        } else
        /*if sbw <= 250E3*/
        {
            bw = 9;
        }

        self.register_write(
            RegMap::RegModemConfig1,
            (self.register_return(RegMap::RegModemConfig1) & 0x0f) as u8 | (bw << 4),
        );
        self.set_ldo_flag();
    }

    pub fn set_ldo_flag(&self) {
        // Section 4.1.1.5
        let symbol_duration =
            1000 / (self.get_signal_bandwidth() / (1 << self.get_spreading_factor()) as f64) as i64;

        // Section 4.1.1.6
        let ldo_on: bool = symbol_duration > 16;

        let config3: u8;
        if ldo_on {
            config3 = self.register_return(RegMap::RegModemConfig3) | 0b1000;
        } else {
            config3 = self.register_return(RegMap::RegModemConfig3);
        }
        self.register_write(RegMap::RegModemConfig3, config3);
    }

    pub fn set_coding_rate4(&self, mut denominator: u8) {
        if denominator < 5 {
            denominator = 5;
        } else if denominator > 8 {
            denominator = 8;
        }

        let cr = denominator - 4 as u8;

        self.register_write(
            RegMap::RegModemConfig1,
            (self.register_return(RegMap::RegModemConfig1) as u8 & 0xf1) | (cr << 1),
        );
    }

    pub fn set_preamble_length(&self, length: i64) {
        self.register_write(RegMap::RegPreambleMsb, (length >> 8) as u8);
        self.register_write(RegMap::RegPreambleLsb, (length >> 0) as u8);
    }

    pub fn set_sync_word(&self, sw: u8) {
        self.register_write(RegMap::RegSyncWord, sw);
    }

    pub fn enable_crc(&self) {
        self.register_write(
            RegMap::RegModemConfig2,
            self.register_return(RegMap::RegModemConfig2) as u8 | 0x04,
        );
    }

    pub fn disable_crc(&self) {
        self.register_write(
            RegMap::RegModemConfig2,
            self.register_return(RegMap::RegModemConfig2) as u8 & 0xfb,
        );
    }

    pub fn enable_invert_iq(&self) {
        self.register_write(RegMap::RegInvertiq, 0x66);
        self.register_write(RegMap::RegInvertiq2, 0x19);
    }

    pub fn disable_invert_iq(&self) {
        self.register_write(RegMap::RegInvertiq, 0x27);
        self.register_write(RegMap::RegInvertiq2, 0x1d);
    }

    pub fn set_ocp(&self, ma: u8) {
        let mut ocp_trim = 27 as u8;

        if ma <= 120 {
            ocp_trim = (ma - 45) / 5;
        } else if ma <= 240 {
            ocp_trim = (ma + 30) / 10;
        }

        self.register_write(RegMap::RegOcp, 0x20 | (0x1F & ocp_trim));
    }
}
