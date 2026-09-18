use crate::pin::read_bit;
use crate::{Tca9554, driver::ExioPin};
use core::fmt::Debug;
use embassy_sync::blocking_mutex::raw::RawMutex;
use embedded_hal_async::digital::Wait;
use embedded_hal_async::i2c::I2c;
use futures::TryFutureExt;

pub struct Interrupts<
    INT,
    const SUBS: usize = 8,
    M: RawMutex = embassy_sync::blocking_mutex::raw::NoopRawMutex,
> {
    pub(crate) int: embassy_sync::mutex::Mutex<M, INT>,
    pub(crate) int_subscribers: embassy_sync::pubsub::PubSubChannel<M, u8, 1, SUBS, 1>,
}

impl<I2C, INT, const SUBS: usize, M: RawMutex> Tca9554<I2C, Interrupts<INT, SUBS, M>> {
    /// Releases the driver, returning ownership of the I²C peripheral and INT pin.
    pub fn release_int(self) -> (I2C, INT) {
        (self.i2c, self.interrupt_handler.int.into_inner())
    }
}

pub enum InterruptWaitError<INT, I2C> {
    InterruptError(INT),
    I2CError(I2C),
    TooManySubscribers,
}

impl<INT: Debug, I2C> Debug for InterruptWaitError<INT, I2C> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            InterruptWaitError::I2CError(_) => f.write_str("I2C Error"),
            InterruptWaitError::InterruptError(int) => write!(f, "Interrupt Error: {int:?}"),
            InterruptWaitError::TooManySubscribers => f.write_str("Too many subscribers"),
        }
    }
}

impl<INT: Debug, I2C> embedded_hal::digital::Error for InterruptWaitError<INT, I2C> {
    fn kind(&self) -> embedded_hal::digital::ErrorKind {
        embedded_hal::digital::ErrorKind::Other
    }
}

impl<I2C, INT, const SUBS: usize, M: RawMutex> Tca9554<I2C, Interrupts<INT, SUBS, M>>
where
    INT: Wait,
    I2C: I2c + Clone,
{
    /// Wait for any of the input pins to change state, and read the new pin state mask.
    /// Will keep waiting until the `stop_waiting_cond` returns true.
    pub async fn wait_for_interrupt_with_cond(
        &self,
        mut stop_waiting_cond: impl FnMut(u8) -> bool,
    ) -> Result<u8, InterruptWaitError<INT::Error, I2C::Error>> {
        match self.interrupt_handler.int.try_lock() {
            Ok(mut int) => {
                // We are the first to wait for the interrupt, we should publish to everyone else
                use embassy_sync::pubsub::PubSubBehavior;

                int.wait_for_low()
                    .await
                    .map_err(InterruptWaitError::InterruptError)?;
                core::mem::drop(int);
                let pins = self
                    .read_input_ref()
                    .await
                    .map_err(InterruptWaitError::I2CError)?;
                self.interrupt_handler
                    .int_subscribers
                    .publish_immediate(pins);
                Ok(pins)
            }
            Err(_) => {
                let mut sub = self
                    .interrupt_handler
                    .int_subscribers
                    .subscriber()
                    .map_err(|_| InterruptWaitError::TooManySubscribers)?;
                loop {
                    let mask = sub.next_message_pure().await;
                    if stop_waiting_cond(mask) {
                        return Ok(mask);
                    }
                }
            }
        }
    }
}

impl<I2C, INT, const SUBS: usize, M: RawMutex> embedded_hal::digital::ErrorType
    for ExioPin<'_, I2C, Interrupts<INT, SUBS, M>>
where
    INT: Wait,
    I2C: I2c + Clone,
{
    type Error = InterruptWaitError<INT::Error, I2C::Error>;
}

impl<I2C, INT, const SUBS: usize, M: RawMutex> ExioPin<'_, I2C, Interrupts<INT, SUBS, M>>
where
    INT: Wait,
    I2C: I2c + Clone,
{
    async fn wait_for(
        &self,
        mut is_wanted_state: impl FnMut(bool) -> bool,
    ) -> Result<(), InterruptWaitError<INT::Error, I2C::Error>> {
        if is_wanted_state(
            self.read_input()
                .await
                .map_err(InterruptWaitError::I2CError)?,
        ) {
            return Ok(());
        }
        self.0
            .wait_for_interrupt_with_cond(move |mask| is_wanted_state(read_bit(mask, self.1)))
            .await?;
        Ok(())
    }
}

impl<I2C, INT, const SUBS: usize, M: RawMutex> Wait for ExioPin<'_, I2C, Interrupts<INT, SUBS, M>>
where
    INT: Wait,
    I2C: I2c + Clone,
{
    fn wait_for_low(&mut self) -> impl Future<Output = Result<(), Self::Error>> {
        self.wait_for(|high| !high)
    }
    fn wait_for_high(&mut self) -> impl Future<Output = Result<(), Self::Error>> {
        self.wait_for(|high| high)
    }
    fn wait_for_rising_edge(&mut self) -> impl Future<Output = Result<(), Self::Error>> {
        self.wait_for(|high| !high)
            .and_then(|_| self.wait_for(|high| high))
    }
    fn wait_for_falling_edge(&mut self) -> impl Future<Output = Result<(), Self::Error>> {
        self.wait_for(|high| high)
            .and_then(|_| self.wait_for(|high| !high))
    }
    fn wait_for_any_edge(&mut self) -> impl Future<Output = Result<(), Self::Error>> {
        self.0
            .wait_for_interrupt_with_cond(|_m| true)
            .map_ok(|_| ())
    }
}
