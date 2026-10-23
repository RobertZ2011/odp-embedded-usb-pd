//! Types for SET_CCOM command, see UCSI spec 6.5.8

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
    /// Rp
    pub bool, rp, set_rp: 7;
    /// Rd
    pub bool, rd, set_rd: 8;
    /// Drp
    pub bool, drp, set_drp: 9;
}

#[cfg(feature = "defmt")]
impl defmt::Format for ArgBitsRaw {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "ArgBitsRaw {{ .0: {}, connector_number: {}, rp: {}, rd: {}, drp: {} }}",
            self.0,
            self.connector_number(),
            self.rp(),
            self.rd(),
            self.drp(),
        )
    }
}

/// Command arguments
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Args<T: PortId> {
    /// Connector number
    pub connector_number: T,
    /// Rp
    pub rp: bool,
    /// Rd
    pub rd: bool,
    /// Drp
    pub drp: bool,
}

impl<T: PortId> Args<T> {
    pub fn validate(&self) -> Result<(), InvalidCcomFlags> {
        InvalidCcomFlags {
            rp: self.rp,
            rd: self.rd,
            drp: self.drp,
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
            rp: self.rp,
            rd: self.rd,
            drp: self.drp,
        }
    }
}

impl<T: PortId> Default for Args<T> {
    fn default() -> Self {
        Self {
            connector_number: T::from(0),
            rp: false,
            rd: false,
            drp: false,
        }
    }
}

/// Error used when RP, RD, and DRP are all false
///
/// Per the UCSI spec, it is illegal to set all these to false
/// See 4.5.8 for more details.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct InvalidCcomFlags {
    /// RP value
    pub rp: bool,
    /// RD value
    pub rd: bool,
    /// DRP value
    pub drp: bool,
}

impl InvalidCcomFlags {
    /// Validates the current combination of flags
    fn validate(&self) -> Result<(), Self> {
        if !self.rp && !self.rd && !self.drp {
            Err(*self)
        } else {
            Ok(())
        }
    }
}

impl<T: PortId> TryFrom<ArgBitsRaw> for Args<T> {
    type Error = InvalidCcomFlags;

    fn try_from(raw: ArgBitsRaw) -> Result<Self, Self::Error> {
        let args = Self {
            connector_number: raw.connector_number().into(),
            rp: raw.rp(),
            rd: raw.rd(),
            drp: raw.drp(),
        };
        args.validate()?;
        Ok(args)
    }
}

impl<T: PortId> TryFrom<Args<T>> for ArgBitsRaw {
    type Error = InvalidCcomFlags;

    fn try_from(args: Args<T>) -> Result<Self, Self::Error> {
        args.validate()?;
        let mut raw = ArgBitsRaw(0);
        raw.set_connector_number(args.connector_number.into());
        raw.set_rp(args.rp);
        raw.set_rd(args.rd);
        raw.set_drp(args.drp);
        Ok(raw)
    }
}

impl<T: PortId> TryFrom<u16> for Args<T> {
    type Error = InvalidCcomFlags;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        ArgBitsRaw(value).try_into()
    }
}

impl<T: PortId> TryFrom<Args<T>> for u16 {
    type Error = InvalidCcomFlags;

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
    type Error = InvalidCcomFlags;

    fn try_from(args: Args<T>) -> Result<Self, Self::Error> {
        Ok(Self {
            bits: U16LE::new(args.try_into()?),
            ..Default::default()
        })
    }
}

impl<T: PortId> TryFrom<ArgsRaw> for Args<T> {
    type Error = InvalidCcomFlags;

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
        // Rp/DRP on connector 3, the Rp bit shares a byte with the connector number
        let encoded: [u8; ArgsRaw::LEN] = [0x83, 0x02, 0x00, 0x00, 0x00, 0x00];
        let expected: Args<GlobalPortId> = Args {
            connector_number: GlobalPortId(3),
            rp: true,
            drp: true,
            ..Default::default()
        };

