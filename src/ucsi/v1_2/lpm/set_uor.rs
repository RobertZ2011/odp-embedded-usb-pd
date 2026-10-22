//! Types for SET_UOR command, see UCSI spec 6.5.9

use bitfield::bitfield;
use bytemuck::{Pod, Zeroable};
use pack1::U16LE;

use crate::ucsi::v1_2::{CommandHeaderRaw, COMMAND_LEN};
use crate::PortId;

/// Command padding
pub const COMMAND_PADDING: usize = COMMAND_LEN - size_of::<CommandHeaderRaw>() - size_of::<ArgBitsRaw>();

bitfield! {
    /// Raw argument bits
    #[derive(Copy, Clone, Default, PartialEq, Eq)]
    pub struct ArgBitsRaw(u16);
    impl Debug;

    /// Connector number
    pub u8, connector_number, set_connector_number: 6, 0;
    /// DFP
    pub bool, dfp, set_dfp: 7;
    /// UFP
    pub bool, ufp, set_ufp: 8;
    /// Accept data-role swap
    pub bool, accept_swap, set_accept_swap: 9;
}

#[cfg(feature = "defmt")]
impl defmt::Format for ArgBitsRaw {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "ArgBitsRaw {{ .0: {}, connector_number: {}, dfp: {}, ufp: {}, accept_swap: {} }}",
            self.0,
            self.connector_number(),
            self.dfp(),
            self.ufp(),
            self.accept_swap()
        )
    }
}

/// Command arguments
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Args<T: PortId> {
    /// Connector number
    pub connector_number: T,
    /// DFP
    pub dfp: bool,
    /// UFP
    pub ufp: bool,
    /// Accept data-role swap
    pub accept_swap: bool,
}

impl<T: PortId> Args<T> {
    /// Validate the data role fields
    pub fn validate(&self) -> Result<(), InvalidDataRole> {
        InvalidDataRole {
            dfp: self.dfp,
            ufp: self.ufp,
        }
        .validate()
    }
}

impl<T: PortId, Out: PortId> crate::ucsi::MapPort<T, Out> for Args<T> {
    type Output = Args<Out>;

    fn map_port<F>(self, f: F) -> Self::Output
    where
        F: Fn(T) -> Out,
    {
        Args {
            connector_number: f(self.connector_number),
            dfp: self.dfp,
            ufp: self.ufp,
            accept_swap: self.accept_swap,
        }
    }
}

impl<T: PortId> Default for Args<T> {
    fn default() -> Self {
        Self {
            connector_number: T::from(0),
            dfp: false,
            ufp: false,
            accept_swap: false,
        }
    }
}

/// Error used when both DFP and UFP are set to invalid values
///
/// Per the UCSI spec, the dfp and ufp fields cannot both be set to the same value.
/// See 4.5.9 for more details
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct InvalidDataRole {
    /// Indicates whether DFP was set
    pub dfp: bool,
    /// Indicates whether UFP was set
    pub ufp: bool,
}

impl InvalidDataRole {
    pub fn validate(&self) -> Result<(), Self> {
        if self.dfp == self.ufp {
            Err(Self {
                dfp: self.dfp,
                ufp: self.ufp,
            })
        } else {
            Ok(())
        }
    }
}

impl<T: PortId> TryFrom<ArgBitsRaw> for Args<T> {
    type Error = InvalidDataRole;

    fn try_from(raw: ArgBitsRaw) -> Result<Self, Self::Error> {
        let new = Self {
            connector_number: raw.connector_number().into(),
            dfp: raw.dfp(),
            ufp: raw.ufp(),
            accept_swap: raw.accept_swap(),
        };
        new.validate()?;
        Ok(new)
    }
}

impl<T: PortId> TryFrom<Args<T>> for ArgBitsRaw {
    type Error = InvalidDataRole;

