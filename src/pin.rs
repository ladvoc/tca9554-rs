use core::sync::atomic::Ordering::Relaxed;
use embedded_hal_async::i2c::I2c;
use futures::TryFutureExt;

use crate::driver::ExioPin;

pub(crate) fn read_bit(mask: u8, bit: u8) -> bool {
    (mask >> bit) & 1 != 0
}

fn with_bit(mask: u8, bit: u8, value: bool) -> u8 {
    let bit = (value as u8) << bit;
    mask & !bit | bit
}

impl<I2C, INT> ExioPin<'_, I2C, INT>
where
    I2C: I2c + Clone,
{
    /// Reads the value of the polarity inversion register for this pin.
    /// Returns: Whether this input's polarity is inverted.
    pub fn read_polarity(&self) -> impl Future<Output = Result<bool, I2C::Error>> {
        self.0
            .read_polarity_ref()
            .map_ok(move |mask| read_bit(mask, self.1))
    }

    /// Reads the value of the input register for this pin.
    /// Returns: Whether the incoming logic level of this pin is high, if it's defined as an input.
    pub fn read_input(&self) -> impl Future<Output = Result<bool, I2C::Error>> {
        self.0
            .read_input_ref()
            .map_ok(move |mask| read_bit(mask, self.1))
    }

    /// Reads the value of the output register for this pin.
    /// Returns: The outgoing logic level of this pin, if it's defined as an output.
    pub fn read_output(&self) -> impl Future<Output = Result<bool, I2C::Error>> {
        self.0
            .read_output_ref()
            .map_ok(move |mask| read_bit(mask, self.1))
    }

    /// Reads the value of the direction register for this pin.
    /// Returns: Whether this pin is configured as an input
    pub fn read_is_input(&self) -> impl Future<Output = Result<bool, I2C::Error>> {
        self.0
            .read_direction_ref()
            .map_ok(move |mask| (mask >> self.1) & 1 != 0)
    }

    /// Sets this output pin low or high.
    pub fn set_output_state(&self, high: bool) -> impl Future<Output = Result<(), I2C::Error>> {
        self.0
            .write_output_ref(with_bit(self.0.output_mask.load(Relaxed), self.1, high))
    }

    /// Sets this output pin low or high.
    pub fn set_polarity(&self, inverted: bool) -> impl Future<Output = Result<(), I2C::Error>> {
        self.0.write_polarity_ref(with_bit(
            self.0.polarity_mask.load(Relaxed),
            self.1,
            inverted,
        ))
    }

    /// Configures this pin as an input or output.
    pub fn set_input(&self, input: bool) -> impl Future<Output = Result<(), I2C::Error>> {
        self.0
            .write_direction_ref(with_bit(self.0.direction_mask.load(Relaxed), self.1, input))
    }
}
