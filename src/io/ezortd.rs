use embedded_hal::blocking::{delay, i2c};

/// Describes potential errors
#[derive(Debug, PartialEq)]
pub enum Error {
    /// I²C bus error
    I2c,
    /// Still processing, not ready
    NotReady,
    /// Syntax Error
    SyntaxError,
    /// Error parsing response
    ParseError,
}

impl From<std::num::ParseFloatError> for Error {
    fn from(_error: std::num::ParseFloatError) -> Self {
        Error::ParseError
    }
}

/// Sensor configuration
pub struct EzoRtd<I2C, Delay> {
    /// I2C master device to use to communicate with the sensor
    i2c: I2C,
    /// Delay device to be able to sleep in-between commands
    delay: Delay,
    /// I2C address
    address: u8,
}

impl<I2C, Delay, HalI2CError> EzoRtd<I2C, Delay>
where
    I2C: i2c::Read<Error = HalI2CError> + i2c::Write<Error = HalI2CError>,
    Delay: delay::DelayMs<u16>,
{
    pub fn new(i2c: I2C, delay: Delay, address: u8) -> Self {
        Self { i2c, delay, address }
    }

    fn send_command(&mut self, command: &str) -> Result<(), Error> {
        self.i2c.write(self.address, command.as_bytes()).map_err(|_| Error::I2c)
    }

    fn read_response(&mut self, mut buf: &mut [u8]) -> Result<(), Error> {
        self.i2c.read(self.address, &mut buf).map_err(|_| Error::I2c)?;
        self.validate_response_code(buf)
    }

    fn validate_response_code(&self, buf: &[u8]) -> Result<(), Error> {
        match buf[0] {
            254 => Err(Error::NotReady),
            2 => Err(Error::SyntaxError),
            1 => Ok(()),
            _ => Err(Error::ParseError),
        }
    }

    fn extract_string(&self, buf: &[u8]) -> Result<String, Error> {
        let end = match buf.iter().position(|&r| r == 0x0) {
            Some(n) => n,
            None => buf.len()
        };

        match String::from_utf8((&buf[1..end]).to_vec()) {
            Ok(t) => Ok(t),
            Err(_) => Err(Error::ParseError)
        }
    }

    pub fn information(&mut self) -> Result<String, Error> {
        self.send_command("i")?;
        self.delay.delay_ms(600);

        let mut buffer = [0u8; 14];
        self.read_response(&mut buffer)?;
        let temperature_string = self.extract_string(&buffer)?;
        
        Ok(temperature_string)
    }

    pub fn read(&mut self) -> Result<f64, Error> {
        self.send_command("R")?;
        self.delay.delay_ms(600);

        let mut buffer = [0u8; 14];
        self.read_response(&mut buffer)?;
        let temperature = self.extract_string(&buffer)?.parse::<f64>()?;

        Ok(temperature)
    }

    pub fn status(&mut self) -> Result<String, Error> {
        self.send_command("Status")?;
        self.delay.delay_ms(300);
        
        let mut buffer = [0u8; 14];
        self.read_response(&mut buffer)?;
        let s = self.extract_string(&buffer)?;
        
        Ok(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::io::ErrorKind;

    use embedded_hal_mock::delay::MockNoop as NoopDelay;
    use embedded_hal_mock::i2c::{Mock as I2cMock, Transaction};
    use embedded_hal_mock::MockError;

    /// Test if the `send_command` function sends the expected bytes to the expected address
    #[test]
    fn send_command() {
        let expectations = [
            Transaction::write(0x66, "i".as_bytes().to_vec()),
        ];
        let mock = I2cMock::new(&expectations);
        let mut rtd = EzoRtd::new(mock, NoopDelay, 0x66);
        let res = rtd.send_command("i");
        assert!(res.is_ok());
    }

    /// Test whether the `send_command` function propagates I²C errors.
    #[test]
    fn send_command_error() {
        let expectations = [
            Transaction::write(0x66, "i".as_bytes().to_vec()).with_error(MockError::Io(ErrorKind::Other))
        ];
        let mock = I2cMock::new(&expectations);
        let mut rtd = EzoRtd::new(mock, NoopDelay, 0x66);
        let err = rtd.send_command("i").unwrap_err();
        assert_eq!(err, Error::I2c);
    }

    /// Test if `read_response` returns string if first byte is 1
    #[test]
    fn read_response_success() {
        let expectations = [
            Transaction::read(0x66, vec![1,4]),
        ];
        let mock = I2cMock::new(&expectations);
        let mut rtd = EzoRtd::new(mock, NoopDelay, 0x66);
        let mut buf = vec![0u8, 2];
        rtd.read_response(&mut buf).unwrap();
        assert_eq!(buf, vec![1, 4]);
    }

    /// Test if `read_response` returns syntax if first byte is 2
    #[test]
    fn read_response_syntax_error() {
        let expectations = [
            Transaction::read(0x66, vec![2,4]),
        ];
        let mock = I2cMock::new(&expectations);
        let mut rtd = EzoRtd::new(mock, NoopDelay, 0x66);
        let mut buf = vec![0u8, 2];
        let err = rtd.read_response(&mut buf).unwrap_err();
        assert_eq!(err, Error::SyntaxError);
    }

    /// Test if `read_response` returns not ready if first byte is 254
    #[test]
    fn read_response_not_ready() {
        let expectations = [
            Transaction::read(0x66, vec![254,4]),
        ];
        let mock = I2cMock::new(&expectations);
        let mut rtd = EzoRtd::new(mock, NoopDelay, 0x66);
        let mut buf = vec![0u8, 2];
        let err = rtd.read_response(&mut buf).unwrap_err();
        assert_eq!(err, Error::NotReady);
    }

    /// Test if `read_response` returns parse error if first byte is other
    #[test]
    fn read_response_parse_error() {
        let expectations = [
            Transaction::read(0x66, vec![4,4]),
        ];
        let mock = I2cMock::new(&expectations);
        let mut rtd = EzoRtd::new(mock, NoopDelay, 0x66);
        let mut buf = vec![0u8, 2];
        let err = rtd.read_response(&mut buf).unwrap_err();
        assert_eq!(err, Error::ParseError);
    }

    /// Test if `read` returns expected value
    #[test]
    fn read_success() {
        let expectations = [
            Transaction::write(0x66, "R".as_bytes().to_vec()),
            Transaction::read(0x66, vec![1, 49, 50, 46, 51, 52, 53, 0, 0, 0, 0, 0, 0, 0]),
        ];
        let mock = I2cMock::new(&expectations);
        let mut rtd = EzoRtd::new(mock, NoopDelay, 0x66);
        let res = rtd.read().unwrap();
        assert_eq!(res, 12.345);
    }

}
