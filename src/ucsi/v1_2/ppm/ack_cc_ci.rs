//! Types for the ACK_CC_CI command
use bitfield::bitfield;
use bytemuck::{Pod, Zeroable};

use crate::ucsi::v1_2::{CommandHeaderRaw, COMMAND_LEN};

bitfield! {
    /// Raw ack flags, see UCSI spec 6.5.4 for details
    #[derive(Copy, Clone, PartialEq, Eq)]
    pub struct AckRaw(u8);
    impl Debug;

    /// Ack connector change
    pub bool, connector_change, set_connector_change: 0;
    /// Ack command complete
    pub bool, command_complete, set_command_complete: 1;
}

#[cfg(feature = "defmt")]
impl defmt::Format for AckRaw {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "AckRaw {{ .0: {}, connector_change: {}, command_complete: {} }}",
            self.0,
            self.connector_change(),
            self.command_complete()
        )
    }
}

/// Higher-level representation of [`AckRaw`]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Ack {
    /// Ack connector change
    pub connector_change: bool,
    /// Ack command complete
    pub command_complete: bool,
}

impl From<AckRaw> for Ack {
    fn from(raw: AckRaw) -> Self {
        Self {
            connector_change: raw.connector_change(),
            command_complete: raw.command_complete(),
        }
    }
}

impl From<Ack> for AckRaw {
    fn from(ack: Ack) -> Self {
        let mut raw = AckRaw(0);
        raw.set_connector_change(ack.connector_change);
        raw.set_command_complete(ack.command_complete);
        raw
    }
}

impl From<u8> for Ack {
    fn from(raw: u8) -> Self {
        AckRaw(raw).into()
    }
}

impl From<Ack> for u8 {
    fn from(ack: Ack) -> Self {
        AckRaw::from(ack).0
    }
}

/// ACK_CC_CI command structure
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Args {
    /// Ack flags
    pub ack: Ack,
}

/// Data length for the ACK_CC_CI command response
pub const RESPONSE_DATA_LEN: u8 = 0;
/// Command padding
pub const COMMAND_PADDING: usize = COMMAND_LEN - size_of::<CommandHeaderRaw>() - size_of::<AckRaw>();

/// Raw wire format of [`Args`]
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Zeroable, Pod)]
pub struct ArgsRaw {
    /// Ack flags
    pub ack: u8,
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
        defmt::write!(fmt, "ArgsRaw {{ ack: {} }}", AckRaw(self.ack))
    }
}

impl From<Args> for ArgsRaw {
    fn from(args: Args) -> Self {
        Self {
            ack: args.ack.into(),
            ..Default::default()
        }
    }
}

impl From<ArgsRaw> for Args {
    fn from(raw: ArgsRaw) -> Self {
        Self { ack: raw.ack.into() }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_raw_len() {
        assert_eq!(ArgsRaw::LEN, COMMAND_LEN - size_of::<CommandHeaderRaw>());
    }

    #[test]
    fn test_ack_raw_roundtrip() {
        for raw in 0..=u8::MAX {
            // Only the two defined bits survive the roundtrip
            let expected = AckRaw(raw & 0x3);
            assert_eq!(AckRaw::from(Ack::from(AckRaw(raw))), expected);
        }

        assert_eq!(
            Ack::from(0x1),
            Ack {
                connector_change: true,
                command_complete: false
            }
        );
        assert_eq!(
            Ack::from(0x2),
            Ack {
                connector_change: false,
                command_complete: true
            }
        );
    }

    #[test]
    fn test_args_raw_roundtrip() {
        let args = Args {
            ack: Ack {
                connector_change: true,
                command_complete: true,
            },
        };

        let mut expected = [0u8; ArgsRaw::LEN];
        expected[0] = 0x3;

        let bytes: [u8; ArgsRaw::LEN] = bytemuck::must_cast(ArgsRaw::from(args));
        assert_eq!(bytes, expected);
        assert_eq!(Args::from(bytemuck::must_cast::<_, ArgsRaw>(expected)), args);
    }
}
