use crate::Address;
use core::sync::atomic::{AtomicU8, Ordering::Relaxed};
use embedded_hal_async::i2c::I2c;

pub struct NoInterrupts;

/// Driver for a TCA9554(A) I/O expander.
pub struct Tca9554<I2C, Int> {
    pub(crate) i2c: I2C,
    pub(crate) address: Address,
    #[cfg_attr(not(feature = "interrupt"), allow(unused))]
    pub(crate) interrupt_handler: Int,
    pub(crate) output_mask: AtomicU8,
    pub(crate) polarity_mask: AtomicU8,
    pub(crate) direction_mask: AtomicU8,
}

impl<I2C> Tca9554<I2C, NoInterrupts> {
    /// Creates a new driver with the given I²C peripheral and address.
    #[must_use]
    pub fn new(i2c: I2C, address: Address) -> Self {
        Self {
            i2c,
            address,
            interrupt_handler: NoInterrupts,
            output_mask: AtomicU8::new(OUTPUT_REGISTER_DEFAULT),
            polarity_mask: AtomicU8::new(POLARITY_REGISTER_DEFAULT),
            direction_mask: AtomicU8::new(DIRECTION_REGISTER_DEFAULT),
        }
    }

    /// Binds an interrupt pin to this peripheral.
    #[must_use]
    #[cfg(feature = "interrupt")]
    pub fn with_int<INT, const SUBS: usize, M: embassy_sync::blocking_mutex::raw::RawMutex>(
        self,
        int: INT,
    ) -> Tca9554<I2C, crate::interrupt::Interrupts<INT, SUBS, M>> {
        let Self {
            i2c,
            address,
            output_mask,
            polarity_mask,
            direction_mask,
            interrupt_handler: _,
        } = self;
        Tca9554 {
            i2c,
            address,
            interrupt_handler: crate::interrupt::Interrupts {
                int: embassy_sync::mutex::Mutex::new(int),
                int_subscribers: embassy_sync::pubsub::PubSubChannel::new(),
            },
            output_mask,
            polarity_mask,
            direction_mask,
        }
    }
}

impl<I2C, INT> Tca9554<I2C, INT> {
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
pub(crate) enum Register {
    Input = 0x00,
    Output = 0x01,
    Polarity = 0x02,
    Direction = 0x03,
}

// Power-on defaults
const OUTPUT_REGISTER_DEFAULT: u8 = 0xFF;
const POLARITY_REGISTER_DEFAULT: u8 = 0x00;
const DIRECTION_REGISTER_DEFAULT: u8 = 0xFF;

static DUMMY_MASK: AtomicU8 = AtomicU8::new(0);

/// Writes a value to a register.
async fn write_register<I: I2c>(
    i2c: &mut I,
    address: Address,
    register: Register,
    value: u8,
    store_into: &AtomicU8,
) -> Result<(), I::Error> {
    i2c.write(address.into(), &[register as u8, value]).await?;
    store_into.store(value, Relaxed);
    Ok(())
}

/// Reads the value from a register.
async fn read_register<I: I2c>(
    i2c: &mut I,
    address: Address,
    register: Register,
    store_into: &AtomicU8,
) -> Result<u8, I::Error> {
    let mut read_buf = [0u8];
    i2c.write_read(address.into(), &[register as u8], &mut read_buf)
        .await?;
    let res = read_buf[0];
    store_into.store(res, Relaxed);
    Ok(res)
}

#[derive(Clone)]
pub struct ExioPin<'io, I2C, INT>(pub(crate) &'io Tca9554<I2C, INT>, pub(crate) u8);

