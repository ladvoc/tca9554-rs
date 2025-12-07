use embedded_hal_async::i2c::SevenBitAddress;

/// I²C address for a TCA9554(A) I/O expander.
///
/// For the standard variant (model not ending in 'A'), use [`Self::standard`].
/// Otherwise, use [`Self::alternate`].
///
/// Datasheet reference: section 8.6.1, figure 19.
///
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Address(SevenBitAddress);

impl Address {
    /// Address for the standard variant of the chip (model TCA9554).
    pub fn standard() -> Self {
        Self(0x40 >> 1)
    }

    /// Address for the alternate variant of the chip (model TCA9554A).
    pub fn alternate() -> Self {
        Self(0x70 >> 1)
    }

    /// Sets the user selectable, rightmost bits in the address: `(a3, a2, a1)`.
    pub fn with_selectable_bits(self, ax: (bool, bool, bool)) -> Self {
        let (a2, a1, a0) = ax;
        let bits = ((a2 as u8) << 2) | ((a1 as u8) << 1) | a0 as u8;
        Self(self.0 | bits)
    }
}

impl From<Address> for SevenBitAddress {
    fn from(val: Address) -> Self {
        val.0
    }
}

#[cfg(test)]
mod tests {
    use super::{Address, SevenBitAddress};

    #[test]
    fn test_standard() {
        let addr: SevenBitAddress = Address::standard().into();
        assert_eq!(addr, 0x20);
    }

    #[test]
    fn test_alternate() {
        let addr: SevenBitAddress = Address::alternate().into();
        assert_eq!(addr, 0x38);
    }

    #[test]
    fn test_with_selectable_bits() {
        let addr: SevenBitAddress = Address::standard()
            .with_selectable_bits((false, true, false))
            .into();
        assert_eq!(addr, 0x22);
    }
}
