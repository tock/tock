use returncode::ReturnCode;

pub trait TempDriver {
    fn init(&self);
    fn take_measurement(&self);
}

pub trait Client {
    fn measurement_done(&self, temp: usize) -> ReturnCode;
}
