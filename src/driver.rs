use crate::Address;
use embedded_hal_async::i2c::I2c;

/// Driver for a TCA9554(A) I/O expander.
pub struct Tca9554<I2C> {
    i2c: I2C,
    address: Address,
}

impl<I2C> Tca9554<I2C> {
    /// Creates a new driver with the given I²C peripheral and address.
    pub fn new(i2c: I2C, address: Address) -> Self {
        Self { i2c, address }
    }

    /// Gets the I²C used by the driver.
    pub fn address(&self) -> Address {
        self.address
    }

    /// Releases the driver, returning ownership of the I²C peripheral.
    pub fn release(self) -> I2C {
        self.i2c
    }
}

/// Device register address.
#[repr(u8)]
enum Register {
    Input = 0x00,
    Output = 0x01,
    Polarity = 0x02,
    Direction = 0x03,
}

// Power-on defaults
const OUTPUT_REGISTER_DEFAULT: u8 = 0xFF;
const POLARITY_REGISTER_DEFAULT: u8 = 0x00;
const DIRECTION_REGISTER_DEFAULT: u8 = 0xFF;

impl<I2C, E> Tca9554<I2C>
where
    I2C: I2c<Error = E>,
{
    /// Reads the value of the input register.
    pub async fn read_input(&mut self) -> Result<u8, E> {
        self.read_register(Register::Input).await
    }

    /// Reads the value of the output register.
    pub async fn read_output(&mut self) -> Result<u8, E> {
        self.read_register(Register::Output).await
    }

    /// Writes the value of the output register.
    pub async fn write_output(&mut self, state: u8) -> Result<(), E> {
        self.write_register(Register::Output, state).await
    }

    /// Reads the value of the polarity inversion register.
    pub async fn read_polarity(&mut self) -> Result<u8, E> {
        self.read_register(Register::Polarity).await
    }

    /// Writes the value of polarity inversion register.
    pub async fn write_polarity(&mut self, state: u8) -> Result<(), E> {
        self.write_register(Register::Polarity, state).await
    }

    /// Reads the value of the direction register.
    pub async fn read_direction(&mut self) -> Result<u8, E> {
        self.read_register(Register::Direction).await
    }

    /// Writes the value of the direction register.
    pub async fn write_direction(&mut self, state: u8) -> Result<(), E> {
        self.write_register(Register::Direction, state).await
    }

    /// Writes the state of the registers to the chip's power-on defaults.
    pub async fn reset(&mut self) -> Result<(), E> {
        self.write_direction(DIRECTION_REGISTER_DEFAULT).await?;
        self.write_polarity(POLARITY_REGISTER_DEFAULT).await?;
        self.write_output(OUTPUT_REGISTER_DEFAULT).await?;
        Ok(())
    }

    /// Returns whether or not the current state of the registers
    /// matches the chip's power-on defaults.
    ///
    /// When the chip is functioning properly, the registers will match
    /// the power-on defaults after power has been applied or after
    /// a call to [`Self::reset()`].
    ///
    pub async fn is_in_default_state(&mut self) -> Result<bool, E> {
        Ok(self.read_direction().await? == DIRECTION_REGISTER_DEFAULT
            && self.read_polarity().await? == POLARITY_REGISTER_DEFAULT
            && self.read_output().await? == OUTPUT_REGISTER_DEFAULT)
    }

    /// Writes a value to a register.
    async fn write_register(&mut self, register: Register, value: u8) -> Result<(), E> {
        self.i2c
            .write(self.address.into(), &[register as u8, value])
            .await?;
        Ok(())
    }

    /// Reads the value from a register.
    async fn read_register(&mut self, register: Register) -> Result<u8, E> {
        let mut read_buf = [0u8];
        self.i2c
            .write_read(self.address.into(), &[register as u8], &mut read_buf)
            .await?;
        Ok(read_buf[0])
    }
}