    fn try_from(args: Args<T>) -> Result<Self, Self::Error> {
        args.validate()?;
        let mut raw = ArgBitsRaw(0);
        raw.set_connector_number(args.connector_number.into());
        raw.set_dfp(args.dfp);
        raw.set_ufp(args.ufp);
        raw.set_accept_swap(args.accept_swap);
        Ok(raw)
    }
}

impl<T: PortId> TryFrom<u16> for Args<T> {
    type Error = InvalidDataRole;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        ArgBitsRaw(value).try_into()
    }
}

impl<T: PortId> TryFrom<Args<T>> for u16 {
    type Error = InvalidDataRole;

    fn try_from(args: Args<T>) -> Result<Self, Self::Error> {
        Ok(ArgBitsRaw::try_from(args)?.0)
    }
}

/// Raw wire format of [`Args`]
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Zeroable, Pod)]
pub struct ArgsRaw {
    /// Argument bits, see [`ArgBitsRaw`]
    pub bits: U16LE,
    /// Reserved bytes, filling out the remainder of the command
    _reserved: [u8; COMMAND_PADDING],
}

impl ArgsRaw {
    /// Length of the raw arguments in bytes
    pub const LEN: usize = size_of::<Self>();
}

#[cfg(feature = "defmt")]
impl defmt::Format for ArgsRaw {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "ArgsRaw {{ bits: {} }}", ArgBitsRaw(self.bits.get()))
    }
}

impl<T: PortId> TryFrom<Args<T>> for ArgsRaw {
    type Error = InvalidDataRole;

    fn try_from(args: Args<T>) -> Result<Self, Self::Error> {
        Ok(Self {
            bits: U16LE::new(u16::try_from(args)?),
            ..Default::default()
        })
    }
}

impl<T: PortId> TryFrom<ArgsRaw> for Args<T> {
    type Error = InvalidDataRole;

