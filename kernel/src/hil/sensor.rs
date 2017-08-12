use returncode::ReturnCode;

pub trait TemperatureDriver {
    fn set_client(&self, client: &'static TemperatureClient);
    fn read_ambient_temperature(&self) -> ReturnCode {
        ReturnCode::ENOSUPPORT
    }
    fn read_cpu_temperature(&self) -> ReturnCode {
        ReturnCode::ENOSUPPORT
    }
}

pub trait TemperatureClient {
    fn callback(&self, value: usize, measurement_type: usize, err: ReturnCode);
}

pub trait HumidityDriver {
    fn set_client(&self, client: &'static HumidityClient);
    fn read_humidity(&self) -> ReturnCode {
        ReturnCode::ENOSUPPORT
    }
}

pub trait HumidityClient {
    fn callback(&self, value: usize, measurement_type: usize, err: ReturnCode);
}
