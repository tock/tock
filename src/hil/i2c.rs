pub trait I2C {
    fn enable(&mut self);
    fn disable(&mut self);
    fn write_sync(&mut self, addr: u16, data: &[u8]);
    fn read_sync(&mut self, addr: u16, buffer: &mut [u8]);
}