    fn try_from(raw: ArgsRaw) -> Result<Self, Self::Error> {
        raw.bits.get().try_into()
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
    fn test_args_raw_roundtrip() {
        // DFP/accept swap on connector 3, the DFP bit shares a byte with the connector number
        let encoded: [u8; ArgsRaw::LEN] = [0x83, 0x02, 0x00, 0x00, 0x00, 0x00];
        let expected: Args<GlobalPortId> = Args {
            connector_number: GlobalPortId(3),
            dfp: true,
            accept_swap: true,
            ..Default::default()
        };

        assert_eq!(Args::try_from(bytemuck::must_cast::<_, ArgsRaw>(encoded)), Ok(expected));
        let bytes: [u8; ArgsRaw::LEN] = bytemuck::must_cast(ArgsRaw::try_from(expected).unwrap());
        assert_eq!(bytes, encoded);
    }

    #[test]
    fn test_args_raw_ignores_reserved() {
        let encoded: [u8; ArgsRaw::LEN] = [0x01, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF];
        let expected: Args<GlobalPortId> = Args {
            connector_number: GlobalPortId(1),
            ufp: true,
            accept_swap: true,
            ..Default::default()
        };
        assert_eq!(Args::try_from(bytemuck::must_cast::<_, ArgsRaw>(encoded)), Ok(expected));
        assert_eq!(
            bytemuck::must_cast::<ArgsRaw, [u8; ArgsRaw::LEN]>(expected.try_into().unwrap()),
            [0x01, 0x03, 0x00, 0x00, 0x00, 0x00]
        );
    }

    #[test]
    fn test_into_raw_validation_logic() {
        // Test basic values
        let raw_args = ArgsRaw::try_from(Args {
            connector_number: GlobalPortId(1),
            dfp: true,
            ufp: false,
            ..Default::default()
        });
        assert_eq!(
            raw_args,
            Ok(ArgsRaw {
                bits: U16LE::new(0x0081),
                ..Default::default()
            })
        );

        let raw_args = ArgsRaw::try_from(Args {
            connector_number: GlobalPortId(1),
            dfp: true,
            ufp: true,
            ..Default::default()
        });
        assert_eq!(raw_args, Err(InvalidDataRole { dfp: true, ufp: true }));

        let raw_args = ArgsRaw::try_from(Args {
            connector_number: GlobalPortId(1),
            dfp: false,
            ufp: false,
            ..Default::default()
        });
        assert_eq!(raw_args, Err(InvalidDataRole { dfp: false, ufp: false }));

        // Verify that the accept swap flag has no effect on validation
        let raw_args = ArgsRaw::try_from(Args {
            connector_number: GlobalPortId(1),
            dfp: true,
            ufp: false,
            accept_swap: true,
            ..Default::default()
        });
        assert_eq!(
            raw_args,
            Ok(ArgsRaw {
                bits: U16LE::new(0x0281),
                ..Default::default()
            })
        );

        let raw_args = ArgsRaw::try_from(Args {
            connector_number: GlobalPortId(1),
            dfp: true,
            ufp: true,
            accept_swap: true,
            ..Default::default()
        });
        assert_eq!(raw_args, Err(InvalidDataRole { dfp: true, ufp: true }));

        let raw_args = ArgsRaw::try_from(Args {
            connector_number: GlobalPortId(1),
            dfp: false,
            ufp: false,
            accept_swap: true,
            ..Default::default()
        });
        assert_eq!(raw_args, Err(InvalidDataRole { dfp: false, ufp: false }));
    }

    #[test]
    fn test_from_raw_validation_logic() {
        let args_raw = ArgsRaw {
            bits: U16LE::new(0x0081),
            ..Default::default()
        };
        assert_eq!(
            Args::<GlobalPortId>::try_from(args_raw),
            Ok(Args {
                connector_number: GlobalPortId(1),
                dfp: true,
                ufp: false,
                accept_swap: false,
            })
        );

        let args_raw = ArgsRaw {
            bits: U16LE::new(0x0101),
            ..Default::default()
        };
        assert_eq!(
            Args::<GlobalPortId>::try_from(args_raw),
            Ok(Args {
                connector_number: GlobalPortId(1),
                dfp: false,
                ufp: true,
                accept_swap: false,
            })
        );

        let args_raw = ArgsRaw {
            bits: U16LE::new(0x0181),
            ..Default::default()
        };
        assert_eq!(
            Args::<GlobalPortId>::try_from(args_raw),
            Err(InvalidDataRole { dfp: true, ufp: true })
        );

        let args_raw = ArgsRaw {
            bits: U16LE::new(0x0001),
            ..Default::default()
        };
        assert_eq!(
            Args::<GlobalPortId>::try_from(args_raw),
            Err(InvalidDataRole { dfp: false, ufp: false })
        );

        // The accept swap flag must not affect data role validation.
        let args_raw = ArgsRaw {
            bits: U16LE::new(0x0281),
            ..Default::default()
        };
        assert_eq!(
            Args::<GlobalPortId>::try_from(args_raw),
            Ok(Args {
                connector_number: GlobalPortId(1),
                dfp: true,
                ufp: false,
                accept_swap: true,
            })
        );

        let args_raw = ArgsRaw {
            bits: U16LE::new(0x0301),
            ..Default::default()
        };
        assert_eq!(
            Args::<GlobalPortId>::try_from(args_raw),
            Ok(Args {
                connector_number: GlobalPortId(1),
                dfp: false,
                ufp: true,
                accept_swap: true,
            })
        );

        let args_raw = ArgsRaw {
            bits: U16LE::new(0x0381),
            ..Default::default()
        };
        assert_eq!(
            Args::<GlobalPortId>::try_from(args_raw),
            Err(InvalidDataRole { dfp: true, ufp: true })
        );

        let args_raw = ArgsRaw {
            bits: U16LE::new(0x0201),
            ..Default::default()
        };
        assert_eq!(
            Args::<GlobalPortId>::try_from(args_raw),
            Err(InvalidDataRole { dfp: false, ufp: false })
        );
    }
}