impl<I2C, INT> Tca9554<I2C, INT>
where
    I2C: I2c,
{
    /// Performs reads of all registers to ensure the device is functioning correctly.
    pub async fn init(&mut self) -> Result<(), I2C::Error> {
        self.read_output().await?;
        self.read_polarity().await?;
        self.read_direction().await?;
        Ok(())
    }

    /// Reads the value of the input register.
    pub fn read_input(&mut self) -> impl Future<Output = Result<u8, I2C::Error>> {
        read_register(&mut self.i2c, self.address, Register::Input, &DUMMY_MASK)
    }

    /// Reads the value of the output register.
    pub fn read_output(&mut self) -> impl Future<Output = Result<u8, I2C::Error>> {
        read_register(
            &mut self.i2c,
            self.address,
            Register::Output,
            &self.output_mask,
        )
    }

    /// Writes the value of the output register.
    pub fn write_output(&mut self, state: u8) -> impl Future<Output = Result<(), I2C::Error>> {
        write_register(
            &mut self.i2c,
            self.address,
            Register::Output,
            state,
            &self.output_mask,
        )
    }

    /// Reads the value of the polarity inversion register.
    pub fn read_polarity(&mut self) -> impl Future<Output = Result<u8, I2C::Error>> {
        read_register(
            &mut self.i2c,
            self.address,
            Register::Polarity,
            &self.polarity_mask,
        )
    }

    /// Writes the value of polarity inversion register.
    pub fn write_polarity(&mut self, state: u8) -> impl Future<Output = Result<(), I2C::Error>> {
        write_register(
            &mut self.i2c,
            self.address,
            Register::Polarity,
            state,
            &self.polarity_mask,
        )
    }

    /// Reads the value of the direction register.
    pub fn read_direction(&mut self) -> impl Future<Output = Result<u8, I2C::Error>> {
        read_register(
            &mut self.i2c,
            self.address,
            Register::Direction,
            &self.direction_mask,
        )
    }

    /// Writes the value of the direction register.
    pub fn write_direction(&mut self, state: u8) -> impl Future<Output = Result<(), I2C::Error>> {
        write_register(
            &mut self.i2c,
            self.address,
            Register::Direction,
            state,
            &self.direction_mask,
        )
    }

    /// Writes the state of the registers to the chip's power-on defaults.
    pub async fn reset(&mut self) -> Result<(), I2C::Error> {
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
    pub async fn is_in_default_state(&mut self) -> Result<bool, I2C::Error> {
        Ok(self.read_direction().await? == DIRECTION_REGISTER_DEFAULT
            && self.read_polarity().await? == POLARITY_REGISTER_DEFAULT
            && self.read_output().await? == OUTPUT_REGISTER_DEFAULT)
    }
}

impl<I2C, INT> Tca9554<I2C, INT>
where
    I2C: I2c + Clone,
{
    /// Reads the value of the input register.
    pub async fn read_input_ref(&self) -> Result<u8, I2C::Error> {
        read_register(
            &mut self.i2c.clone(),
            self.address,
            Register::Input,
            &DUMMY_MASK,
        )
        .await
    }

    /// Reads the value of the output register.
    pub async fn read_output_ref(&self) -> Result<u8, I2C::Error> {
        read_register(
            &mut self.i2c.clone(),
            self.address,
            Register::Output,
            &self.output_mask,
        )
        .await
    }

    /// Writes the value of the output register.
    pub async fn write_output_ref(&self, state: u8) -> Result<(), I2C::Error> {
        write_register(
            &mut self.i2c.clone(),
            self.address,
            Register::Output,
            state,
            &self.output_mask,
        )
        .await?;
        Ok(())
    }

    /// Reads the value of the polarity inversion register.
    pub async fn read_polarity_ref(&self) -> Result<u8, I2C::Error> {
        read_register(
            &mut self.i2c.clone(),
            self.address,
            Register::Polarity,
            &self.polarity_mask,
        )
        .await
    }

    /// Writes the value of polarity inversion register.
    pub async fn write_polarity_ref(&self, state: u8) -> Result<(), I2C::Error> {
        write_register(
            &mut self.i2c.clone(),
            self.address,
            Register::Polarity,
            state,
            &self.polarity_mask,
        )
        .await?;
        Ok(())
    }

    /// Reads the value of the direction register.
    pub async fn read_direction_ref(&self) -> Result<u8, I2C::Error> {
        read_register(
            &mut self.i2c.clone(),
            self.address,
            Register::Direction,
            &self.direction_mask,
        )
        .await
    }

    /// Writes the value of the direction register.
    pub async fn write_direction_ref(&self, state: u8) -> Result<(), I2C::Error> {
        write_register(
            &mut self.i2c.clone(),
            self.address,
            Register::Direction,
            state,
            &self.direction_mask,
        )
        .await?;
        Ok(())
    }

    /// Writes the state of the registers to the chip's power-on defaults.
    pub async fn reset_ref(&self) -> Result<(), I2C::Error> {
        self.write_direction_ref(DIRECTION_REGISTER_DEFAULT).await?;
        self.write_polarity_ref(POLARITY_REGISTER_DEFAULT).await?;
        self.write_output_ref(OUTPUT_REGISTER_DEFAULT).await?;
        Ok(())
    }

    /// Returns whether or not the current state of the registers
    /// matches the chip's power-on defaults.
    ///
    /// When the chip is functioning properly, the registers will match
    /// the power-on defaults after power has been applied or after
    /// a call to [`Self::reset_ref()`].
    pub fn is_in_default_state_ref(&self) -> bool {
        self.direction_mask.load(Relaxed) == DIRECTION_REGISTER_DEFAULT
            && self.polarity_mask.load(Relaxed) == POLARITY_REGISTER_DEFAULT
            && self.output_mask.load(Relaxed) == OUTPUT_REGISTER_DEFAULT
    }

    /// Creates a view restricted to a specific pin of the IO Extender.
    pub fn pin<'io>(&'io self, pin: u8) -> ExioPin<'io, I2C, INT> {
        ExioPin(self, pin)
    }
}
