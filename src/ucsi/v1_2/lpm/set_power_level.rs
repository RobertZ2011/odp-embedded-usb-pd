//! Types for SET_POWER_LEVEL command, see UCSI spec 6.5.19

use bitfield::bitfield;
use bytemuck::{Pod, Zeroable};

use crate::pdo::MW500_UNIT;
use crate::ucsi::v1_2::{CommandHeaderRaw, COMMAND_LEN};
use crate::{type_c, PortId, PowerRole};

/// Length of the raw argument bits, this command uses the entire payload
pub const ARG_BITS_LEN: usize = COMMAND_LEN - size_of::<CommandHeaderRaw>();

bitfield! {
    /// Raw argument bits
    ///
    /// Several fields cross byte boundaries, so this covers the entire payload.
    #[derive(Copy, Clone, Default, PartialEq, Eq)]
    pub struct ArgBitsRaw([u8]);
    impl Debug;

    /// Connector number
    pub u8, connector_number, set_connector_number: 6, 0;
    /// Power role, 1 = source
    pub bool, power_role, set_power_role: 7;
    /// Max PD power, in 0.5W/1W units depending on [`Self::lsb_control`]
    pub u8, max_power, set_max_power: 15, 8;
    /// Type-C current
    pub u8, type_c_current, set_type_c_current: 18, 16;
}

#[cfg(feature = "defmt")]
impl defmt::Format for ArgBitsRaw<[u8; ARG_BITS_LEN]> {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "ArgBitsRaw {{ .0: {}, \
            connector_number: {}, \
            power_role: {}, \
            max_power: {}, \
            type_c_current: {} }}",
            self.0,
            self.connector_number(),
            self.power_role(),
            self.max_power(),
            self.type_c_current(),
        )
    }
}

/// Type-C current
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Current {
    /// PPM default
    #[default]
    PpmDefault,
    /// Type-C current
    Current(type_c::Current),
}

/// Type-C current decode error, contains the invalid value
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct InvalidCurrent(pub u8);

impl TryFrom<u8> for Current {
    type Error = InvalidCurrent;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(Current::PpmDefault),
            0x01 => Ok(Current::Current(type_c::Current::Current3A0)),
            0x02 => Ok(Current::Current(type_c::Current::Current1A5)),
            0x03 => Ok(Current::Current(type_c::Current::UsbDefault)),
            v => Err(InvalidCurrent(v)),
        }
    }
}

impl From<Current> for u8 {
    fn from(value: Current) -> Self {
        match value {
            Current::PpmDefault => 0x00,
            Current::Current(type_c::Current::Current3A0) => 0x01,
            Current::Current(type_c::Current::Current1A5) => 0x02,
            Current::Current(type_c::Current::UsbDefault) => 0x03,
        }
    }
}

/// Command arguments
///
/// Power, current and voltage are stored in mW, mA and mV. Converting to the raw representation
/// truncates them to the wire resolution selected by [`Self::lsb_control`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Args<T: PortId> {
    /// Connector number
    pub connector_number: T,
    /// Power role
    pub power_role: PowerRole,
    /// Max PD power in mW
    pub max_power: u32,
    /// Type-C current
    pub type_c_current: Current,
}

impl<T: PortId, Out: PortId> crate::ucsi::MapPort<T, Out> for Args<T> {
    type Output = Args<Out>;

    fn map_port<F>(self, f: F) -> Self::Output
    where
        F: Fn(T) -> Out,
    {
        Args {
            connector_number: f(self.connector_number),
            power_role: self.power_role,
            max_power: self.max_power,
            type_c_current: self.type_c_current,
        }
    }
}

impl<T: PortId> Default for Args<T> {
    fn default() -> Self {
        Self {
            connector_number: T::from(0),
            power_role: PowerRole::default(),
            max_power: 0,
            type_c_current: Current::default(),
        }
    }
}

impl<T: PortId> TryFrom<ArgBitsRaw<[u8; ARG_BITS_LEN]>> for Args<T> {
    type Error = InvalidCurrent;

    fn try_from(raw: ArgBitsRaw<[u8; ARG_BITS_LEN]>) -> Result<Self, Self::Error> {
        Ok(Self {
            connector_number: raw.connector_number().into(),
            power_role: if raw.power_role() {
                PowerRole::Source
            } else {
                PowerRole::Sink
            },
            max_power: u32::from(raw.max_power()) * MW500_UNIT,
            type_c_current: Current::try_from(raw.type_c_current())?,
        })
    }
}

