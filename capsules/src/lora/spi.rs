
use kernel::hil::spi;
use base;

impl<S: spi::SpiMasterDevice> Radio<'a, S> {
    fn handle_interrupt(&self) {
        //TODO
    }

    fn register_write(&self, reg: RegMap, val: u8) -> ReturnCode {
        //digitalWrite(ss_pin, Low);
        if (self.spi_busy.get() || self.spi_tx.is_none() || self.spi_rx.is_none()) {
            return ReturnCode::EBUSY;
        }
        let wbuf = self.spi_tx.take().unwrap();
        let rbuf = self.spi_rx.take().unwrap();
        wbuf[0] = (reg as u8) | 0x80;
        wbuf[1] = val;
        self.spi.read_write_bytes(wbuf, Some(rbuf), 2);
        self.spi_busy.set(true);
        //digitalWrite(ss_pin, High);
        ReturnCode::SUCCESS
    }

    fn register_read(&self, reg: RegMap) -> ReturnCode {
        //digitalWrite(ss_pin, Low);
        if (self.spi_busy.get() || self.spi_tx.is_none() || self.spi_rx.is_none()) {
            return ReturnCode::EBUSY;
        }
        let wbuf = self.spi_tx.take().unwrap();
        let rbuf = self.spi_rx.take().unwrap();
        wbuf[0] = (reg as u8) | 0x7f;
        wbuf[1] = 0;
        self.spi.read_write_bytes(wbuf, Some(rbuf), 2);
        self.spi_busy.set(true);
        //digitalWrite(ss_pin, High);
        ReturnCode::SUCCESS
    }

    fn frame_write(&self, buf: &'static mut [u8], frame_len: u8) -> ReturnCode {
        //TODO
    }

    fn frame_read(&self, buf: &'static mut [u8], frame_len: u8) -> ReturnCode {
        //TODO
    }

    //Do we need this?
    fn state_transition_write(&self, reg: RegMap, val: u8, state: InternalState) {
        self.state.set(state);
        self.register_write(reg, val);
    }

    fn state_transition_read(&self, reg: RegMap, state: InternalState) {
        self.state.set(state);
        self.register_read(reg);
    }
}