        assert_eq!(Args::try_from(bytemuck::must_cast::<_, ArgsRaw>(encoded)), Ok(expected));
        let bytes: [u8; ArgsRaw::LEN] = bytemuck::must_cast(ArgsRaw::try_from(expected).unwrap());
        assert_eq!(bytes, encoded);
    }

    #[test]
    fn test_args_raw_ignores_reserved() {
        let encoded: [u8; ArgsRaw::LEN] = [0x01, 0xFB, 0xFF, 0xFF, 0xFF, 0xFF];
        let expected: Args<GlobalPortId> = Args {
            connector_number: GlobalPortId(1),
            rd: true,
            drp: true,
            ..Default::default()
        };
        assert_eq!(Args::try_from(bytemuck::must_cast::<_, ArgsRaw>(encoded)), Ok(expected));
    }

    #[test]
    fn test_into_raw_validation_logic() {
        assert_eq!(
            ArgsRaw::try_from(Args {
                connector_number: GlobalPortId(1),
                rp: false,
                rd: false,
                drp: false,
            }),
            Err(InvalidCcomFlags {
                rp: false,
                rd: false,
                drp: false
            })
        );

        let encoded: [u8; ArgsRaw::LEN] = [0x01, 0x02, 0x00, 0x00, 0x00, 0x00];
        let expected: Args<GlobalPortId> = Args {
            connector_number: GlobalPortId(1),
            rp: false,
            rd: false,
            drp: true,
        };
        assert_eq!(Args::try_from(bytemuck::must_cast::<_, ArgsRaw>(encoded)), Ok(expected));

        let encoded: [u8; ArgsRaw::LEN] = [0x01, 0x01, 0x00, 0x00, 0x00, 0x00];
        let expected: Args<GlobalPortId> = Args {
            connector_number: GlobalPortId(1),
            rp: false,
            rd: true,
            drp: false,
        };
        assert_eq!(Args::try_from(bytemuck::must_cast::<_, ArgsRaw>(encoded)), Ok(expected));

        let encoded: [u8; ArgsRaw::LEN] = [0x01, 0x03, 0x00, 0x00, 0x00, 0x00];
        let expected: Args<GlobalPortId> = Args {
            connector_number: GlobalPortId(1),
            rp: false,
            rd: true,
            drp: true,
        };
        assert_eq!(Args::try_from(bytemuck::must_cast::<_, ArgsRaw>(encoded)), Ok(expected));

        let encoded: [u8; ArgsRaw::LEN] = [0x81, 0x00, 0x00, 0x00, 0x00, 0x00];
        let expected: Args<GlobalPortId> = Args {
            connector_number: GlobalPortId(1),
            rp: true,
            rd: false,
            drp: false,
        };
        assert_eq!(Args::try_from(bytemuck::must_cast::<_, ArgsRaw>(encoded)), Ok(expected));

        let encoded: [u8; ArgsRaw::LEN] = [0x81, 0x02, 0x00, 0x00, 0x00, 0x00];
        let expected: Args<GlobalPortId> = Args {
            connector_number: GlobalPortId(1),
            rp: true,
            rd: false,
            drp: true,
        };
        assert_eq!(Args::try_from(bytemuck::must_cast::<_, ArgsRaw>(encoded)), Ok(expected));

        let encoded: [u8; ArgsRaw::LEN] = [0x81, 0x01, 0x00, 0x00, 0x00, 0x00];
        let expected: Args<GlobalPortId> = Args {
            connector_number: GlobalPortId(1),
            rp: true,
            rd: true,
            drp: false,
        };
        assert_eq!(Args::try_from(bytemuck::must_cast::<_, ArgsRaw>(encoded)), Ok(expected));

        let encoded: [u8; ArgsRaw::LEN] = [0x81, 0x03, 0x00, 0x00, 0x00, 0x00];
        let expected: Args<GlobalPortId> = Args {
            connector_number: GlobalPortId(1),
            rp: true,
            rd: true,
            drp: true,
        };
        assert_eq!(Args::try_from(bytemuck::must_cast::<_, ArgsRaw>(encoded)), Ok(expected));
    }

    #[test]
    fn test_from_raw_validation_logic() {
        let base: Args<GlobalPortId> = Args {
            connector_number: GlobalPortId(1),
            ..Default::default()
        };

        // 8 possible values, try them all
        let none = Args {
            rp: false,
            rd: false,
            drp: false,
            ..base
        };
        assert_eq!(
            ArgsRaw::try_from(none),
            Err(InvalidCcomFlags {
                rp: false,
                rd: false,
                drp: false
            })
        );

        let drp = Args {
            rp: false,
            rd: false,
            drp: true,
            ..base
        };
        let bytes: [u8; ArgsRaw::LEN] = bytemuck::must_cast(ArgsRaw::try_from(drp).unwrap());
        assert_eq!(bytes, [0x01, 0x02, 0x00, 0x00, 0x00, 0x00]);

        let rd = Args {
            rp: false,
            rd: true,
            drp: false,
            ..base
        };
        let bytes: [u8; ArgsRaw::LEN] = bytemuck::must_cast(ArgsRaw::try_from(rd).unwrap());
        assert_eq!(bytes, [0x01, 0x01, 0x00, 0x00, 0x00, 0x00]);

        let rd_drp = Args {
            rp: false,
            rd: true,
            drp: true,
            ..base
        };
        let bytes: [u8; ArgsRaw::LEN] = bytemuck::must_cast(ArgsRaw::try_from(rd_drp).unwrap());
        assert_eq!(bytes, [0x01, 0x03, 0x00, 0x00, 0x00, 0x00]);

        let rp = Args {
            rp: true,
            rd: false,
            drp: false,
            ..base
        };
        let bytes: [u8; ArgsRaw::LEN] = bytemuck::must_cast(ArgsRaw::try_from(rp).unwrap());
        assert_eq!(bytes, [0x81, 0x00, 0x00, 0x00, 0x00, 0x00]);

        let rp_drp = Args {
            rp: true,
            rd: false,
            drp: true,
            ..base
        };
        let bytes: [u8; ArgsRaw::LEN] = bytemuck::must_cast(ArgsRaw::try_from(rp_drp).unwrap());
        assert_eq!(bytes, [0x81, 0x02, 0x00, 0x00, 0x00, 0x00]);

        let rp_rd = Args {
            rp: true,
            rd: true,
            drp: false,
            ..base
        };
        let bytes: [u8; ArgsRaw::LEN] = bytemuck::must_cast(ArgsRaw::try_from(rp_rd).unwrap());
        assert_eq!(bytes, [0x81, 0x01, 0x00, 0x00, 0x00, 0x00]);

        let all = Args {
            rp: true,
            rd: true,
            drp: true,
            ..base
        };
        let bytes: [u8; ArgsRaw::LEN] = bytemuck::must_cast(ArgsRaw::try_from(all).unwrap());
        assert_eq!(bytes, [0x81, 0x03, 0x00, 0x00, 0x00, 0x00]);
    }
}
