use returncode::ReturnCode;

pub trait TemperatureDriver {
    fn take_measurement(&self);
}

pub trait Client {
    fn measurement_done(&self, temp: usize) -> ReturnCode;
}