impl<T: PortId> TryFrom<Args<T>> for ArgBitsRaw<[u8; ARG_BITS_LEN]> {
    type Error = OverflowError;

    fn try_from(args: Args<T>) -> Result<Self, Self::Error> {
        let mut raw = ArgBitsRaw([0; ARG_BITS_LEN]);
        raw.set_connector_number(args.connector_number.into());
        raw.set_power_role(args.power_role == PowerRole::Source);

        let max_power = args.max_power / MW500_UNIT;
        if max_power > u8::MAX as u32 {
            return Err(OverflowError::MaxPowerOverflow(args.max_power));
        }
        raw.set_max_power(max_power as u8);

        raw.set_type_c_current(args.type_c_current.into());
        Ok(raw)
    }
}

/// Error type for [`Args`] conversion failures
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum OverflowError {
    /// The max power value cannot fit in 8 bits
    MaxPowerOverflow(u32),
}

/// Raw wire format of [`Args`]
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Zeroable, Pod)]
pub struct ArgsRaw {
    /// Argument bits, see [`ArgBitsRaw`]
    pub bits: [u8; ARG_BITS_LEN],
}

impl ArgsRaw {
    /// Length of the raw arguments in bytes
    pub const LEN: usize = size_of::<Self>();
}

#[cfg(feature = "defmt")]
impl defmt::Format for ArgsRaw {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "ArgsRaw {{ bits: {} }}", ArgBitsRaw(self.bits))
    }
}

impl<T: PortId> TryFrom<Args<T>> for ArgsRaw {
    type Error = OverflowError;

    fn try_from(args: Args<T>) -> Result<Self, Self::Error> {
        Ok(Self {
            bits: ArgBitsRaw::try_from(args)?.0,
        })
    }
}

impl<T: PortId> TryFrom<ArgsRaw> for Args<T> {
    type Error = InvalidCurrent;

    fn try_from(raw: ArgsRaw) -> Result<Self, Self::Error> {
        ArgBitsRaw(raw.bits).try_into()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::GlobalPortId;

    #[test]
    fn test_raw_len() {
        assert_eq!(ArgsRaw::LEN, COMMAND_LEN - size_of::<CommandHeaderRaw>());
    }

    #[test]
    fn test_current_roundtrip() {
        for raw in 0..=u8::MAX {
            match Current::try_from(raw) {
                Ok(current) => assert_eq!(u8::from(current), raw),
                Err(err) => {
                    assert!(raw > 0x03);
                    assert_eq!(err, InvalidCurrent(raw));
                }
            }
        }
    }

    #[test]
    fn test_args_raw_roundtrip() {
        // Source on connector 3
        // 1W max power
        // 1.5A Type-C current
        let encoded: [u8; ArgsRaw::LEN] = [0x83, 0x02, 0x02, 0x00, 0x00, 0x00];
        let expected: Args<GlobalPortId> = Args {
            connector_number: GlobalPortId(3),
            power_role: PowerRole::Source,
            max_power: 1000,
            type_c_current: Current::Current(type_c::Current::Current1A5),
        };

        assert_eq!(Args::try_from(bytemuck::must_cast::<_, ArgsRaw>(encoded)), Ok(expected));
        let bytes: [u8; ArgsRaw::LEN] = bytemuck::must_cast(ArgsRaw::try_from(expected).unwrap());
        assert_eq!(bytes, encoded);
    }

    #[test]
    fn test_args_raw_truncates_to_units() {
        let args: Args<GlobalPortId> = Args {
            max_power: 1499,
            ..Default::default()
        };
        let expected: Args<GlobalPortId> = Args {
            max_power: 1000,
            ..Default::default()
        };

        assert_eq!(Args::try_from(ArgsRaw::try_from(args).unwrap()), Ok(expected));
    }

    #[test]
    fn test_args_raw_invalid_current() {
        // Connector 0, invalid type_c_current value (0x4) at bits 18:16
        let encoded: [u8; ArgsRaw::LEN] = [0x00, 0x00, 0x04, 0x00, 0x00, 0x00];
        assert_eq!(
            Args::<GlobalPortId>::try_from(bytemuck::must_cast::<_, ArgsRaw>(encoded)),
            Err(InvalidCurrent(0x04))
        );
    }

    #[test]
    fn test_args_raw_invalid_max_power() {
        // Connector 0, max_power overflow
        let args: Args<GlobalPortId> = Args {
            // 256 * 500 mW
            max_power: 128000,
            ..Default::default()
        };
        assert_eq!(ArgsRaw::try_from(args), Err(OverflowError::MaxPowerOverflow(128000)));
    }
}
