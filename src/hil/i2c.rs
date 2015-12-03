pub trait I2C {
    fn enable(&self);
    fn disable(&self);
    fn write_sync(&self, addr: u16, data: &[u8]);
    fn read_sync(&self, addr: u16, buffer: &mut [u8]);
}
